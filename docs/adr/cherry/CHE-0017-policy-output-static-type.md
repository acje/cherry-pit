# CHE-0017. Policy Output as Static Associated Type

Date: 2026-04-25
Last-reviewed: 2026-04-25
Tier: B

## Status

Accepted

## Related

- References: CHE-0005

## Context

Policies react to domain events by producing outputs — typically
commands to dispatch to other aggregates. The output type must be
defined somewhere. Options:

1. **`Box<dyn Any>`** — fully erased output. Policies can produce
   anything. Infrastructure must downcast at runtime. No compile-time
   verification of output types.
2. **`Box<dyn Command>`** — type-erased but constrained to commands.
   Still requires runtime downcasting for dispatch. Heap allocation
   per output.
3. **Static associated type** — `type Output: Send + Sync + 'static`.
   The agent defines an enum of possible outputs. The compiler
   verifies exhaustive matching when infrastructure dispatches them.

## Decision

```rust
pub trait Policy: Send + Sync + 'static {
    type Event: DomainEvent;
    type Output: Send + Sync + 'static;
    fn react(&self, event: &EventEnvelope<Self::Event>) -> Vec<Self::Output>;
}
```

`Output` is a static associated type with minimal bounds
(`Send + Sync + 'static`). It is deliberately NOT bounded by
`Command`.

The agent defines an output enum:

```rust
enum OrderPolicyOutput {
    NotifyWarehouse(NotifyWarehouseCommand),
    UpdateInventory(AggregateId, UpdateInventoryCommand),
}
```

Infrastructure dispatches by matching on the enum — the compiler
verifies all variants are handled.

## Consequences

- No runtime type errors. The output enum is exhaustively matched at
  compile time. A new output variant causes compilation failures at
  all dispatch sites until handled.
- No heap allocation per output. The enum lives on the stack or
  inline in the `Vec`.
- A policy cannot accidentally produce an output type it was not
  designed to produce. The type system makes the boundary explicit.
- **`Output` is not bounded by `Command`.** This is intentional:
  policy outputs may need to carry additional context beyond the
  command itself (e.g., the target `AggregateId` for cross-aggregate
  dispatch, routing metadata for cross-context dispatch). Requiring
  `Command` on `Output` would force this context into the command
  type, conflating routing with intent.
- The infrastructure layer that dispatches policy outputs must know
  the output enum type. This creates a coupling between the policy
  and its dispatch wiring — resolved by `cherry-pit-agent` (the composition
  layer).
- Policies receive `EventEnvelope`, not raw events — they need
  metadata (`aggregate_id`, `timestamp`, `event_id`) to construct
  correctly targeted outputs.
