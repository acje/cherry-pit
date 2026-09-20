//! Run lock management to prevent concurrent collection runs.
//!
//! A lock file is written to the working directory before a collection run
//! starts. The lock contains metadata (run ID, PID, hostname, creation time)
//! so that stale locks left by crashed processes can be identified and
//! reclaimed.
//!
//! **Stale lock policy:** A lock is considered stale if its holder is dead on
//! the current host, or if its `created_at` timestamp exceeds the configured
//! TTL. The default TTL is 15 minutes.
//! Manual recovery is available via `--force-unlock`.
//!
//! **Lock ownership:** The lock file is a stable coordination inode that is
//! never unlinked. An OS-backed advisory lock (`File::try_lock`) is taken on
//! that inode only for the duration of a single metadata transaction —
//! acquire, reclaim, renew or release — never for the lifetime of the guard.
//! Each successful acquisition records a fresh unique generation inside the
//! coordination record; renew, release and drop compare that generation and
//! mutate under the same short guard. A TTL-expired, forced or dead holder
//! can therefore still be taken over while alive, and the displaced holder's
//! renew reports lost ownership while its release and drop become no-ops.
//! Every guard is non-blocking: contention yields an error immediately, and
//! drop never waits.
//!
//! The coordination inode is opened without following symlinks and is
//! rejected unless it is a regular file with a single link. This fences the
//! crate's own lock metadata transactions for cooperating clients in a
//! trusted lock directory on a local filesystem supporting advisory locking.
//! It makes no claim against arbitrary external unlinking, against an
//! uncooperative writer, or over application writes performed by a stale
//! holder.

use std::path::{Path, PathBuf};
use std::time::Duration;

use jiff::{SignedDuration, Timestamp};
use serde::{Deserialize, Serialize};

use tracing::{info, warn};

use crate::error::PersistenceError;

/// Default stale-lock TTL: 15 minutes.
pub const DEFAULT_LOCK_TTL: Duration = Duration::from_mins(15);

/// Default lock file name.
pub const DEFAULT_LOCK_FILENAME: &str = "collector.lock";

/// Metadata stored inside the lock file.
///
/// Serde DTO; forward/backward schema evolution is handled by serde's
/// field-presence semantics + `#[serde(default)]` rather than
/// `#[non_exhaustive]`. CHE-0021's `#[non_exhaustive]` rule is scoped
/// to public error types in cherry-pit-core, not infrastructure DTOs.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LockMetadata {
    /// Run ID that holds the lock.
    pub run_id: String,
    /// Process ID of the lock holder (diagnostic only).
    pub pid: u32,
    /// Hostname of the lock holder.
    #[serde(default)]
    pub hostname: String,
    /// When the lock was created (UTC).
    pub created_at: Timestamp,
}

impl LockMetadata {
    /// Create lock metadata for the current process.
    #[must_use]
    pub fn current(run_id: &str) -> Self {
        Self::current_with_clock(run_id, Timestamp::now)
    }

    fn current_with_clock(run_id: &str, now: impl Fn() -> Timestamp) -> Self {
        Self {
            run_id: run_id.to_string(),
            pid: std::process::id(),
            hostname: current_hostname(),
            created_at: now(),
        }
    }
}

fn current_hostname() -> String {
    gethostname::gethostname().to_string_lossy().into_owned()
}

/// RAII guard over an owned run lock.
///
/// Ownership is recorded as a unique generation inside the coordination
/// inode. Dropping the guard clears that record only while the generation
/// still matches, so a holder displaced by a TTL, dead-holder or forced
/// takeover cannot disturb its successor.
///
/// Fields are private; construction is via [`acquire`] only.
/// `#[non_exhaustive]` is not needed (CHE-0021 scope is error types
/// in cherry-pit-core, and the private fields already prevent
/// external literal construction).
#[derive(Debug)]
pub struct RunLock {
    path: PathBuf,
    metadata: LockMetadata,
    generation: Generation,
    file: std::fs::File,
    released: bool,
}

impl RunLock {
    /// The lock metadata.
    #[must_use]
    pub fn metadata(&self) -> &LockMetadata {
        &self.metadata
    }

    /// The path to the lock file.
    #[must_use]
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Explicitly release the lock.
    ///
    /// Clears the coordination record while this guard's generation is still
    /// the recorded one, then drops the short advisory guard. The
    /// coordination inode is never unlinked, and a release by a displaced
    /// holder is a no-op. Calling this explicitly disarms the drop path.
    ///
    /// # Errors
    ///
    /// Returns [`PersistenceError::LockFailed`] if the coordination guard is
    /// contended, or [`PersistenceError::Io`] if the record cannot be read or
    /// cleared.
    pub fn release(mut self) -> Result<(), PersistenceError> {
        let result = self.relinquish();
        self.released = true;
        result
    }

    /// Refresh the lock's `created_at` to now, so a long-running holder
    /// is not mistaken for a stale lock and reclaimed by another
    /// process. Callers (typically long-running daemons) invoke this
    /// well inside the configured stale-lock TTL.
    ///
    /// Rewrites the coordination record in place under a short advisory
    /// guard, after confirming this guard still owns the recorded
    /// generation. The in-memory `metadata` is updated only on success.
    ///
    /// # Errors
    ///
    /// Returns [`PersistenceError::LockFailed`] if metadata serialization
    /// fails, if the coordination guard is contended, or if ownership has
    /// been taken over by a later holder; otherwise propagates the
    /// underlying I/O failure as [`PersistenceError::Io`].
    pub fn renew(&mut self) -> Result<(), PersistenceError> {
        self.renew_with_clock(Timestamp::now)
    }

    fn renew_with_clock(&mut self, now: impl Fn() -> Timestamp) -> Result<(), PersistenceError> {
        let refreshed = LockMetadata {
            run_id: self.metadata.run_id.clone(),
            pid: self.metadata.pid,
            hostname: self.metadata.hostname.clone(),
            created_at: now(),
        };
        let guard = Guard::acquire(&self.file, &self.path)?;
        match read_state(&self.file)? {
            LockState::Owned(_, Some(recorded)) if recorded == self.generation => {}
            _ => {
                return Err(PersistenceError::LockFailed {
                    reason: format!(
                        "lock ownership lost: run {} no longer holds {}",
                        self.metadata.run_id,
                        self.path.display()
                    ),
                });
            }
        }
        write_record(&self.file, &refreshed, self.generation)?;
        drop(guard);
        self.metadata = refreshed;
        Ok(())
    }

    fn relinquish(&mut self) -> Result<(), PersistenceError> {
        let guard = Guard::acquire(&self.file, &self.path)?;
        let owned_by_us = matches!(
            read_state(&self.file)?,
            LockState::Owned(_, Some(recorded)) if recorded == self.generation
        );
        let result = if owned_by_us {
            clear_record(&self.file)
        } else {
            Ok(())
        };
        drop(guard);
        result
    }
}

impl Drop for RunLock {
    fn drop(&mut self) {
        if !self.released
            && let Err(e) = self.relinquish()
        {
            warn!(
                path = %self.path.display(),
                error = %e,
                "failed to release lock file on drop"
            );
        }
    }
}

type Generation = uuid::Uuid;

fn new_generation() -> Generation {
    uuid::Uuid::now_v7()
}

struct Guard<'a> {
    file: &'a std::fs::File,
}

impl<'a> Guard<'a> {
    fn acquire(file: &'a std::fs::File, path: &Path) -> Result<Self, PersistenceError> {
        match file.try_lock() {
            Ok(()) => Ok(Self { file }),
            Err(std::fs::TryLockError::WouldBlock) => Err(PersistenceError::LockFailed {
                reason: format!("lock coordination is contended: {}", path.display()),
            }),
            Err(std::fs::TryLockError::Error(e)) => Err(PersistenceError::Io(e)),
        }
    }
}

impl Drop for Guard<'_> {
    fn drop(&mut self) {
        if let Err(e) = self.file.unlock() {
            warn!(error = %e, "failed to drop lock coordination guard");
        }
    }
}

fn open_coordination(lock_path: &Path) -> Result<std::fs::File, PersistenceError> {
    let mut options = std::fs::OpenOptions::new();
    options.read(true).write(true).create(true).truncate(false);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        let flags = rustix::fs::OFlags::NOFOLLOW | rustix::fs::OFlags::NONBLOCK;
        options.custom_flags(i32::try_from(flags.bits()).unwrap_or(0));
    }
    let file = options.open(lock_path).map_err(PersistenceError::Io)?;
    let meta = file.metadata().map_err(PersistenceError::Io)?;
    if !meta.file_type().is_file() {
        return Err(PersistenceError::LockFailed {
            reason: format!("lock path is not a regular file: {}", lock_path.display()),
        });
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        if meta.nlink() != 1 {
            return Err(PersistenceError::LockFailed {
                reason: format!(
                    "lock path has {} links; a coordination inode must be unshared: {}",
                    meta.nlink(),
                    lock_path.display()
                ),
            });
        }
    }
    Ok(file)
}

fn describe_live_holder(file: &std::fs::File, lock_path: &Path) -> String {
    match read_state(file) {
        Ok(LockState::Owned(existing, _)) => format!(
            "lock held by run {} (pid {}, since {})",
            existing.run_id, existing.pid, existing.created_at,
        ),
        _ => format!("lock held by a live process ({})", lock_path.display()),
    }
}

/// Attempt to acquire a run lock.
///
/// If a lock record already exists, checks whether it is stale using
/// `stale_ttl`. A lock is stale when its `created_at` timestamp plus
/// `stale_ttl` is in the past, or when its holder is dead on this host.
///
/// When `force` is true, any existing record is taken over regardless of
/// stale/alive status. The previous holder's details are logged at `warn`
/// level. A TTL-expired or forced takeover succeeds even against a *live*
/// holder: the displaced holder loses its recorded generation, so its later
/// renew fails and its release and drop become no-ops.
///
/// The coordination inode is validated and never unlinked; metadata
/// transactions run under a short non-blocking advisory guard (CHE-0051:R6).
///
/// # Errors
///
/// Returns `PersistenceError::LockFailed` if a live, non-stale holder owns
/// the record, if the coordination guard is contended, or if the lock path
/// is not a valid coordination inode.
pub fn acquire(
    lock_dir: &Path,
    run_id: &str,
    stale_ttl: Duration,
    force: bool,
    lock_filename: &str,
) -> Result<RunLock, PersistenceError> {
    acquire_with_clock(
        lock_dir,
        run_id,
        stale_ttl,
        force,
        lock_filename,
        Timestamp::now,
    )
}

fn acquire_with_clock(
    lock_dir: &Path,
    run_id: &str,
    stale_ttl: Duration,
    force: bool,
    lock_filename: &str,
    now: impl Fn() -> Timestamp,
) -> Result<RunLock, PersistenceError> {
    let lock_dir = crate::fs::dir_or_current(lock_dir);
    let lock_path = lock_dir.join(lock_filename);

    std::fs::create_dir_all(lock_dir).map_err(PersistenceError::Io)?;

    let file = open_coordination(&lock_path)?;
    let guard = Guard::acquire(&file, &lock_path)?;

    match read_state(&file)? {
        LockState::Unowned => {}
        LockState::Invalid(reason) => {
            warn!(
                path = %lock_path.display(),
                reason = %reason,
                "replacing corrupt lock file contents"
            );
        }
        LockState::Owned(existing, _) => {
            if force {
                warn!(
                    run_id = %existing.run_id,
                    pid = existing.pid,
                    created_at = %existing.created_at,
                    "force-removing existing lock"
                );
            } else if let Some(reclaim_reason) = reclaim_reason(&existing, stale_ttl, &now) {
                warn!(
                    run_id = %existing.run_id,
                    pid = existing.pid,
                    host = %existing.hostname,
                    created_at = %existing.created_at,
                    reason = reclaim_reason.as_str(),
                    "reclaiming stale lock"
                );
            } else {
                return Err(PersistenceError::LockFailed {
                    reason: describe_live_holder(&file, &lock_path),
                });
            }
        }
    }

    let metadata = LockMetadata::current_with_clock(run_id, &now);
    let generation = new_generation();
    write_record(&file, &metadata, generation)?;
    drop(guard);

    info!(
        run_id = %metadata.run_id,
        pid = metadata.pid,
        "lock acquired"
    );

    Ok(RunLock {
        path: lock_path,
        metadata,
        generation,
        file,
        released: false,
    })
}

#[cfg(test)]
fn is_stale(meta: &LockMetadata, ttl: Duration) -> bool {
    reclaim_reason(meta, ttl, Timestamp::now).is_some()
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
enum ReclaimReason {
    TtlExpired,
    DeadHolderSameHost,
}

impl ReclaimReason {
    fn as_str(self) -> &'static str {
        match self {
            Self::TtlExpired => "ttl-expired",
            Self::DeadHolderSameHost => "dead-holder-same-host",
        }
    }
}

fn reclaim_reason(
    meta: &LockMetadata,
    ttl: Duration,
    now: impl Fn() -> Timestamp,
) -> Option<ReclaimReason> {
    if same_host_dead_holder(meta) {
        return Some(ReclaimReason::DeadHolderSameHost);
    }
    if ttl_expired(meta, ttl, now) {
        return Some(ReclaimReason::TtlExpired);
    }
    None
}

fn same_host_dead_holder(meta: &LockMetadata) -> bool {
    !meta.hostname.is_empty() && meta.hostname == current_hostname() && !pid_is_alive(meta.pid)
}

fn ttl_expired(meta: &LockMetadata, ttl: Duration, now: impl Fn() -> Timestamp) -> bool {
    let ttl_jiff = SignedDuration::try_from(ttl).unwrap_or_else(|_| {
        SignedDuration::try_from(DEFAULT_LOCK_TTL).unwrap_or(SignedDuration::from_mins(15))
    });
    now().duration_since(meta.created_at) > ttl_jiff
}

#[cfg(unix)]
fn pid_is_alive(pid: u32) -> bool {
    let Ok(raw_pid) = i32::try_from(pid) else {
        return true;
    };
    let Some(pid) = rustix::process::Pid::from_raw(raw_pid) else {
        return true;
    };
    !matches!(
        rustix::process::test_kill_process(pid),
        Err(err) if err == rustix::io::Errno::SRCH
    )
}

#[cfg(not(unix))]
fn pid_is_alive(_pid: u32) -> bool {
    true
}
#[derive(Serialize, Deserialize)]
struct LockRecord {
    #[serde(flatten)]
    metadata: LockMetadata,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    generation: Option<Generation>,
}

fn write_record(
    file: &std::fs::File,
    metadata: &LockMetadata,
    generation: Generation,
) -> Result<(), PersistenceError> {
    use std::io::{Seek, SeekFrom, Write};

    let record = LockRecord {
        metadata: metadata.clone(),
        generation: Some(generation),
    };
    let json = serde_json::to_string_pretty(&record).map_err(|e| PersistenceError::LockFailed {
        reason: format!("failed to serialize lock metadata: {e}"),
    })?;
    let mut handle = file;
    handle.set_len(0).map_err(PersistenceError::Io)?;
    handle
        .seek(SeekFrom::Start(0))
        .map_err(PersistenceError::Io)?;
    handle
        .write_all(json.as_bytes())
        .map_err(PersistenceError::Io)?;
    handle.sync_all().map_err(PersistenceError::Io)
}

fn clear_record(file: &std::fs::File) -> Result<(), PersistenceError> {
    file.set_len(0).map_err(PersistenceError::Io)?;
    file.sync_all().map_err(PersistenceError::Io)
}

#[derive(Debug)]
enum LockState {
    Unowned,
    Owned(LockMetadata, Option<Generation>),
    Invalid(String),
}

fn read_state(file: &std::fs::File) -> Result<LockState, PersistenceError> {
    use std::io::{Read, Seek, SeekFrom};

    let mut handle = file;
    handle
        .seek(SeekFrom::Start(0))
        .map_err(PersistenceError::Io)?;
    let mut buf = bounded_buffer();
    handle
        .take(MAX_LOCK_FILE_BYTES + 1)
        .read_to_end(&mut buf)
        .map_err(PersistenceError::Io)?;
    parse_state(&buf)
}

fn bounded_buffer() -> Vec<u8> {
    Vec::with_capacity(usize::try_from(MAX_LOCK_FILE_BYTES + 1).unwrap_or(usize::MAX))
}

fn parse_state(buf: &[u8]) -> Result<LockState, PersistenceError> {
    if u64::try_from(buf.len()).unwrap_or(u64::MAX) > MAX_LOCK_FILE_BYTES {
        return Err(PersistenceError::LockFailed {
            reason: format!("lock file too large: exceeds {MAX_LOCK_FILE_BYTES} bytes"),
        });
    }
    if buf.iter().all(u8::is_ascii_whitespace) {
        return Ok(LockState::Unowned);
    }
    match serde_json::from_slice::<LockRecord>(buf) {
        Ok(record) => Ok(LockState::Owned(record.metadata, record.generation)),
        Err(e) => Ok(LockState::Invalid(format!("lock file is corrupt: {e}"))),
    }
}

#[cfg(test)]
fn write_lock(path: &Path, metadata: &LockMetadata) -> Result<(), PersistenceError> {
    let json =
        serde_json::to_string_pretty(metadata).map_err(|e| PersistenceError::LockFailed {
            reason: format!("failed to serialize lock metadata: {e}"),
        })?;
    crate::fs::atomic_write_text(path, &json).map_err(|e| PersistenceError::LockFailed {
        reason: format!("failed to write lock file: {e}"),
    })
}

const MAX_LOCK_FILE_BYTES: u64 = 1_048_576;

#[cfg(test)]
fn read_lock(path: &Path) -> Result<LockMetadata, PersistenceError> {
    use std::io::Read;

    let file = open_coordination(path)?;
    let mut buf = bounded_buffer();
    (&file)
        .take(MAX_LOCK_FILE_BYTES + 1)
        .read_to_end(&mut buf)
        .map_err(PersistenceError::Io)?;
    match parse_state(&buf)? {
        LockState::Owned(metadata, _) => Ok(metadata),
        LockState::Unowned => Err(PersistenceError::LockFailed {
            reason: "lock file is unowned".to_string(),
        }),
        LockState::Invalid(reason) => Err(PersistenceError::LockFailed { reason }),
    }
}

#[must_use]
pub fn lock_path(lock_dir: &Path, lock_filename: &str) -> PathBuf {
    lock_dir.join(lock_filename)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn lock_metadata_current_populates_fields() {
        let meta = LockMetadata::current("test-run-123");
        assert_eq!(meta.run_id, "test-run-123");
        assert_eq!(meta.pid, std::process::id());
        assert!(!meta.hostname.is_empty());
    }

    #[test]
    fn acquire_creates_lock_file() {
        let dir = TempDir::new().unwrap();
        let lock = acquire(
            dir.path(),
            "run-1",
            DEFAULT_LOCK_TTL,
            false,
            DEFAULT_LOCK_FILENAME,
        )
        .unwrap();

        assert!(lock.path().exists());
        assert_eq!(lock.metadata().run_id, "run-1");

        let meta = read_lock(lock.path()).unwrap();
        assert_eq!(meta.run_id, "run-1");
        assert_eq!(meta.pid, std::process::id());
    }

    #[test]
    fn acquire_fails_when_lock_held() {
        let dir = TempDir::new().unwrap();
        let _lock = acquire(
            dir.path(),
            "run-1",
            DEFAULT_LOCK_TTL,
            false,
            DEFAULT_LOCK_FILENAME,
        )
        .unwrap();

        let result = acquire(
            dir.path(),
            "run-2",
            DEFAULT_LOCK_TTL,
            false,
            DEFAULT_LOCK_FILENAME,
        );
        assert!(result.is_err());
        let err = result.unwrap_err();
        let msg = format!("{err}");
        assert!(msg.contains("collection lock failed:"), "got: {msg}");
    }

    #[test]
    fn lock_released_on_drop() {
        let dir = TempDir::new().unwrap();
        let lock_file_path;
        {
            let lock = acquire(
                dir.path(),
                "run-1",
                DEFAULT_LOCK_TTL,
                false,
                DEFAULT_LOCK_FILENAME,
            )
            .unwrap();
            lock_file_path = lock.path().to_path_buf();
            assert!(lock_file_path.exists());
        }
        assert!(
            lock_file_path.exists(),
            "coordination inode is never unlinked (H3 fencing)"
        );
        assert_eq!(
            std::fs::metadata(&lock_file_path).unwrap().len(),
            0,
            "drop must clear ownership"
        );
        assert!(
            acquire(
                dir.path(),
                "run-2",
                DEFAULT_LOCK_TTL,
                false,
                DEFAULT_LOCK_FILENAME
            )
            .is_ok(),
            "dropped lock must be immediately re-acquirable"
        );
    }

    #[test]
    fn lock_released_explicitly() {
        let dir = TempDir::new().unwrap();
        let lock = acquire(
            dir.path(),
            "run-1",
            DEFAULT_LOCK_TTL,
            false,
            DEFAULT_LOCK_FILENAME,
        )
        .unwrap();
        let path = lock.path().to_path_buf();
        assert!(path.exists());

        lock.release().unwrap();
        assert!(
            path.exists(),
            "coordination inode is never unlinked (H3 fencing)"
        );
        assert!(
            acquire(
                dir.path(),
                "run-2",
                DEFAULT_LOCK_TTL,
                false,
                DEFAULT_LOCK_FILENAME
            )
            .is_ok(),
            "released lock must be immediately re-acquirable"
        );
    }

    #[test]
    fn stale_lock_is_reclaimed() {
        let dir = TempDir::new().unwrap();

        let stale_meta = LockMetadata {
            run_id: "old-run".to_string(),
            pid: 999_999_999,
            hostname: LockMetadata::current("host-probe").hostname,
            created_at: Timestamp::now() - SignedDuration::from_hours(5),
        };
        let lock_file = dir.path().join(DEFAULT_LOCK_FILENAME);
        write_lock(&lock_file, &stale_meta).unwrap();

        let lock = acquire(
            dir.path(),
            "new-run",
            DEFAULT_LOCK_TTL,
            false,
            DEFAULT_LOCK_FILENAME,
        )
        .unwrap();
        assert_eq!(lock.metadata().run_id, "new-run");
    }

    #[test]
    fn fresh_lock_is_not_reclaimed() {
        let dir = TempDir::new().unwrap();

        let meta = LockMetadata {
            run_id: "active-run".to_string(),
            pid: std::process::id(),
            hostname: LockMetadata::current("host-probe").hostname,
            created_at: Timestamp::now(),
        };
        let lock_file = dir.path().join(DEFAULT_LOCK_FILENAME);
        write_lock(&lock_file, &meta).unwrap();

        let result = acquire(
            dir.path(),
            "new-run",
            DEFAULT_LOCK_TTL,
            false,
            DEFAULT_LOCK_FILENAME,
        );
        assert!(result.is_err());
    }

    #[test]
    fn corrupt_lock_file_is_replaced() {
        let dir = TempDir::new().unwrap();
        let lock_file = dir.path().join(DEFAULT_LOCK_FILENAME);
        std::fs::write(&lock_file, "not-json").unwrap();

        let lock = acquire(
            dir.path(),
            "new-run",
            DEFAULT_LOCK_TTL,
            false,
            DEFAULT_LOCK_FILENAME,
        )
        .unwrap();
        assert_eq!(lock.metadata().run_id, "new-run");
    }

    #[test]
    fn acquire_after_release_succeeds() {
        let dir = TempDir::new().unwrap();
        let lock = acquire(
            dir.path(),
            "run-1",
            DEFAULT_LOCK_TTL,
            false,
            DEFAULT_LOCK_FILENAME,
        )
        .unwrap();
        lock.release().unwrap();

        let lock2 = acquire(
            dir.path(),
            "run-2",
            DEFAULT_LOCK_TTL,
            false,
            DEFAULT_LOCK_FILENAME,
        )
        .unwrap();
        assert_eq!(lock2.metadata().run_id, "run-2");
    }

    #[test]
    fn lock_path_returns_expected() {
        let path = lock_path(Path::new("/tmp/work"), DEFAULT_LOCK_FILENAME);
        assert_eq!(path, PathBuf::from("/tmp/work/collector.lock"));
    }

    #[test]
    fn lock_path_custom_filename() {
        let path = lock_path(Path::new("/tmp/work"), "my-service.lock");
        assert_eq!(path, PathBuf::from("/tmp/work/my-service.lock"));
    }

    #[test]
    fn acquire_with_custom_filename() {
        let dir = TempDir::new().unwrap();
        let lock = acquire(dir.path(), "run-1", DEFAULT_LOCK_TTL, false, "custom.lock").unwrap();
        assert!(lock.path().ends_with("custom.lock"));
    }

    #[test]
    fn is_stale_fresh_lock_is_not_stale() {
        let meta = LockMetadata {
            run_id: "run".to_string(),
            pid: std::process::id(),
            hostname: LockMetadata::current("host-probe").hostname,
            created_at: Timestamp::now(),
        };
        assert!(!is_stale(&meta, DEFAULT_LOCK_TTL));
    }

    #[test]
    fn is_stale_old_lock_is_stale() {
        let meta = LockMetadata {
            run_id: "run".to_string(),
            pid: 999_999_999,
            hostname: LockMetadata::current("host-probe").hostname,
            created_at: Timestamp::now() - SignedDuration::from_hours(5),
        };
        assert!(is_stale(&meta, DEFAULT_LOCK_TTL));
    }

    #[test]
    fn cross_host_lock_at_default_ttl_boundary_is_not_stale() {
        let meta = LockMetadata {
            run_id: "run".to_string(),
            pid: 1,
            hostname: "foreign-host.example.com".to_string(),
            created_at: Timestamp::now() - SignedDuration::from_mins(14),
        };
        assert!(!is_stale(&meta, DEFAULT_LOCK_TTL));
    }

    #[test]
    fn default_ttl_is_fifteen_minutes() {
        assert_eq!(DEFAULT_LOCK_TTL, Duration::from_mins(15));
    }

    #[test]
    fn is_stale_custom_short_ttl() {
        let meta = LockMetadata {
            run_id: "run".to_string(),
            pid: 1,
            hostname: LockMetadata::current("host-probe").hostname,
            created_at: Timestamp::now() - SignedDuration::from_secs(61),
        };
        assert!(is_stale(&meta, Duration::from_mins(1)));
    }

    #[test]
    fn concurrent_acquire_exactly_one_wins() {
        use std::sync::{Arc, Barrier};
        use std::thread;

        let dir = TempDir::new().unwrap();
        let dir_path = dir.path().to_path_buf();
        let num_threads = 10;
        let barrier = Arc::new(Barrier::new(num_threads));

        #[expect(
            clippy::needless_collect,
            reason = "all ten workers must be spawned before the first join so Barrier::new(10) can release; lazy fusion would join worker 0 before worker 1 is spawned and deadlock"
        )]
        let handles: Vec<_> = (0..num_threads)
            .map(|i| {
                let dir = dir_path.clone();
                let barrier = Arc::clone(&barrier);
                thread::spawn(move || {
                    barrier.wait();
                    acquire(
                        &dir,
                        &format!("run-{i}"),
                        DEFAULT_LOCK_TTL,
                        false,
                        DEFAULT_LOCK_FILENAME,
                    )
                })
            })
            .collect();

        let results: Vec<_> = handles.into_iter().map(|h| h.join().unwrap()).collect();
        let successes = results.iter().filter(|r| r.is_ok()).count();
        assert_eq!(
            successes, 1,
            "exactly one thread should acquire the lock, got {successes}"
        );
    }

    #[test]
    fn acquire_with_force_reclaims_fresh_lock() {
        let dir = TempDir::new().unwrap();

        let meta = LockMetadata {
            run_id: "active-run".to_string(),
            pid: std::process::id(),
            hostname: LockMetadata::current("host-probe").hostname,
            created_at: Timestamp::now(),
        };
        let lock_file = dir.path().join(DEFAULT_LOCK_FILENAME);
        write_lock(&lock_file, &meta).unwrap();

        let lock = acquire(
            dir.path(),
            "forced-run",
            DEFAULT_LOCK_TTL,
            true,
            DEFAULT_LOCK_FILENAME,
        )
        .unwrap();
        assert_eq!(lock.metadata().run_id, "forced-run");
    }

    #[test]
    fn acquire_with_force_handles_corrupt_lock() {
        let dir = TempDir::new().unwrap();
        let lock_file = dir.path().join(DEFAULT_LOCK_FILENAME);
        std::fs::write(&lock_file, "not-json-garbage").unwrap();

        let lock = acquire(
            dir.path(),
            "forced-run",
            DEFAULT_LOCK_TTL,
            true,
            DEFAULT_LOCK_FILENAME,
        )
        .unwrap();
        assert_eq!(lock.metadata().run_id, "forced-run");
    }

    #[test]
    fn acquire_with_force_no_existing_lock() {
        let dir = TempDir::new().unwrap();
        let lock = acquire(
            dir.path(),
            "run-1",
            DEFAULT_LOCK_TTL,
            true,
            DEFAULT_LOCK_FILENAME,
        )
        .unwrap();
        assert_eq!(lock.metadata().run_id, "run-1");
    }

    #[tokio::test]
    async fn lock_released_via_arc_mutex_take() {
        use std::sync::Arc;

        let dir = TempDir::new().unwrap();
        let lock = acquire(
            dir.path(),
            "run-1",
            DEFAULT_LOCK_TTL,
            false,
            DEFAULT_LOCK_FILENAME,
        )
        .unwrap();
        let lock_path = lock.path().to_path_buf();
        assert!(lock_path.exists());

        let handle: Arc<tokio::sync::Mutex<Option<RunLock>>> =
            Arc::new(tokio::sync::Mutex::new(Some(lock)));

        {
            let mut guard = handle.lock().await;
            let taken = guard.take().unwrap();
            taken.release().unwrap();
        }

        assert_eq!(
            std::fs::metadata(&lock_path).unwrap().len(),
            0,
            "take+release must clear ownership without unlinking the inode"
        );
    }

    #[test]
    fn read_lock_rejects_oversized_file() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("oversized.lock");
        let f = std::fs::File::create(&path).unwrap();
        f.set_len(MAX_LOCK_FILE_BYTES + 1).unwrap();
        drop(f);

        let err = read_lock(&path).unwrap_err();
        match &err {
            PersistenceError::LockFailed { reason } => {
                assert!(
                    reason.contains("too large"),
                    "expected 'too large' in reason: {reason}"
                );
            }
            other => panic!("expected LockFailed, got: {other:?}"),
        }
    }

    #[test]
    fn read_write_lock_round_trip() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("test.lock");

        let meta = LockMetadata::current("test-run");
        write_lock(&path, &meta).unwrap();
        let loaded = read_lock(&path).unwrap();

        assert_eq!(loaded.run_id, meta.run_id);
        assert_eq!(loaded.pid, meta.pid);
        assert_eq!(loaded.hostname, meta.hostname);
    }

    #[test]
    fn release_on_already_deleted_lock_file_succeeds() {
        let dir = TempDir::new().unwrap();
        let lock = acquire(
            dir.path(),
            "run-1",
            DEFAULT_LOCK_TTL,
            false,
            DEFAULT_LOCK_FILENAME,
        )
        .unwrap();
        let path = lock.path().to_path_buf();

        std::fs::remove_file(&path).unwrap();
        assert!(!path.exists());

        lock.release().unwrap();
    }

    #[test]
    fn old_format_lock_with_hostname_is_readable() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("old.lock");
        let json = r#"{
            "run_id": "old-run",
            "pid": 12345,
            "hostname": "old-host.example.com",
            "created_at": "2026-01-01T00:00:00Z"
        }"#;
        std::fs::write(&path, json).unwrap();

        let meta = read_lock(&path).unwrap();
        assert_eq!(meta.run_id, "old-run");
        assert_eq!(meta.pid, 12345);
        assert_eq!(meta.hostname, "old-host.example.com");
    }

    #[test]
    fn old_format_lock_without_hostname_is_ttl_only() {
        let dir = TempDir::new().unwrap();
        let lock_file = dir.path().join(DEFAULT_LOCK_FILENAME);
        let json = format!(
            r#"{{
            "run_id": "old-run",
            "pid": 999999999,
            "created_at": "{}"
        }}"#,
            Timestamp::now()
        );
        std::fs::write(&lock_file, json).unwrap();

        let meta = read_lock(&lock_file).unwrap();
        assert!(meta.hostname.is_empty());

        let result = acquire(
            dir.path(),
            "new-run",
            DEFAULT_LOCK_TTL,
            false,
            DEFAULT_LOCK_FILENAME,
        );
        assert!(
            result.is_err(),
            "missing hostname must prevent same-host dead-pid auto-steal"
        );
    }

    #[test]
    fn same_host_dead_pid_lock_is_reclaimed_immediately() {
        let dir = TempDir::new().unwrap();
        let meta = LockMetadata {
            run_id: "dead-run".to_string(),
            pid: 999_999_999,
            hostname: LockMetadata::current("host-probe").hostname,
            created_at: Timestamp::now(),
        };
        let lock_file = dir.path().join(DEFAULT_LOCK_FILENAME);
        write_lock(&lock_file, &meta).unwrap();

        let lock = acquire(
            dir.path(),
            "new-run",
            DEFAULT_LOCK_TTL,
            false,
            DEFAULT_LOCK_FILENAME,
        )
        .unwrap();

        assert_eq!(lock.metadata().run_id, "new-run");
    }

    #[test]
    fn same_host_alive_pid_lock_is_not_reclaimed() {
        let dir = TempDir::new().unwrap();
        let meta = LockMetadata {
            run_id: "alive-run".to_string(),
            pid: std::process::id(),
            hostname: LockMetadata::current("host-probe").hostname,
            created_at: Timestamp::now(),
        };
        let lock_file = dir.path().join(DEFAULT_LOCK_FILENAME);
        write_lock(&lock_file, &meta).unwrap();

        let result = acquire(
            dir.path(),
            "new-run",
            DEFAULT_LOCK_TTL,
            false,
            DEFAULT_LOCK_FILENAME,
        );

        assert!(result.is_err());
    }

    #[test]
    fn different_host_dead_pid_lock_reclaims_only_after_ttl() {
        let dir = TempDir::new().unwrap();
        let lock_file = dir.path().join(DEFAULT_LOCK_FILENAME);
        let fresh_meta = LockMetadata {
            run_id: "foreign-run".to_string(),
            pid: 999_999_999,
            hostname: "foreign-host.example.com".to_string(),
            created_at: Timestamp::now(),
        };
        write_lock(&lock_file, &fresh_meta).unwrap();

        let fresh_result = acquire(
            dir.path(),
            "new-run",
            DEFAULT_LOCK_TTL,
            false,
            DEFAULT_LOCK_FILENAME,
        );
        assert!(fresh_result.is_err());

        let aged_meta = LockMetadata {
            created_at: Timestamp::now() - SignedDuration::from_mins(16),
            ..fresh_meta
        };
        write_lock(&lock_file, &aged_meta).unwrap();

        let lock = acquire(
            dir.path(),
            "ttl-run",
            DEFAULT_LOCK_TTL,
            false,
            DEFAULT_LOCK_FILENAME,
        )
        .unwrap();

        assert_eq!(lock.metadata().run_id, "ttl-run");
    }

    #[test]
    fn auto_steal_emits_warn_with_holder_evidence() {
        let dir = TempDir::new().unwrap();
        let meta = LockMetadata {
            run_id: "dead-run".to_string(),
            pid: 999_999_999,
            hostname: LockMetadata::current("host-probe").hostname,
            created_at: Timestamp::now(),
        };
        let lock_file = dir.path().join(DEFAULT_LOCK_FILENAME);
        write_lock(&lock_file, &meta).unwrap();

        let events = capture_lock_events(|| {
            acquire(
                dir.path(),
                "new-run",
                DEFAULT_LOCK_TTL,
                false,
                DEFAULT_LOCK_FILENAME,
            )
            .unwrap();
        });

        assert!(events.contains("level=WARN"), "got: {events}");
        assert!(events.contains("run_id=dead-run"), "got: {events}");
        assert!(events.contains("pid=999999999"), "got: {events}");
        assert!(events.contains("host="), "got: {events}");
        assert!(
            events.contains("reason=\"dead-holder-same-host\""),
            "got: {events}"
        );
    }

    #[test]
    fn externally_created_empty_lock_file_is_replaced() {
        let dir = TempDir::new().unwrap();
        let lock_file = dir.path().join(DEFAULT_LOCK_FILENAME);

        std::fs::write(&lock_file, "").unwrap();

        let result = acquire(
            dir.path(),
            "new-run",
            DEFAULT_LOCK_TTL,
            false,
            DEFAULT_LOCK_FILENAME,
        );
        let lock = result.expect("acquire should succeed by replacing empty lock");
        assert_eq!(lock.metadata.run_id, "new-run");
    }

    #[test]
    fn renew_updates_created_at_and_keeps_run_id_and_pid() {
        let dir = TempDir::new().unwrap();
        let mut lock = acquire(
            dir.path(),
            "long-running",
            DEFAULT_LOCK_TTL,
            false,
            DEFAULT_LOCK_FILENAME,
        )
        .unwrap();
        let original_created_at = lock.metadata.created_at;
        let original_pid = lock.metadata.pid;
        let original_hostname = lock.metadata.hostname.clone();

        std::thread::sleep(Duration::from_millis(20));
        lock.renew().unwrap();

        assert_eq!(lock.metadata.run_id, "long-running");
        assert_eq!(lock.metadata.pid, original_pid);
        assert_eq!(lock.metadata.hostname, original_hostname);
        assert!(
            lock.metadata.created_at > original_created_at,
            "renew() must advance created_at"
        );

        let on_disk = read_lock(lock.path()).unwrap();
        assert_eq!(on_disk.run_id, "long-running");
        assert_eq!(on_disk.created_at, lock.metadata.created_at);
        assert_eq!(on_disk.hostname, lock.metadata.hostname);
    }

    #[test]
    fn renewed_lock_is_not_reclaimed_by_other_acquire() {
        let dir = TempDir::new().unwrap();
        let start: Timestamp = "2026-01-01T00:00:00Z".parse().unwrap();
        let mut lock = acquire_with_clock(
            dir.path(),
            "long-running",
            Duration::from_secs(1),
            false,
            DEFAULT_LOCK_FILENAME,
            || start,
        )
        .unwrap();
        assert_eq!(
            lock.metadata.created_at, start,
            "acquire must use injected time"
        );
        let original = lock.metadata.clone();
        let renewed_at = start + SignedDuration::from_millis(500);
        lock.renew_with_clock(|| renewed_at).unwrap();
        let expected = LockMetadata {
            created_at: renewed_at,
            ..original
        };
        assert_eq!(lock.metadata, expected);
        assert_eq!(read_lock(lock.path()).unwrap(), expected);

        for elapsed_ms in [1200, 1500] {
            let result = acquire_with_clock(
                dir.path(),
                "other",
                Duration::from_secs(1),
                false,
                DEFAULT_LOCK_FILENAME,
                || start + SignedDuration::from_millis(elapsed_ms),
            );
            assert!(
                matches!(result, Err(PersistenceError::LockFailed { .. })),
                "renewed lock must remain held before and exactly at TTL"
            );
            assert_eq!(read_lock(lock.path()).unwrap(), expected);
        }

        let after_ttl = renewed_at + SignedDuration::from_secs(1) + SignedDuration::from_nanos(1);
        let replacement = acquire_with_clock(
            dir.path(),
            "other",
            Duration::from_secs(1),
            false,
            DEFAULT_LOCK_FILENAME,
            || after_ttl,
        )
        .expect("a TTL-expired lock must be reclaimable even from a live holder");
        assert!(
            lock.renew_with_clock(|| after_ttl).is_err(),
            "the displaced holder must learn it lost ownership"
        );
        drop(lock);
        assert_eq!(
            read_lock(replacement.path()).unwrap().run_id,
            "other",
            "the displaced holder's drop must not disturb the replacement"
        );
        assert_eq!(replacement.metadata.run_id, "other");
        assert_eq!(replacement.metadata.created_at, after_ttl);
        assert_eq!(read_lock(replacement.path()).unwrap(), replacement.metadata);
    }

    #[test]
    fn capture_survives_wrong_thread_first_registration() {
        for order in [
            RegistrationOrder::CompetitorFirst,
            RegistrationOrder::CaptureFirst,
        ] {
            let mut command = capture_child_command("lock::tests::capture_registration_child");
            command.env(REGISTRATION_ORDER, order.as_str());
            let mut child = command.spawn().unwrap();
            let outcome = wait_for_capture_child(&mut child, CAPTURE_CHILD_TIMEOUT).unwrap();
            assert!(
                matches!(outcome, CaptureChildOutcome::Exited(status) if status.success()),
                "{order:?}: {outcome:?}; retained child output max=0 bytes"
            );
            eprintln!("verified-order={}", order.as_str());
        }
    }

    const REGISTRATION_ORDER: &str = "CHERRY_PIT_CAPTURE_REGISTRATION_ORDER";
    const CAPTURE_CHILD_TIMEOUT: Duration = Duration::from_secs(10);

    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    enum RegistrationOrder {
        CompetitorFirst,
        CaptureFirst,
    }

    impl RegistrationOrder {
        fn parse(value: &str) -> Result<Self, &'static str> {
            match value {
                "competitor-first" => Ok(Self::CompetitorFirst),
                "capture-first" => Ok(Self::CaptureFirst),
                _ => Err("unknown capture registration order"),
            }
        }

        fn as_str(self) -> &'static str {
            match self {
                Self::CompetitorFirst => "competitor-first",
                Self::CaptureFirst => "capture-first",
            }
        }
    }

    fn capture_child_command(target: &str) -> std::process::Command {
        let mut command = std::process::Command::new(std::env::current_exe().unwrap());
        command
            .args(["--exact", target, "--ignored", "--nocapture"])
            .stdin(std::process::Stdio::null())
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null());
        command
    }

    #[derive(Debug)]
    enum CaptureChildOutcome {
        Exited(std::process::ExitStatus),
        TimedOut(std::process::ExitStatus),
    }

    fn wait_for_capture_child(
        child: &mut std::process::Child,
        timeout: Duration,
    ) -> std::io::Result<CaptureChildOutcome> {
        let started = std::time::Instant::now();
        loop {
            match child.try_wait() {
                Ok(Some(status)) => return Ok(CaptureChildOutcome::Exited(status)),
                Ok(None) if started.elapsed() >= timeout => {
                    return kill_and_reap_capture_child(child).map(CaptureChildOutcome::TimedOut);
                }
                Ok(None) => std::thread::sleep(Duration::from_millis(5)),
                Err(error) => {
                    kill_and_reap_capture_child(child)?;
                    return Err(error);
                }
            }
        }
    }

    fn kill_and_reap_capture_child(
        child: &mut std::process::Child,
    ) -> std::io::Result<std::process::ExitStatus> {
        let killed = match child.kill() {
            Ok(()) => Ok(()),
            Err(error) => match child.try_wait() {
                Ok(Some(_)) => Ok(()),
                Ok(None) => Err(error),
                Err(wait_error) => Err(wait_error),
            },
        };
        let reaped = child.wait();
        killed?;
        reaped
    }

    #[test]
    fn capture_child_timeout_kills_and_reaps() {
        let mut child = capture_child_command("lock::tests::capture_timeout_child")
            .spawn()
            .unwrap();
        let started = std::time::Instant::now();
        let outcome = wait_for_capture_child(&mut child, Duration::from_millis(50)).unwrap();
        assert!(
            matches!(outcome, CaptureChildOutcome::TimedOut(status) if !status.success()),
            "must kill and reap timed-out child: {outcome:?}"
        );
        assert!(child.try_wait().unwrap().is_some());
        assert!(started.elapsed() < Duration::from_secs(5));
    }

    #[test]
    fn capture_timeout_child_exits_without_parent_kill() {
        let mut child = capture_child_command("lock::tests::capture_timeout_child")
            .spawn()
            .unwrap();
        let outcome = wait_for_capture_child(&mut child, Duration::from_secs(3)).unwrap();
        assert!(
            matches!(outcome, CaptureChildOutcome::Exited(status) if status.success()),
            "fixture must self-terminate: {outcome:?}"
        );
    }

    #[test]
    #[ignore = "dedicated subprocess deadline fixture"]
    fn capture_timeout_child() {
        std::thread::sleep(Duration::from_secs(2));
    }

    #[test]
    fn capture_registration_order_rejects_unknown() {
        assert_eq!(
            RegistrationOrder::parse("unknown"),
            Err("unknown capture registration order")
        );
    }

    #[test]
    #[ignore = "parent executes both registration orders in fresh processes"]
    fn capture_registration_child() {
        let order = RegistrationOrder::parse(&std::env::var(REGISTRATION_ORDER).unwrap()).unwrap();

        let events = capture_lock_events(|| {
            if order == RegistrationOrder::CaptureFirst {
                reclaim_dead_holder("captured-first");
            }
            std::thread::spawn(|| reclaim_dead_holder("uncaptured-competitor"))
                .join()
                .unwrap();
            let max_level = tracing::metadata::LevelFilter::current();
            eprintln!("order={order:?} max_level={max_level}");
            assert!(max_level >= tracing::metadata::LevelFilter::WARN);
            reclaim_dead_holder("captured-after");
        });
        let expected_host = format!("host={} ", LockMetadata::current("host-probe").hostname);
        assert!(
            events.lines().any(|event| [
                "run_id=captured-after",
                "level=WARN",
                "pid=999999999",
                &expected_host,
                "reason=\"dead-holder-same-host\""
            ]
            .iter()
            .all(|field| event.contains(field))),
            "got: {events}"
        );
        assert!(!events.contains("uncaptured-competitor"), "got: {events}");
        if order == RegistrationOrder::CaptureFirst {
            assert!(events.contains("run_id=captured-first"), "got: {events}");
        }
    }

    fn reclaim_dead_holder(run_id: &str) {
        let dir = TempDir::new().unwrap();
        let meta = LockMetadata {
            run_id: run_id.to_owned(),
            pid: 999_999_999,
            hostname: LockMetadata::current("host-probe").hostname,
            created_at: Timestamp::now(),
        };
        write_lock(&dir.path().join(DEFAULT_LOCK_FILENAME), &meta).unwrap();
        let lock = acquire(
            dir.path(),
            "replacement",
            DEFAULT_LOCK_TTL,
            false,
            DEFAULT_LOCK_FILENAME,
        )
        .unwrap();
        assert_eq!(lock.metadata.run_id, "replacement");
    }

    fn capture_lock_events(f: impl FnOnce()) -> String {
        use std::fmt::Write;
        use std::sync::{Arc, Mutex};
        use tracing::Event;
        use tracing::Level;
        use tracing::Metadata;
        use tracing::Subscriber;
        use tracing::field::{Field, Visit};
        use tracing::span;
        use tracing::subscriber::Interest;

        #[derive(Clone, Default)]
        struct CaptureSubscriber {
            events: Arc<Mutex<String>>,
        }

        impl Subscriber for CaptureSubscriber {
            fn enabled(&self, metadata: &Metadata<'_>) -> bool {
                metadata.level() <= &Level::WARN
            }

            fn new_span(&self, _span: &span::Attributes<'_>) -> span::Id {
                span::Id::from_u64(1)
            }

            fn record(&self, _span: &span::Id, _values: &span::Record<'_>) {}

            fn record_follows_from(&self, _span: &span::Id, _follows: &span::Id) {}

            fn event(&self, event: &Event<'_>) {
                let mut visitor = EventVisitor::default();
                event.record(&mut visitor);
                let mut events = self.events.lock().unwrap();
                writeln!(
                    events,
                    "level={} {}",
                    event.metadata().level(),
                    visitor.fields
                )
                .unwrap();
            }

            fn enter(&self, _span: &span::Id) {}

            fn exit(&self, _span: &span::Id) {}

            fn register_callsite(&self, metadata: &'static Metadata<'static>) -> Interest {
                if self.enabled(metadata) {
                    Interest::always()
                } else {
                    Interest::never()
                }
            }

            fn max_level_hint(&self) -> Option<tracing::metadata::LevelFilter> {
                Some(tracing::metadata::LevelFilter::WARN)
            }
        }

        #[derive(Default)]
        struct EventVisitor {
            fields: String,
        }

        impl Visit for EventVisitor {
            fn record_debug(&mut self, field: &Field, value: &dyn std::fmt::Debug) {
                write!(self.fields, "{}={value:?} ", field.name()).unwrap();
            }
        }

        let subscriber = CaptureSubscriber::default();
        let events = Arc::clone(&subscriber.events);
        let registration_dispatch = tracing::Dispatch::new(subscriber.clone());
        tracing::subscriber::with_default(subscriber, f);
        drop(registration_dispatch);
        events.lock().unwrap().clone()
    }
}

#[cfg(test)]
mod ownership_regression {
    use super::*;
    use tempfile::TempDir;

    fn acquire_at(
        dir: &Path,
        run_id: &str,
        ttl: Duration,
        at: Timestamp,
    ) -> Result<RunLock, PersistenceError> {
        acquire_with_clock(dir, run_id, ttl, false, DEFAULT_LOCK_FILENAME, || at)
    }

    #[test]
    fn h3_stale_owner_drop_does_not_destroy_replacement() {
        let dir = TempDir::new().unwrap();
        let start: Timestamp = "2026-01-01T00:00:00Z".parse().unwrap();
        let ttl = Duration::from_mins(1);
        let stale = acquire_at(dir.path(), "old", ttl, start).unwrap();
        let taken_over = start + SignedDuration::from_secs(600);

        let replacement = acquire_at(dir.path(), "new", ttl, taken_over)
            .expect("TTL takeover must succeed against a live stale holder");
        drop(stale);

        assert_eq!(replacement.metadata().run_id, "new");
        assert_eq!(
            read_lock(replacement.path()).unwrap().run_id,
            "new",
            "replacement ownership must survive the displaced holder's drop"
        );
        let fresh = acquire(
            dir.path(),
            "probe",
            DEFAULT_LOCK_TTL,
            false,
            DEFAULT_LOCK_FILENAME,
        )
        .expect("a fresh acquisition may take over the ancient replacement record");
        assert!(
            replacement.release().is_ok(),
            "the displaced replacement's release must be a harmless no-op"
        );
        assert_eq!(
            read_lock(fresh.path()).unwrap().run_id,
            "probe",
            "the newest owner's record must be intact"
        );
    }

    #[test]
    fn h3_stale_owner_renew_does_not_overwrite_replacement() {
        let dir = TempDir::new().unwrap();
        let start: Timestamp = "2026-01-01T00:00:00Z".parse().unwrap();
        let ttl = Duration::from_mins(1);
        let mut stale = acquire_at(dir.path(), "old", ttl, start).unwrap();
        let replacement = acquire_at(
            dir.path(),
            "new",
            ttl,
            start + SignedDuration::from_secs(600),
        )
        .expect("TTL takeover must succeed against a live stale holder");

        assert!(
            stale
                .renew_with_clock(|| start + SignedDuration::from_secs(900))
                .is_err(),
            "a displaced holder's renew must report lost ownership"
        );
        assert!(
            stale.release().is_ok(),
            "a displaced holder's release must be a harmless no-op"
        );

        let on_disk = read_lock(replacement.path()).unwrap();
        assert_eq!(
            on_disk.run_id, "new",
            "the replacement's record must survive the displaced holder"
        );
    }

    #[test]
    fn h3_explicit_release_cannot_double_remove_a_later_owner() {
        let dir = TempDir::new().unwrap();
        let first = acquire(
            dir.path(),
            "first",
            DEFAULT_LOCK_TTL,
            false,
            DEFAULT_LOCK_FILENAME,
        )
        .unwrap();
        first.release().unwrap();

        let second = acquire(
            dir.path(),
            "second",
            DEFAULT_LOCK_TTL,
            false,
            DEFAULT_LOCK_FILENAME,
        )
        .unwrap();

        assert_eq!(second.metadata().run_id, "second");
        assert_eq!(
            read_lock(second.path()).unwrap().run_id,
            "second",
            "released guard must not remove a later owner's record"
        );
    }

    #[cfg(unix)]
    #[test]
    fn h4_unreadable_live_lock_errors_without_mutation() {
        use std::os::unix::fs::PermissionsExt;

        let dir = TempDir::new().unwrap();
        let path = dir.path().join(DEFAULT_LOCK_FILENAME);
        write_lock(&path, &LockMetadata::current("live")).unwrap();
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o000)).unwrap();

        let result = acquire(
            dir.path(),
            "contender",
            DEFAULT_LOCK_TTL,
            false,
            DEFAULT_LOCK_FILENAME,
        );

        assert!(result.is_err(), "unreadable lock must not be acquirable");
        assert!(
            path.exists(),
            "unreadable lock must not be deleted as corrupt"
        );
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o600)).unwrap();
        let survivor = read_lock(&path).unwrap();
        assert_eq!(survivor.run_id, "live");
    }

    #[test]
    fn m2_oversize_lock_is_rejected_without_mutation() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join(DEFAULT_LOCK_FILENAME);
        let file = std::fs::File::create(&path).unwrap();
        file.set_len(MAX_LOCK_FILE_BYTES + 1).unwrap();
        drop(file);

        let result = acquire(
            dir.path(),
            "contender",
            DEFAULT_LOCK_TTL,
            false,
            DEFAULT_LOCK_FILENAME,
        );

        let err = result.expect_err("oversize lock must be rejected");
        assert!(
            format!("{err}").contains("too large"),
            "expected oversize rejection, got: {err}"
        );
        assert_eq!(
            std::fs::metadata(&path).unwrap().len(),
            MAX_LOCK_FILE_BYTES + 1,
            "oversize lock must not be mutated"
        );
    }

    #[test]
    fn m2_at_limit_lock_is_read_within_bound() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("at-limit.lock");
        let file = std::fs::File::create(&path).unwrap();
        file.set_len(MAX_LOCK_FILE_BYTES).unwrap();
        drop(file);

        let err = read_lock(&path).unwrap_err();
        assert!(
            !format!("{err}").contains("too large"),
            "at-limit file must not be rejected as oversize: {err}"
        );
    }

    #[test]
    fn h1_competing_reclaimers_are_serialized_to_one_generation() {
        use std::sync::Arc;
        use std::sync::atomic::{AtomicUsize, Ordering};

        let dir = Arc::new(TempDir::new().unwrap());
        let start: Timestamp = "2026-01-01T00:00:00Z".parse().unwrap();
        let ttl = Duration::from_mins(1);
        let stale = acquire_at(dir.path(), "old", ttl, start).unwrap();
        let takeover = start + SignedDuration::from_secs(600);

        let count = Arc::new(AtomicUsize::new(0));
        let held = std::sync::Mutex::new(Vec::new());
        std::thread::scope(|scope| {
            for i in 0..8 {
                let dir = Arc::clone(&dir);
                let count = Arc::clone(&count);
                let held = &held;
                scope.spawn(move || {
                    if let Ok(lock) = acquire_at(dir.path(), &format!("c{i}"), ttl, takeover) {
                        count.fetch_add(1, Ordering::Relaxed);
                        held.lock().unwrap().push(lock);
                    }
                });
            }
        });

        assert!(
            count.load(Ordering::Relaxed) >= 1,
            "at least one reclaimer must win the expired lock"
        );
        let held = held.into_inner().unwrap();
        let lock_path = dir.path().join(DEFAULT_LOCK_FILENAME);
        let on_disk = read_lock(&lock_path).unwrap().run_id;
        let mut owner = None;
        for candidate in held {
            if candidate.metadata().run_id == on_disk {
                owner = Some(candidate);
            } else {
                assert!(
                    candidate.release().is_ok(),
                    "a displaced reclaimer's release must be a no-op"
                );
            }
        }
        let owner = owner.expect("the recorded run must be one of the reclaimers");
        assert!(
            stale.release().is_ok(),
            "the displaced original holder's release must be a no-op"
        );
        let survivor = read_lock(owner.path()).unwrap();
        assert_eq!(
            survivor.run_id,
            owner.metadata().run_id,
            "exactly one reclaimer's record must survive the others' releases"
        );
    }

    #[cfg(unix)]
    #[test]
    fn h2_symlinked_coordination_path_is_rejected_without_mutating_target() {
        let dir = TempDir::new().unwrap();
        let target = dir.path().join("innocent.txt");
        std::fs::write(&target, b"not json, not ours").unwrap();
        std::os::unix::fs::symlink(&target, dir.path().join(DEFAULT_LOCK_FILENAME)).unwrap();

        let err = acquire(
            dir.path(),
            "contender",
            DEFAULT_LOCK_TTL,
            false,
            DEFAULT_LOCK_FILENAME,
        )
        .expect_err("a symlinked coordination path must be rejected");
        assert!(
            matches!(
                err,
                PersistenceError::Io(_) | PersistenceError::LockFailed { .. }
            ),
            "unexpected rejection: {err}"
        );
        assert_eq!(
            std::fs::read(&target).unwrap(),
            b"not json, not ours",
            "the symlink target must not be mutated"
        );
    }

    #[cfg(unix)]
    #[test]
    fn h2_hardlinked_coordination_inode_is_rejected_without_mutating_target() {
        let dir = TempDir::new().unwrap();
        let target = dir.path().join("innocent.txt");
        std::fs::write(&target, b"not json, not ours").unwrap();
        std::fs::hard_link(&target, dir.path().join(DEFAULT_LOCK_FILENAME)).unwrap();

        let err = acquire(
            dir.path(),
            "contender",
            DEFAULT_LOCK_TTL,
            false,
            DEFAULT_LOCK_FILENAME,
        )
        .expect_err("a shared coordination inode must be rejected");
        assert!(
            format!("{err}").contains("links"),
            "expected a link-policy rejection, got: {err}"
        );
        assert_eq!(
            std::fs::read(&target).unwrap(),
            b"not json, not ours",
            "the hard-link target must not be mutated"
        );
    }

    #[cfg(unix)]
    #[test]
    fn h2_directory_coordination_path_is_rejected() {
        let dir = TempDir::new().unwrap();
        std::fs::create_dir(dir.path().join(DEFAULT_LOCK_FILENAME)).unwrap();

        assert!(
            acquire(
                dir.path(),
                "contender",
                DEFAULT_LOCK_TTL,
                false,
                DEFAULT_LOCK_FILENAME,
            )
            .is_err(),
            "a directory must never be used as a coordination inode"
        );
    }

    #[cfg(unix)]
    #[test]
    fn h2_fifo_coordination_path_is_rejected_without_hanging() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join(DEFAULT_LOCK_FILENAME);
        let status = std::process::Command::new("mkfifo")
            .arg(&path)
            .status()
            .expect("mkfifo must be available");
        assert!(status.success(), "mkfifo failed");

        let (tx, rx) = std::sync::mpsc::channel();
        let probe_dir = dir.path().to_path_buf();
        std::thread::spawn(move || {
            let outcome = acquire(
                &probe_dir,
                "contender",
                DEFAULT_LOCK_TTL,
                false,
                DEFAULT_LOCK_FILENAME,
            )
            .map_or_else(|e| e.to_string(), |_| "acquired".to_string());
            let _ = tx.send(outcome);
        });

        let reason = rx
            .recv_timeout(Duration::from_secs(5))
            .expect("opening a FIFO coordination path must not block");
        assert!(
            reason.contains("not a regular file"),
            "a FIFO must be rejected by the file-kind guard, got: {reason}"
        );
    }

    #[test]
    fn m1_valid_at_limit_record_is_accepted_within_the_read_budget() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("at-limit-valid.lock");
        let mut meta = LockMetadata::current("padded");
        let base = serde_json::to_string_pretty(&meta).unwrap().len();
        let pad = usize::try_from(MAX_LOCK_FILE_BYTES).unwrap() - base;
        meta.run_id = "x".repeat(pad + "padded".len());
        let json = serde_json::to_string_pretty(&meta).unwrap();
        assert_eq!(
            u64::try_from(json.len()).unwrap(),
            MAX_LOCK_FILE_BYTES,
            "fixture must sit exactly at the read bound"
        );
        std::fs::write(&path, &json).unwrap();

        let read = read_lock(&path).expect("a valid at-limit record must be accepted");
        assert_eq!(read.run_id, meta.run_id);
    }

    #[test]
    fn m1_excess_byte_is_rejected_and_read_buffer_never_grows() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("excess.lock");
        let mut json = serde_json::to_string_pretty(&LockMetadata::current("padded")).unwrap();
        json.push_str(&" ".repeat(usize::try_from(MAX_LOCK_FILE_BYTES).unwrap() + 1 - json.len()));
        std::fs::write(&path, &json).unwrap();

        let err = read_lock(&path).expect_err("one excess byte must be rejected");
        assert!(
            format!("{err}").contains("too large"),
            "expected oversize rejection, got: {err}"
        );

        let buf = bounded_buffer();
        let capacity = buf.capacity();
        assert_eq!(
            u64::try_from(capacity).unwrap(),
            MAX_LOCK_FILE_BYTES + 1,
            "the read buffer must be allocated once at exactly the budget"
        );
    }
}
