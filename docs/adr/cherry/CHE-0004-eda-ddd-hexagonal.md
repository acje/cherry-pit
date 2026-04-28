# CHE-0004. Event-Driven Architecture with DDD and Hexagonal Architecture

Date: 2026-04-24
Last-reviewed: 2026-04-27
Tier: S
Status: Accepted

## Related

- References: CHE-0001

## Context

Cherry-pit is a composable systems-kernel for agent-first building.
Three forces are in tension:

1. **Audit completeness.** Agents require full, replayable history of
   every state change. Audit is not a bolt-on; it is the primary data
   model. Any architecture that does not store the complete causal
   chain of state transitions fails this requirement.

2. **Domain model fidelity.** The system must preserve command intent
   (what the user asked for) separately from state changes (what
   happened). CRUD systems collapse intent into state mutations —
   the "why" is lost, making replay non-deterministic and debugging
   forensically impossible.

3. **Infrastructure decoupling.** The kernel handles "undifferentiated
   heavy lifting" — persistence, transport, and fan-out — so users
   focus on domain logic. Domain code must not know about
   serialization formats, database schemas, or message brokers.

Four architectural approaches were evaluated:

| Approach | Audit | Intent | Decoupling | Complexity |
|----------|-------|--------|------------|------------|
| EDA + Event Sourcing + DDD + Hex | Full | Preserved | Full | High |
| CRUD + Change Data Capture | Partial | Lost | Partial | Medium |
| Bi-temporal Database | Full | Partial | Low | High |
| State-based + Audit Log | Partial | Partial | Partial | Low |

CRUD+CDC loses command intent: a CDC record says "field X changed
to Y" but not "user issued command Z that caused the change."
Bi-temporal databases preserve temporal state but do not naturally
decompose into aggregates and bounded contexts. State-based+audit-log
duplicates data (state + log) with no guarantee of consistency
between them.

The chosen approach composes three mutually reinforcing patterns:

- **Event-Driven Architecture (EDA)** structures the system around
  events as the primary communication and coordination mechanism.
  Event sourcing — persisting all state changes as an immutable
  event log — is a core pattern within this approach, providing
  full audit trails and state reconstruction by replay.
- **Domain-Driven Design** provides aggregates as consistency
  boundaries with clear command/event semantics.
- **Hexagonal architecture (ports and adapters)** decouples domain
  logic from infrastructure, enabling testability and composability.

EDA supplies the communication and data model, DDD supplies the
consistency model, and hexagonal architecture supplies the
integration model.

## Decision

Cherry-pit combines EDA, DDD, and hexagonal architecture as
its foundational pattern. All domain logic lives behind trait-based
ports. All infrastructure lives in adapter crates. Events are the
source of truth; aggregates are the consistency boundary.

## Consequences

- Users must understand the event-driven mental model (commands →
  events → state, not direct mutation) and the event-sourcing
  pattern that underpins it.
- Read models are eventually consistent (CQRS separation).
- Migrating away from event-driven architecture and event sourcing
  is extremely difficult once committed — but for a framework,
  this is the point: users opt in knowingly.
- The three patterns together provide auditability, replayability,
  testability, and composability.
- **Steep learning curve.** EDA + event sourcing + DDD is one of the
  most conceptually demanding architectural patterns in software
  engineering. Developers must internalize commands, events,
  aggregates, projections, policies, and eventual consistency before
  being productive. This is the primary adoption barrier.
- **Eventual consistency is inherent.** Read models are projections
  rebuilt from events. They are always eventually consistent with
  the write model. Developers accustomed to strong consistency
  (read-after-write) must adapt their mental model. No amount of
  framework design can eliminate this fundamental property.
- **Event schema is append-only forever.** Once an event type is
  persisted, it cannot be removed or renamed without migration
  infrastructure. The event log is an append-only ledger of the
  system's entire history — every schema decision is permanent.
