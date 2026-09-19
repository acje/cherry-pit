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
