# COM-0007. Information Hiding — Minimize Leakage

Date: 2026-04-26
Last-reviewed: 2026-04-28
Tier: B
Status: Accepted

## Related

References: COM-0001, COM-0002

## Context

Ousterhout (Ch. 5, "Information Hiding (and Leakage)") refines the deep-module principle (COM-0002) from *how much* to hide to *what specifically* to hide. A deep module is necessary but not sufficient — if the wrong details leak, the module is deep in implementation but shallow in abstraction. Information hiding means each module encapsulates its design decisions so other modules cannot depend on them. Information leakage — when a decision is reflected in multiple modules — creates hidden coupling requiring coordinated changes.

Leakage takes several forms: interface leakage through method signatures or public fields; temporal decomposition where sequential phases share knowledge of formats and intermediate state; and back-channel leakage through documentation or implicit contracts that create practical coupling despite formal decoupling.

Cherry-pit demonstrates information hiding at several boundaries. `EventEnvelope` fields are `pub(crate)`, with consumers accessing data through methods so internal layout can change freely. The MsgPack format is invisible to trait users — `EventStore` consumes and produces domain types. File layout (one file per stream, atomic rename) is hidden behind the `EventStore` trait; callers never construct file paths or manage handles.

## Decision

Design decisions should be encapsulated within the module that owns
them. No other module should need to know — or be able to depend
on — the decision.

R1 [5]: Before implementing a module, list its design decisions —
  data representation, algorithm, protocol, resource strategy —
  as candidates for hiding
R2 [5]: Types exposed through interfaces describe the abstraction,
  not the implementation; return AggregateId not NonZeroU64, accept
  impl Iterator not Vec
R3 [6]: When multiple modules must change together for a format or
  protocol change, consolidate them into a single module that owns
  the full sequence
R4 [5]: Default to private visibility; promote to pub(crate) only
  when needed within the crate, and to pub only when needed by
  another crate

## Consequences

- `EventEnvelope` accessors are the primary example: field layout
  is a hidden decision. Adding `correlation_id` and `causation_id`
  (CHE-0016, CHE-0039) did not change the consumer API because
  fields were already hidden behind methods.
- Serialization format changes (CHE-0031: MessagePack, CHE-0045:
  serialization scope) are isolated to implementation modules. No
  trait user knows about `rmp_serde`.
- New infrastructure ports (CHE-0044: object store) can change the
  storage mechanism without leaking storage concepts through the
  `EventStore` trait.
- Temporal decomposition is prevented by the store's atomic
  create/load/append design — envelope construction, sequencing,
  and persistence are consolidated, not split into sequential
  phases.
- Overuse of information hiding can make debugging harder. The
  mitigation is the same as COM-0003: expose internal state through
  error messages and structured logging during failures, not
  through the interface during normal operation.
