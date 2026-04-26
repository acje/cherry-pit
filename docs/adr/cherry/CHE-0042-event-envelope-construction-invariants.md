# CHE-0042. EventEnvelope Construction Invariants

Date: 2026-04-25
Last-reviewed: 2026-04-25
Tier: A

## Status

Accepted

## Related

- References: CHE-0002, CHE-0010, CHE-0016, CHE-0033, CHE-0034, CHE-0039

## Context

`EventEnvelope<E>` is the infrastructure wrapper around every domain
event. It carries metadata (event_id, aggregate_id, sequence,
timestamp, correlation_id, causation_id) stamped by the `EventStore`
during `create` and `append`. CHE-0016 establishes: "Callers never
construct envelopes directly."

However, all seven fields are `pub`:

```rust
pub struct EventEnvelope<E: DomainEvent> {
    pub event_id: uuid::Uuid,
    pub aggregate_id: AggregateId,
    pub sequence: u64,
    pub timestamp: jiff::Timestamp,
    pub correlation_id: Option<uuid::Uuid>,
    pub causation_id: Option<uuid::Uuid>,
    pub payload: E,
}
```

Any code — including user code — can construct an envelope via
struct literal with wrong `sequence`, wrong `aggregate_id`, stale
`timestamp`, nil `event_id`, or arbitrary correlation metadata.
The safety guarantee "only the store constructs envelopes" is
convention, not enforcement.

CHE-0002 says: "Every cherry-pit type must encode its invariants at
the type level." `EventEnvelope` violates this — illegal envelopes
are representable.

Three options were evaluated:

1. **Private fields + validated public constructor + accessor
   methods** — external code cannot construct malformed envelopes.
   The constructor validates invariants (non-nil event_id, non-zero
   sequence). All field reads go through methods. Breaking change.
2. **`#[non_exhaustive]` on struct** — prevents external struct
   literal construction. Fields remain publicly readable. Same
   cross-crate constructor problem as option 1 because
   `EventEnvelope` is defined in `cherry-pit-core` but constructed in
   `cherry-pit-gateway`.
3. **Accept with documentation** — keep `pub` fields, document the
   convention. Zero breaking changes. CHE-0002 violation persists.

## Decision

Option 1: Private fields + validated public constructor + accessor
methods.

### Struct definition

```rust
#[derive(Debug, Clone, Serialize)]
#[serde(bound(serialize = "E: Serialize"))]
pub struct EventEnvelope<E: DomainEvent> {
    event_id: uuid::Uuid,
    aggregate_id: AggregateId,
    sequence: NonZeroU64,
    timestamp: jiff::Timestamp,
    correlation_id: Option<uuid::Uuid>,
    causation_id: Option<uuid::Uuid>,
    payload: E,
}
```

All fields private. `sequence` uses `NonZeroU64` to eliminate zero
sequences at the type level — serde rejects zero on
deserialization automatically. `Serialize` derived (serde can
access private fields within the defining module). `Deserialize`
handled separately (see below).

### Validated constructor

`EventEnvelope::new()` accepts all 7 fields as parameters and returns
`Result<Self, EnvelopeError>`. It validates that `event_id` is non-nil
(returns `EnvelopeError::NilEventId` otherwise), then assigns all
fields. Zero sequence is impossible at the type level via `NonZeroU64`.
See `cherry-pit-core/src/envelope.rs` for full implementation.

The constructor is `pub` (not `pub(crate)`) because `cherry-pit-gateway`
is a separate crate that must call it. The constructor is safe for
external use because it validates invariants — external callers
cannot create malformed envelopes through this path.

### Accessor methods

```rust
impl<E: DomainEvent> EventEnvelope<E> {
    pub fn event_id(&self) -> uuid::Uuid { self.event_id }
    pub fn aggregate_id(&self) -> AggregateId { self.aggregate_id }
    /// Returns the 1-based sequence as `u64` for ergonomic use.
    pub fn sequence(&self) -> u64 { self.sequence.get() }
    pub fn timestamp(&self) -> jiff::Timestamp { self.timestamp }
    pub fn correlation_id(&self) -> Option<uuid::Uuid> {
        self.correlation_id
    }
    pub fn causation_id(&self) -> Option<uuid::Uuid> {
        self.causation_id
    }
    pub fn payload(&self) -> &E { &self.payload }
}
```

### Error type

```rust
#[derive(Debug)]
pub enum EnvelopeError {
    /// event_id must not be the nil UUID.
    NilEventId,
}
```

Single variant — `ZeroSequence` is eliminated by using `NonZeroU64`
for the `sequence` field. Manual `Display` and `Error` impls,
consistent with CHE-0027.

### Serde deserialization

Rust's `#[derive(Deserialize)]` can access private fields within
the defining module. Deserialization reconstructs `EventEnvelope`
without calling `new()` — this bypasses constructor validation.

Mitigation: **post-deserialization validation in `EventStore::load`
implementations.** After deserializing envelopes from storage, the
store calls a `validate()` method on each envelope:

```rust
impl<E: DomainEvent> EventEnvelope<E> {
    /// Validate invariants on a deserialized envelope.
    ///
    /// Called by EventStore::load after deserialization. Returns
    /// an error if the envelope violates construction invariants
    /// (nil event_id). Zero sequence is impossible — `NonZeroU64`
    /// serde rejects zero on deserialization.
    pub fn validate(&self) -> Result<(), EnvelopeError> {
        if self.event_id.is_nil() {
            return Err(EnvelopeError::NilEventId);
        }
        Ok(())
    }
}
```

This keeps the `Deserialize` derive simple (no custom impl with
complex generic bounds) while ensuring invalid data from corrupt
storage is caught at load time.

## Consequences

- **CHE-0002 compliance restored.** External code cannot construct
  malformed envelopes via struct literal. The validated constructor
  rejects nil event_id. Zero sequence is eliminated at the type
  level via `NonZeroU64`. Post-deserialization validation catches
  corrupt stored data.
- **Breaking change.** 48 locations in `cherry-pit-gateway` change:
  1 production field access → accessor, 42 test field accesses →
  accessors, 3 struct literal constructions → `EventEnvelope::new()`
  calls, 2 new `CorrelationContext` parameters (from CHE-0039).
  All changes are mechanical.
- **Sequencing dependency on CHE-0039.** The constructor accepts
  `correlation_id` and `causation_id` as `Option<uuid::Uuid>`
  parameters (extracted from `CorrelationContext` at the call
  site). CHE-0039 must be implemented first so the
  `CorrelationContext` type exists for the store to propagate.
- **Test migration.** Test code that constructs envelopes with
  specific metadata (e.g., the correlation roundtrip test) uses
  `EventEnvelope::new()` with a `CorrelationContext::new(corr,
  cause)`. No test-only constructor needed — the public validated
  constructor serves both production and test code.
- **Serde bypass is defense-in-depth.** The `validate()` method in
  `load()` catches corrupt data. The window where an invalid
  envelope exists in memory (between deserialization and validation)
  is contained within the store implementation.
- **`pub fn new()` visibility.** External code (user code) CAN call
  `EventEnvelope::new()`. This is acceptable because the constructor
  validates invariants. The convention "only the store constructs
  envelopes" remains the design intent, but violations are no longer
  dangerous — they produce valid envelopes with potentially wrong
  metadata, not structurally invalid envelopes.
- **Future compile-fail test.** A `trybuild` test should verify
  that direct struct literal construction of `EventEnvelope` fails
  (CHE-0028). This proves the private-field invariant holds.
