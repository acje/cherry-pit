# CHE-0020. Infrastructure-Owned Aggregate Identity

Date: 2026-04-25
Last-reviewed: 2026-04-25
Tier: B

## Status

Accepted

## Related

- Depends on: CHE-0011, CHE-0013, CHE-0018
- References: CHE-0012, CHE-0013, CHE-0018, CHE-0033

## Context

Every aggregate instance needs an identifier. The question is: who
creates and owns it — the domain or the infrastructure?

CHE-0011 and CHE-0013 establish the type and API for aggregate
identity but neither states the ownership principle: the domain
layer never creates its own `AggregateId`.

Two approaches:

1. **Domain-assigned identity** — the aggregate has an `id()` method
   or the command carries a caller-chosen ID. The domain decides
   what the aggregate is called. Requires uniqueness coordination
   (UUIDs, sequences, or a registry).
2. **Infrastructure-assigned identity** — the event store assigns the
   ID during `create()`. The domain never invents IDs. The aggregate
   trait has no `id()` method. If domain logic needs its own ID, it
   stores it as a field set during the first event's `apply`.

## Decision

The infrastructure layer owns aggregate identity assignment. The
domain layer is identity-agnostic.

Concrete rules:

1. **`Aggregate` has no `id()` method.** Aggregate identity is
   managed by the infrastructure layer (event store, command bus,
   gateway). The aggregate itself has no way to query its own ID.
2. **`EventStore::create` assigns the ID.** The store auto-increments
   a `u64` counter and wraps it in `AggregateId(NonZeroU64)`.
   Callers receive the ID as a return value.
3. **Callers never invent IDs.** There is no public constructor
   that takes a raw `u64` and creates an `AggregateId` for use in
   `create()`. The `AggregateId::new(NonZeroU64)` constructor exists
   for the store's internal use and for test helpers, not for domain
   code to generate IDs.
4. **If the domain needs its own identity** (e.g., a `user_id` or
   `order_number`), it stores it as a domain field populated during
   the first event's `apply`. This domain identity is separate from
   the infrastructure `AggregateId`.

```rust
// Aggregate trait — no id() method
pub trait Aggregate: Default + Send + Sync + 'static {
    type Event: DomainEvent;
    fn apply(&mut self, event: &Self::Event);
}

// EventStore::create — store assigns the ID
fn create(&self, events: Vec<Self::Event>)
    -> impl Future<Output = Result<(AggregateId, Vec<EventEnvelope<Self::Event>>), StoreError>> + Send;
```

## Consequences

- **No identity conflicts** — the store guarantees uniqueness via
  sequential assignment. No coordination protocol, no UUID collision
  risk for aggregate IDs (event IDs use UUID v7 per CHE-0033, but
  aggregate IDs are sequential `u64`s).
- **Aggregate is a pure state machine** — it has no infrastructure
  concerns. It does not know its ID, its store, or its bus. This
  maximizes testability and keeps the domain layer runtime-agnostic
  (CHE-0018).
- **First-event pattern** — since the aggregate starts as `Default`
  (CHE-0012) and does not receive its ID, any domain logic that
  needs the aggregate's own ID must extract it from the first
  event's envelope. The `EventEnvelope.aggregate_id` field is
  available to policies and projections (which receive envelopes),
  but the aggregate itself only receives `&Event` (not the
  envelope). This is an intentional asymmetry: the aggregate
  operates on domain facts, not infrastructure metadata.
- **No aggregate self-reference** — an aggregate cannot include its
  own `AggregateId` in the events it produces during `handle`. If
  a command handler needs to reference "this aggregate" in an event,
  the ID must be passed in as part of the command or stored as a
  domain field during a prior `apply`.
- **Cross-reference between ADRs** — this principle is a consequence
  of CHE-0011 (type), CHE-0013 (API split), and CHE-0018 (domain
  purity). It completes the picture by stating the ownership rule
  explicitly.
