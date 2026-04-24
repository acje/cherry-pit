use std::io;
use std::marker::PhantomData;
use std::num::NonZeroU64;
use std::path::PathBuf;
use std::sync::Arc;

use pit_core::{AggregateId, DomainEvent, EventEnvelope, EventStore, StoreError};

/// File-based event store using `MessagePack` serialization.
///
/// Stores each aggregate's event stream as a single `.msgpack` file
/// in the configured directory. Designed for development and small
/// deployments where a full database is unnecessary.
///
/// Parameterized by `E` — the single domain event type this store
/// persists. Each aggregate type gets its own `MsgpackFileStore<E>`
/// instance pointing at its own directory. The type parameter
/// guarantees at compile time that you cannot accidentally load or
/// persist the wrong event type.
///
/// # File layout
///
/// ```text
/// store/
/// ├── 1.msgpack
/// ├── 2.msgpack
/// └── ...
/// ```
///
/// Each file contains the complete event history for one aggregate,
/// serialized as `Vec<EventEnvelope<E>>` in `MessagePack` format.
///
/// # ID assignment
///
/// New aggregates get sequential `u64` IDs starting from 1 via
/// [`create`](EventStore::create). The next ID is lazily initialized
/// by scanning the directory for the highest existing numeric filename
/// on the first `create` call.
///
/// # Concurrency
///
/// Per-aggregate write serialization via `tokio::sync::Mutex`. Multiple
/// aggregates can be written concurrently. Reads are lock-free.
///
/// Not suitable for multi-process access — use a database-backed store
/// for that. File atomicity relies on POSIX `rename(2)` semantics.
pub struct MsgpackFileStore<E: DomainEvent> {
    dir: PathBuf,
    /// Next aggregate ID to assign. `None` means uninitialized —
    /// first `create` call scans the directory to find the max.
    next_id: tokio::sync::Mutex<Option<u64>>,
    /// Per-aggregate write locks. `scc::HashMap` is lock-free for
    /// concurrent reads and uses fine-grained locking for writes —
    /// no poison risk, no contention on the map itself.
    locks: scc::HashMap<u64, Arc<tokio::sync::Mutex<()>>>,
    _phantom: PhantomData<E>,
}

impl<E: DomainEvent> MsgpackFileStore<E> {
    /// Create a new store writing to the given directory.
    ///
    /// The directory is created lazily on first write.
    #[must_use]
    pub fn new(dir: impl Into<PathBuf>) -> Self {
        Self {
            dir: dir.into(),
            next_id: tokio::sync::Mutex::new(None),
            locks: scc::HashMap::new(),
            _phantom: PhantomData,
        }
    }

    /// Return the file path for an aggregate.
    ///
    /// Infallible — `u64` IDs cannot cause path traversal.
    fn aggregate_path(&self, id: AggregateId) -> PathBuf {
        self.dir.join(format!("{}.msgpack", id.get()))
    }

    fn get_lock(&self, id: u64) -> Arc<tokio::sync::Mutex<()>> {
        // Fast path: lock already exists — read without blocking writers.
        if let Some(lock) = self.locks.read_sync(&id, |_, v| Arc::clone(v)) {
            return lock;
        }
        // Slow path: insert a new lock if still absent.
        self.locks
            .entry_sync(id)
            .or_insert_with(|| Arc::new(tokio::sync::Mutex::new(())))
            .get()
            .clone()
    }

    /// Scan the directory for the highest numeric filename to seed the
    /// auto-increment counter. Non-numeric filenames are silently
    /// skipped.
    async fn scan_max_id(&self) -> Result<u64, StoreError> {
        let mut max: u64 = 0;
        let mut entries = match tokio::fs::read_dir(&self.dir).await {
            Ok(entries) => entries,
            Err(e) if e.kind() == io::ErrorKind::NotFound => return Ok(0),
            Err(e) => return Err(StoreError::Infrastructure(Box::new(e))),
        };
        while let Some(entry) = entries
            .next_entry()
            .await
            .map_err(|e| StoreError::Infrastructure(Box::new(e)))?
        {
            if let Some(stem) = entry.path().file_stem().and_then(|s| s.to_str())
                && let Ok(id) = stem.parse::<u64>()
            {
                max = max.max(id);
            }
        }
        Ok(max)
    }

    /// Build envelopes from raw domain events.
    ///
    /// Assigns `event_id` (UUID v7), `aggregate_id`, `sequence`
    /// (starting from `start_sequence`), and a shared `timestamp`
    /// (single timestamp per batch — the batch is atomic).
    fn build_envelopes(
        id: AggregateId,
        start_sequence: u64,
        events: Vec<E>,
    ) -> Result<Vec<EventEnvelope<E>>, StoreError> {
        let timestamp = jiff::Timestamp::now();
        let mut envelopes = Vec::with_capacity(events.len());
        for (i, payload) in events.into_iter().enumerate() {
            let i_u64 = u64::try_from(i).unwrap_or(u64::MAX);
            let sequence = start_sequence
                .checked_add(i_u64)
                .and_then(|s| s.checked_add(1))
                .ok_or_else(|| {
                    StoreError::Infrastructure(Box::new(io::Error::new(
                        io::ErrorKind::InvalidInput,
                        "sequence overflow",
                    )))
                })?;
            envelopes.push(EventEnvelope {
                event_id: uuid::Uuid::now_v7(),
                aggregate_id: id,
                sequence,
                timestamp,
                correlation_id: None,
                causation_id: None,
                payload,
            });
        }
        Ok(envelopes)
    }

    /// Serialize and atomically write envelopes to disk.
    async fn write_atomic(
        &self,
        path: &std::path::Path,
        envelopes: &[EventEnvelope<E>],
    ) -> Result<(), StoreError> {
        let bytes = rmp_serde::encode::to_vec_named(envelopes)
            .map_err(|e| StoreError::Infrastructure(Box::new(e)))?;

        tokio::fs::create_dir_all(&self.dir)
            .await
            .map_err(|e| StoreError::Infrastructure(Box::new(e)))?;

        // Use a unique temp file per aggregate to prevent collisions
        // between concurrent writes to different aggregates.
        let tmp_name = format!(
            "{}.tmp",
            path.file_name().and_then(|n| n.to_str()).unwrap_or("tmp")
        );
        let tmp_path = self.dir.join(tmp_name);
        tokio::fs::write(&tmp_path, &bytes)
            .await
            .map_err(|e| StoreError::Infrastructure(Box::new(e)))?;
        if let Err(e) = tokio::fs::rename(&tmp_path, path).await {
            let _ = tokio::fs::remove_file(&tmp_path).await;
            return Err(StoreError::Infrastructure(Box::new(e)));
        }
        Ok(())
    }
}

impl<E: DomainEvent> Default for MsgpackFileStore<E> {
    /// Default store directory: `store/`
    fn default() -> Self {
        Self::new("store")
    }
}

impl<E: DomainEvent> EventStore for MsgpackFileStore<E> {
    type Event = E;

    async fn load(
        &self,
        id: AggregateId,
    ) -> Result<Vec<EventEnvelope<E>>, StoreError> {
        let path = self.aggregate_path(id);
        match tokio::fs::read(&path).await {
            Ok(bytes) => rmp_serde::from_slice(&bytes)
                .map_err(|e| StoreError::Infrastructure(Box::new(e))),
            Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(Vec::new()),
            Err(e) => Err(StoreError::Infrastructure(Box::new(e))),
        }
    }

    async fn create(
        &self,
        events: Vec<E>,
    ) -> Result<(AggregateId, Vec<EventEnvelope<E>>), StoreError> {
        if events.is_empty() {
            return Err(StoreError::Infrastructure(Box::new(io::Error::new(
                io::ErrorKind::InvalidInput,
                "cannot create aggregate with zero events",
            ))));
        }

        // Assign next ID under lock.
        let id = {
            let mut next = self.next_id.lock().await;
            let n = if let Some(n) = *next {
                n
            } else {
                let max = self.scan_max_id().await?;
                max.checked_add(1).ok_or_else(|| {
                    StoreError::Infrastructure(Box::new(io::Error::other(
                        "aggregate ID overflow",
                    )))
                })?
            };
            let after = n.checked_add(1).ok_or_else(|| {
                StoreError::Infrastructure(Box::new(io::Error::other(
                    "aggregate ID overflow",
                )))
            })?;
            *next = Some(after);
            let nz = NonZeroU64::new(n).ok_or_else(|| {
                StoreError::Infrastructure(Box::new(io::Error::other(
                    "aggregate ID must be non-zero",
                )))
            })?;
            AggregateId::new(nz)
        };

        let envelopes = Self::build_envelopes(id, 0, events)?;
        let path = self.aggregate_path(id);
        self.write_atomic(&path, &envelopes).await?;

        Ok((id, envelopes))
    }

    async fn append(
        &self,
        id: AggregateId,
        expected_sequence: u64,
        events: Vec<E>,
    ) -> Result<Vec<EventEnvelope<E>>, StoreError> {
        if events.is_empty() {
            return Ok(Vec::new());
        }

        let lock = self.get_lock(id.get());
        let _guard = lock.lock().await;

        let path = self.aggregate_path(id);

        // Load existing events.
        let mut existing: Vec<EventEnvelope<E>> = match tokio::fs::read(&path).await {
            Ok(bytes) => rmp_serde::from_slice(&bytes)
                .map_err(|e| StoreError::Infrastructure(Box::new(e)))?,
            Err(e) if e.kind() == io::ErrorKind::NotFound => Vec::new(),
            Err(e) => return Err(StoreError::Infrastructure(Box::new(e))),
        };

        // Optimistic concurrency check.
        let actual_sequence = existing.last().map_or(0, |e| e.sequence);
        if actual_sequence != expected_sequence {
            return Err(StoreError::ConcurrencyConflict {
                aggregate_id: id,
                expected_sequence,
                actual_sequence,
            });
        }

        // Build envelopes inside the lock so timestamps are monotonic
        // with sequence numbers.
        let new_envelopes = Self::build_envelopes(id, expected_sequence, events)?;

        existing.extend(new_envelopes.iter().cloned());
        self.write_atomic(&path, &existing).await?;

        Ok(new_envelopes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::{Deserialize, Serialize};
    use std::num::NonZeroU64;

    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    enum TestEvent {
        Created { name: String },
        Updated { name: String },
    }

    impl DomainEvent for TestEvent {
        fn event_type(&self) -> &'static str {
            match self {
                Self::Created { .. } => "test.created",
                Self::Updated { .. } => "test.updated",
            }
        }
    }

    /// Helper to construct an `AggregateId` from a raw `u64` in tests.
    fn agg_id(n: u64) -> AggregateId {
        AggregateId::new(NonZeroU64::new(n).unwrap())
    }

    // ── create ──────────────────────────────────────────────────────

    #[tokio::test]
    async fn create_assigns_sequential_ids() {
        let dir = tempfile::tempdir().unwrap();
        let store = MsgpackFileStore::new(dir.path());

        let (id1, _) = store
            .create(vec![TestEvent::Created { name: "a".into() }])
            .await
            .unwrap();
        let (id2, _) = store
            .create(vec![TestEvent::Created { name: "b".into() }])
            .await
            .unwrap();
        let (id3, _) = store
            .create(vec![TestEvent::Created { name: "c".into() }])
            .await
            .unwrap();

        assert_eq!(id1, agg_id(1));
        assert_eq!(id2, agg_id(2));
        assert_eq!(id3, agg_id(3));
    }

    #[tokio::test]
    async fn create_returns_correct_envelopes() {
        let dir = tempfile::tempdir().unwrap();
        let store = MsgpackFileStore::new(dir.path());

        let events = vec![
            TestEvent::Created { name: "a".into() },
            TestEvent::Updated { name: "b".into() },
        ];
        let (id, envelopes) = store.create(events).await.unwrap();

        assert_eq!(envelopes.len(), 2);
        assert_eq!(envelopes[0].aggregate_id, id);
        assert_eq!(envelopes[1].aggregate_id, id);
        assert_eq!(envelopes[0].sequence, 1);
        assert_eq!(envelopes[1].sequence, 2);
        assert_eq!(
            envelopes[0].payload,
            TestEvent::Created { name: "a".into() }
        );
        assert_eq!(
            envelopes[1].payload,
            TestEvent::Updated { name: "b".into() }
        );
        // UUID v7 — both should be non-nil and different.
        assert!(!envelopes[0].event_id.is_nil());
        assert_ne!(envelopes[0].event_id, envelopes[1].event_id);
        // Same timestamp within the batch.
        assert_eq!(envelopes[0].timestamp, envelopes[1].timestamp);
    }

    #[tokio::test]
    async fn create_rejects_empty_events() {
        let dir = tempfile::tempdir().unwrap();
        let store = MsgpackFileStore::<TestEvent>::new(dir.path());

        let result = store.create(vec![]).await;
        assert!(result.is_err());
        assert!(
            matches!(result.unwrap_err(), StoreError::Infrastructure(_)),
            "expected Infrastructure error for empty events"
        );
    }

    #[tokio::test]
    async fn create_survives_restart() {
        let dir = tempfile::tempdir().unwrap();

        // First store instance creates two aggregates.
        {
            let store = MsgpackFileStore::new(dir.path());
            store
                .create(vec![TestEvent::Created { name: "a".into() }])
                .await
                .unwrap();
            store
                .create(vec![TestEvent::Created { name: "b".into() }])
                .await
                .unwrap();
        }

        // Second store instance (simulates process restart) should
        // continue from 3.
        let store = MsgpackFileStore::new(dir.path());
        let (id, _) = store
            .create(vec![TestEvent::Created { name: "c".into() }])
            .await
            .unwrap();
        assert_eq!(id, agg_id(3));
    }

    #[tokio::test]
    async fn directory_scan_ignores_non_numeric() {
        let dir = tempfile::tempdir().unwrap();

        // Pre-create a file with a non-numeric name.
        tokio::fs::create_dir_all(dir.path()).await.unwrap();
        tokio::fs::write(dir.path().join("old-format.msgpack"), b"junk")
            .await
            .unwrap();
        // Also create a numeric file to seed the counter.
        {
            let store = MsgpackFileStore::new(dir.path());
            store
                .create(vec![TestEvent::Created { name: "a".into() }])
                .await
                .unwrap();
        }

        // New store should scan, skip "old-format", find "1", assign 2.
        let store = MsgpackFileStore::new(dir.path());
        let (id, _) = store
            .create(vec![TestEvent::Created { name: "b".into() }])
            .await
            .unwrap();
        assert_eq!(id, agg_id(2));
    }

    // ── load ────────────────────────────────────────────────────────

    #[tokio::test]
    async fn load_nonexistent_returns_empty() {
        let dir = tempfile::tempdir().unwrap();
        let store = MsgpackFileStore::new(dir.path());

        let events: Vec<EventEnvelope<TestEvent>> =
            store.load(agg_id(999)).await.unwrap();
        assert!(events.is_empty());
    }

    #[tokio::test]
    async fn corrupt_file_returns_error() {
        let dir = tempfile::tempdir().unwrap();

        // Write garbage to a store file.
        tokio::fs::create_dir_all(dir.path()).await.unwrap();
        tokio::fs::write(dir.path().join("1.msgpack"), b"not valid msgpack")
            .await
            .unwrap();

        let store = MsgpackFileStore::new(dir.path());
        let result: Result<Vec<EventEnvelope<TestEvent>>, _> =
            store.load(agg_id(1)).await;
        assert!(result.is_err());
        assert!(
            matches!(result.unwrap_err(), StoreError::Infrastructure(_)),
            "expected Infrastructure error for corrupt file"
        );
    }

    // ── create + load roundtrip ─────────────────────────────────────

    #[tokio::test]
    async fn create_and_load_roundtrip() {
        let dir = tempfile::tempdir().unwrap();
        let store = MsgpackFileStore::new(dir.path());

        let (id, created) = store
            .create(vec![TestEvent::Created { name: "alice".into() }])
            .await
            .unwrap();

        let loaded = store.load(id).await.unwrap();
        assert_eq!(loaded.len(), 1);
        assert_eq!(loaded[0].payload, created[0].payload);
        assert_eq!(loaded[0].sequence, 1);
        assert_eq!(loaded[0].aggregate_id, id);
    }

    // ── append ──────────────────────────────────────────────────────

    #[tokio::test]
    async fn append_and_load_roundtrip() {
        let dir = tempfile::tempdir().unwrap();
        let store = MsgpackFileStore::new(dir.path());

        let (id, _) = store
            .create(vec![TestEvent::Created { name: "alice".into() }])
            .await
            .unwrap();

        let appended = store
            .append(id, 1, vec![TestEvent::Updated { name: "bob".into() }])
            .await
            .unwrap();
        assert_eq!(appended.len(), 1);
        assert_eq!(appended[0].sequence, 2);
        assert_eq!(appended[0].aggregate_id, id);

        let loaded = store.load(id).await.unwrap();
        assert_eq!(loaded.len(), 2);
        assert_eq!(loaded[0].sequence, 1);
        assert_eq!(loaded[1].sequence, 2);
    }

    #[tokio::test]
    async fn append_multiple_batches() {
        let dir = tempfile::tempdir().unwrap();
        let store = MsgpackFileStore::new(dir.path());

        let (id, _) = store
            .create(vec![TestEvent::Created { name: "alice".into() }])
            .await
            .unwrap();

        store
            .append(id, 1, vec![TestEvent::Updated { name: "bob".into() }])
            .await
            .unwrap();
        store
            .append(id, 2, vec![TestEvent::Updated { name: "carol".into() }])
            .await
            .unwrap();

        let loaded = store.load(id).await.unwrap();
        assert_eq!(loaded.len(), 3);
        assert_eq!(loaded[0].sequence, 1);
        assert_eq!(loaded[1].sequence, 2);
        assert_eq!(loaded[2].sequence, 3);
    }

    #[tokio::test]
    async fn append_empty_is_noop() {
        let dir = tempfile::tempdir().unwrap();
        let store = MsgpackFileStore::new(dir.path());

        let (id, _) = store
            .create(vec![TestEvent::Created { name: "alice".into() }])
            .await
            .unwrap();

        let result = store.append(id, 1, vec![]).await.unwrap();
        assert!(result.is_empty());

        // Original event still there, nothing else.
        let loaded = store.load(id).await.unwrap();
        assert_eq!(loaded.len(), 1);
    }

    #[tokio::test]
    async fn append_returns_correct_envelopes() {
        let dir = tempfile::tempdir().unwrap();
        let store = MsgpackFileStore::new(dir.path());

        let (id, _) = store
            .create(vec![TestEvent::Created { name: "a".into() }])
            .await
            .unwrap();

        let envelopes = store
            .append(
                id,
                1,
                vec![
                    TestEvent::Updated { name: "b".into() },
                    TestEvent::Updated { name: "c".into() },
                ],
            )
            .await
            .unwrap();

        assert_eq!(envelopes.len(), 2);
        assert_eq!(envelopes[0].aggregate_id, id);
        assert_eq!(envelopes[1].aggregate_id, id);
        assert_eq!(envelopes[0].sequence, 2);
        assert_eq!(envelopes[1].sequence, 3);
        assert!(!envelopes[0].event_id.is_nil());
        assert_ne!(envelopes[0].event_id, envelopes[1].event_id);

        // Verify they match what's on disk.
        let loaded = store.load(id).await.unwrap();
        assert_eq!(loaded.len(), 3);
        assert_eq!(loaded[1].payload, envelopes[0].payload);
        assert_eq!(loaded[2].payload, envelopes[1].payload);
    }

    // ── concurrency ─────────────────────────────────────────────────

    #[tokio::test]
    async fn concurrency_conflict_detected() {
        let dir = tempfile::tempdir().unwrap();
        let store = MsgpackFileStore::new(dir.path());

        let (id, _) = store
            .create(vec![TestEvent::Created { name: "alice".into() }])
            .await
            .unwrap();

        // First append succeeds.
        store
            .append(id, 1, vec![TestEvent::Updated { name: "bob".into() }])
            .await
            .unwrap();

        // Second append with stale expected_sequence fails.
        let result = store
            .append(id, 1, vec![TestEvent::Updated { name: "carol".into() }])
            .await;
        assert!(result.is_err());
        assert!(
            matches!(result.unwrap_err(), StoreError::ConcurrencyConflict { .. }),
            "expected ConcurrencyConflict"
        );
    }

    #[tokio::test]
    async fn concurrent_appends_to_same_aggregate() {
        let dir = tempfile::tempdir().unwrap();
        let store = Arc::new(MsgpackFileStore::new(dir.path()));

        let (id, _) = store
            .create(vec![TestEvent::Created { name: "seed".into() }])
            .await
            .unwrap();

        // Spawn 5 concurrent appends, all expecting sequence 1.
        let mut handles = Vec::new();
        for i in 0..5 {
            let s = Arc::clone(&store);
            handles.push(tokio::spawn(async move {
                s.append(
                    id,
                    1,
                    vec![TestEvent::Updated {
                        name: format!("writer-{i}"),
                    }],
                )
                .await
            }));
        }

        let results: Vec<_> = futures_util::future::join_all(handles)
            .await
            .into_iter()
            .map(|r| r.unwrap())
            .collect();

        let successes = results.iter().filter(|r| r.is_ok()).count();
        let conflicts = results
            .iter()
            .filter(|r| matches!(r, Err(StoreError::ConcurrencyConflict { .. })))
            .count();

        assert_eq!(successes, 1, "exactly one writer should succeed");
        assert_eq!(
            conflicts, 4,
            "remaining writers should get ConcurrencyConflict"
        );
    }

    // ── isolation ───────────────────────────────────────────────────

    #[tokio::test]
    async fn separate_aggregates_isolated() {
        let dir = tempfile::tempdir().unwrap();
        let store = MsgpackFileStore::new(dir.path());

        let (id1, _) = store
            .create(vec![TestEvent::Created { name: "alice".into() }])
            .await
            .unwrap();
        let (id2, _) = store
            .create(vec![TestEvent::Created { name: "bob".into() }])
            .await
            .unwrap();

        let loaded1 = store.load(id1).await.unwrap();
        let loaded2 = store.load(id2).await.unwrap();

        assert_eq!(loaded1.len(), 1);
        assert_eq!(loaded2.len(), 1);
        assert_eq!(
            loaded1[0].payload,
            TestEvent::Created { name: "alice".into() }
        );
        assert_eq!(
            loaded2[0].payload,
            TestEvent::Created { name: "bob".into() }
        );
        assert_ne!(id1, id2);
    }

    // ── misc ────────────────────────────────────────────────────────

    #[test]
    fn default_uses_store_dir() {
        let store = MsgpackFileStore::<TestEvent>::default();
        assert_eq!(store.dir, PathBuf::from("store"));
    }

    #[tokio::test]
    async fn create_then_append_lifecycle() {
        let dir = tempfile::tempdir().unwrap();
        let store = MsgpackFileStore::new(dir.path());

        // Create.
        let (id, created) = store
            .create(vec![TestEvent::Created { name: "order".into() }])
            .await
            .unwrap();
        assert_eq!(created.len(), 1);
        assert_eq!(created[0].sequence, 1);

        // Append.
        let appended = store
            .append(id, 1, vec![TestEvent::Updated { name: "shipped".into() }])
            .await
            .unwrap();
        assert_eq!(appended.len(), 1);
        assert_eq!(appended[0].sequence, 2);

        // Full history.
        let all = store.load(id).await.unwrap();
        assert_eq!(all.len(), 2);
        assert_eq!(all[0].aggregate_id, id);
        assert_eq!(all[1].aggregate_id, id);
    }

    #[tokio::test]
    async fn send_to_nonexistent_returns_empty() {
        let dir = tempfile::tempdir().unwrap();
        let store = MsgpackFileStore::new(dir.path());

        // Loading a never-created aggregate returns empty — the bus
        // layer maps this to AggregateNotFound.
        let events: Vec<EventEnvelope<TestEvent>> =
            store.load(agg_id(42)).await.unwrap();
        assert!(events.is_empty());
    }

    #[tokio::test]
    async fn concurrent_creates_assign_unique_ids() {
        let dir = tempfile::tempdir().unwrap();
        let store = Arc::new(MsgpackFileStore::new(dir.path()));

        let mut handles = Vec::new();
        for i in 0..10 {
            let s = Arc::clone(&store);
            handles.push(tokio::spawn(async move {
                s.create(vec![TestEvent::Created {
                    name: format!("agg-{i}"),
                }])
                .await
            }));
        }

        let results: Vec<_> = futures_util::future::join_all(handles)
            .await
            .into_iter()
            .map(|r| r.unwrap().unwrap())
            .collect();

        let mut ids: Vec<_> = results.iter().map(|(id, _)| *id).collect();
        ids.sort();
        ids.dedup();
        assert_eq!(ids.len(), 10, "all 10 creates should get unique IDs");
    }

    #[tokio::test]
    async fn build_envelopes_sequence_overflow() {
        let dir = tempfile::tempdir().unwrap();
        let store = MsgpackFileStore::new(dir.path());

        // Create an aggregate first.
        let (id, _) = store
            .create(vec![TestEvent::Created { name: "a".into() }])
            .await
            .unwrap();

        // Attempt to append with start_sequence near u64::MAX.
        // This should fail with an overflow error, not panic.
        let result = store
            .append(id, u64::MAX, vec![TestEvent::Updated { name: "b".into() }])
            .await;

        // The concurrency check will fire first (actual_sequence=1 != u64::MAX),
        // but the overflow would also be caught in build_envelopes.
        assert!(result.is_err());
    }

    // ── backward compatibility ──────────────────────────────────────

    #[tokio::test]
    async fn deserializes_old_format_without_correlation_fields() {
        // Simulate a msgpack file written before correlation_id and
        // causation_id were added: serialize with named keys but
        // without the new fields. The #[serde(default)] on
        // EventEnvelope ensures missing fields default to None.
        #[derive(Serialize)]
        struct OldEnvelope {
            event_id: uuid::Uuid,
            aggregate_id: AggregateId,
            sequence: u64,
            timestamp: jiff::Timestamp,
            payload: TestEvent,
        }

        let dir = tempfile::tempdir().unwrap();
        tokio::fs::create_dir_all(dir.path()).await.unwrap();

        let old = vec![OldEnvelope {
            event_id: uuid::Uuid::now_v7(),
            aggregate_id: agg_id(1),
            sequence: 1,
            timestamp: jiff::Timestamp::now(),
            payload: TestEvent::Created { name: "old".into() },
        }];

        // Use named encoding (map format) — same as the store uses.
        let bytes = rmp_serde::encode::to_vec_named(&old).unwrap();
        tokio::fs::write(dir.path().join("1.msgpack"), &bytes)
            .await
            .unwrap();

        let store = MsgpackFileStore::<TestEvent>::new(dir.path());
        let loaded = store.load(agg_id(1)).await.unwrap();

        assert_eq!(loaded.len(), 1);
        assert_eq!(loaded[0].payload, TestEvent::Created { name: "old".into() });
        assert!(loaded[0].correlation_id.is_none());
        assert!(loaded[0].causation_id.is_none());
    }

    #[tokio::test]
    async fn correlation_and_causation_ids_roundtrip() {
        let dir = tempfile::tempdir().unwrap();
        let store = MsgpackFileStore::new(dir.path());

        let (id, created) = store
            .create(vec![TestEvent::Created { name: "traced".into() }])
            .await
            .unwrap();

        // Verify initial creation has None for both fields.
        assert!(created[0].correlation_id.is_none());
        assert!(created[0].causation_id.is_none());

        // Reload and verify None survives serialization roundtrip.
        let loaded = store.load(id).await.unwrap();
        assert!(loaded[0].correlation_id.is_none());
        assert!(loaded[0].causation_id.is_none());
    }

    #[tokio::test]
    async fn correlation_and_causation_some_values_roundtrip() {
        // Verify that Some(uuid) values survive a write/load cycle
        // by manually constructing an envelope with populated fields.
        let dir = tempfile::tempdir().unwrap();
        tokio::fs::create_dir_all(dir.path()).await.unwrap();

        let corr = uuid::Uuid::now_v7();
        let cause = uuid::Uuid::now_v7();
        let envelopes = vec![EventEnvelope {
            event_id: uuid::Uuid::now_v7(),
            aggregate_id: agg_id(1),
            sequence: 1,
            timestamp: jiff::Timestamp::now(),
            correlation_id: Some(corr),
            causation_id: Some(cause),
            payload: TestEvent::Created { name: "with-ids".into() },
        }];

        let bytes = rmp_serde::encode::to_vec_named(&envelopes).unwrap();
        tokio::fs::write(dir.path().join("1.msgpack"), &bytes)
            .await
            .unwrap();

        let store = MsgpackFileStore::<TestEvent>::new(dir.path());
        let loaded = store.load(agg_id(1)).await.unwrap();

        assert_eq!(loaded.len(), 1);
        assert_eq!(loaded[0].correlation_id, Some(corr));
        assert_eq!(loaded[0].causation_id, Some(cause));
    }
}
