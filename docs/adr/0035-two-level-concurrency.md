# 35. Two-Level Concurrency Architecture

Date: 2026-04-25
Last-reviewed: 2026-04-25

## Status

Accepted

## Related

- Depends on: ADR 0006, ADR 0032
- Referenced by: ADR 0036 (indirect), ADR 0043

## Context

`MsgpackFileStore` must handle concurrent operations safely:

- Multiple aggregates can be written simultaneously.
- Two appends to the same aggregate must be serialized.
- New aggregate ID assignment must be globally unique.
- Reads should not block writes or other reads.

Options for concurrent access:

1. **Global lock** — one `Mutex` protecting all operations. Simple but
   eliminates all write parallelism. Reads blocked by writes.
2. **Per-aggregate lock** — fine-grained locking. Different aggregates
   proceed in parallel. Same-aggregate writes are serialized.
3. **Lock-free** — atomic operations only. Complex, hard to reason
   about with file I/O.

## Decision

`MsgpackFileStore` uses a two-level concurrency architecture:

1. **Global ID mutex** (`tokio::sync::Mutex<Option<u64>>`) — held only
   during aggregate ID assignment in `create`. Serializes ID
   generation to guarantee uniqueness. Lazy initialization: `None`
   on first access triggers `scan_max_id()` to read the directory.

2. **Per-aggregate write locks** (`scc::HashMap<u64,
   Arc<tokio::sync::Mutex<()>>>`) — `scc::HashMap` is a lock-free
   concurrent hash map. Two access patterns:
   - Fast path: `read_sync` — lock-free read for existing entries.
   - Slow path: `entry_sync` + `or_insert_with` — fine-grained insert
     for new entries.
   Each aggregate gets its own `tokio::sync::Mutex<()>` wrapped in
   `Arc` for sharing across tasks.

3. **Lock-free reads** — `load()` reads files directly without
   acquiring any lock. This is safe because writes are atomic (temp
   file + rename) — a concurrent read sees either the old or new
   version, never a partial write.

`create` does NOT acquire a per-aggregate write lock. This is safe
because the global ID mutex guarantees the assigned ID is unique — no
other operation can target a freshly assigned ID. The write to disk
happens after the mutex is released but before any other operation can
know the new ID.

## Consequences

- Different aggregates can be read and written concurrently without
  contention.
- Same-aggregate writes are serialized, preventing the read-check-
  write race in optimistic concurrency.
- Reads never block. A read during a concurrent write sees the
  pre-write state (atomic rename has not yet occurred).
- The `scc::HashMap` grows monotonically — locks for aggregate IDs are
  never removed. For long-running processes with many aggregates, this
  is a minor memory leak. Acceptable for the target use case.
- `scan_max_id` is one-shot: it runs once on the first `create` call.
  Files added externally after initialization are invisible to the
  counter. This is consistent with the single-writer assumption
  (ADR 0006) — external modification is undefined behavior.
- The coupling between `create` (no per-aggregate lock) and
  `write_atomic` (temp file naming) means `write_atomic` must not be
  called concurrently for the same target path without external
  serialization. This invariant is maintained by the current design
  but would break if `create` were parallelized.
