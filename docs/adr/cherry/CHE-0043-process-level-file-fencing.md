# CHE-0043. Process-Level File Fencing

Date: 2026-04-25
Last-reviewed: 2026-04-25
Tier: D

## Status

Accepted

Amended 2026-04-25 — added COM cross-reference

## Related

- References: CHE-0006, CHE-0021, CHE-0032, CHE-0035, COM-0003, PAR-0004

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

- **Two processes, same directory** → second process fails fast with
  `StoreError::StoreLocked` instead of silently corrupting data.
- **Single process, multiple store instances** → each `File::create()`
  opens a new file description; `flock` is per-description on both
  macOS and Linux, so two `MsgpackFileStore` instances in the same
  process contending for the same directory will also conflict
  (desirable — only one store should own a directory).
- **Read-only access unaffected** — `load` does not call
  `ensure_fenced`. Multiple readers can coexist with a single writer.
- **Zero new dependencies** — uses `std::fs::File::try_lock()`,
  available since Rust 1.95.
- **Advisory, not mandatory** — a process that ignores the `.lock`
  file can still write. This is defense-in-depth, not a security
  boundary.
- **NFS caveat** — `flock` is not reliable on NFS. The store
  directory must be on a local filesystem. This is consistent with
  the single-process deployment model (CHE-0006).
- **`.lock` file left on disk** — the sentinel file remains after
  the process exits. This is harmless — the lock is on the open
  file descriptor, not the file's existence. The file is covered
  by `.gitignore` (`store/` directory).
