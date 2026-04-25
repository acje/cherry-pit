# 4. Event Sourcing with DDD and Hexagonal Architecture

Date: 2026-04-24
Last-reviewed: 2026-04-25

## Status

Accepted

## Related

- Depends on: ADR 0001
- Informs: ADR 0005, ADR 0006, ADR 0008, ADR 0009, ADR 0010, ADR 0014, ADR 0016, ADR 0024, ADR 0039

## Context

Cherry-pit is a composable systems-kernel for agent-first building.
Agents need full audit trails, replayable history, and decoupled I/O.
The kernel must handle "undifferentiated heavy lifting" — persistence,
transport, and fan-out — so users focus on domain logic.

Three architectural patterns were evaluated:

- **Event sourcing** provides a complete, immutable audit log and
  enables state reconstruction by replay.
- **Domain-Driven Design** provides aggregates as consistency
  boundaries with clear command/event semantics.
- **Hexagonal architecture (ports and adapters)** decouples domain
  logic from infrastructure, enabling testability and composability.

These patterns are mutually reinforcing: event sourcing supplies the
data model, DDD supplies the consistency model, and hexagonal
architecture supplies the integration model.

Alternative: CRUD with change-data-capture was considered but rejected
because it loses intent (commands) and makes replay non-deterministic.

## Decision

Cherry-pit combines event sourcing, DDD, and hexagonal architecture as
its foundational pattern. All domain logic lives behind trait-based
ports. All infrastructure lives in adapter crates. Events are the
source of truth; aggregates are the consistency boundary.

## Consequences

- Users must understand the event-sourcing mental model (commands →
  events → state, not direct mutation).
- Read models are eventually consistent (CQRS separation).
- Migrating away from event sourcing is extremely difficult once
  committed — but for a framework, this is the point: users opt in
  knowingly.
- The three patterns together provide auditability, replayability,
  testability, and composability.
