# Cherry pit

A composable systems-kernel for agent-first building. Focus on domain logic
rather than system design and infrastructure.

The cherry pit provides the undifferentiated heavy lifting — architecture patterns,
event infrastructure, message transport, web serving — as composable
components. Agents select which components to include, configure them for
the target runtime environment, and build adapters for existing systems.

Like a cherry, the value grows around the pit.

Without a pit, you wire event stores, build serialization layers, debug
message transport, and design concurrency control — before writing a single
line of domain logic. Cherry-pit is designed to provide that foundation.
You describe aggregates, commands, and events; the pit handles persistence,
transport, and fan-out.

## Who is this for?

For developers and architects building event-sourced systems in Rust, and
for domain experts who want to ship business value without negotiating
infrastructure. Cherry-pit is designed so that AI agents can compose and
configure the system on their behalf.

New to these terms? See [Key concepts](docs/glossary.md).

## Why cherry-pit?

Cherry-pit is not a framework you install and configure. It is a set of
composable crates. Select the components your system needs, leave the rest
out. No runtime cost for capabilities you don't use.

Cherry-pit is designed to be composed programmatically. AI agents select
components, generate domain models, and wire adapters — the same workflow
a human developer follows, made automatable by narrow, typed interfaces.

## Status

Design phase. This repository is the authoritative design document and will
become the cargo workspace as components stabilize.

## Design priorities

Every design decision is evaluated against these priorities, in strict
rank order:

1. **Correctness** — Make invalid states unrepresentable. Lean on the type
   system to reject wrong code at compile time, not at runtime. Total
   functions. No unsafe code. Use latest versions of crates and run clippy
   pedantic after completed plans. Use idiomatic rust, DDD and EDA-architecture.
2. **Secure** — No accidental data leakage across bounded contexts. Validate
   at the boundary. No unsafe unless proven necessary and audited.
3. **Energy efficient** — Do less work, not faster work. Avoid unnecessary
   allocations, cloning, and serialization. Prefer borrowing over owning.
4. **Response time** — Fast, but never at the cost of correctness, security,
   or energy.

When priorities conflict, higher-ranked concerns win.

## Architectural styles

cherry-pit combines three architectural styles:

- Business level: **Domain-Driven Design** — Aggregates, value objects, domain events,
  commands, bounded contexts.
- Systems level: **Event-Driven Architecture** — Commands produce events. Events drive
  policies, projections, and integrations.
- Component level: **Hexagonal (Ports & Adapters)** — Narrow, well-typed ports for all I/O.
  Domain logic never touches infrastructure directly.

```text
                        ┌─────────────────────────────────┐
                        │         Agent Surface           │
                        │                                 │
                        │  1. Compose the pit             │
                        │  2. Describe the domain         │
                        │  3. Build brownfield adapters   │
                        └────────────┬──────┬─────────────┘
                                     │      │
                   ┌─────────────────┘      └───────────────────┐
                   ▼                                            ▼
          ┌──────────────────┐                        ┌──────────────────┐
          │  Domain Ports    │                        │  Adapter Toolkit │
          │  (narrow, typed, │                        │  (traits, build- │
          │   opinionated)   │                        │   ing blocks for │
          │                  │                        │   brownfield     │
          │                  │                        │   integrations)  │
          └────────┬─────────┘                        └─────────┬────────┘
                   │                                            │
                   └───────────────────┬────────────────────────┘
                                       ▼
    ┌────────────────────────────────────────────────────────────────────┐
    │                   THE CHERRY PIT (composable)                      │
    │                                                                    │
    │  ┌─────────────┐  ┌─────────────┐  ┌───────────┐  ┌─────────────┐  │
    │  │  DDD Core   │  │   Pardosa   │  │   NATS /  │  │    Infra    │  │
    │  │             │  │             │  │ Jetstream │  │             │  │
    │  │ Aggregates  │  │ Serde events│  │           │  │ Gateway     │  │
    │  │ Commands    │  │ Append-only │  │ Message   │  │ Web serving │  │
    │  │ Events      │  │ logs        │  │ transport │  │ Webhooks    │  │
    │  │ Policies    │  │ Schema evo  │  │ Stream    │  │ Projections │  │
    │  │ Projections │  │ Log migra-  │  │ persist   │  │ Persistence │  │
    │  │ Gateway port│  │ tion        │  │           │  │             │  │
    │  │ Bus port    │  │             │  │           │  │             │  │
    │  │ Store port  │  │             │  │           │  │             │  │
    │  │ EventBus    │  │             │  │           │  │             │  │
    │  └─────────────┘  └─────────────┘  └───────────┘  └─────────────┘  │
    │                                                                    │
    └────────────────────────────────────────────────────────────────────┘
```

## Components

| Component          | Required | Description                                           |
|------------------  |----------|------------------------------------------------------ |
| **DDD Core**       | always   | Aggregate, command, event, policy, projection traits. |
|                    |          | Port traits: CommandGateway, CommandBus (driving);    |
|                    |          | EventStore, EventBus (driven)                         |
| **Pardosa**        | optional | Event serialization and append-only logs. Best fit    |
|                    |          | for ECST patterns where data deletion is required     |
| **NATS/Jetstream** | optional | Message transport and stream persistence              |
| **Infra**          | optional | Gateway impl, web serving, projection storage,        |
|                    |          | data aggregation                                      |

## Agent-first design

The agent's responsibilities span three layers:

1. **Compose the pit** — select pit-* components, configure for the target runtime
2. **Describe the domain** — work with the domain expert to define aggregates, commands, events, policies, projections
3. **Build brownfield adapters** — integrate with existing systems using the adapter toolkit

## Documentation

| Document | Contents |
|----------|----------|
| [pit-core trait design](docs/pit-core.md) | DomainEvent, Command, Aggregate, HandleCommand, EventEnvelope, CommandGateway, CommandBus, DispatchError, EventStore, StoreError, EventBus, BusError, Policy, Projection traits with design rationale |
| [Pardosa](docs/pardosa.md) | Event serialization, transport, append-only log format, schema evolution |
| [Infrastructure crates](docs/infrastructure.md) | Adapted crates for gateway, web, projection, aggregation |
| [Key concepts](docs/glossary.md) | Glossary of DDD and EDA terms used in this project |

## Get involved

Cherry-pit is in design phase. Feedback on trait design, architecture
decisions, and use cases is welcome.
[Open an issue](https://github.com/acje/cherry-pit/issues) to start a
conversation.

## Repository structure (planned)

```sh
cherry-pit/
├── README.md
├── docs/
│   ├── pit-core.md            # Trait design and rationale
│   ├── pardosa.md             # Serialization and transport
│   ├── infrastructure.md      # Infrastructure crate catalogue
│   ├── architecture.dot       # System architecture (Graphviz)
│   ├── hexagonal.dot          # Ports and adapters diagram
│   ├── event-flow.dot         # Event lifecycle diagram
│   └── migration.dot          # Pardosa log migration diagram
├── crates/
│   ├── pit-core/              # Aggregate, command, event, gateway, bus, store traits
│   ├── pit-agent/             # Agent surface and adapter toolkit
│   ├── pardosa/               # Event serializer and transport
│   ├── pardosa-genome/        # Append-only file format and migration
│   ├── pit-gateway/           # CommandGateway and CommandBus impls, adapters
│   ├── pit-web/               # Web serving adapter
│   └── pit-projection/        # Read model storage and query serving
├── examples/
├── Cargo.toml                 # Workspace manifest
└── LICENSE
```

Only `docs/` and `LICENSE` exist today. Crate directories will appear as
components stabilize.

Licensed under [MIT](LICENSE).
