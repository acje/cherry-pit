# 24. Event Delivery Model

Date: 2026-04-25
Last-reviewed: 2026-04-25

## Status

Accepted

## Related

- Depends on: ADR 0004, ADR 0017
- Informs: ADR 0040

## Context

`EventBus` defines `publish()` but no `subscribe()`. No code wires
policies or projections to the bus. The CommandBus (unbuilt) owns the
persist-then-publish lifecycle. The gap: how events flow from
`EventBus::publish()` to `Policy::react()` and `Projection::apply()`.

## Decision

Persist-then-publish with non-fatal delivery:

1. **CommandBus calls `EventBus::publish()`** after successful
   persistence. Events are the source of truth once stored — publish
   is notification, not commit.
2. **Publication failure is non-fatal** — events are safely stored.
   Tracking-style processors catch up by replaying from the store.
3. **No `subscribe` on the port trait** — subscription is inherently
   implementation-specific (in-process channels vs NATS subjects vs
   polling).
4. **`pit-agent` wires the graph** — the planned composition layer
   registers policies and projections with concrete `EventBus`
   implementations. Manual wiring until `pit-agent` exists.
5. **In-process delivery is synchronous within `publish`** — the bus
   calls each registered handler before returning.

## Consequences

- No at-least-once delivery guarantee at the `EventBus` level.
  Guarantee comes from store-based replay (catch-up subscription).
- Cross-process delivery (NATS) deferred to Pardosa transport.
- Policies/projections that miss a publication can rebuild by
  replaying from the event store.
- The wiring layer (`pit-agent`) is the missing piece — its design
  determines how policies and projections are discovered, registered,
  and dispatched to.
