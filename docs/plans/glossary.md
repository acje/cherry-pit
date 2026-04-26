# Key concepts

Terms used throughout cherry-pit documentation. Definitions match the
trait designs in [cherry-pit-core](cherry-pit-core.md).

| Term | Meaning |
|------|---------|
| **Aggregate** | The consistency and transactional boundary. Reconstructs its state by replaying events. The only place where business invariants are enforced. |
| **Command** | A request to change state — represents intent. May be accepted or rejected by an aggregate. |
| **Domain event** | An immutable fact — something that happened. Source of truth in event-sourced systems. |
| **Policy** | Reacts to events by producing commands. The mechanism for cross-aggregate and cross-context coordination. |
| **Projection** | A read-optimized view built by folding events. Can be rebuilt from scratch at any time. |
| **Bounded context** | A boundary within which a domain model is defined and applicable. Enforces data isolation between contexts. |
| **Event store** | The persistence layer for aggregate event streams. Single source of truth in event-sourced systems. |
| **Adapter** | A component that connects domain ports to external systems — webhooks, APIs, databases, message brokers. |
