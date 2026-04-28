# CHE-0024. Event Delivery Model

Date: 2026-04-25
Last-reviewed: 2026-04-28
Tier: C
Status: Accepted

## Related

References: CHE-0001, CHE-0004, CHE-0017

## Context

Event delivery systems offer three guarantee levels: at-most-once (fire and forget, risk of lost events), at-least-once (ACK + retry, risk of duplicates), and exactly-once (distributed transaction, high complexity). `EventBus` defines `publish()` but no `subscribe()`. No code wires policies or projections to the bus. The `CommandBus` (unbuilt) owns the persist-then-publish lifecycle. Cherry-pit's position: the `EventStore` provides durable persistence, the `EventBus` provides best-effort notification, and consumers catch up by replaying from the store — effectively at-least-once at the system level.

## Decision

Persist-then-publish with non-fatal delivery:

R1 [7]: Persist events before publishing; publication failure is
  non-fatal because events are safely stored
R2 [7]: No subscribe method on the EventBus port trait; subscription
  is implementation-specific
R3 [7]: Consumers that miss a publication catch up by replaying from
  the event store

1. **CommandBus calls `EventBus::publish()`** after successful
   persistence. Events are the source of truth once stored — publish
   is notification, not commit.
2. **Publication failure is non-fatal** — events are safely stored.
   Tracking-style processors catch up by replaying from the store.
3. **No `subscribe` on the port trait** — subscription is inherently
   implementation-specific (in-process channels vs NATS subjects vs
   polling).
4. **`cherry-pit-agent` wires the graph** — the planned composition layer
   registers policies and projections with concrete `EventBus`
   implementations. Manual wiring until `cherry-pit-agent` exists.
5. **In-process delivery is synchronous within `publish`** — the bus
   calls each registered handler before returning.

## Consequences

- No at-least-once delivery guarantee at the `EventBus` level.
  Guarantee comes from store-based replay (catch-up subscription).
- Cross-process delivery (NATS) deferred to Pardosa transport.
- Policies/projections that miss a publication can rebuild by
  replaying from the event store.
- The wiring layer (`cherry-pit-agent`) is the missing piece — its design
  determines how policies and projections are discovered, registered,
  and dispatched to.
