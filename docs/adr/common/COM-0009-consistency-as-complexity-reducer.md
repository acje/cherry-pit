# COM-0009. Consistency as Complexity Reducer

Date: 2026-04-26
Last-reviewed: 2026-04-28
Tier: B
Status: Accepted

## Related

- References: COM-0001

## Context

Ousterhout (Ch. 17, "Consistency") identifies consistency as one of
the most powerful tools for reducing cognitive complexity. When
similar things are done in similar ways, developers can leverage
knowledge from one context to understand another without re-reading
code. Inconsistency forces developers to treat each instance as
novel, even when the underlying pattern is the same.

**Categories of consistency:**

- **Naming** — the same concept uses the same name everywhere.
  Different concepts use different names. A name never means two
  different things in two contexts.

- **Coding patterns** — similar operations follow the same
  structural pattern. If event application is always
  `fn apply(&mut self, event: &E)`, every aggregate follows this
  shape. A developer who has seen one aggregate knows how to read
  all of them.

- **Design patterns** — architectural decisions are applied
  uniformly. If errors are defined out of existence (COM-0005) in
  one subsystem, the same approach is used in all comparable
  subsystems. Mixed strategies force developers to remember which
  strategy applies where.

- **Invariants** — properties that are always true reduce the space
  of possibilities a developer must consider. "apply() is always
  infallible" (CHE-0009) means developers never need to check
  whether a particular aggregate's apply can fail.

**The cost of inconsistency is invisible.** Each inconsistency is
small — a different naming convention here, a different error
pattern there. But they accumulate into a system where every module
must be read from scratch because patterns do not transfer.

Cherry-pit applies consistency extensively:

- **Vocabulary** — `create`, `load`, `append` across all store
  implementations. `apply` across all aggregates. `handle` across
  all command handlers.
- **Infallibility pattern** — `apply()` returns `()` on every
  aggregate and projection (CHE-0009). No exceptions.
- **Error-per-command** — every command has its own error type
  (CHE-0015). No mixed approaches with shared error enums.
- **Envelope construction** — the store always constructs envelopes
  (CHE-0016). No caller-constructed envelopes anywhere.

## Decision

When a pattern is established, apply it uniformly. Inconsistency is
permitted only when the difference reflects a genuine semantic
distinction, not convenience or historical accident.

### Rules

1. **Same concept, same name.** Establish canonical names for
   recurring concepts and use them everywhere. The glossary of names
   is implicit in the codebase and explicit in the ADR system.
   Synonyms are inconsistencies.

2. **Same problem, same pattern.** When a new module solves a
   problem that an existing module has already solved, use the same
   structural approach. Deviation requires justification — "this
   case is genuinely different because X."

3. **Enforce through convention, then tooling.** Document the pattern
   in an ADR. Enforce mechanically where possible (trait signatures,
   compile-fail tests, lints). Manual enforcement through code
   review is the fallback.

4. **Inconsistency requires justification.** When a PR introduces a
   pattern that differs from the established approach, the author
   must explain why the difference is semantically necessary. "I
   didn't know about the existing pattern" is not justification —
   it is a signal that the pattern needs better visibility.

5. **Update holistically.** When a pattern changes, update all
   instances. Partial migration creates the worst form of
   inconsistency: the developer cannot know which pattern is
   current without checking each instance.

## Consequences

- Trait method naming follows a consistent vocabulary: `create`,
  `load`, `append`, `apply`, `handle`. New traits are expected to
  follow this vocabulary or justify a different one.
- The infallibility invariant (CHE-0009) eliminates an entire class
  of "which aggregates can fail during apply?" questions. Developers
  learn the pattern once.
- Error-per-command (CHE-0015) is applied uniformly. No aggregate
  uses a shared error enum while others use per-command errors.
- The ADR system itself is a consistency tool: every ADR follows
  the same template, uses the same vocabulary (References,
  Supersedes, Root), and is validated by the same linter.
- Consistency creates inertia. Changing a well-established pattern
  requires updating all instances, which raises the bar for change.
  This is intentional for high-tier patterns (S, A) and may be
  relaxed for lower tiers (D) where local variation is acceptable.
