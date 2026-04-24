# Build plan: cherry-pit crates

Status: phases 1 and 4 (partial) implemented  
Rust edition: 2024 · rust-version: 1.85 (minimum for edition 2024)  
Resolver: 3

## Design decisions

### 1. Timestamp type: jiff

**Decision: use `jiff::Timestamp`.**

Rationale:
- jiff is the modern Rust datetime library (by BurntSushi)
- `Timestamp` is a UTC instant — equivalent to `DateTime<Utc>`
- Lossless RFC 9557/RFC 3339 serde roundtrips
- DST-safe arithmetic by default
- No need for separate `chrono-tz` for IANA zones

`EventEnvelope` uses `jiff::Timestamp` and `AggregateId`:
```rust
pub struct EventEnvelope<E: DomainEvent> {
    pub event_id: Uuid,
    pub aggregate_id: AggregateId,
    pub sequence: u64,
    pub timestamp: jiff::Timestamp,
    pub payload: E,
}
```

### 2. Event IDs: UUID v7

UUID v7 is time-ordered — natural sort by creation time. Perfect for
event IDs where chronological ordering is meaningful.

```toml
uuid = { version = "1", features = ["v7", "serde"] }
```

### 3. pardosa-genome-derive

Proc macros require a separate crate. `pardosa-genome-derive` is
included in the workspace plan.

### 4. No async_trait

All async port traits use RPITIT (`impl Future` in trait return
position). Requires Rust 1.75+ (we target 1.85+). Zero-cost: no
`Box<dyn Future>` heap allocation per dispatch.

## Crate dependency DAG

Build order follows the DAG — leaves first, dependents after.

```
                    ┌──────────────┐
                    │   pit-core   │  ← leaf, build first
                    └──────┬───────┘
                           │
              ┌────────────┼─────────────────┐
              │            │                 │
              ▼            ▼                 ▼
      ┌──────────────┐ ┌──────────┐  ┌──────────────┐
      │ pit-gateway  │ │ pardosa- │  │pit-projection│
      │              │ │ genome   │  │              │
      └──────┬───────┘ └────┬─────┘  └──────────────┘
             │              │
             │         ┌────┴──────┐
             │         │ pardosa-  │
             │         │ genome-   │
             │         │ derive    │
             │         └────┬──────┘
             │              │
             │         ┌────┴─────┐
             │         │ pardosa  │
             │         └──────────┘
             │
      ┌──────┴───────┐
      │   pit-web    │
      └──────────────┘
             │
      ┌──────┴───────┐
      │  pit-agent   │  ← depends on everything, build last
      └──────────────┘
```

## Phase 1: pit-core (foundational traits)

Zero external cherry-pit dependencies. Pure trait definitions +
EventEnvelope + error types. This is the crate everything else depends
on, so it ships first.

### Dependencies

| Crate      | Version | Features              | Why                                        |
|------------|---------|----------------------|--------------------------------------------|
| serde      | 1       | derive               | DomainEvent: Serialize + DeserializeOwned  |
| uuid       | 1       | v7, serde            | EventEnvelope.event_id                     |
| jiff       | 0.2     | serde                | EventEnvelope.timestamp                    |

### Dev dependencies

| Crate      | Version | Why                    |
|------------|---------|------------------------|
| trybuild   | 1       | Compile-fail tests     |
| serde_json | 1       | Serialization tests    |
| rmp-serde  | 1       | MsgPack serde tests    |

### Contents (implemented)

- `DomainEvent` trait
- `Command` trait
- `Aggregate` trait (with `type Event` associated type)
- `HandleCommand<C>` trait (with `type Error` per command)
- `EventEnvelope<E>` struct (UUID v7, `jiff::Timestamp`)
- `Policy` trait (with `type Event`, `type Output`)
- `Projection` trait (with `type Event`)
- `CommandGateway` trait (async, RPITIT, `type Aggregate`)
- `CommandBus` trait (async, RPITIT, `type Aggregate`)
- `EventStore` trait (async, RPITIT, `type Event`)
- `EventBus` trait (async, RPITIT, `type Event`)
- `DispatchError<E>` enum
- `DispatchResult<A, C>` type alias
- `StoreError` enum
- `BusError` struct

### Module layout

```
pit-core/
├── Cargo.toml
└── src/
    ├── lib.rs          # Re-exports
    ├── event.rs        # DomainEvent, EventEnvelope
    ├── command.rs      # Command
    ├── aggregate.rs    # Aggregate, HandleCommand
    ├── policy.rs       # Policy
    ├── projection.rs   # Projection
    ├── gateway.rs      # CommandGateway
    ├── bus.rs          # CommandBus, EventBus
    ├── store.rs        # EventStore
    └── error.rs        # DispatchError, StoreError, BusError
```

## Phase 2: pardosa-genome + pardosa-genome-derive

Append-only file format, log versioning, migration engine. Can be built
in parallel with pit-gateway since they share no dependencies beyond
pit-core.

### pardosa-genome dependencies

| Crate      | Version | Features | Why                            |
|------------|---------|----------|--------------------------------|
| serde      | 1       | derive   | Log entry serialization        |
| serde_json | 1       | —        | Default wire format            |
| sha2       | 0.11    | —        | Content integrity hashing      |
| bytes      | 1       | —        | Zero-copy byte buffers         |
| thiserror  | 2       | —        | Error types                    |
| jiff       | 0.2     | serde    | Log timestamps                 |

### pardosa-genome dev dependencies

| Crate    | Version | Why                    |
|----------|---------|------------------------|
| proptest | 1       | Property-based tests   |
| tempfile | 3       | Temp dirs for log tests|

### pardosa-genome-derive dependencies

| Crate      | Version | Why             |
|------------|---------|-----------------|
| syn        | 2       | Parsing         |
| quote      | 1       | Code generation |
| proc-macro2| 1       | Token streams   |

### pardosa-genome module layout

```
pardosa-genome/
├── Cargo.toml
└── src/
    ├── lib.rs          # Re-exports
    ├── log.rs          # Append-only log structure
    ├── entry.rs        # Log entry format
    ├── version.rs      # Log versioning
    ├── migration.rs    # Migration engine
    ├── integrity.rs    # SHA-256 content hashing
    └── error.rs        # GenomeError types
```

## Phase 3: pardosa (serialization + transport)

Depends on pardosa-genome and pit-core.

### Dependencies

| Crate      | Version | Features                      | Why                    |
|------------|---------|-------------------------------|------------------------|
| pit-core   | path    | —                             | DomainEvent trait      |
| pardosa-genome | path | —                            | File format            |
| serde      | 1       | derive                        | Serialization          |
| serde_json | 1       | —                             | Wire format            |
| tokio      | 1       | macros, rt-multi-thread, sync | Async runtime          |
| async-nats | 0.47    | —                             | NATS/JetStream client  |
| bytes      | 1       | —                             | Zero-copy buffers      |
| tracing    | 0.1     | —                             | Observability          |
| thiserror  | 2       | —                             | Error types            |

## Phase 4: pit-gateway (port implementations)

Concrete implementations of pit-core port traits. Currently provides
`MsgpackFileStore<E>` — a file-based event store with auto-increment
IDs, store-created envelopes, and optimistic concurrency.

### Dependencies

| Crate      | Version | Features              | Why                     |
|------------|---------|----------------------|-------------------------|
| pit-core   | path    | —                    | Port traits             |
| serde      | 1       | derive               | Event serialization     |
| rmp-serde  | 0.15    | —                    | MessagePack format      |
| scc        | 3       | —                    | Concurrent collections  |
| tokio      | 1       | fs                   | Async file I/O          |
| uuid       | 1       | v7, serde            | Event ID generation     |
| jiff       | 0.2     | serde                | Timestamps              |

### Dev dependencies

| Crate        | Version | Why                |
|--------------|---------|---------------------|
| tempfile     | 3       | Temp dirs for tests |
| futures-util | 0.3     | Concurrent test helpers |
| tokio        | 1       | macros, rt-multi-thread |

### Module layout

```
pit-gateway/
├── Cargo.toml
└── src/
    ├── lib.rs              # Re-exports
    └── event_store/
        ├── mod.rs          # Re-exports
        └── msgpack_file.rs # MsgpackFileStore<E> + 20 tests
```

## Phase 5: pit-web (HTTP adapter)

Web serving via axum. Inbound webhooks, query-side API serving.

### Dependencies

| Crate       | Version | Features                                 | Why                |
|-------------|---------|------------------------------------------|--------------------|
| pit-core    | path    | —                                        | Port traits        |
| axum        | 0.8     | http1, http2, tokio, json, ws            | Web framework      |
| tokio       | 1       | macros, net, rt-multi-thread, signal, sync, time | Runtime   |
| tower-http  | 0.6     | limit, trace                             | HTTP middleware     |
| serde       | 1       | derive                                   | Request/response    |
| serde_json  | 1       | —                                        | JSON bodies        |
| tracing     | 0.1     | —                                        | Observability      |
| thiserror   | 2       | —                                        | Error types        |
| futures-util| 0.3     | —                                        | WebSocket splitting |

### Dev dependencies

| Crate             | Version | Why                |
|-------------------|---------|---------------------|
| tokio-tungstenite | 0.29    | WebSocket tests    |
| reqwest           | 0.13    | HTTP client tests  |

## Phase 6: pit-projection (read models)

Read model storage and query serving.

### Dependencies

| Crate      | Version | Features       | Why                |
|------------|---------|----------------|--------------------|
| pit-core   | path    | —              | Projection trait   |
| serde      | 1       | derive         | Model serialization|
| serde_json | 1       | —              | Wire format        |
| tokio      | 1       | macros, sync   | Async runtime      |
| tracing    | 0.1     | —              | Observability      |
| thiserror  | 2       | —              | Error types        |

## Phase 7: pit-agent (agent surface)

Last to build. Depends on all other crates. Provides the composition
API that agents use to assemble a cherry-pit system.

### Dependencies

All workspace crates + selected external deps depending on final design.

## Workspace Cargo.toml structure

All shared dependencies are declared at workspace level via
`[workspace.dependencies]` and inherited by member crates using
`dep.workspace = true`. This ensures version consistency across the
workspace.

### Workspace members (build order)

1. `crates/pit-core`
2. `crates/pardosa-genome`
3. `crates/pardosa-genome-derive`
4. `crates/pardosa`
5. `crates/pit-gateway`
6. `crates/pit-web`
7. `crates/pit-projection`
8. `crates/pit-agent`

### Release profile

```toml
[profile.release]
lto = true
strip = true
codegen-units = 1
overflow-checks = true     # correctness > speed (P1)
```

### Clippy configuration

Per design priorities (P1 correctness):
```toml
[workspace.lints.clippy]
pedantic = { level = "warn", priority = -1 }
```

## Key design decisions

| Area                              | Decision                           | Reason                                         |
|-----------------------------------|------------------------------------|-------------------------------------------------|
| Timestamp type                    | `jiff::Timestamp`                  | Modern Rust datetime, lossless serde roundtrips |
| Event IDs                         | `uuid::Uuid` with v7 feature      | Time-ordered event IDs                          |
| Port trait generics               | Associated types on ports          | Single-aggregate design for compile-time safety |
| Aggregate identity                | `AggregateId(u64)` newtype         | Copy semantics, store-assigned, type-safe       |
| Envelope construction             | Store creates envelopes            | Eliminates redundancy, impossible to malform    |
| Aggregate lifecycle               | create/send split                  | ID not known until store assigns it             |
| Error types                       | Manual `Display`/`Error` impls     | No thiserror dependency in pit-core             |
| Async traits                      | RPITIT (`impl Future`)             | Zero-cost, no `Box<dyn Future>` allocation      |

## Open questions

1. **NATS client version** — async-nats 0.47 is current. Should pardosa
   depend on it directly or behind a feature flag for offline-first
   setups?
2. **pit-projection storage** — what backing store? In-memory for now,
   with trait-based port for future SQLite/PostgreSQL?
3. **pit-agent scope** — what exactly does the agent surface crate
   contain? Builder API? Configuration DSL? This needs design work.
4. **Feature flags** — which crates should have optional features vs
   always-on? E.g., pardosa could gate NATS behind a `nats` feature.
