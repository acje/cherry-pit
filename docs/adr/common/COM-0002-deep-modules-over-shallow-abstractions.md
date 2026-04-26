# COM-0002. Deep Modules Over Shallow Abstractions

Date: 2026-04-25
Last-reviewed: 2026-04-25
Tier: S

## Status

Accepted

## Related

- References: COM-0001

## Context

Ousterhout (Ch. 4, "Modules Should Be Deep") introduces a
cost-benefit model for abstraction: every module has an *interface*
(the complexity visible to users) and an *implementation* (the
complexity hidden inside). A module's value is the ratio of hidden
complexity to interface complexity.

- **Deep modules** have simple interfaces and powerful
  implementations. The classic example is Unix file I/O: five system
  calls (`open`, `read`, `write`, `lseek`, `close`) hide an enormous
  implementation (buffering, caching, device drivers, disk scheduling,
  file system layout, permissions, locking).

- **Shallow modules** have interfaces that are nearly as complex as
  their implementations. They provide little abstraction — the caller
  must understand almost as much as if they had written the code
  themselves. Shallow modules are not inherently wrong, but they fail
  the cost-benefit test: they add interface complexity without
  proportional implementation hiding.

**Red flags for shallow modules** (Ousterhout, Ch. 4–5):

- A trait with many methods where each does trivial work
- A wrapper type that mirrors the interface of what it wraps
- A "classitis" pattern where every concept gets its own type
  regardless of whether it hides complexity
- An interface that exposes implementation details (leaky
  abstraction)

Cherry-pit's `EventStore` trait demonstrates depth: three methods
(`create`, `load`, `append`) hide file I/O, MessagePack
serialization, concurrency locking, atomic writes, sequence
validation, envelope construction, and aggregate ID assignment.

## Decision

Prefer deep modules: simple interfaces hiding substantial
implementation complexity. Measure module depth by the ratio of
interface complexity to implementation complexity.

### Rules

1. **Interface-to-implementation ratio test.** Before adding a public
   method, trait method, or type parameter, ask: "Does this addition
   hide more complexity than it exposes?" If not, the module is
   becoming shallower.

2. **Combine related functionality.** Small methods that are always
   called together should be a single method. Small types that are
   always used together should be a single type. The Unix I/O model
   is the exemplar: one `read` call, not separate `allocate_buffer`,
   `seek_position`, `read_bytes`, `check_permissions` calls.

3. **General-purpose interfaces, special-purpose implementations.**
   Interfaces should be general enough to support multiple use cases
   without configuration. Implementations may be highly specialized.
   The `EventStore` trait works for any aggregate type — the
   implementation (`MsgpackFileStore`) makes specific choices about
   serialization format and storage layout.

4. **Red flags that trigger refactoring:**
   - Trait with >5 required methods — likely too shallow
   - Wrapper type with pass-through methods — no abstraction value
   - Type parameter that only appears in one method — over-generic
   - Configuration struct with >3 fields — complexity pushed to
     caller (see COM-0003)

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
