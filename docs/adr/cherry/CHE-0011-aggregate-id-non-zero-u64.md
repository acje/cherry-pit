# CHE-0011. AggregateId as NonZeroU64 Newtype

Date: 2026-04-24
Last-reviewed: 2026-04-25
Tier: A

## Status

Accepted

## Related

- References: CHE-0002, CHE-0006

## Context

Aggregates need identifiers. Options considered:

1. **UUID** — globally unique, no coordination needed. 128 bits, no
   Copy without Clone, serialization overhead.
2. **Plain `u64`** — simple, fast, Copy. But zero is constructible
   and never assigned by the store (IDs start from 1).
3. **`NonZeroU64` newtype** — same as `u64` but eliminates the zero
   hole at the type level. Niche optimization makes
   `Option<AggregateId>` the same size as `AggregateId`.

The original design used `u64`. During architectural review,
`AggregateId(0)` was identified as a latent invariant violation:
constructible via `AggregateId::new(0)` or `From<u64>` but never
assigned by the store.

## Decision

`AggregateId` wraps `NonZeroU64`. The constructor takes `NonZeroU64`
directly. `TryFrom<u64>` is provided for fallible conversion from raw
values. `From<u64>` is removed (replaced by `TryFrom`).

Store-assigned IDs auto-increment from 1 via `NonZeroU64`.

## Consequences

- Zero is no longer a valid aggregate ID at the type level — no
  runtime guard needed.
- `Option<AggregateId>` benefits from niche optimization (same size
  as `AggregateId`).
- Copy semantics preserved — `NonZeroU64` is `Copy`.
- Serde deserializes as `u64` but rejects zero automatically via
  `NonZeroU64`'s `Deserialize` impl.
- `AggregateId::get()` returns plain `u64` for ergonomic use in
  format strings, file paths, etc.
- `TryFrom<u64>` replaces `From<u64>` — callers converting from raw
  `u64` must handle the error case. This is the correct tradeoff:
  the error surfaces at the point of construction, not deep inside
  the store.
- IDs are not globally unique across aggregate types — two different
  aggregates will both have `AggregateId(1)`. Cross-context references
  require domain-level external IDs.
