# CHE-0009. Infallible apply() on Aggregate and Projection

Date: 2026-04-24
Last-reviewed: 2026-04-25
Tier: A

## Status

Accepted

Amended 2026-04-25 — added COM cross-reference

## Related

- Depends on: CHE-0004
- Illustrates: COM-0005

## Context

In an event-sourced system, aggregate state is reconstructed by
replaying all historical events through an `apply` method. If `apply`
can return an error:

- Replaying history could fail, making aggregates unloadable.
- The fundamental event-sourcing guarantee (state = f(events))
  collapses.
- Error handling during replay introduces complexity with no clear
  recovery path — the events are already persisted and immutable.

The same reasoning applies to projections, which fold events into
read-optimized views and must be rebuildable from scratch.

## Decision

Both `Aggregate::apply(&mut self, event: &Event)` and
`Projection::apply(&mut self, event: &EventEnvelope<Event>)` return
`()`. They cannot fail. If they encounter a truly unhandleable event
(corrupted data, unknown variant), the only valid response is a panic
— this represents a bug, not a runtime condition.

## Consequences

- Event replay always succeeds — aggregate state reconstruction is
  guaranteed.
- Projections can always be rebuilt from scratch by replaying the full
  event history.
- Schema evolution creates pressure: when event enums grow new
  variants, all existing `apply` implementations must handle them.
  Pardosa's planned migration-time pruning addresses this; until then,
  users must handle all variants forever.
- Panic is the only error path for truly corrupt data. The CommandBus
  (when implemented) must decide how to handle panics during replay:
  propagate, wrap in `Infrastructure` error, or `catch_unwind`.
- The asymmetry between `Aggregate::apply(&Event)` and
  `Projection::apply(&EventEnvelope<Event>)` is intentional:
  projections need metadata (timestamp, aggregate_id) for time-based
  views; aggregates don't care about envelope metadata.
