# CHE-0044. Object Store Backend (Planned)

Date: 2026-04-25
Last-reviewed: 2026-04-25
Tier: D
Status: Proposed

## Related

- References: CHE-0004, CHE-0006, CHE-0031, CHE-0032, CHE-0035, CHE-0036, CHE-0037, CHE-0043

## Context

`MsgpackFileStore` is the sole `EventStore` implementation. It uses
local-filesystem atomic writes (CHE-0032), per-aggregate locking
(CHE-0035), and process-level fencing (CHE-0043). This design is
correct and simple but limits deployment to a single machine with
local storage.

The `object_store` crate (Apache-licensed, maintained by the Apache
Arrow project) provides a unified API for local filesystem, S3,
GCS, Azure Blob Storage, and in-memory backends. Using it as the
storage layer would:

1. **Decouple the store from local filesystem semantics** — remove
   the POSIX `rename(2)` and `flock(2)` assumptions.
2. **Enable cloud-native deployment** — S3-compatible storage is
   available in every major cloud and many on-premise setups.
3. **Preserve the existing concurrency model** — the `object_store`
   API supports conditional put (`PutMode::Update` with ETags),
   which maps naturally to the optimistic concurrency pattern.
4. **Keep the MessagePack wire format** — the serialization format
   (CHE-0031) is orthogonal to the storage backend.

### Alternatives considered

- **Direct S3 SDK** — couples to one cloud provider. The
  `object_store` crate abstracts this away while remaining thin.
- **Database-backed store** (PostgreSQL, DynamoDB) — more complex,
  introduces connection pooling and schema management. Appropriate
  for a future Tier D ADR if needed.
- **Keep file-only** — acceptable for development but limits
  production deployment options.

### Concurrency mapping

| MsgpackFileStore mechanism     | object_store equivalent          |
|-------------------------------|----------------------------------|
| `flock(2)` advisory lock      | Not needed — conditional put     |
| Temp file + `rename(2)`       | `put_opts` with `PutMode::Update`|
| Per-aggregate `Mutex`         | ETag-based compare-and-swap      |
| `scan_max_id` directory walk  | `list_with_delimiter` prefix scan|

Conditional put with ETags replaces both the advisory lock and the
temp-file-rename pattern. The ETag serves as the fencing token: a
write succeeds only if the ETag matches, preventing lost updates
without process-level coordination.

### Snapshot implications

CHE-0037 defers snapshot support. An `object_store` backend does not
change this — full-stream reads from object storage have higher
latency than local disk, which may accelerate the need for snapshots
(revisit criterion 3 in CHE-0037).

## Decision

Adopt the `object_store` crate as the storage abstraction for a new
`EventStore` implementation alongside `MsgpackFileStore`. The new
implementation will:

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
