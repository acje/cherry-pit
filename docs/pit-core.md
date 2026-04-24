# pit-core: trait design

pit-core defines the foundational traits that every cherry-pit system
implements. These are the narrow, typed ports that agents program against.
All domain logic lives behind these traits. All infrastructure lives on
the other side.

## Single-aggregate design

Every infrastructure port (`EventStore`, `EventBus`, `CommandBus`,
`CommandGateway`) is bound to a single aggregate/event type via
associated types. The compiler enforces end-to-end type safety from
command dispatch through event persistence and publication.

Multiple aggregates are supported at the system level by deploying
separate bounded contexts — each with its own typed infrastructure
stack. Cross-context communication happens through event subscriptions
(e.g. NATS subjects), not shared stores.

This design makes illegal states unrepresentable: you cannot load one
aggregate's events as another's, publish to a bus typed for a different
event, or dispatch a command through a gateway bound to the wrong
aggregate. These errors are rejected at compile time, not at runtime.

## Single-writer aggregates

Cherry-pit assumes single-writer aggregates: each aggregate instance
is owned by exactly one process. No distributed coordination is
needed — the owning process serializes commands internally.

This enables sequential `u64` aggregate IDs without distributed ID
generation. Optimistic concurrency (`expected_sequence` on `append`)
serves as defense-in-depth within the single writer.

## AggregateId

```rust
/// Validated aggregate instance identifier — the stream partition key.
///
/// Identifies a specific aggregate instance within an event store.
/// Each aggregate's event stream is keyed by its AggregateId.
/// The (AggregateId, sequence) tuple is the globally unique
/// coordinate for any single event.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord,
         Serialize, Deserialize)]
pub struct AggregateId(u64);
```

**Design rationale:**
- **Copy semantics** — `u64` is cheap to copy. No `.clone()`, no
  references, no lifetimes. Pass by value everywhere.
- **Type safety** — prevents mixing aggregate IDs with other `u64`
  values (sequence numbers, counts, etc.) at compile time.
- **Store-assigned** — IDs are auto-incremented by the
  [`EventStore::create`] method starting from 1. Callers never
  invent IDs. `AggregateId(0)` is valid at the type level but
  never assigned.
- **No validation** — all `u64` values are valid. No `Result`, no
  error type, no runtime checks.
- **Serde** — serializes as a plain `u64` for compact wire/storage
  representation.

## DomainEvent

```rust
/// Marker trait for domain events.
///
/// Events are immutable facts — something that happened. They are the
/// source of truth in an event-sourced system. Every event must be
/// serializable (for persistence/transport) and cloneable (for fan-out
/// to multiple consumers).
pub trait DomainEvent:
    Serialize + DeserializeOwned + Clone + Send + Sync + 'static
{
    /// A stable string identifier for this event type.
    /// Used for routing, schema registry, and deserialization dispatch.
    /// Must not change once events of this type exist in a log.
    fn event_type(&self) -> &'static str;
}
```

**Design rationale:**
- `Serialize + DeserializeOwned` — events cross process boundaries
  (Pardosa, NATS). Serde is non-negotiable.
- `Clone` — events fan out to projections, policies, and integrations
  without shared ownership.
- `event_type()` — a stable discriminator. Enums give you this at the
  Rust level, but serialized logs and external consumers need a string
  key that survives refactoring.

## Command

```rust
/// Marker trait for commands.
///
/// Commands represent intent — a request to change state. A command
/// may be rejected. Commands are consumed on handling (moved, not
/// borrowed) because they represent a one-time intent.
pub trait Command: Send + Sync + 'static {}
```

**Design rationale:**
- **Correctness (P1):** Command routing is enforced by the
  `HandleCommand<C>` trait on the aggregate side, not by an associated
  type on the command. The compiler rejects any attempt to send a
  command to an aggregate that doesn't implement `HandleCommand<C>`
  for it.
- **Energy (P3):** Commands are not required to be serializable by
  default. Only commands that cross process boundaries (via NATS) need
  serde derives. In-process commands avoid serialization overhead
  entirely.
- The trait is deliberately minimal — a marker with thread-safety
  bounds. All behavior lives in `HandleCommand`.

## Aggregate and HandleCommand

The generic `handle` method is split into a separate `HandleCommand` trait
so the compiler verifies each command→aggregate pair individually. No
runtime dispatch, no forgotten match arms.

```rust
/// The aggregate root — the consistency and transactional boundary.
///
/// An aggregate reconstructs its state by replaying events. It is the
/// only place where business invariants are enforced. The aggregate
/// itself only knows how to apply events — command handling is added
/// via the HandleCommand trait.
pub trait Aggregate: Default + Send + Sync + 'static {
    /// Events this aggregate produces and is reconstructed from.
    type Event: DomainEvent;

    /// Apply an event to update internal state.
    ///
    /// Must be deterministic and total — it must never fail. This
    /// method is called during state reconstruction (replaying history)
    /// and after handling new commands. If apply could fail, the
    /// aggregate's history would become unloadable.
    fn apply(&mut self, event: &Self::Event);
}

/// Command handling is a separate trait so each command→aggregate
/// pair is verified at compile time. An aggregate implements
/// HandleCommand once per command type it accepts.
pub trait HandleCommand<C: Command>: Aggregate {
    /// Domain-specific error for invariant violations.
    type Error: Error + Send + Sync;

    /// Handle a command against the current state.
    ///
    /// Returns zero or more events on success. Zero events means the
    /// command was accepted but no state change occurred (idempotent).
    /// Must be pure — no I/O, no side effects.
    fn handle(&self, cmd: C) -> Result<Vec<Self::Event>, Self::Error>;
}
```

**Design rationale:**
- **Correctness (P1):** Splitting `handle` into `HandleCommand<C>` means
  each command type is a separate impl block. The compiler guarantees
  exhaustive handling — you cannot forget to implement a command. No
  runtime downcasting, no match-arm gaps.
- `Default` — the aggregate starts as a blank slate. State is built
  entirely by replaying events through `apply`. There is no constructor
  with arguments.
- `handle` takes `self` by shared reference — the aggregate inspects its
  current state but does not mutate directly. State changes happen only
  through events returned by `handle`, then applied via `apply`.
- `handle` takes ownership of the command — a command represents one-time
  intent. After handling, it is consumed.
- `apply` takes `&Self::Event` by reference — the event is stored
  separately by the infrastructure. The aggregate just updates its
  in-memory state from it.
- **Energy (P3):** `Error` is an associated type on `HandleCommand`, not
  on `Aggregate` — different commands may have different error types,
  and aggregates that don't handle a given command pay no cost for its
  error type.
- No `id()` method on the trait — aggregate identity is managed by the
  infrastructure layer (event store, repository). The store assigns
  `AggregateId` values on creation. If the domain logic needs its own
  ID, it stores it as a field set during the first event's `apply`.

## EventEnvelope

```rust
/// Infrastructure wrapper around a domain event.
///
/// Provided by pit-core, not implemented by the agent. This is what
/// gets persisted and transported. The domain event is the payload;
/// the envelope adds the metadata needed for ordering, routing, and
/// idempotency.
///
/// Envelopes are created by the EventStore during create and append
/// — callers pass raw domain events, the store stamps on the metadata.
pub struct EventEnvelope<E: DomainEvent> {
    pub event_id: Uuid,              // UUID v7, time-ordered
    pub aggregate_id: AggregateId,   // stream partition key
    pub sequence: u64,               // monotonic within aggregate stream
    pub timestamp: jiff::Timestamp,  // UTC instant (single per batch)
    pub payload: E,
}
```

**Design rationale:**
- `sequence` — monotonically increasing within an aggregate's stream.
  Enables optimistic concurrency and ordered replay.
- `aggregate_id` + `sequence` together form the unique position in the
  aggregate's history.
- The agent never constructs envelopes. The `EventStore` creates them
  when persisting events returned by `handle`. This eliminates
  redundancy between method parameters and envelope fields, and makes
  malformed envelopes impossible by construction.
- `timestamp` — single timestamp per batch. A batch (all events from
  one `create` or `append` call) is an atomic operation and shares
  one timestamp.

## Policy

```rust
/// A policy reacts to domain events by producing commands.
///
/// Policies are the mechanism for cross-aggregate and cross-context
/// coordination. They observe what happened (events) and decide what
/// should happen next (commands). Policies are eventually consistent
/// by nature.
///
/// The Output type is an enum defined by the agent that encompasses
/// all command types this policy can emit. This keeps dispatch static
/// and avoids heap allocation.
pub trait Policy: Send + Sync + 'static {
    /// The event type this policy reacts to.
    type Event: DomainEvent;

    /// The output type — typically an enum of possible commands.
    type Output: Send + Sync + 'static;

    /// React to an event. Returns zero or more outputs to dispatch.
    ///
    /// An empty vec means this event is not relevant to this policy.
    /// Policies must be idempotent — reacting to the same event
    /// twice must produce the same outputs.
    fn react(
        &self,
        event: &EventEnvelope<Self::Event>,
    ) -> Vec<Self::Output>;
}
```

**Design rationale:**
- **Correctness (P1):** `Output` is a static associated type, not
  `Box<dyn AnyCommand>`. The agent defines an enum of possible
  command outputs, and the compiler verifies exhaustive matching when
  the infrastructure dispatches them. No runtime type errors.
- **Energy (P3):** No heap allocation per emitted command. The enum
  lives on the stack or inline in the Vec.
- **Security (P2):** Static output types mean a policy cannot
  accidentally emit a command type it was not designed to produce.
  The boundary is explicit in the type system.
- Policies receive `EventEnvelope`, not raw events — they often need
  metadata (timestamp, aggregate_id) to construct correctly targeted
  commands.
- Idempotency requirement — since event delivery may be at-least-once
  (especially over NATS), policies must tolerate replays.

## Projection

```rust
/// A projection folds events into a query-optimized read model.
///
/// Projections are the read side of CQRS. They consume events and
/// build denormalized views optimized for specific query patterns.
/// Like apply on Aggregate, projections must be deterministic and
/// total.
pub trait Projection: Default + Send + Sync + 'static {
    /// The event type this projection consumes.
    type Event: DomainEvent;

    /// Apply an event to update the read model.
    ///
    /// Must be deterministic and total. A projection can always be
    /// rebuilt from scratch by replaying all events.
    fn apply(&mut self, event: &EventEnvelope<Self::Event>);
}
```

**Design rationale:**
- `Default` — projections can be rebuilt from scratch at any time by
  replaying the full event history. There is no migration story for
  projections; you just rebuild them.
- Receives `EventEnvelope` — projections often use metadata (timestamp
  for time-based views, sequence for ordering guarantees).
- No error return — like `Aggregate::apply`, projection application must
  be total. If a projection cannot handle an event, that is a bug, not
  a runtime error.

## CommandGateway

```rust
/// The primary entry point for dispatching commands into the system.
///
/// Bound to a single aggregate type. The compiler verifies that every
/// command dispatched through this gateway is accepted by the bound
/// aggregate — no runtime routing errors possible.
///
/// The gateway adds cross-cutting concerns (interceptors, retry,
/// logging) on top of the CommandBus.
pub trait CommandGateway: Send + Sync + 'static {
    /// The single aggregate type this gateway dispatches to.
    type Aggregate: Aggregate;

    /// Create a new aggregate instance. Store assigns the ID.
    fn create<C>(
        &self,
        cmd: C,
    ) -> impl Future<Output = CreateResult<Self::Aggregate, C>> + Send
    where
        Self::Aggregate: HandleCommand<C>,
        C: Command;

    /// Dispatch a command to an existing aggregate instance.
    fn send<C>(
        &self,
        id: AggregateId,
        cmd: C,
    ) -> impl Future<Output = DispatchResult<Self::Aggregate, C>> + Send
    where
        Self::Aggregate: HandleCommand<C>,
        C: Command;
}
```

**Design rationale:**
- **Correctness (P1):** The `Aggregate` associated type binds this
  gateway to exactly one aggregate. `send` requires
  `Self::Aggregate: HandleCommand<C>` — the compiler proves at the
  call site that this aggregate accepts this command. A call like
  `gateway.send(id, cmd)` will not compile if the bound aggregate
  does not implement `HandleCommand` for the command type. No
  turbofish required — the aggregate is fixed, the command is inferred.
- **Create/send split:** `create` handles new aggregates (store assigns
  the ID and returns it). `send` handles existing aggregates (caller
  provides the known ID). This makes aggregate lifecycle explicit.
- **Energy (P3):** Uses `impl Future` (RPITIT) instead of `async_trait`
  to avoid the `Box<dyn Future>` heap allocation per dispatch. Requires
  Rust 1.75+.
- The gateway is a **port** defined in pit-core. Concrete implementations
  live in the infrastructure layer (`pit-gateway` crate). The domain
  never depends on a specific gateway implementation.
- Returns `Vec<EventEnvelope<Event>>` via `DispatchResult` / `CreateResult`
  — callers that need synchronous confirmation can inspect the produced
  events and their metadata.

## CommandBus

```rust
/// The internal command routing and execution mechanism.
///
/// Bound to a single aggregate type via the Aggregate associated
/// type. The compiler proves that commands, events, store, and bus
/// all agree on the same aggregate — no cross-aggregate ID/type
/// mismatches are possible.
///
/// The bus performs the actual work:
/// 1. Load the aggregate from the event store (replay via apply).
/// 2. Call HandleCommand::handle() with the command.
/// 3. Persist the produced events (store creates envelopes).
/// 4. Publish envelopes to the event bus for fan-out.
///
/// The CommandGateway wraps the bus and adds cross-cutting middleware.
pub trait CommandBus: Send + Sync + 'static {
    /// The single aggregate type this bus manages.
    type Aggregate: Aggregate;

    /// Create a new aggregate — full lifecycle without a known ID.
    fn create<C>(
        &self,
        cmd: C,
    ) -> impl Future<Output = CreateResult<Self::Aggregate, C>> + Send
    where
        Self::Aggregate: HandleCommand<C>,
        C: Command;

    /// Load, handle, persist, publish — the full command lifecycle.
    fn dispatch<C>(
        &self,
        id: AggregateId,
        cmd: C,
    ) -> impl Future<Output = DispatchResult<Self::Aggregate, C>> + Send
    where
        Self::Aggregate: HandleCommand<C>,
        C: Command;
}
```

**Design rationale:**
- **Correctness (P1):** The `Aggregate` associated type means the bus,
  its event store, and its event bus all operate on the same aggregate
  and event types. The compiler enforces this — you cannot wire a bus
  with a mismatched store.
- **Create/dispatch split:** `create` handles new aggregates (Default →
  handle → store.create → publish → return (id, envelopes)). `dispatch`
  handles existing aggregates (load → replay → handle → store.append →
  publish → return envelopes). Both paths produce envelopes for the
  event bus.
- Separating Bus from Gateway is a deliberate layering: the Bus handles
  the aggregate lifecycle (load → handle → persist → publish), while the
  Gateway adds operational concerns (retry, interceptors, timeout).
- The Bus is responsible for the unit of work. Events are only published
  after successful persistence. This prevents the scenario where
  listeners react to events that were never stored.
- **Security (P2):** The Bus enforces optimistic concurrency via the
  `sequence` field on `EventEnvelope`. If two commands race against the
  same aggregate, one will receive `ConcurrencyConflict`.

## EventStore

```rust
/// Port for loading and persisting a single aggregate's event streams.
///
/// Each event store instance is bound to exactly one domain event type
/// via the Event associated type. This gives compile-time proof that
/// every load/append operates on the correct event type — the caller
/// cannot accidentally deserialize one aggregate's events as another's.
///
/// The store creates EventEnvelopes — callers pass raw domain events.
/// The store assigns event_id (UUID v7), aggregate_id, sequence, and
/// timestamp. This eliminates redundancy and makes malformed envelopes
/// impossible by construction.
pub trait EventStore: Send + Sync + 'static {
    /// The single domain event type this store persists.
    type Event: DomainEvent;

    /// Load all events for an aggregate, ordered by sequence.
    ///
    /// Returns an empty Vec if no events exist for this aggregate.
    fn load(
        &self,
        id: AggregateId,
    ) -> impl Future<Output = Result<Vec<EventEnvelope<Self::Event>>,
                                     StoreError>> + Send;

    /// Create a new aggregate — the store assigns the next ID.
    ///
    /// Returns the assigned AggregateId and the created envelopes.
    /// Rejects empty events (no aggregate without at least one event).
    fn create(
        &self,
        events: Vec<Self::Event>,
    ) -> impl Future<Output = Result<(AggregateId,
                                      Vec<EventEnvelope<Self::Event>>),
                                     StoreError>> + Send;

    /// Append events to an existing aggregate's stream.
    ///
    /// Store creates envelopes, checks optimistic concurrency via
    /// expected_sequence. Empty events is a no-op.
    /// Atomic — all events persist, or none do.
    fn append(
        &self,
        id: AggregateId,
        expected_sequence: u64,
        events: Vec<Self::Event>,
    ) -> impl Future<Output = Result<Vec<EventEnvelope<Self::Event>>,
                                     StoreError>> + Send;
}
```

```rust
/// Errors from event store operations.
#[non_exhaustive]
pub enum StoreError {
    ConcurrencyConflict {
        aggregate_id: AggregateId,
        expected_sequence: u64,
        actual_sequence: u64,
    },
    Infrastructure(Box<dyn Error + Send + Sync>),
}
```

**Design rationale:**
- **Correctness (P1):** The `Event` associated type means a
  `MsgpackFileStore<OrderEvent>` can only load and persist
  `OrderEvent` — the type parameter is fixed at construction, not
  chosen per-call. This eliminates an entire class of runtime
  deserialization bugs.
- **Correctness (P1):** The store creates envelopes from raw domain
  events. Callers pass `Vec<Self::Event>`, the store assigns all
  metadata (`event_id`, `aggregate_id`, `sequence`, `timestamp`).
  This eliminates redundancy between method parameters and envelope
  fields, making malformed envelopes impossible by construction.
- **Correctness (P1):** `expected_sequence` on `append` makes optimistic
  concurrency explicit in the API. There is no "blind append" — every
  write declares what it expects the current state to be.
- **Correctness (P1):** `append` is atomic — all events persist or none do.
  Partial writes would leave the aggregate in an inconsistent state.
- **Create/append split:** `create` auto-increments a `u64` counter to
  assign the ID. This is safe under the single-writer assumption.
  `append` targets an existing aggregate by its known ID.
- **Energy (P3):** `append` takes `Vec<Self::Event>` by value — events
  are moved into envelopes without cloning. `create` does the same.
- **Energy (P3):** Uses `impl Future` (RPITIT) for zero-cost async.
- `load` returns empty Vec for unknown aggregates rather than a NotFound
  error. An unknown aggregate is an empty event stream, not an error
  condition. The `CommandBus` maps empty load to `AggregateNotFound`
  for dispatch operations.
- `StoreError` deliberately does not include `NotFound`. The
  `AggregateNotFound` variant in `DispatchError` is a CommandBus-level
  decision, not a store-level error.

## EventBus

```rust
/// Port for publishing events to downstream consumers.
///
/// Each bus instance is bound to a single domain event type. In a
/// distributed system, each bounded context has its own EventBus
/// publishing its aggregate's events (e.g. to a dedicated NATS
/// subject). Cross-context consumption uses separate subscriptions
/// typed to the foreign event type.
pub trait EventBus: Send + Sync + 'static {
    /// The single domain event type this bus publishes.
    type Event: DomainEvent;

    /// Publish events to all registered consumers.
    ///
    /// Called by the CommandBus after events are successfully
    /// persisted. Because events are already safely stored, publication
    /// failure is non-fatal — tracking-style processors can catch up
    /// on missed publications.
    fn publish(
        &self,
        events: &[EventEnvelope<Self::Event>],
    ) -> impl Future<Output = Result<(), BusError>> + Send;
}
```

```rust
/// Error from event bus publication.
pub struct BusError(Box<dyn Error + Send + Sync>);
```

**Design rationale:**
- **Correctness (P1):** The `Event` associated type means each bus
  publishes exactly one event type — the compiler prevents
  cross-aggregate event pollution.
- **Energy (P3):** Takes `&[EventEnvelope<Self::Event>]` — events are
  borrowed. For synchronous in-process delivery, no cloning is needed.
- **Energy (P3):** `impl Future` (RPITIT) for zero-cost async.
- **Separation from EventStore:** Cherry-pit deliberately separates
  store (persist) from bus (fan-out). An agent may want fan-out without
  persistence (volatile event bus for testing or stateless reactive
  systems). The CommandBus composes both — explicit in the type system.
- **Publication failure semantics:** Events are persisted FIRST, THEN
  published. Publication failure cannot un-persist events.
- **No subscribe method on the trait:** Subscription is inherently
  implementation-specific. Putting `subscribe` on the port trait would
  force a single subscription model.

## DispatchError, DispatchResult, and CreateResult

```rust
/// Errors that can occur during command dispatch.
///
/// Generic over E — the domain-specific error type from
/// HandleCommand<C>::Error. This preserves full type information
/// through the gateway and bus, allowing callers to match on
/// domain errors without downcasting.
#[non_exhaustive]
pub enum DispatchError<E: Error + Send + Sync> {
    Rejected(E),
    AggregateNotFound { aggregate_id: AggregateId },
    ConcurrencyConflict {
        aggregate_id: AggregateId,
        expected_sequence: u64,
        actual_sequence: u64,
    },
    Infrastructure(Box<dyn Error + Send + Sync>),
}

/// Result type for command dispatch through the bus or gateway.
///
/// Returns the event envelopes produced and persisted on success.
pub type DispatchResult<A, C> = Result<
    Vec<EventEnvelope<<A as Aggregate>::Event>>,
    DispatchError<<A as HandleCommand<C>>::Error>,
>;

/// Result type for aggregate creation through the bus or gateway.
///
/// Returns the store-assigned AggregateId and the event envelopes.
pub type CreateResult<A, C> = Result<
    (AggregateId, Vec<EventEnvelope<<A as Aggregate>::Event>>),
    DispatchError<<A as HandleCommand<C>>::Error>,
>;
```

**Design rationale:**
- **Correctness (P1):** `DispatchError<E>` is generic over the domain
  error type. When an adapter calls `gateway.send(id, cmd)`, the
  error type is `DispatchError<ShipOrderError>`. The adapter can match
  on `Rejected(ShipOrderError::NotConfirmed)` without downcasting.
- `DispatchResult<A, C>` returns `Vec<EventEnvelope<Event>>` — callers
  receive full metadata (event_id, sequence, timestamp) alongside the
  domain events.
- `CreateResult<A, C>` additionally returns the store-assigned
  `AggregateId`, which the caller needs to target future commands.
- `AggregateNotFound` is separate from `Rejected` because it is not a
  domain decision — the aggregate never ran. HTTP adapters typically
  map this to 404.
- `ConcurrencyConflict` carries diagnostic fields. The Gateway's retry
  scheduler can automatically retry on this variant.
- `Infrastructure` uses `Box<dyn Error>` — infrastructure errors are
  inherently open-ended. Type-erasing them here keeps the domain side
  clean.

## Trait dependency graph

```
                    DRIVING (primary) SIDE
                    ═══════════════════════

  Primary Adapters ──► CommandGateway ──► CommandBus
  (webhook, REST        (type Agg)        (type Agg)
   poller, Policy)          │                │
                   create<C>│       create<C>│
                     send<C>│     dispatch<C>│
                            │                │
                  where Agg: HandleCommand<C>│
                            │                │
                            └────────────────┘
                                     │
                                     │  uses
                                     ▼
                    Command ──► HandleCommand<C>
                                     │
                                 produces
                                     │
                                     ▼
                              Aggregate::Event
                                     │
                    ═══════════════════════
                    DRIVEN (secondary) SIDE
                                     │
                    ┌────────────────┼────────────────┐
                    │                                 │
                    ▼                                 ▼
               EventStore                        EventBus
             (type Event)                      (type Event)
         (load, create, append)                (publish)
               │                                     │
         assigns AggregateId                    fan-out to:
         creates envelopes          ┌──────────┼──────────┐
                                    │          │          │
                                    ▼          ▼          ▼
                                 Policy   Projection  (Pardosa)
                                    │
                                produces
                                (Output)
                                    │
                                    ▼
                       CommandGateway (back to top)
```

All ports on the driving side (`CommandGateway`, `CommandBus`) and
driven side (`EventStore`, `EventBus`) carry an associated type that
locks them to a single aggregate and its events. The compiler proves
end-to-end that commands, events, persistence, and publication all
agree on the same types — no runtime routing or deserialization
errors are possible.
