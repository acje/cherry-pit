# CHE-0044. Object Store Backend (Planned)

Date: 2026-04-25
Last-reviewed: 2026-04-25
Tier: D
Status: Proposed

## Related

References: CHE-0001, CHE-0004, CHE-0006, CHE-0031, CHE-0032, CHE-0035, CHE-0036, CHE-0037, CHE-0043

## Context

`MsgpackFileStore` is the sole `EventStore` implementation, using
local-filesystem atomic writes (CHE-0032), per-aggregate locking
(CHE-0035), and process-level fencing (CHE-0043). This limits
deployment to a single machine with local storage.

The `object_store` crate (Apache Arrow project) provides a unified
API for local filesystem, S3, GCS, Azure Blob, and in-memory
backends. It decouples the store from POSIX semantics, enables
cloud-native deployment, and supports conditional put
(`PutMode::Update` with ETags) mapping naturally to optimistic
concurrency. The MessagePack wire format (CHE-0031) is orthogonal
to the storage backend.

Alternatives considered: direct S3 SDK (couples to one provider),
database-backed store (adds connection pooling and schema management),
and keeping file-only (limits production options). Conditional put
with ETags replaces both advisory locking and temp-file-rename;
the ETag serves as a fencing token preventing lost updates without
process-level coordination. CHE-0037 defers snapshot support; an
object store backend does not change this, though higher per-request
latency may accelerate the need for snapshots.

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

- **New dependency** — `object_store` and its feature flags (aws,
  gcp, azure) increase compile time and binary size. Feature-gated
  to minimize impact when unused.
- **Latency profile changes** — object storage has higher per-request
  latency than local disk. Full-stream reads are more expensive,
  increasing pressure toward snapshot support (CHE-0037).
- **No flock needed** — conditional put replaces advisory file locking.
  CHE-0043's fencing mechanism becomes specific to `MsgpackFileStore`,
  not a general requirement.
- **ETag semantics vary** — S3 ETags for single-part uploads are
  MD5 hashes; GCS uses CRC32C-based generation numbers; Azure uses
  opaque ETags. The `object_store` crate abstracts this, but tests
  must cover each backend's conditional-put behavior.
- **ID assignment changes** — `scan_max_id` via directory listing is
  eventually consistent on S3 (list-after-write consistency was added
  in 2020, but prefix listing has edge cases). The ID assignment
  strategy may need to move to a separate metadata object or atomic
  counter.
- **Migration path** — existing `MsgpackFileStore` data can be
  migrated by copying `.msgpack` files to object storage. The wire
  format is identical; only the storage and concurrency layer changes.
