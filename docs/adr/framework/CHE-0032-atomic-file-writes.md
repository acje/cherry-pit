# CHE-0032. Atomic File Writes via Temp-File and Rename

Date: 2026-04-25
Last-reviewed: 2026-04-25
Tier: D

## Status

Accepted

## Related

- Depends on: CHE-0006
- Informs: CHE-0035, CHE-0036
- Extended by: CHE-0043, CHE-0044

## Context

`MsgpackFileStore` persists aggregate event streams as files. If the
process crashes during a write, the file on disk must remain in a
consistent state — either the old version or the new version, never a
partial write.

Write strategies considered:

1. **Direct write** — `write()` to the target file. A crash mid-write
   leaves a truncated or corrupt file. Unacceptable.
2. **Write-ahead log** — append changes to a WAL, then apply. Complex,
   introduces a second file format, requires recovery on startup.
3. **Temp-file + rename** — write to a temporary file, then
   `rename()` to the target path. On POSIX systems, `rename(2)` is
   atomic — the directory entry is updated in a single operation.

## Decision

All writes in `MsgpackFileStore` go through `write_atomic`:

1. Serialize envelopes to bytes in memory.
2. Write bytes to `{filename}.tmp` in the store directory.
3. `rename()` the temp file to the target path.
4. On rename failure, clean up the temp file (best-effort).

Temp file naming uses `{aggregate_filename}.tmp` (e.g.,
`1.msgpack.tmp`). This is safe because:

- `append` holds a per-aggregate write lock — no two appends to the
  same aggregate run concurrently, so their temp files cannot collide.
- `create` assigns unique sequential IDs under a global mutex — no
  two creates target the same filename.

## Consequences

- Crash during write leaves only the temp file. The aggregate file is
  either the old version or absent. No corrupt data.
- **Platform constraint:** POSIX `rename(2)` is atomic for both new
  files and overwrites. On Windows, `rename` fails if the destination
  exists — the `append` path (which overwrites) would fail on Windows.
  Cherry-pit currently targets POSIX systems only for file-based
  storage.
- Temp file safety is coupled to the concurrency design: if per-
  aggregate locking or sequential ID assignment were removed,
  concurrent `write_atomic` calls to the same path would corrupt data.
  This coupling is intentional and documented.
- Orphaned `.tmp` files (from crashes) accumulate in the store
  directory. No automatic cleanup is implemented — manual cleanup or
  startup sweep is a future concern.
- The entire aggregate history is rewritten on every `append` (read-
  modify-write under lock). This is simple but not suitable for
  aggregates with thousands of events.
