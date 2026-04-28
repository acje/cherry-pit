# CHE-0038. Testing Strategy and Property-Based Verification

Date: 2026-04-25
Last-reviewed: 2026-04-28
Tier: B
Status: Accepted

## Related

- References: CHE-0001, CHE-0003, CHE-0028, COM-0001

## Context

Cherry-pit uses three testing patterns today, none documented as a
deliberate strategy:

1. **Unit tests** (`#[test]`) — 17 synchronous tests in
   `cherry-pit-core/src/aggregate_id.rs` covering `AggregateId` construction,
   serde roundtrips, ordering, and niche optimization.
2. **Async integration tests** (`#[tokio::test]`) — 30 tests in
   `cherry-pit-gateway/src/event_store/msgpack_file.rs` covering store
   lifecycle, concurrency, optimistic locking, and serialization
   compatibility.
3. **Compile-fail tests** (`trybuild`) — 3 test cases in
   `cherry-pit-core/tests/compile_fail/` verifying that the type system
   rejects cross-aggregate confusion, unhandled commands, and non-
   command types (CHE-0028).

Missing from the current practice:

- **Property-based tests** — `proptest` is declared as a workspace
  dev-dependency but has zero usage. No property tests exist for
  core invariants such as aggregate replay determinism, sequence
  monotonicity, or serialization roundtrip identity.
- **No documented test boundary** — the split between sync domain
  tests and async infrastructure tests mirrors CHE-0018 (sync domain,
  async infrastructure) but is implicit, not stated.
- **No guidance for new tests** — contributors have no documented
  rule for which test type to use when adding new invariants.

CHE-0001 places correctness first (P1). CHE-0003 prefers compile-time
verification. CHE-0028 establishes compile-fail tests for type
contracts. This ADR completes the testing picture for runtime
invariants that the type system cannot enforce.

## Decision

### Test boundary rule

The test boundary mirrors CHE-0018's sync/async split:

| Layer | Test type | Runtime dependency |
|-------|-----------|-------------------|
| Domain logic (`Aggregate::apply`, `HandleCommand::handle`, `Policy::react`, `Projection::apply`) | `#[test]` (synchronous) | None |
| Infrastructure (`EventStore`, `EventBus`, `CommandBus`, `CommandGateway` impls) | `#[tokio::test]` (async) | tokio |
| Type contracts (trait bounds, associated types) | `trybuild` compile-fail | None at runtime |
| Core invariants (replay determinism, sequence monotonicity, serde roundtrips) | `proptest` property-based | None |
| Serde format regression (wire compatibility) | Golden-file comparison | None |

### Property-based tests for core invariants

`proptest` is used for invariants that hold over arbitrary inputs:

1. **Serialization roundtrip identity** — for any `EventEnvelope<E>`
   where `E: DomainEvent + Arbitrary`, serializing then deserializing
   produces an equal value.
2. **Sequence monotonicity** — for any sequence of `build_envelopes`
   calls, output sequences are strictly monotonically increasing.
3. **`AggregateId` roundtrip** — for any `NonZeroU64`, constructing
   an `AggregateId` and converting back yields the original value.
   For any `u64`, `TryFrom` succeeds iff the value is non-zero.

Property tests live alongside unit tests in `#[cfg(test)]` modules.

### Compile-fail tests for type contracts

Per CHE-0028. Extended rule: when a new trait bound or associated
type constraint is introduced, a corresponding compile-fail test
must be added. The test suite grows with the type safety surface
area.

### No mock frameworks

Infrastructure tests use real adapters with `tempfile` for isolation.
No mock `EventStore`, no mock `EventBus`. This is consistent with
hexagonal architecture — adapters are tested against real
infrastructure, not simulations. Test doubles are plain structs
implementing the port traits when needed for domain-level testing.

### Test naming convention

`{action}_{condition}_{expected_result}` — already used throughout
the codebase (e.g., `create_rejects_empty_events`,
`append_to_uncreated_aggregate_fails`). Formalized as the standard.

## Consequences

- `proptest` transitions from dead dependency to active use. If
  property-based testing is rejected in a future review, the
  dependency must be removed from the workspace.
- New invariants added by future ADRs must specify which test type
  verifies them: unit, async integration, compile-fail, or property.
- Domain test suites have zero runtime dependencies — they run
  without tokio, without file system access, without network.
- The test boundary is auditable: any test importing `tokio` is an
  infrastructure test. Any `#[test]` function is a domain or
  property test.
- No snapshot testing. Golden-file testing is limited to **one**
  specific purpose: serde format regression (see below). Compile-fail
  tests use `trybuild`'s stderr comparison, which is structurally
  similar but serves a different purpose. Compiler version upgrades
  may require updating `.stderr` files.

Golden-file testing for serde format regression: a deterministic
`EventEnvelope` (fixed UUID bytes, fixed `jiff::Timestamp`, fixed
payload) is serialized to MessagePack and compared byte-for-byte
against a committed fixture at
`cherry-pit-core/tests/fixtures/envelope_golden.msgpack`. This catches
accidental serialization format changes from dependency updates
(jiff, rmp-serde, uuid) before any data is written with an
incompatible wire format.

This is not general snapshot testing — the golden file is a single
regression gate for the most critical serialization path. The
fixture is regenerated by deleting it and running the test.
