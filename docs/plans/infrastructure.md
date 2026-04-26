# Infrastructure crates

Concrete implementations of cherry-pit-core port traits. Components are
pulled in as narrow, purpose-built crates under the `pit-*` namespace.

## Components

- **Gateway** (`cherry-pit-gateway`) — Concrete implementations of cherry-pit-core
  port traits. Currently provides `MsgpackFileStore<E>` — a file-based
  event store parameterized by domain event type for compile-time safety.
  Each aggregate type gets its own store instance pointing at its own
  directory. Designed for development and small deployments.
- **Web serving** (`cherry-pit-web`, planned) — HTTP endpoint scaffolding for
  inbound webhooks and query-side API serving.
- **Projection** (`cherry-pit-projection`, planned) — read model storage and
  query serving.

## MsgpackFileStore

File-based event store using MessagePack serialization. Stores each
aggregate's event stream as a single `.msgpack` file. Features:

- Named/map encoding via `rmp_serde::encode::to_vec_named` for
  forward-compatible schema evolution (`#[serde(default)]` on new fields)
- Per-aggregate write serialization via `tokio::sync::Mutex`
- Optimistic concurrency via `expected_sequence`
- Atomic writes via temp file + rename
- Path traversal protection on aggregate IDs
- Type-safe: parameterized by `E: DomainEvent`, cannot load wrong type
