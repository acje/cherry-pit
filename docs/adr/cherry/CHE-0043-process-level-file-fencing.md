# CHE-0043. Process-Level File Fencing

Date: 2026-04-25
Last-reviewed: 2026-04-25
Tier: D
Status: Accepted

## Related

References: CHE-0001, CHE-0006, CHE-0021, CHE-0032, CHE-0035, COM-0003, PAR-0004

## Context

CHE-0006 establishes the single-writer assumption: each aggregate
instance is owned by exactly one OS process. However, no enforcement
mechanism existed — if two processes accidentally shared a store
directory, concurrent writes could corrupt data silently.

The gap: CHE-0006 documented the risk ("No fencing mechanism exists
at the storage level. If two processes accidentally share a store
directory, data corruption is possible") but deferred mitigation.

## Decision

`MsgpackFileStore` acquires an exclusive advisory file lock on
`{store_dir}/.lock` before its first write operation (`create` or
`append`). The lock is:

R1 [10]: Acquire an exclusive advisory file lock on {store_dir}/.lock
  before the first write operation
R2 [10]: Lock acquisition is lazy via OnceCell, triggered on first
  write not on construction
R3 [10]: A second process on the same directory gets
  StoreError::StoreLocked instead of silent data corruption

1. **Lazy** — acquired on first write via `tokio::sync::OnceCell`,
   not on construction. Read-only operations (`load`) do not fence.
2. **Exclusive** — uses `std::fs::File::try_lock()` (Rust 1.95+
   native `flock(2)` wrapper). A second process attempting the same
   directory gets `StoreError::StoreLocked`.
3. **Process-scoped** — the `std::fs::File` handle lives in the
   `OnceCell` for the `MsgpackFileStore` lifetime. The OS releases
   the lock when the file descriptor closes (on drop).
4. **Advisory** — the lock is advisory, not mandatory. It prevents
   accidental dual-writer scenarios; it does not protect against
   malicious processes ignoring the lock.

```rust
// StoreError variant
StoreLocked { path: PathBuf }

// MsgpackFileStore field
dir_lock: tokio::sync::OnceCell<std::fs::File>

// Acquisition (inside ensure_fenced)
file.try_lock().map_err(|e| match e {
    std::fs::TryLockError::WouldBlock => StoreError::StoreLocked { path },
    std::fs::TryLockError::Error(io) => StoreError::Infrastructure(Box::new(io)),
})?;
```

No external crate is needed — Rust 1.95 stabilized `File::try_lock()`
and `File::lock()` in `std::fs`.

## Consequences

- Two processes on same directory → second fails fast with `StoreError::StoreLocked` instead of silent corruption.
- Two `MsgpackFileStore` instances in the same process contending for the same directory also conflict (desirable).
- Read-only access unaffected — `load` does not call `ensure_fenced`.
- Zero new dependencies — uses `std::fs::File::try_lock()` (Rust 1.95+).
- Advisory, not mandatory — defense-in-depth, not a security boundary.
- `flock` is unreliable on NFS; store directory must be local filesystem (consistent with CHE-0006).
- `.lock` file remains after exit — harmless; the lock is on the file descriptor, not the file's existence.
