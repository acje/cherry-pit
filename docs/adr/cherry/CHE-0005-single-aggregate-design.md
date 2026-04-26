# CHE-0005. Single-Aggregate Design with Compile-Time Type Safety

Date: 2026-04-24
Last-reviewed: 2026-04-25
Tier: S

## Status

Accepted

Amended 2026-04-25 — added COM cross-reference

## Related

- Depends on: CHE-0004
- Illustrates: CHE-0001, COM-0002
- References: CHE-0011, CHE-0013

## Context

Infrastructure ports (EventStore, EventBus, CommandBus, CommandGateway)
need to handle aggregate-specific event and command types. Two design
approaches were considered:

1. **Generic per-call** — ports accept any event/command type per
   method call, with runtime type checking.
2. **Single-aggregate binding** — each port instance is locked to
   exactly one aggregate/event type via associated types, with
   compile-time enforcement.

Option 1 is flexible but allows cross-aggregate type confusion at
runtime (loading Order events as Inventory events, publishing to a bus
typed for a different event). Option 2 trades wiring verbosity for
compile-time safety.

## Decision

Every infrastructure port is bound to a single aggregate/event type via
associated types. Multiple aggregates require separate bounded contexts,
each with its own typed infrastructure stack. Cross-context communication
happens through event subscriptions (e.g. NATS subjects), not shared
stores.

Compile-fail tests in cherry-pit-core prove the guarantees hold.

## Consequences

- The compiler prevents cross-aggregate type confusion at every layer.
  No runtime downcasting, no `Any`, no type-erased envelopes.
- Wiring complexity scales linearly with aggregate count. `cherry-pit-agent`
  (builder API) must solve this ergonomically.
- Object safety is sacrificed — no `Box<dyn EventStore>`.
- Cross-aggregate coordination (sagas) is implemented via `Policy`
  traits reacting to events from one aggregate and emitting commands
  to another.
- `Policy::Output` is typed as `Send + Sync + 'static` without a
  `Command` bound, allowing flexible output enums.
