# COM-0007. Information Hiding — Minimize Leakage

Date: 2026-04-26
Last-reviewed: 2026-04-26
Tier: A
Status: Accepted

## Related

- References: COM-0002

## Context

Ousterhout (Ch. 5, "Information Hiding (and Leakage)") refines the
deep-module principle (COM-0002) from *how much* to hide to *what
specifically* to hide. A deep module is necessary but not sufficient
— if the wrong details leak through the interface, the module is
deep in implementation but shallow in abstraction.

**Information hiding** means each module encapsulates knowledge of
its design decisions — data representations, algorithms, low-level
mechanisms — so that other modules cannot depend on them. The
complement is **information leakage**: when a design decision is
reflected in multiple modules, creating hidden coupling. Changing
the decision requires coordinated changes across all leaked sites.

**Forms of leakage:**

- **Interface leakage** — implementation details exposed through
  method signatures, type parameters, or public fields. Callers
  become coupled to the representation.

- **Temporal decomposition** — splitting a task into sequential
  phases where each phase becomes a module. The modules share
  knowledge about the overall sequence, data formats, and
  intermediate state. Example: separate "read config," "parse
  config," and "validate config" modules that all know the config
  format.

- **Back-channel leakage** — implementation details shared through
  documentation, conventions, or implicit contracts rather than
  through the interface itself. Formally decoupled but practically
  coupled.

Cherry-pit demonstrates information hiding at several boundaries:

- **EventEnvelope fields** are `pub(crate)` — consumers access
  envelope data through methods, not direct field access. The
  internal representation (field layout, optional fields) can
  change without affecting consumers.

- **MsgPack format** is invisible to trait users — `EventStore`
  consumes and produces domain types. The serialization format is a
  hidden design decision inside `MsgpackFileStore`.

- **File layout** (one file per stream, atomic rename) is hidden
  behind the `EventStore` trait. Callers never construct file paths
  or manage file handles.

## Decision

Design decisions should be encapsulated within the module that owns
them. No other module should need to know — or be able to depend
on — the decision.

### Rules

1. **Identify the design decisions.** Before implementing a module,
   list the design decisions it embodies: data representation,
   algorithm choice, protocol details, resource management strategy.
   These are candidates for hiding.

2. **Hide representation.** Types exposed through interfaces should
   describe the abstraction, not the implementation. Return
   `AggregateId`, not `NonZeroU64`. Accept `impl Iterator`, not
   `Vec`. Expose methods, not fields.

3. **Detect temporal decomposition.** If multiple modules must be
   modified together when a format, protocol, or sequence changes,
   they share leaked knowledge. Consolidate them into a single
   module that owns the full sequence.

4. **Minimize pub surface.** Default to private. Promote to
   `pub(crate)` only when another module within the crate needs
   access. Promote to `pub` only when another crate needs access.
   Every visibility increase is a leakage risk.

5. **Red flags for leakage:**
   - Two modules parsing the same data format — format knowledge
     leaked
   - A type that mirrors an internal data structure in its public
     API — representation leaked
   - A caller that must sequence calls in a specific order because
     the module does not manage its own state — temporal leakage
   - Documentation that says "this must match the implementation
     in module X" — back-channel leakage

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
