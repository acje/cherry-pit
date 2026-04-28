# CHE-0001. Design Priority Ordering

Date: 2026-04-25
Last-reviewed: 2026-04-25
Tier: S
Status: Accepted

## Related

- References: COM-0001

## Context

Cherry-pit makes many tradeoff decisions: type-system strictness vs
ergonomics, runtime checks vs performance, dependency count vs
convenience, compile time vs binary quality. Without a shared ranking,
each decision becomes a local argument with no resolution criterion.

Several existing ADRs already reference "P1: correctness > speed"
(CHE-0026), "no unsafe" (CHE-0007), and "pure, deterministic command
handling" (CHE-0008). These citations point to an implicit priority
system that has never been formally established as its own decision.

Four candidate priorities:

1. **Correctness** — reject wrong code at compile time. Total
   functions. No undefined behavior.
2. **Security** — no data leakage across bounded contexts. Validate
   at boundaries.
3. **Energy efficiency** — do less work, not faster work. Avoid
   unnecessary allocations, cloning, and serialization.
4. **Response time** — fast, but never at the cost of correctness.

The question is not whether these matter — all do — but their strict
rank order when they conflict.

## Decision

Every design decision is evaluated against these priorities in strict
rank order:

| Priority | Name | Principle |
|----------|------|-----------|
| P1 | Correctness | Make illegal states unrepresentable. Lean on the type system. Total functions. No unsafe. |
| P2 | Secure | No data leakage across bounded contexts. Validate at boundaries. |
| P3 | Energy efficient | Do less work, not faster work. Avoid unnecessary allocations, cloning, serialization. |
| P4 | Response time | Fast, but never at the cost of correctness. |

"Strict rank order" means: when P1 and P4 conflict, P1 wins without
debate. When P2 and P3 conflict, P2 wins. No decision may optimize a
lower-priority concern at the expense of a higher one.

This ADR is the canonical source for the priority system. Other ADRs
cite it by number when invoking a priority tradeoff.

## Consequences

- **`overflow-checks = true` in release** (CHE-0026) — P1 overrides
  P4. Integer overflow panics even in production.
- **`#![forbid(unsafe_code)]`** (CHE-0007) — P1 eliminates an entire
  class of correctness failures. Performance-sensitive unsafe
  patterns are unavailable.
- **Pure command handling** (CHE-0008) — P1 prefers deterministic,
  testable handlers over convenient mutable receivers.
- **Associated types over trait objects** (CHE-0005) — P1 enforces
  compile-time type safety at the cost of object-safety ergonomics.
- **Single-aggregate typing** (CHE-0005) — P2 prevents cross-context
  data leakage at the type level.
- **RPITIT over async_trait** (CHE-0025) — P3 avoids heap allocation
  per dispatch. Does not conflict with P1 or P2.
- **`DeserializeOwned` on `DomainEvent`** (CHE-0010) — P1 (owned
  values simplify lifetime reasoning) overrides P3 (zero-copy
  deserialization would be more efficient).
- New ADRs should explicitly cite the priority tradeoff that drove
  the decision. This forces authors to articulate *which* priority
  won and *which* was sacrificed.
- The ranking is intentionally opinionated. Projects that prioritize
  response time over correctness should not adopt cherry-pit.
