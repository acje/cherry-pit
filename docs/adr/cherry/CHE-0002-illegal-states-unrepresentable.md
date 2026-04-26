# CHE-0002. Make Illegal States Unrepresentable

Date: 2026-04-25
Last-reviewed: 2026-04-25
Tier: S

## Status

Accepted

## Related

- Depends on: CHE-0001

## Context

P1 (CHE-0001) mandates correctness first. The most effective
correctness technique is designing types where invalid values cannot
be constructed — the compiler rejects illegal states rather than
runtime guards catching them.

This principle is applied throughout cherry-pit but never stated as
its own decision. It informs: `AggregateId(NonZeroU64)` (CHE-0011),
associated types preventing cross-aggregate confusion (CHE-0005),
exhaustive event enums (CHE-0022), and infallible apply (CHE-0009).

## Decision

Every cherry-pit type must encode its invariants at the type level.

- Newtypes prevent primitive confusion (`AggregateId` vs raw `u64`).
- `NonZero*` types eliminate zero holes without runtime checks.
- Associated types fix relationships at compile time, not per-call.
- Exhaustive enums force handling of every variant — no `_` wildcards
  on domain types.
- Validated constructors return `Result`, preventing invalid instances.

Runtime guards (e.g., `expected_sequence` on `append`) are
defense-in-depth, not primary enforcement.

## Consequences

- Higher type complexity — more newtypes, associated types, and
  `where` clauses than a permissive design.
- More verbose APIs at definition sites. Dramatically fewer runtime
  failure modes at call sites.
- Users defining aggregates, events, and commands inherit this
  discipline — the framework's trait bounds enforce it transitively.
- See `docs/agent-guidance.md` for prescriptive application of this
  principle during framework customization.
