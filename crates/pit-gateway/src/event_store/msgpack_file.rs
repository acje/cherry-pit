use std::io;
use std::marker::PhantomData;
use std::num::NonZeroU64;
use std::path::PathBuf;
use std::sync::Arc;

use pit_core::{AggregateId, CorrelationContext, DomainEvent, EventEnvelope, EventStore, StoreError};

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
///
/// # Process fencing
///
/// On first write, the store acquires an advisory `flock` on a `.lock`
/// sentinel file in the store directory. If another process already
/// holds the lock, the store returns `StoreError::StoreLocked`. This
/// detects accidental multi-process access but does not prevent
/// deliberate circumvention (advisory locks are not mandatory).
pub struct MsgpackFileStore<E: DomainEvent> {
    dir: PathBuf,
    /// Next aggregate ID to assign. `None` means uninitialized —
    /// first `create` call scans the directory to find the max.
    next_id: tokio::sync::Mutex<Option<u64>>,
    /// Per-aggregate write locks. `scc::HashMap` is lock-free for
    /// concurrent reads and uses fine-grained locking for writes —
    /// no poison risk, no contention on the map itself.
    locks: scc::HashMap<u64, Arc<tokio::sync::Mutex<()>>>,
    /// Advisory file lock on `{dir}/.lock`. Acquired lazily on first
    /// write operation, held for the store's lifetime. Detects
    /// accidental multi-process access to the same store directory.
    /// The `std::fs::File` handle keeps the flock alive — releasing
    /// happens automatically on drop.
    dir_lock: tokio::sync::OnceCell<std::fs::File>,
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
            dir_lock: tokio::sync::OnceCell::new(),
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

    /// Acquire an advisory file lock on the store directory.
    ///
    /// Called lazily on the first write operation (`create` or
    /// `append`). Uses `flock(2)` via `std::fs::File::try_lock` —
    /// the lock is held for the `MsgpackFileStore` lifetime (the
    /// `std::fs::File` handle lives in the `OnceCell`). Released
    /// automatically on drop.
    ///
    /// # Errors
    ///
    /// Returns `StoreError::StoreLocked` if another process already
    /// holds an exclusive lock on the same directory's `.lock` file.
    async fn ensure_fenced(&self) -> Result<(), StoreError> {
        self.dir_lock
            .get_or_try_init(|| async {
                let dir = self.dir.clone();
                tokio::task::spawn_blocking(move || {
                    std::fs::create_dir_all(&dir).map_err(|e| {
                        StoreError::Infrastructure(Box::new(e))
                    })?;

                    let lock_path = dir.join(".lock");
                    let file = std::fs::File::create(&lock_path).map_err(|e| {
                        StoreError::Infrastructure(Box::new(e))
                    })?;

                    file.try_lock().map_err(|e| match e {
                        std::fs::TryLockError::WouldBlock => {
                            StoreError::StoreLocked { path: dir }
                        }
                        std::fs::TryLockError::Error(io_err) => {
                            StoreError::Infrastructure(Box::new(io_err))
                        }
                    })?;

                    Ok(file)
                })
                .await
                .map_err(|e| StoreError::Infrastructure(Box::new(e)))?
            })
            .await?;
        Ok(())
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
    /// (starting from `start_sequence`), a shared `timestamp`
    /// (single timestamp per batch — the batch is atomic), and
    /// `correlation_id`/`causation_id` from the provided context.
    fn build_envelopes(
        id: AggregateId,
        start_sequence: u64,
        events: Vec<E>,
        context: &CorrelationContext,
    ) -> Result<Vec<EventEnvelope<E>>, StoreError> {
        let timestamp = jiff::Timestamp::now();
        let mut envelopes = Vec::with_capacity(events.len());
        for (i, payload) in events.into_iter().enumerate() {
            let i_u64 = u64::try_from(i).unwrap_or(u64::MAX);
            let sequence_raw = start_sequence
                .checked_add(i_u64)
                .and_then(|s| s.checked_add(1))
                .ok_or_else(|| {
                    StoreError::Infrastructure(Box::new(io::Error::new(
                        io::ErrorKind::InvalidInput,
                        "sequence overflow",
                    )))
                })?;
            let sequence = NonZeroU64::new(sequence_raw).ok_or_else(|| {
                StoreError::Infrastructure(Box::new(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "sequence must be non-zero",
                )))
            })?;
            let envelope = EventEnvelope::new(
                uuid::Uuid::now_v7(),
                id,
                sequence,
                timestamp,
                context.correlation_id(),
                context.causation_id(),
                payload,
            )
            .map_err(|e| StoreError::Infrastructure(Box::new(e)))?;
            envelopes.push(envelope);
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
            Ok(bytes) => {
                let envelopes: Vec<EventEnvelope<E>> = rmp_serde::from_slice(&bytes)
                    .map_err(|e| StoreError::Infrastructure(Box::new(e)))?;
                for envelope in &envelopes {
                    envelope
                        .validate()
                        .map_err(|e| StoreError::Infrastructure(Box::new(e)))?;
                }
                Ok(envelopes)
            }
            Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(Vec::new()),
            Err(e) => Err(StoreError::Infrastructure(Box::new(e))),
        }
    }

    async fn create(
        &self,
        events: Vec<E>,
        context: CorrelationContext,
    ) -> Result<(AggregateId, Vec<EventEnvelope<E>>), StoreError> {
        self.ensure_fenced().await?;

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

        let envelopes = Self::build_envelopes(id, 0, events, &context)?;
        let path = self.aggregate_path(id);
        self.write_atomic(&path, &envelopes).await?;

        Ok((id, envelopes))
    }

    async fn append(
        &self,
        id: AggregateId,
        expected_sequence: NonZeroU64,
        events: Vec<E>,
        context: CorrelationContext,
    ) -> Result<Vec<EventEnvelope<E>>, StoreError> {
        if events.is_empty() {
            return Ok(Vec::new());
        }

        self.ensure_fenced().await?;

        let lock = self.get_lock(id.get());
        let _guard = lock.lock().await;

        let path = self.aggregate_path(id);

        // Load existing events — the aggregate must have been created
        // via `create()` first. If the file does not exist, the
        // aggregate was never created and append is not valid.
        let mut existing: Vec<EventEnvelope<E>> = match tokio::fs::read(&path).await {
            Ok(bytes) => {
                let envelopes: Vec<EventEnvelope<E>> = rmp_serde::from_slice(&bytes)
                    .map_err(|e| StoreError::Infrastructure(Box::new(e)))?;
                for envelope in &envelopes {
                    envelope
                        .validate()
                        .map_err(|e| StoreError::Infrastructure(Box::new(e)))?;
                }
                envelopes
            }
            Err(e) if e.kind() == io::ErrorKind::NotFound => {
                return Err(StoreError::Infrastructure(Box::from(format!(
                    "cannot append to aggregate {id}: not created (use create() first)"
                ))));
            }
            Err(e) => return Err(StoreError::Infrastructure(Box::new(e))),
        };

        // Optimistic concurrency check.
        let actual_sequence = existing.last().map_or(0, |e| e.sequence());
        if actual_sequence != expected_sequence.get() {
            return Err(StoreError::ConcurrencyConflict {
                aggregate_id: id,
                expected_sequence: expected_sequence.get(),
                actual_sequence,
            });
        }

        // Build envelopes inside the lock so timestamps are monotonic
        // with sequence numbers.
        let new_envelopes = Self::build_envelopes(id, expected_sequence.get(), events, &context)?;

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

    /// Shorthand — most tests don't need correlation.
    fn no_ctx() -> CorrelationContext {
        CorrelationContext::none()
    }

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

    /// Helper to construct a `NonZeroU64` from a raw `u64` in tests.
    fn nz(n: u64) -> NonZeroU64 {
        NonZeroU64::new(n).unwrap()
    }

    // ── create ──────────────────────────────────────────────────────

    #[tokio::test]
    async fn create_assigns_sequential_ids() {
        let dir = tempfile::tempdir().unwrap();
        let store = MsgpackFileStore::new(dir.path());

        let (id1, _) = store
            .create(vec![TestEvent::Created { name: "a".into() }], no_ctx())
            .await
            .unwrap();
        let (id2, _) = store
            .create(vec![TestEvent::Created { name: "b".into() }], no_ctx())
            .await
            .unwrap();
        let (id3, _) = store
            .create(vec![TestEvent::Created { name: "c".into() }], no_ctx())
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
        let (id, envelopes) = store.create(events, no_ctx()).await.unwrap();

        assert_eq!(envelopes.len(), 2);
        assert_eq!(envelopes[0].aggregate_id(), id);
        assert_eq!(envelopes[1].aggregate_id(), id);
        assert_eq!(envelopes[0].sequence(), 1);
        assert_eq!(envelopes[1].sequence(), 2);
        assert_eq!(
            *envelopes[0].payload(),
            TestEvent::Created { name: "a".into() }
        );
        assert_eq!(
            *envelopes[1].payload(),
            TestEvent::Updated { name: "b".into() }
        );
        // UUID v7 — both should be non-nil and different.
        assert!(!envelopes[0].event_id().is_nil());
        assert_ne!(envelopes[0].event_id(), envelopes[1].event_id());
        // Same timestamp within the batch.
        assert_eq!(envelopes[0].timestamp(), envelopes[1].timestamp());
    }

    #[tokio::test]
    async fn create_rejects_empty_events() {
        let dir = tempfile::tempdir().unwrap();
        let store = MsgpackFileStore::<TestEvent>::new(dir.path());

        let result = store.create(vec![], no_ctx()).await;
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
                .create(vec![TestEvent::Created { name: "a".into() }], no_ctx())
                .await
                .unwrap();
            store
                .create(vec![TestEvent::Created { name: "b".into() }], no_ctx())
                .await
                .unwrap();
        }

        // Second store instance (simulates process restart) should
        // continue from 3.
        let store = MsgpackFileStore::new(dir.path());
        let (id, _) = store
            .create(vec![TestEvent::Created { name: "c".into() }], no_ctx())
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
                .create(vec![TestEvent::Created { name: "a".into() }], no_ctx())
                .await
                .unwrap();
        }

        // New store should scan, skip "old-format", find "1", assign 2.
        let store = MsgpackFileStore::new(dir.path());
        let (id, _) = store
            .create(vec![TestEvent::Created { name: "b".into() }], no_ctx())
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
            .create(vec![TestEvent::Created { name: "alice".into() }], no_ctx())
            .await
            .unwrap();

        let loaded = store.load(id).await.unwrap();
        assert_eq!(loaded.len(), 1);
        assert_eq!(*loaded[0].payload(), *created[0].payload());
        assert_eq!(loaded[0].sequence(), 1);
        assert_eq!(loaded[0].aggregate_id(), id);
    }

    // ── append ──────────────────────────────────────────────────────

    #[tokio::test]
    async fn append_and_load_roundtrip() {
        let dir = tempfile::tempdir().unwrap();
        let store = MsgpackFileStore::new(dir.path());

        let (id, _) = store
            .create(vec![TestEvent::Created { name: "alice".into() }], no_ctx())
            .await
            .unwrap();

        let appended = store
            .append(id, nz(1), vec![TestEvent::Updated { name: "bob".into() }], no_ctx())
            .await
            .unwrap();
        assert_eq!(appended.len(), 1);
        assert_eq!(appended[0].sequence(), 2);
        assert_eq!(appended[0].aggregate_id(), id);

        let loaded = store.load(id).await.unwrap();
        assert_eq!(loaded.len(), 2);
        assert_eq!(loaded[0].sequence(), 1);
        assert_eq!(loaded[1].sequence(), 2);
    }

    #[tokio::test]
    async fn append_multiple_batches() {
        let dir = tempfile::tempdir().unwrap();
        let store = MsgpackFileStore::new(dir.path());

        let (id, _) = store
            .create(vec![TestEvent::Created { name: "alice".into() }], no_ctx())
            .await
            .unwrap();

        store
            .append(id, nz(1), vec![TestEvent::Updated { name: "bob".into() }], no_ctx())
            .await
            .unwrap();
        store
            .append(id, nz(2), vec![TestEvent::Updated { name: "carol".into() }], no_ctx())
            .await
            .unwrap();

        let loaded = store.load(id).await.unwrap();
        assert_eq!(loaded.len(), 3);
        assert_eq!(loaded[0].sequence(), 1);
        assert_eq!(loaded[1].sequence(), 2);
        assert_eq!(loaded[2].sequence(), 3);
    }

    #[tokio::test]
    async fn append_empty_is_noop() {
        let dir = tempfile::tempdir().unwrap();
        let store = MsgpackFileStore::new(dir.path());

        let (id, _) = store
            .create(vec![TestEvent::Created { name: "alice".into() }], no_ctx())
            .await
            .unwrap();

        let result = store.append(id, nz(1), vec![], no_ctx()).await.unwrap();
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
            .create(vec![TestEvent::Created { name: "a".into() }], no_ctx())
            .await
            .unwrap();

        let envelopes = store
            .append(
                id,
                nz(1),
                vec![
                    TestEvent::Updated { name: "b".into() },
                    TestEvent::Updated { name: "c".into() },
                ],
                no_ctx(),
            )
            .await
            .unwrap();

        assert_eq!(envelopes.len(), 2);
        assert_eq!(envelopes[0].aggregate_id(), id);
        assert_eq!(envelopes[1].aggregate_id(), id);
        assert_eq!(envelopes[0].sequence(), 2);
        assert_eq!(envelopes[1].sequence(), 3);
        assert!(!envelopes[0].event_id().is_nil());
        assert_ne!(envelopes[0].event_id(), envelopes[1].event_id());

        // Verify they match what's on disk.
        let loaded = store.load(id).await.unwrap();
        assert_eq!(loaded.len(), 3);
        assert_eq!(*loaded[1].payload(), *envelopes[0].payload());
        assert_eq!(*loaded[2].payload(), *envelopes[1].payload());
    }

    // ── concurrency ─────────────────────────────────────────────────

    #[tokio::test]
    async fn concurrency_conflict_detected() {
        let dir = tempfile::tempdir().unwrap();
        let store = MsgpackFileStore::new(dir.path());

        let (id, _) = store
            .create(vec![TestEvent::Created { name: "alice".into() }], no_ctx())
            .await
            .unwrap();

        // First append succeeds.
        store
            .append(id, nz(1), vec![TestEvent::Updated { name: "bob".into() }], no_ctx())
            .await
            .unwrap();

        // Second append with stale expected_sequence fails.
        let result = store
            .append(id, nz(1), vec![TestEvent::Updated { name: "carol".into() }], no_ctx())
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
            .create(vec![TestEvent::Created { name: "seed".into() }], no_ctx())
            .await
            .unwrap();

        // Spawn 5 concurrent appends, all expecting sequence 1.
        let mut handles = Vec::new();
        for i in 0..5 {
            let s = Arc::clone(&store);
            handles.push(tokio::spawn(async move {
                s.append(
                    id,
                    nz(1),
                    vec![TestEvent::Updated {
                        name: format!("writer-{i}"),
                    }],
                    no_ctx(),
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
            .create(vec![TestEvent::Created { name: "alice".into() }], no_ctx())
            .await
            .unwrap();
        let (id2, _) = store
            .create(vec![TestEvent::Created { name: "bob".into() }], no_ctx())
            .await
            .unwrap();

        let loaded1 = store.load(id1).await.unwrap();
        let loaded2 = store.load(id2).await.unwrap();

        assert_eq!(loaded1.len(), 1);
        assert_eq!(loaded2.len(), 1);
        assert_eq!(
            *loaded1[0].payload(),
            TestEvent::Created { name: "alice".into() }
        );
        assert_eq!(
            *loaded2[0].payload(),
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
            .create(vec![TestEvent::Created { name: "order".into() }], no_ctx())
            .await
            .unwrap();
        assert_eq!(created.len(), 1);
        assert_eq!(created[0].sequence(), 1);

        // Append.
        let appended = store
            .append(id, nz(1), vec![TestEvent::Updated { name: "shipped".into() }], no_ctx())
            .await
            .unwrap();
        assert_eq!(appended.len(), 1);
        assert_eq!(appended[0].sequence(), 2);

        // Full history.
        let all = store.load(id).await.unwrap();
        assert_eq!(all.len(), 2);
        assert_eq!(all[0].aggregate_id(), id);
        assert_eq!(all[1].aggregate_id(), id);
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
                }], no_ctx())
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
            .create(vec![TestEvent::Created { name: "a".into() }], no_ctx())
            .await
            .unwrap();

        // Attempt to append with start_sequence near u64::MAX.
        // This should fail with an overflow error, not panic.
        let result = store
            .append(id, nz(u64::MAX), vec![TestEvent::Updated { name: "b".into() }], no_ctx())
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
        assert_eq!(*loaded[0].payload(), TestEvent::Created { name: "old".into() });
        assert!(loaded[0].correlation_id().is_none());
        assert!(loaded[0].causation_id().is_none());
    }

    #[tokio::test]
    async fn correlation_and_causation_ids_roundtrip() {
        let dir = tempfile::tempdir().unwrap();
        let store = MsgpackFileStore::new(dir.path());

        let (id, created) = store
            .create(vec![TestEvent::Created { name: "traced".into() }], no_ctx())
            .await
            .unwrap();

        // Verify initial creation has None for both fields.
        assert!(created[0].correlation_id().is_none());
        assert!(created[0].causation_id().is_none());

        // Reload and verify None survives serialization roundtrip.
        let loaded = store.load(id).await.unwrap();
        assert!(loaded[0].correlation_id().is_none());
        assert!(loaded[0].causation_id().is_none());
    }

    #[tokio::test]
    async fn correlation_and_causation_some_values_roundtrip() {
        // Verify that Some(uuid) values survive a write/load cycle
        // by manually constructing an envelope with populated fields.
        let dir = tempfile::tempdir().unwrap();
        tokio::fs::create_dir_all(dir.path()).await.unwrap();

        let corr = uuid::Uuid::now_v7();
        let cause = uuid::Uuid::now_v7();
        let envelopes = vec![EventEnvelope::new(
            uuid::Uuid::now_v7(),
            agg_id(1),
            NonZeroU64::new(1).unwrap(),
            jiff::Timestamp::now(),
            Some(corr),
            Some(cause),
            TestEvent::Created { name: "with-ids".into() },
        ).unwrap()];

        let bytes = rmp_serde::encode::to_vec_named(&envelopes).unwrap();
        tokio::fs::write(dir.path().join("1.msgpack"), &bytes)
            .await
            .unwrap();

        let store = MsgpackFileStore::<TestEvent>::new(dir.path());
        let loaded = store.load(agg_id(1)).await.unwrap();

        assert_eq!(loaded.len(), 1);
        assert_eq!(loaded[0].correlation_id(), Some(corr));
        assert_eq!(loaded[0].causation_id(), Some(cause));
    }

    #[tokio::test]
    async fn create_with_correlation_context_stamps_envelopes() {
        let dir = tempfile::tempdir().unwrap();
        let store = MsgpackFileStore::new(dir.path());

        let corr = uuid::Uuid::now_v7();
        let cause = uuid::Uuid::now_v7();
        let ctx = CorrelationContext::new(corr, cause);

        let (id, created) = store
            .create(vec![TestEvent::Created { name: "ctx".into() }], ctx)
            .await
            .unwrap();

        assert_eq!(created[0].correlation_id(), Some(corr));
        assert_eq!(created[0].causation_id(), Some(cause));

        // Verify values survive load roundtrip.
        let loaded = store.load(id).await.unwrap();
        assert_eq!(loaded[0].correlation_id(), Some(corr));
        assert_eq!(loaded[0].causation_id(), Some(cause));
    }

    #[tokio::test]
    async fn append_with_correlation_context_stamps_envelopes() {
        let dir = tempfile::tempdir().unwrap();
        let store = MsgpackFileStore::new(dir.path());

        let (id, _) = store
            .create(vec![TestEvent::Created { name: "seed".into() }], no_ctx())
            .await
            .unwrap();

        let corr = uuid::Uuid::now_v7();
        let cause = uuid::Uuid::now_v7();
        let ctx = CorrelationContext::new(corr, cause);

        let appended = store
            .append(id, nz(1), vec![TestEvent::Updated { name: "ctx".into() }], ctx)
            .await
            .unwrap();

        assert_eq!(appended[0].correlation_id(), Some(corr));
        assert_eq!(appended[0].causation_id(), Some(cause));

        // Original event should still have None.
        let loaded = store.load(id).await.unwrap();
        assert!(loaded[0].correlation_id().is_none());
        assert_eq!(loaded[1].correlation_id(), Some(corr));
    }

    #[tokio::test]
    async fn create_with_correlated_context_stamps_correlation_only() {
        let dir = tempfile::tempdir().unwrap();
        let store = MsgpackFileStore::new(dir.path());

        let corr = uuid::Uuid::now_v7();
        let ctx = CorrelationContext::correlated(corr);

        let (id, created) = store
            .create(vec![TestEvent::Created { name: "corr-only".into() }], ctx)
            .await
            .unwrap();

        assert_eq!(created[0].correlation_id(), Some(corr));
        assert!(created[0].causation_id().is_none());

        // Verify roundtrip.
        let loaded = store.load(id).await.unwrap();
        assert_eq!(loaded[0].correlation_id(), Some(corr));
        assert!(loaded[0].causation_id().is_none());
    }

    #[tokio::test]
    async fn old_format_with_zero_sequence_rejected_on_load() {
        // Serialize an envelope with sequence=0 (invalid) and verify
        // that loading it fails — NonZeroU64 serde rejects zero.
        #[derive(serde::Serialize)]
        struct BadEnvelope {
            event_id: uuid::Uuid,
            aggregate_id: AggregateId,
            sequence: u64,
            timestamp: jiff::Timestamp,
            payload: TestEvent,
        }

        let dir = tempfile::tempdir().unwrap();
        tokio::fs::create_dir_all(dir.path()).await.unwrap();

        let bad = vec![BadEnvelope {
            event_id: uuid::Uuid::now_v7(),
            aggregate_id: agg_id(1),
            sequence: 0,
            timestamp: jiff::Timestamp::now(),
            payload: TestEvent::Created { name: "zero-seq".into() },
        }];

        let bytes = rmp_serde::encode::to_vec_named(&bad).unwrap();
        tokio::fs::write(dir.path().join("1.msgpack"), &bytes)
            .await
            .unwrap();

        let store = MsgpackFileStore::<TestEvent>::new(dir.path());
        let result = store.load(agg_id(1)).await;

        assert!(result.is_err());
        assert!(
            matches!(result.unwrap_err(), StoreError::Infrastructure(_)),
            "expected Infrastructure error for zero sequence"
        );
    }

    // ── append-to-uncreated guard ───────────────────────────────────

    #[tokio::test]
    async fn append_to_uncreated_aggregate_fails() {
        let dir = tempfile::tempdir().unwrap();
        let store = MsgpackFileStore::<TestEvent>::new(dir.path());

        // Append to a never-created aggregate must fail with a
        // file-not-found guard — callers must use create() first.
        // The sequence value is irrelevant; the guard fires before
        // the concurrency check.
        let result = store
            .append(
                agg_id(999),
                nz(1),
                vec![TestEvent::Created {
                    name: "sneaky".into(),
                }],
                no_ctx(),
            )
            .await;

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(
            matches!(err, StoreError::Infrastructure(_)),
            "expected Infrastructure error, got: {err}"
        );
        let msg = err.to_string();
        assert!(
            msg.contains("not created"),
            "error message should mention 'not created', got: {msg}"
        );
    }

    // ── additional coverage ─────────────────────────────────────────

    #[tokio::test]
    async fn create_does_not_overwrite_existing_file() {
        // Manually place a file at the path that create() would
        // assign. Verify the store skips that ID and does not
        // silently overwrite the existing file.
        let dir = tempfile::tempdir().unwrap();
        tokio::fs::create_dir_all(dir.path()).await.unwrap();

        // Plant a sentinel file at 1.msgpack.
        let sentinel = b"sentinel data";
        tokio::fs::write(dir.path().join("1.msgpack"), sentinel)
            .await
            .unwrap();

        let store = MsgpackFileStore::<TestEvent>::new(dir.path());

        // create() should scan and discover 1.msgpack, then assign ID 2.
        let (id, _) = store
            .create(vec![TestEvent::Created {
                name: "safe".into(),
            }], no_ctx())
            .await
            .unwrap();

        assert_eq!(id.get(), 2, "should skip the occupied ID");

        // Verify the sentinel file is untouched.
        let data = tokio::fs::read(dir.path().join("1.msgpack"))
            .await
            .unwrap();
        assert_eq!(data, sentinel, "existing file must not be overwritten");
    }

    #[tokio::test]
    async fn scan_max_id_with_gaps() {
        // Files 1.msgpack and 5.msgpack exist (gap at 2-4).
        // After restart, next ID should be 6.
        let dir = tempfile::tempdir().unwrap();
        let store = MsgpackFileStore::<TestEvent>::new(dir.path());

        // Create IDs 1–5 by writing files directly.
        tokio::fs::create_dir_all(dir.path()).await.unwrap();
        for id_val in [1u64, 5] {
            let id = agg_id(id_val);
            let envelopes = vec![EventEnvelope::new(
                uuid::Uuid::now_v7(),
                id,
                NonZeroU64::new(1).unwrap(),
                jiff::Timestamp::now(),
                None,
                None,
                TestEvent::Created {
                    name: format!("agg-{id_val}"),
                },
            ).unwrap()];
            let bytes = rmp_serde::encode::to_vec_named(&envelopes).unwrap();
            tokio::fs::write(
                dir.path().join(format!("{id_val}.msgpack")),
                &bytes,
            )
            .await
            .unwrap();
        }

        // Simulate restart — new store instance.
        let store2 = MsgpackFileStore::<TestEvent>::new(dir.path());

        let (id, _) = store2
            .create(vec![TestEvent::Created {
                name: "after-gap".into(),
            }], no_ctx())
            .await
            .unwrap();

        assert_eq!(id.get(), 6, "next ID should be max(1,5)+1 = 6");
        drop(store);
    }

    #[tokio::test]
    async fn concurrent_create_and_append() {
        // Interleave create and append operations concurrently.
        // Verify no data corruption.
        let dir = tempfile::tempdir().unwrap();
        let store = Arc::new(MsgpackFileStore::new(dir.path()));

        // Seed an aggregate so append has a target.
        let (seed_id, _) = store
            .create(vec![TestEvent::Created {
                name: "seed".into(),
            }], no_ctx())
            .await
            .unwrap();

        let mut handles = Vec::new();

        // 5 concurrent creates.
        for i in 0..5 {
            let s = Arc::clone(&store);
            handles.push(tokio::spawn(async move {
                s.create(vec![TestEvent::Created {
                    name: format!("new-{i}"),
                }], no_ctx())
                .await
                .map(|r| ("create", r.0))
            }));
        }

        // 5 concurrent appends to the seed aggregate — at most one
        // can succeed (all use expected_sequence 1).
        for i in 0..5 {
            let s = Arc::clone(&store);
            handles.push(tokio::spawn(async move {
                s.append(
                    seed_id,
                    nz(1),
                    vec![TestEvent::Updated {
                        name: format!("upd-{i}"),
                    }],
                    no_ctx(),
                )
                .await
                .map(|_| ("append", seed_id))
            }));
        }

        let results: Vec<_> = futures_util::future::join_all(handles)
            .await
            .into_iter()
            .map(|r| r.unwrap())
            .collect();

        // All 5 creates must succeed.
        let creates: Vec<_> = results.iter().filter(|r| r.as_ref().ok().is_some_and(|v| v.0 == "create")).collect();
        assert_eq!(creates.len(), 5, "all creates should succeed");

        // Exactly 1 append succeeds, 4 get ConcurrencyConflict.
        let append_ok = results
            .iter()
            .filter(|r| r.as_ref().ok().is_some_and(|v| v.0 == "append"))
            .count();
        let append_err = results
            .iter()
            .filter(|r| r.is_err())
            .count();
        assert_eq!(
            append_ok, 1,
            "exactly one append should win"
        );
        assert_eq!(
            append_err, 4,
            "four appends should get ConcurrencyConflict"
        );

        // All created aggregates should have unique IDs.
        let mut created_ids: Vec<u64> = creates
            .iter()
            .map(|r| r.as_ref().unwrap().1.get())
            .collect();
        created_ids.sort();
        created_ids.dedup();
        assert_eq!(created_ids.len(), 5, "all created IDs must be unique");
    }

    #[tokio::test]
    async fn temp_file_cleaned_up_after_successful_write() {
        // After a successful write, no .tmp file should remain.
        let dir = tempfile::tempdir().unwrap();
        let store = MsgpackFileStore::new(dir.path());

        let (id, _) = store
            .create(vec![TestEvent::Created {
                name: "clean".into(),
            }], no_ctx())
            .await
            .unwrap();

        // Check that no .tmp files remain.
        let mut entries = tokio::fs::read_dir(dir.path()).await.unwrap();
        while let Some(entry) = entries.next_entry().await.unwrap() {
            let name = entry.file_name();
            let name_str = name.to_string_lossy();
            assert!(
                !name_str.ends_with(".tmp"),
                "temp file should be cleaned up: {name_str}"
            );
        }

        // The actual file should exist.
        let path = dir.path().join(format!("{}.msgpack", id.get()));
        assert!(path.exists(), "aggregate file should exist");
    }

    // ── fencing ──────────────────────────────────────────────────────

    #[tokio::test]
    async fn single_store_acquires_lock_successfully() {
        let dir = tempfile::tempdir().unwrap();
        let store = MsgpackFileStore::new(dir.path());

        // First create triggers ensure_fenced — should succeed.
        let result = store
            .create(vec![TestEvent::Created { name: "fenced".into() }], no_ctx())
            .await;
        assert!(result.is_ok(), "first store should acquire lock");

        // .lock file should exist.
        assert!(
            dir.path().join(".lock").exists(),
            ".lock sentinel file should be created"
        );
    }

    #[tokio::test]
    async fn second_store_same_dir_fails_with_store_locked() {
        let dir = tempfile::tempdir().unwrap();

        // First store acquires the lock.
        let store1 = MsgpackFileStore::new(dir.path());
        store1
            .create(vec![TestEvent::Created { name: "first".into() }], no_ctx())
            .await
            .unwrap();

        // Second store on the same directory should fail.
        let store2 = MsgpackFileStore::<TestEvent>::new(dir.path());
        let result = store2
            .create(vec![TestEvent::Created { name: "second".into() }], no_ctx())
            .await;

        assert!(
            matches!(result, Err(StoreError::StoreLocked { .. })),
            "second store should get StoreLocked, got: {result:?}"
        );
    }

    #[tokio::test]
    async fn lock_released_on_drop_allows_reacquisition() {
        let dir = tempfile::tempdir().unwrap();

        // First store acquires and then drops the lock.
        {
            let store = MsgpackFileStore::new(dir.path());
            store
                .create(vec![TestEvent::Created { name: "first".into() }], no_ctx())
                .await
                .unwrap();
        }
        // store is dropped here — lock released.

        // New store should acquire the lock successfully.
        let store2 = MsgpackFileStore::<TestEvent>::new(dir.path());
        let result = store2
            .append(
                agg_id(1),
                nz(1),
                vec![TestEvent::Updated { name: "after-drop".into() }],
                no_ctx(),
            )
            .await;
        assert!(result.is_ok(), "should reacquire lock after drop");
    }

    #[tokio::test]
    async fn concurrent_ensure_fenced_does_not_deadlock() {
        // Two concurrent create() calls on a fresh store both trigger
        // ensure_fenced. OnceCell serializes them — one wins, the
        // other waits and reuses. Neither should deadlock or error.
        let dir = tempfile::tempdir().unwrap();
        let store = Arc::new(MsgpackFileStore::new(dir.path()));

        let mut handles = Vec::new();
        for i in 0..5 {
            let s = Arc::clone(&store);
            handles.push(tokio::spawn(async move {
                s.create(vec![TestEvent::Created {
                    name: format!("concurrent-{i}"),
                }], no_ctx())
                .await
            }));
        }

        let results: Vec<_> = futures_util::future::join_all(handles)
            .await
            .into_iter()
            .map(|r| r.unwrap())
            .collect();

        // All should succeed — same process, same OnceCell.
        assert!(
            results.iter().all(|r| r.is_ok()),
            "all concurrent creates should succeed within same store"
        );
    }

    // ── proptest ────────────────────────────────────────────────────

    mod proptests {
        use super::*;
        use proptest::prelude::*;

        proptest! {
            #[test]
            fn build_envelopes_sequences_are_monotonic(
                count in 1..20usize,
                start in 0..u64::MAX - 20,
            ) {
                let id = agg_id(1);
                let events: Vec<TestEvent> = (0..count)
                    .map(|i| TestEvent::Created { name: format!("e{i}") })
                    .collect();
                let ctx = no_ctx();

                let envelopes = MsgpackFileStore::build_envelopes(id, start, events, &ctx)
                    .unwrap();

                prop_assert_eq!(envelopes.len(), count);

                // Sequences must be strictly monotonically increasing.
                for window in envelopes.windows(2) {
                    prop_assert!(
                        window[1].sequence() > window[0].sequence(),
                        "sequence not monotonically increasing: {} <= {}",
                        window[1].sequence(),
                        window[0].sequence()
                    );
                }

                // First sequence = start + 1.
                prop_assert_eq!(envelopes[0].sequence(), start + 1);
                // Last sequence = start + count.
                prop_assert_eq!(
                    envelopes.last().unwrap().sequence(),
                    start + count as u64
                );
            }
        }
    }
}
