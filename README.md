# Cherry pit

A composable systems-kernel for agent-first building. Focus on domain
logic rather than system design and infrastructure.

Cherry-pit provides the undifferentiated heavy lifting — architecture
patterns, event infrastructure, message transport, web serving — as
composable components. You describe aggregates, commands, and events;
the pit handles persistence, transport, and fan-out.

## Design priorities

Every decision is evaluated against these priorities, in strict rank order:

1. **Correctness** — Make illegal states unrepresentable. Validated
   constructors enforce invariants in all build profiles, derive macros
   reject non-deterministic types at compile time, and every decode path
   must run the full verification suite — no opt-out. Total functions.
   `#![forbid(unsafe_code)]` in every crate.
2. **Secure** — Treat every input as hostile. Validate structurally at
   boundaries, bound resource consumption before allocation, and reject
   malformed data as a hard error — no lenient modes. Module boundaries
   hide design decisions; the crate DAG prevents domain code from depending
   on infrastructure.
3. **Energy efficient** — Do less work, not faster work. Prefer zero-copy
   reads over deserialization, exact-size buffers over reallocation, and
   compact wire representations over convenient ones. Avoid unnecessary
   allocations, cloning, and serialization.
4. **Response time** — Fast, but never at the cost of correctness. Bound
   latency with timeouts and circuit breakers that preserve read
   availability during write-path failures. Trade simplicity for throughput
   only when measured, not estimated.

## Architecture

Cherry-pit combines three architectural styles:

- **Domain-Driven Design** — aggregates, value objects, domain events, commands
- **Event-Driven Architecture** — commands produce events; events drive policies and projections
- **Hexagonal (Ports & Adapters)** — narrow, typed ports for all I/O

### Type safety by construction

Every infrastructure port is bound to a single aggregate type via
associated types. The compiler proves end-to-end that commands, events,
persistence, and publication all agree on the same types. Multiple
aggregates are supported by deploying separate bounded contexts — each
with its own typed infrastructure stack.

What cannot compile:

- Dispatching a command to an aggregate that doesn't handle it
- Loading one aggregate's events as another's
- Publishing events through a bus typed for a different aggregate
- Downcasting domain errors — `DispatchError<E>` preserves the exact type

## Status

Active development. `cherry-pit-core` traits are implemented and stable.
`cherry-pit-gateway` provides a working `MsgpackFileStore` event store with
atomic writes, process-level fencing, and optimistic concurrency. `pardosa`
has a complete fiber state machine and dragline (append-only log with fiber
lookup). `pardosa-genome` has the crate scaffold — traits (`GenomeSafe`,
`GenomeOrd`), binary format constants, error catalog, and derive macro — but
the serializer and deserializer are not yet implemented. Remaining
infrastructure crates (`cherry-pit-web`, `cherry-pit-projection`) are planned.

## Components

| Component      | Status      | Description                                         |
|----------------|-------------|-----------------------------------------------------|
| **cherry-pit-core**   | implemented | Aggregate, command, event, policy, projection traits. Port traits: CommandGateway, CommandBus, EventStore, EventBus |
| **cherry-pit-gateway**| implemented | `MsgpackFileStore` event store with atomic writes, process fencing, optimistic concurrency |
| **pardosa**    | implemented | Fiber state machine, dragline (append-only log), CRUD + migration operations. Persistence and NATS integration not yet built |
| **pardosa-genome** | scaffold | `GenomeSafe`/`GenomeOrd` traits, binary format constants, error catalog. Serializer and deserializer not yet implemented |
| **pardosa-genome-derive** | implemented | `#[derive(GenomeSafe)]` proc macro with compile-time serde attribute validation |
| **adr-fmt**    | implemented | ADR governance CLI: template validation, naming, relationship integrity, README index generation |
| **cherry-pit-web**    | planned     | Web serving adapter (axum)                          |
| **cherry-pit-projection** | planned | Read model storage and query serving                |

## Repository structure

```
cherry-pit/
├── crates/
│   ├── cherry-pit-core/       # Aggregate, command, event, port traits
│   ├── cherry-pit-gateway/    # EventStore implementations
│   ├── pardosa/               # EDA storage layer (fiber semantics)
│   ├── pardosa-genome/        # Binary serialization format
│   ├── pardosa-genome-derive/ # GenomeSafe derive macro
│   └── adr-fmt/               # ADR governance tool
├── docs/
│   ├── adr/                   # Architecture decision records (governed by adr-fmt)
│   ├── plans/                 # Ephemeral working drafts (consumed into code and ADRs)
│   └── glossary.md            # Domain vocabulary across all crates
└── Cargo.toml                 # Workspace manifest (edition 2024, rust 1.95+)
```

Licensed under [MIT](LICENSE).
