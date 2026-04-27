# CHE-0005. Single-Aggregate Design with Compile-Time Type Safety

Date: 2026-04-24
Last-reviewed: 2026-04-27
Tier: S

## Status

Accepted

Amended 2026-04-25 — added COM cross-reference
Amended 2026-04-27 — expanded alternatives analysis and object-safety
  consequences

## Related

- References: CHE-0001, CHE-0004, CHE-0011, CHE-0013, COM-0002

## Context

Infrastructure ports (EventStore, EventBus, CommandBus, CommandGateway)
need to handle aggregate-specific event and command types. Two design
approaches were considered:

1. **Generic per-call (type-erased).** Ports accept any event/command
   type per method call, with runtime type checking.

   ```rust
   trait EventStore {
       fn load<E: DomainEvent>(&self, id: AggregateId) -> Vec<EventEnvelope<E>>;
   }
   ```

   The store accepts any event type per call. Callers choose the type
   at each call site. Runtime type checking prevents cross-aggregate
   confusion, but errors are discovered at runtime (deserialization
   failure), not compile time. The store is object-safe
   (`Box<dyn EventStore>`), enabling runtime polymorphism.

2. **Single-aggregate binding (associated types).** Each port instance
   is locked to exactly one aggregate/event type via associated types,
   with compile-time enforcement.

   ```rust
   trait EventStore {
       type Event: DomainEvent;
       fn load(&self, id: AggregateId) -> Vec<EventEnvelope<Self::Event>>;
   }
   ```

   Each store instance is locked to one aggregate/event type.
   Cross-aggregate confusion is a compile error, not a runtime error.
   The store is NOT object-safe — `Box<dyn EventStore>` is impossible
   because the associated type prevents type erasure.

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
  `EventStore`, `EventBus`, `CommandBus`, and `CommandGateway` cannot
  be used as trait objects. This means: no heterogeneous collections
  of stores for different aggregate types, no runtime selection of
  store implementations based on configuration, and every aggregate
  type requires its own concrete store instance wired at compile time.
  The `cherry-pit-agent` builder API must solve the wiring ergonomics
  so users don't manually construct typed infrastructure stacks for
  each aggregate.
- Cross-aggregate coordination (sagas) is implemented via `Policy`
  traits reacting to events from one aggregate and emitting commands
  to another.
- `Policy::Output` is typed as `Send + Sync + 'static` without a
  `Command` bound, allowing flexible output enums.
