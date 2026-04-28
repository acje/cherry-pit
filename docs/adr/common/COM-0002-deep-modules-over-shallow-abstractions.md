# COM-0002. Deep Modules Over Shallow Abstractions

Date: 2026-04-25
Last-reviewed: 2026-04-25
Tier: S
Status: Accepted

## Related

References: COM-0001

## Context

Ousterhout (Ch. 4, "Modules Should Be Deep") models abstraction as a cost-benefit ratio: every module has an *interface* (complexity visible to users) and an *implementation* (complexity hidden inside). A module's value is the ratio of hidden complexity to interface complexity. Deep modules have simple interfaces hiding powerful implementations — Unix file I/O's five system calls concealing buffering, caching, device drivers, and file system layout. Shallow modules have interfaces nearly as complex as their implementations, failing the cost-benefit test.

Red flags for shallow modules (Ousterhout, Ch. 4–5) include traits with many trivial methods, wrapper types mirroring what they wrap, "classitis" patterns where every concept gets its own type regardless of hidden complexity, and interfaces exposing implementation details.

Cherry-pit's `EventStore` trait demonstrates depth: three methods (`create`, `load`, `append`) hide file I/O, MessagePack serialization, concurrency locking, atomic writes, sequence validation, envelope construction, and aggregate ID assignment.

## Decision

Prefer deep modules: simple interfaces hiding substantial
implementation complexity. Measure module depth by the ratio of
interface complexity to implementation complexity.

R1 [2]: Every module exposes a simple interface and hides
  implementation complexity behind it
R2 [2]: Before adding a public method or type parameter, verify it
  hides more complexity than it exposes
R3 [2]: Combine small methods always called together into a single
  method; combine small types always used together into a single type
R4 [2]: Interfaces should be general-purpose while implementations
  may be highly specialized
R5 [3]: Red flags — trait with more than five required methods, wrapper
  with pass-through methods, type parameter in only one method —
  trigger refactoring toward deeper modules

### Exceptions

Some modules are intentionally shallow for type safety reasons.
`AggregateId(NonZeroU64)` is a newtype wrapper that adds no
implementation depth but prevents type confusion. These are justified
under COM-0001 (correctness > simplicity) and should be documented
as deliberate exceptions.

## Consequences

- New traits are evaluated for depth before acceptance. A trait
  proposal with many trivial methods is challenged to consolidate.
- The `EventStore` pattern (3 methods hiding 7+ concerns) is the
  benchmark for future infrastructure ports.
- Newtype wrappers for type safety are accepted as justified shallow
  modules — their value comes from the type system, not from
  implementation hiding.
- This principle creates tension with fine-grained error types
  (CHE-0015, CHE-0021): each error variant adds interface complexity.
  COM-0005 (define errors out of existence) resolves this tension by
  eliminating unnecessary error variants.
- "Classitis" refactoring becomes a named practice: when a PR
  introduces multiple small types that could be consolidated,
  reviewers can cite COM-0002.
