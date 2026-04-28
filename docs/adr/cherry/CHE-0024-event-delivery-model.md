# CHE-0024. Event Delivery Model

Date: 2026-04-25
Last-reviewed: 2026-04-27
Tier: B
Status: Accepted

## Related

- References: CHE-0004, CHE-0017

## Context

Event delivery systems offer three guarantee levels, each with
different costs:

| Guarantee | Mechanism | Cost | Risk |
|-----------|-----------|------|------|
| At-most-once | Fire and forget | None | Lost events |
| At-least-once | ACK + retry | Retry logic, dedup | Duplicate events |
| Exactly-once | Distributed transaction | 2PC or idempotent consumers | Complexity, latency |

For event-sourced systems, exactly-once is typically achieved by
combining at-least-once delivery with idempotent consumers — the
event store provides the deduplication mechanism (events have unique
IDs and are immutable once stored).

`EventBus` defines `publish()` but no `subscribe()`. No code wires
policies or projections to the bus. The CommandBus (unbuilt) owns the
persist-then-publish lifecycle. The gap: how events flow from
`EventBus::publish()` to `Policy::react()` and `Projection::apply()`.

Cherry-pit's position: the `EventStore` provides the persistence
guarantee (events are durably stored). The `EventBus` provides
best-effort notification. If notification fails, consumers catch up
by replaying from the store. This is effectively at-least-once
delivery at the system level, even though the bus itself provides no
delivery guarantee.

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
