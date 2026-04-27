# Glossary

Domain vocabulary used across the cherry-pit workspace. Grouped by domain.

## Event sourcing & DDD (cherry-pit-core)

| Term | Meaning |
|------|---------|
| **Aggregate** | Consistency and transactional boundary. Reconstructs state by replaying events. The only place where business invariants are enforced. |
| **Command** | A request to change state — represents intent. May be accepted or rejected by an aggregate. Consumed on handling. |
| **Domain event** | An immutable fact — something that happened. Source of truth in event-sourced systems. |
| **HandleCommand** | Compile-time verified command→aggregate binding. Each pair produces zero or more events on success. |
| **EventEnvelope** | Infrastructure wrapper around a domain event. Adds metadata for ordering, routing, and idempotency (`event_id`, `aggregate_id`, `sequence`, `timestamp`, `correlation_id`, `causation_id`). |
| **Policy** | Reacts to events by producing commands. The mechanism for cross-aggregate and cross-context coordination (eventually consistent). |
| **Projection** | Read-optimized view built by folding events. The read side of CQRS — can be rebuilt from scratch at any time. |
| **Bounded context** | A boundary within which a domain model is defined and applicable. Enforces data isolation between contexts. |
| **AggregateId** | Stream partition key — auto-assigned `u64` wrapped in `NonZeroU64`. |
| **CorrelationContext** | Explicit correlation/causation propagation for tracing related events across aggregates. |

## Ports & adapters (cherry-pit-core)

| Term | Meaning |
|------|---------|
| **EventStore** | Port for loading and persisting a single aggregate's event streams. Single source of truth for aggregate state. |
| **EventBus** | Port for publishing events to downstream consumers (policies, projections, external integrations) after persistence. |
| **CommandBus** | Internal command routing and execution: load aggregate → handle command → persist events → publish envelopes. |
| **CommandGateway** | Primary entry point for dispatching commands. Outermost port on the driving side of the hexagon, adding cross-cutting concerns atop `CommandBus`. |
| **Adapter** | A component that connects domain ports to external systems — webhooks, APIs, databases, message brokers. |
| **DispatchError** | Errors from command dispatch: `Rejected` (business invariant violation), `AggregateNotFound`, `ConcurrencyConflict`, or `Infrastructure`. |

## Fiber semantics (pardosa)

| Term | Meaning |
|------|---------|
| **Fiber** | A single domain entity's event history — a singly linked list of immutable events interleaved in the dragline. |
| **FiberState** | Lifecycle state of a fiber: `Undefined`, `Defined`, `Detached`, `Purged`, `Locked`. |
| **FiberAction** | Action applied to a fiber: `Create`, `Update`, `Detach`, `Rescue`, `Migrate(policy)`. |
| **Dragline** | The core append-only log with fiber lookup. Contains the event line, fiber index, and bookkeeping state. |
| **Line** | The append-only sequence of events from all fibers, ordered by write time. |
| **MigrationPolicy** | Deletion policy during schema migration: `Keep`, `Purge` (removed, key reusable), `LockAndPrune` (pruned, key not reusable). |
| **Index** | Position in the append-only line — single `u64` (`u64::MAX` reserved as `NONE` sentinel). |
| **DomainId** | Unique identifier for a domain entity / fiber — single `u64`. |

## Binary serialization (pardosa-genome)

| Term | Meaning |
|------|---------|
| **GenomeSafe** | Marker trait enforcing deterministic, fixed-layout binary serialization at compile time. Carries `SCHEMA_HASH` and `SCHEMA_SOURCE`. |
| **GenomeOrd** | Marker trait for types with a deterministic total `Ord` — suitable for `BTreeMap` keys in genome-encoded data. |
| **PageClass** | Per-message element budget stored in file headers: `Page0` (256) through `Page3` (1,048,576) via `256 × 16^N`. |
| **SCHEMA_HASH** | xxHash64 fingerprint of a type's canonical representation. Used for schema evolution detection. |
| **SCHEMA_SOURCE** | Human-readable Rust type definition embedded in file headers for tooling and debugging. |
