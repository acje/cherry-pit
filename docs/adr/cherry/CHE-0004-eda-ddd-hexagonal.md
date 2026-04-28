# CHE-0004. Event-Driven Architecture with DDD and Hexagonal Architecture

Date: 2026-04-24
Last-reviewed: 2026-04-27
Tier: S
Status: Accepted

## Related

References: CHE-0001

## Context

Cherry-pit is a composable systems-kernel for agent-first building.
Three forces are in tension: audit completeness (agents need full,
replayable history of every state change), domain model fidelity
(command intent must be preserved separately from state mutations),
and infrastructure decoupling (domain code must not know about
serialization, databases, or message brokers).

CRUD+CDC loses command intent. Bi-temporal databases preserve
temporal state but do not decompose into aggregates and bounded
contexts. State-based audit logs duplicate data with no consistency
guarantee.

The chosen approach composes three mutually reinforcing patterns.
Event-Driven Architecture (EDA) structures the system around events
as the primary communication mechanism, with event sourcing
providing full audit trails and state reconstruction by replay.
Domain-Driven Design provides aggregates as consistency boundaries
with clear command/event semantics. Hexagonal architecture (ports
and adapters) decouples domain logic from infrastructure, enabling
testability and composability. EDA supplies the data model, DDD
the consistency model, and hexagonal architecture the integration
model.

## Decision

Cherry-pit combines EDA, DDD, and hexagonal architecture as
its foundational pattern. All domain logic lives behind trait-based
ports. All infrastructure lives in adapter crates. Events are the
source of truth; aggregates are the consistency boundary.

R1 [1]: Use events as the source of truth for all state, persisted as
  an immutable append-only log
R2 [1]: Place all domain logic behind trait-based ports and all
  infrastructure in adapter crates
R3 [1]: Use aggregates as the consistency boundary for all write
  operations

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
