# CHE-0044. Object Store Backend (Planned)

Date: 2026-04-25
Last-reviewed: 2026-04-25
Tier: D
Status: Proposed

## Related

References: CHE-0001, CHE-0004, CHE-0006, CHE-0031, CHE-0032, CHE-0035, CHE-0036, CHE-0037, CHE-0043

## Context

`MsgpackFileStore` is the sole `EventStore` implementation, using local-filesystem atomic writes (CHE-0032), per-aggregate locking (CHE-0035), and process-level fencing (CHE-0043). This limits deployment to a single machine. The `object_store` crate (Apache Arrow) provides a unified API for local filesystem, S3, GCS, Azure Blob, and in-memory backends. It supports conditional put (`PutMode::Update` with ETags) mapping naturally to optimistic concurrency, replacing both advisory locking and temp-file-rename. The MessagePack wire format (CHE-0031) is orthogonal to the storage backend.

## Decision

Adopt the `object_store` crate as the storage abstraction for a new
`EventStore` implementation alongside `MsgpackFileStore`. The new
implementation will:

R1 [10]: Use conditional put (PutMode::Update) for optimistic
  concurrency instead of file locks and rename
R2 [10]: Preserve the MessagePack named-fields wire format across
  all storage backends
R3 [10]: Keep MsgpackFileStore available for single-machine
  zero-dependency deployments alongside the object store backend

1. Implement `EventStore` with the same `MessagePack + named fields`
   wire format.
2. Use conditional put (`PutMode::Update`) for optimistic concurrency
   instead of file locks and rename.
3. Support local filesystem via `object_store::local::LocalFileSystem`
   as a drop-in replacement for `MsgpackFileStore` in tests.
4. Support S3-compatible backends via `object_store::aws::AmazonS3`.
5. Remove the need for process-level fencing (CHE-0043) — the storage
   backend provides distributed concurrency control.

The `MsgpackFileStore` remains available for single-machine,
zero-dependency deployments. The two implementations coexist — users
choose at wiring time.

## Consequences

- **New dependency** — `object_store` and feature flags increase compile time and binary size. Feature-gated to minimize impact.
- **Latency profile changes** — object storage has higher per-request latency, increasing pressure toward snapshot support (CHE-0037).
- **No flock needed** — conditional put replaces advisory locking. CHE-0043's fencing becomes `MsgpackFileStore`-specific.
- **ETag semantics vary** by provider; the `object_store` crate abstracts this but tests must cover each backend.
- **ID assignment** — `scan_max_id` via directory listing has eventual consistency edge cases on S3; may need a separate metadata object.
- **Migration path** — copy `.msgpack` files to object storage. Wire format is identical; only storage and concurrency layers change.
