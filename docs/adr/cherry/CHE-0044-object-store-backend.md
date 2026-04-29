# CHE-0044. Object Store Backend (Planned)

Date: 2026-04-25
Last-reviewed: 2026-04-28
Tier: D
Status: Proposed

## Related

References: CHE-0001, CHE-0006, CHE-0031, CHE-0032, CHE-0043

## Context

`MsgpackFileStore` is the sole `EventStore` implementation, limited to single-machine deployment. The `object_store` crate (Apache Arrow) provides a unified API for local filesystem, S3, GCS, and Azure Blob. Its conditional put (`PutMode::Update` with ETags) maps naturally to optimistic concurrency, replacing advisory locking and temp-file-rename.

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

- **New dependency** — `object_store` increases compile time. Feature-gated.
- **Higher latency** — object storage per-request latency increases pressure toward snapshots.
- **No flock needed** — conditional put replaces advisory locking.
- **ETag semantics vary** by provider; tests must cover each backend.
- **Migration path** — copy `.msgpack` files to object storage. Wire format identical.
