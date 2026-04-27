# CHE-0002. Make Illegal States Unrepresentable

Date: 2026-04-25
Last-reviewed: 2026-04-27
Tier: S

## Status

Accepted

Amended 2026-04-27 — added quantitative enforcement asymmetry to
  Context

## Related

- References: CHE-0001

## Context

P1 (CHE-0001) mandates correctness first. The most effective
correctness technique is designing types where invalid values cannot
be constructed — the compiler rejects illegal states rather than
runtime guards catching them.

Two enforcement strategies exist for invariants:

1. **Runtime guards** — `assert!(id != 0)`, `if seq == 0 { return
   Err(...) }`. Guards run on every invocation. They can be bypassed
   by bugs, disabled in release builds (`debug_assert!`), or
   forgotten in new code paths. The invariant holds only as long as
   every code path checks it. Maintenance cost scales with call
   sites.

2. **Type-level encoding** — `AggregateId(NonZeroU64)`. The invariant
   is enforced once, at construction time. Every subsequent use of the
   value benefits without any runtime cost. New code paths inherit the
   guarantee automatically — the type system carries it. Maintenance
   cost is O(1).

The asymmetry is fundamental: runtime guards are O(call sites),
type-level invariants are O(1). As the codebase grows, runtime guards
become increasingly likely to miss a path. Type-level invariants
cannot be circumvented by any amount of code growth.

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
