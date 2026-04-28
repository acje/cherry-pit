# COM-0009. Consistency as Complexity Reducer

Date: 2026-04-26
Last-reviewed: 2026-04-28
Tier: B
Status: Accepted

## Related

References: COM-0001

## Context

Ousterhout (Ch. 17, "Consistency") identifies consistency as one of the most powerful tools for reducing cognitive complexity. When similar things are done in similar ways, developers leverage knowledge across contexts without re-reading code. Inconsistency forces each instance to be treated as novel. Consistency spans naming (same concept, same name everywhere), coding patterns (if `apply` always takes `&mut self, event: &E`, one aggregate teaches all), design patterns (COM-0005 applied uniformly, not mixed strategies), and invariants ("apply() is always infallible" per CHE-0009 eliminates per-aggregate checking). The cost of inconsistency is invisible but cumulative — eventually every module must be read from scratch.

Cherry-pit applies consistency through uniform vocabulary (`create`, `load`, `append`, `apply`, `handle`), infallible `apply()` returning `()` on every aggregate (CHE-0009), error-per-command with dedicated types (CHE-0015), and store-owned envelope construction (CHE-0016).

## Decision

When a pattern is established, apply it uniformly. Inconsistency is
permitted only when the difference reflects a genuine semantic
distinction, not convenience or historical accident.

R1 [5]: Establish canonical names for recurring concepts and use them
  everywhere; synonyms are inconsistencies
R2 [5]: When a new module solves a problem an existing module has
  solved, use the same structural approach; deviation requires
  justification that the case is genuinely different
R3 [6]: Document patterns in ADRs and enforce mechanically where
  possible through trait signatures, compile-fail tests, or lints
R4 [5]: Inconsistency in a PR requires the author to explain why the
  difference is semantically necessary
R5 [5]: When a pattern changes, update all instances; partial
  migration creates the worst form of inconsistency

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
