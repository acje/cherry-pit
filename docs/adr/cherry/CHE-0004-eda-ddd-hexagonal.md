# CHE-0004. Event-Driven Architecture with DDD and Hexagonal Architecture

Date: 2026-04-24
Last-reviewed: 2026-04-25
Tier: S

## Status

Accepted

## Related

- Depends on: CHE-0001

## Context

Cherry-pit is a composable systems-kernel for agent-first building.
Agents need full audit trails, replayable history, and decoupled I/O.
The kernel must handle "undifferentiated heavy lifting" — persistence,
transport, and fan-out — so users focus on domain logic.

Three architectural patterns were evaluated:

- **Event-Driven Architecture (EDA)** structures the system around
  events as the primary communication and coordination mechanism.
  Event sourcing — persisting all state changes as an immutable
  event log — is a core pattern within this approach, providing
  full audit trails and state reconstruction by replay.
- **Domain-Driven Design** provides aggregates as consistency
  boundaries with clear command/event semantics.
- **Hexagonal architecture (ports and adapters)** decouples domain
  logic from infrastructure, enabling testability and composability.

These patterns are mutually reinforcing: EDA supplies the
communication and data model, DDD supplies the consistency model, and
hexagonal architecture supplies the integration model.

Alternative: CRUD with change-data-capture was considered but rejected
because it loses intent (commands) and makes replay non-deterministic.

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
