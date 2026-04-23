# pit-core: trait design

pit-core defines the foundational traits that every cherry-pit system
implements. These are the narrow, typed ports that agents program against.
All domain logic lives behind these traits. All infrastructure lives on
the other side.

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
  infrastructure layer (event store, repository). The aggregate itself
  is pure domain state. If the domain logic needs its own ID, it stores
  it as a field set during the first event's `apply`.

## EventEnvelope

```rust
/// Infrastructure wrapper around a domain event.
///
/// Provided by pit-core, not implemented by the agent. This is what
/// gets persisted and transported. The domain event is the payload;
/// the envelope adds the metadata needed for ordering, routing, and
/// idempotency.
pub struct EventEnvelope<E: DomainEvent> {
    pub event_id: Uuid,
    pub aggregate_id: String,
    pub sequence: u64,
    pub timestamp: DateTime<Utc>,
    pub payload: E,
}
```

**Design rationale:**
- `sequence` — monotonically increasing within an aggregate's stream.
  Enables optimistic concurrency and ordered replay.
- `aggregate_id` + `sequence` together form the unique position in the
  aggregate's history.
- The agent never constructs envelopes. pit-core creates them when
  persisting events returned by `handle`.

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
/// Every primary adapter (webhook listener, REST API poller, scheduled
/// job) and every Policy dispatches commands through the gateway. The
/// gateway is the outermost port on the driving side of the hexagon.
///
/// The gateway adds cross-cutting concerns (interceptors, retry,
/// logging) on top of the CommandBus.
///
/// Generic methods allow the compiler to verify each
/// command→aggregate pair at the call site — no runtime routing errors.
pub trait CommandGateway: Send + Sync + 'static {
    /// Dispatch a command targeting a specific aggregate instance.
    ///
    /// The gateway:
    /// 1. Runs dispatch interceptors (logging, metadata, validation).
    /// 2. Delegates to the CommandBus.
    /// 3. Optionally retries on transient infrastructure failure.
    ///
    /// Returns the events produced by the aggregate on success.
    /// Returns a typed DispatchError on failure — either a domain
    /// rejection or an infrastructure error.
    fn send<A, C>(
        &self,
        aggregate_id: &str,
        cmd: C,
    ) -> impl Future<Output = Result<Vec<A::Event>, DispatchError<<A as HandleCommand<C>>::Error>>> + Send
    where
        A: HandleCommand<C>,
        C: Command;
}
```

**Design rationale:**
- **Correctness (P1):** `send` is generic over `A: HandleCommand<C>`.
  The compiler enforces that the command type `C` is accepted by aggregate
  type `A`. A call like `gateway.send::<OrderAggregate, ShipOrder>(id, cmd)`
  will not compile if `OrderAggregate` does not implement
  `HandleCommand<ShipOrder>`. Routing correctness is verified at compile
  time, not runtime.
- **Energy (P3):** Uses `impl Future` (RPITIT) instead of `async_trait`
  to avoid the `Box<dyn Future>` heap allocation per dispatch. Requires
  Rust 1.75+.
- The gateway is a **port** defined in pit-core. Concrete implementations
  live in the infrastructure layer (`pit-gateway` crate). The domain
  never depends on a specific gateway implementation.
- Returns `Vec<A::Event>` — callers that need synchronous confirmation
  (like HTTP adapters returning a response) can inspect the produced
  events. Callers that don't care (fire-and-forget policies) can ignore
  the return value.

## CommandBus

```rust
/// The internal command routing and execution mechanism.
///
/// The bus is the lower-level port that performs the actual work:
/// 1. Load the aggregate from the event store (replay via apply).
/// 2. Call HandleCommand::handle() with the command.
/// 3. Persist the produced events.
/// 4. Publish events to the event bus for fan-out.
///
/// The CommandGateway wraps the bus and adds cross-cutting middleware.
/// Direct bus access is available for testing and infrastructure code
/// that needs to bypass gateway interceptors.
pub trait CommandBus: Send + Sync + 'static {
    /// Load, handle, persist, publish — the full command lifecycle.
    ///
    /// Implementors manage the unit of work: if event persistence
    /// fails, no events are published. If optimistic concurrency
    /// is violated, a ConcurrencyConflict error is returned.
    fn dispatch<A, C>(
        &self,
        aggregate_id: &str,
        cmd: C,
    ) -> impl Future<Output = Result<Vec<A::Event>, DispatchError<<A as HandleCommand<C>>::Error>>> + Send
    where
        A: HandleCommand<C>,
        C: Command;
}
```

**Design rationale:**
- Separating Bus from Gateway is a deliberate layering: the Bus handles
  the aggregate lifecycle (load → handle → persist → publish), while the
  Gateway adds operational concerns (retry, interceptors, timeout).
- Same generic signature as the Gateway — the Gateway delegates directly
  to the Bus after running its middleware chain.
- **Correctness (P1):** The Bus is responsible for the unit of work.
  Events are only published after successful persistence. This prevents
  the scenario where listeners react to events that were never stored.
- **Security (P2):** The Bus enforces optimistic concurrency via the
  `sequence` field on `EventEnvelope`. If two commands race against the
  same aggregate, one will receive `ConcurrencyConflict`.
- The Bus is also a **port** in pit-core. Concrete implementations
  (in-memory for testing, event-store-backed for production) live in
  infrastructure crates.

## DispatchError

```rust
/// Errors that can occur during command dispatch.
///
/// Generic over E — the domain-specific error type from
/// HandleCommand<C>::Error. This preserves full type information
/// through the gateway and bus, allowing callers to match on
/// domain errors without downcasting.
pub enum DispatchError<E: Error + Send + Sync> {
    /// The aggregate rejected the command (business invariant violation).
    /// Contains the domain-specific error from HandleCommand::handle().
    Rejected(E),

    /// No events exist for this aggregate_id — it has never been created.
    AggregateNotFound { aggregate_id: String },

    /// Another command was persisted against this aggregate between our
    /// load and our persist. The caller may retry.
    ConcurrencyConflict {
        aggregate_id: String,
        expected_sequence: u64,
        actual_sequence: u64,
    },

    /// Infrastructure failure (event store unavailable, serialization
    /// error, transport timeout, etc.).
    Infrastructure(Box<dyn Error + Send + Sync>),
}
```

**Design rationale:**
- **Correctness (P1):** `DispatchError<E>` is generic over the domain
  error type. When an adapter calls
  `gateway.send::<OrderAggregate, ShipOrder>(id, cmd)`, the error type
  is `DispatchError<ShipOrderError>`. The adapter can match on
  `Rejected(ShipOrderError::NotConfirmed)` without downcasting.
- `AggregateNotFound` is separate from `Rejected` because it is not a
  domain decision — the aggregate never ran. HTTP adapters typically
  map this to 404.
- `ConcurrencyConflict` carries the expected and actual sequence numbers
  for diagnostic logging. The Gateway's retry scheduler can automatically
  retry on this error variant.
- `Infrastructure` uses `Box<dyn Error>` — infrastructure errors are
  inherently open-ended (network, disk, serialization). Type-erasing
  them here keeps the domain side clean while still surfacing the cause.

## EventStore

```rust
/// Port for loading and persisting aggregate event streams.
///
/// The event store is the single source of truth for aggregate state
/// in an event-sourced system. Every aggregate's history is an ordered
/// sequence of EventEnvelopes keyed by (aggregate_id, sequence).
///
/// The CommandBus implementation uses this port to reconstruct
/// aggregates (load events → replay via Aggregate::apply) and to
/// persist new events after HandleCommand::handle() succeeds.
///
/// This is a secondary (driven) port — the domain tells infrastructure
/// when to load and persist. Concrete implementations (in-memory for
/// testing, Pardosa-backed, PostgreSQL-backed) live in infrastructure
/// crates.
///
/// Cherry-pit separates the store (persist) from the bus (fan-out)
/// for composability. An agent that only needs batch replay doesn't
/// need a bus. An agent that only needs volatile fan-out doesn't
/// need a store.
pub trait EventStore: Send + Sync + 'static {
    /// Load all events for an aggregate, ordered by sequence.
    ///
    /// Returns an empty Vec if no events exist for this aggregate_id.
    /// This is not an error — it means the aggregate has never been
    /// created. The CommandBus decides whether to proceed (creation
    /// command against a Default aggregate) or return
    /// DispatchError::AggregateNotFound.
    fn load<E: DomainEvent>(
        &self,
        aggregate_id: &str,
    ) -> impl Future<Output = Result<Vec<EventEnvelope<E>>, StoreError>> + Send;

    /// Append new events to an aggregate's stream.
    ///
    /// `expected_sequence` is the sequence number of the last event
    /// the caller loaded (0 if creating a new aggregate with no prior
    /// events). If the store's actual last sequence for this aggregate
    /// does not match `expected_sequence`, the append is rejected with
    /// `StoreError::ConcurrencyConflict`.
    ///
    /// The caller (CommandBus) constructs the EventEnvelopes with
    /// sequence numbers starting from `expected_sequence + 1`. The
    /// store validates and persists atomically — either all events
    /// in the slice are persisted, or none are.
    fn append<E: DomainEvent>(
        &self,
        aggregate_id: &str,
        expected_sequence: u64,
        events: &[EventEnvelope<E>],
    ) -> impl Future<Output = Result<(), StoreError>> + Send;
}
```

```rust
/// Errors from event store operations.
pub enum StoreError {
    /// Optimistic concurrency violation — another writer persisted
    /// events after our load. The CommandBus maps this to
    /// DispatchError::ConcurrencyConflict. The gateway's retry
    /// scheduler may automatically retry.
    ConcurrencyConflict {
        aggregate_id: String,
        expected_sequence: u64,
        actual_sequence: u64,
    },

    /// Infrastructure failure (disk I/O, network, serialization).
    Infrastructure(Box<dyn Error + Send + Sync>),
}
```

**Design rationale:**
- **Correctness (P1):** `load` is generic over `E: DomainEvent`. The
  CommandBus calls `store.load::<A::Event>(id)` — the compiler ensures
  events are deserialized into the correct type. No runtime type confusion.
- **Correctness (P1):** `expected_sequence` on `append` makes optimistic
  concurrency explicit in the API. There is no "blind append" — every
  write declares what it expects the current state to be. The storage
  layer enforces a unique constraint on `(aggregate_id, sequence)`.
- **Correctness (P1):** `append` is atomic — all events persist or none do.
  Partial writes would leave the aggregate in an inconsistent state.
- **Energy (P3):** `append` takes `&[EventEnvelope<E>]` — events are
  borrowed, not moved. The store serializes from references, avoiding
  unnecessary ownership transfer.
- **Energy (P3):** Uses `impl Future` (RPITIT) for zero-cost async.
- `load` returns empty Vec for unknown aggregates rather than a NotFound
  error. An unknown aggregate is an empty event stream, not an error
  condition. This simplifies creation flows — the first command against
  a new aggregate_id gets a `Default` aggregate with an empty history.
  The aggregate's `handle()` method is responsible for rejecting commands
  that don't make sense against default state.
- `StoreError` deliberately does not include `NotFound`. The
  `AggregateNotFound` variant in `DispatchError` is a CommandBus-level
  decision (the bus may choose to return it when `load` yields an empty
  vec for a non-creation command), not a store-level error.
- No `load_from_sequence` or snapshot methods yet. Snapshots are an
  optimization — the aggregate can always be reconstructed from the
  full event stream. When aggregate streams grow large enough to warrant
  snapshots, `load_from_sequence` can be added without breaking the trait
  (it's additive).

## EventBus

```rust
/// Port for publishing events to downstream consumers.
///
/// After the CommandBus persists new events via the EventStore, it
/// publishes them through the EventBus for fan-out to Policies,
/// Projections, and external integrations (e.g. NATS subjects).
///
/// This is a secondary (driven) port. The CommandBus calls publish
/// after successful persistence. Concrete implementations (in-memory
/// synchronous fan-out, NATS-backed, channel-based) live in
/// infrastructure crates.
///
/// The subscribe/wiring side — connecting specific Policies and
/// Projections to the bus — is infrastructure configuration, not a
/// port trait. The agent (or the pit-gateway setup code) registers
/// handlers at startup. This keeps the port trait minimal: the
/// CommandBus only needs to publish.
///
/// Cherry-pit separates EventBus from EventStore for composability —
/// volatile fan-out without persistence is a valid configuration for
/// systems that don't need event sourcing replay.
pub trait EventBus: Send + Sync + 'static {
    /// Publish events to all registered consumers.
    ///
    /// Called by the CommandBus after events are successfully persisted
    /// in the EventStore. Because events are already safely stored,
    /// publication failure is non-fatal to the command dispatch — the
    /// events will not be lost. Tracking-style processors (which poll
    /// the EventStore directly) can catch up on missed publications.
    ///
    /// Implementations may deliver synchronously (in-process fan-out
    /// in the caller's task) or asynchronously (enqueue for later
    /// delivery by a background processor).
    fn publish<E: DomainEvent>(
        &self,
        events: &[EventEnvelope<E>],
    ) -> impl Future<Output = Result<(), BusError>> + Send;
}
```

```rust
/// Error from event bus publication.
///
/// Intentionally simple — publication errors are infrastructure-level.
/// There are no domain error variants because event publication is
/// best-effort from the command dispatch perspective. The CommandBus
/// may log this error but does not propagate it as a DispatchError
/// to the caller — the command already succeeded (events are persisted).
pub struct BusError(pub Box<dyn Error + Send + Sync>);
```

**Design rationale:**
- **Correctness (P1):** `publish` is generic over `E: DomainEvent` — the
  compiler ensures only correctly typed events enter the bus. No runtime
  type confusion between different aggregate event types.
- **Energy (P3):** Takes `&[EventEnvelope<E>]` — events are borrowed from
  the CommandBus's scope. For synchronous in-process delivery (calling
  `policy.react(event)` and `projection.apply(event)` directly), no
  cloning is needed.
- **Energy (P3):** `impl Future` (RPITIT) for zero-cost async.
- **Separation from EventStore:** Some frameworks merge the event store
  and event bus into a single component. Cherry-pit deliberately separates
  them. Reasons:
  - **Composability:** An agent may want fan-out without persistence
    (volatile event bus, useful for testing or stateless reactive systems).
  - **Single Responsibility:** The store persists; the bus distributes.
  - **Testability:** Easier to mock one without the other.
  - The CommandBus implementation composes both: it depends on an
    `EventStore` AND an `EventBus`. This is explicit in the type system.
- **Publication failure semantics:** The CommandBus persists events FIRST
  (via EventStore), THEN publishes (via EventBus). If publication fails,
  the events are safe. Events are staged, persisted, then dispatched —
  persistence failure prevents dispatch, but dispatch failure cannot
  un-persist.
- **No subscribe method on the trait:** Subscription is inherently
  implementation-specific. An in-memory bus holds a list of closures.
  A NATS-backed bus creates subject subscriptions. A tracking processor
  polls the EventStore directly (bypassing the bus entirely). Putting
  `subscribe` on the port trait would force a single subscription model.
  Instead, the infrastructure configures wiring at startup.
- Two event delivery models are supported:
  - **Subscribing:** The bus calls handlers synchronously during
    `publish()`. Handlers run in the publisher's context. Simple,
    low-latency, but handlers block the command return path.
  - **Tracking/Streaming:** Handlers poll the EventStore independently,
    maintaining their own position cursor. Decoupled, replayable, but
    adds latency. The EventBus is not involved — handlers go directly
    to the EventStore.
  - Both models are valid. The choice is infrastructure configuration,
    not a trait-level decision.

## Trait dependency graph

```
                    DRIVING (primary) SIDE
                    ═══════════════════════

  Primary Adapters ──► CommandGateway ──► CommandBus
  (webhook, REST                          │       │
   poller, Policy)                  handle │       │ uses
                                          │       │
                   Command ──► HandleCommand       │
                                   │               │
                               produces            │
                                   │               │
                                   ▼               │
                              DomainEvent          │
                                                   │
                    ═══════════════════════         │
                    DRIVEN (secondary) SIDE        │
                                                   │
                              ┌─────────────────────┤
                              │                     │
                              ▼                     ▼
                         EventStore            EventBus
                        (load, persist)       (publish)
                                                   │
                                              fan-out to:
                                    ┌──────────┼──────────┐
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

Primary adapters and policies dispatch commands through the
CommandGateway. The gateway delegates to the CommandBus.

The CommandBus orchestrates the full command lifecycle using two
secondary ports:
1. **EventStore** — loads the aggregate's event history (for
   reconstruction via `Aggregate::apply`), then persists new events
   produced by `HandleCommand::handle()`.
2. **EventBus** — publishes the persisted events for fan-out.

Events fan out to policies (which produce typed outputs dispatched back
through the gateway), projections (which build read models), and
optionally to Pardosa for serialization and transport.

The cycle Gateway → Bus → HandleCommand → Aggregate → Event →
(persist to EventStore) → (publish via EventBus) → Policy → Gateway
is the heartbeat of the system.

Publication is best-effort — events are already safely persisted in
the EventStore before the EventBus is called. Tracking-style processors
that poll the EventStore directly are immune to publication failures.
