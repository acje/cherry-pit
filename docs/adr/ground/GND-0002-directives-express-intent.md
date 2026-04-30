# GND-0002. Directives Express Intent, Not Mechanism

Date: 2026-04-30
Last-reviewed: 2026-04-30
Tier: S
Status: Accepted

## Related

References: GND-0001

## Context

Under irreducible uncertainty (GND-0001), the executor of a directive
will encounter circumstances the issuer did not foresee. Von Moltke
distinguished *Direktive* (intent + boundaries) from *Befehl*
(prescribed action). Bungay shows that Befehl-style directives become
brittle the moment reality diverges from the plan; Direktive-style
directives let the executor adapt without violating intent.

Three options:

1. **Prescribe action** — directives state what to do step by step.
   Executors stall or violate when reality diverges.
2. **State only outcomes** — directives state goals with no boundaries.
   Executors freelance; alignment collapses.
3. **State intent and boundaries; leave action to the executor** —
   directive answers *what to achieve*, *why*, *what is in bounds*,
   and *what is out of bounds*. Executor chooses *how*.

Option 3 chosen: it preserves alignment and grants the autonomy
needed to absorb friction at the point of contact.

## Decision

Every directive — every ADR, every charter, every binding policy —
states intent, rationale, and boundaries. Implementation choices
belong to the executor unless the implementation choice is itself
the load-bearing decision.

R1 [2]: Express every ADR's tagged rules as statements of *what to
  achieve* and *why*, naming the artifact constrained but leaving
  the executor's choice of approach explicit where the approach is
  not the decision
R2 [2]: Treat ADRs that prescribe step-by-step procedure as a smell;
  refactor toward intent-plus-boundary or move the steps into a
  separate operational protocol document
R3 [3]: Distinguish in every ADR whether the decision constrains
  *the structure* (and the implementation may vary) or *the
  implementation* (because the implementation is the structure)

## Consequences

- **Closes the alignment gap by design.** Executors aligned on intent
  can adapt action to circumstance without re-issuing the directive.
- **Moves implementation discretion downward.** Per-crate work, per-
  module choices, and per-incident responses live with the executor,
  not the directive.
- **Smell signal for the corpus.** ADRs whose tagged rules read as
  procedures are flagged for refactoring. Some legitimately prescribe
  mechanism (the type-level safety ADRs); those are explicit cases,
  not the default.
- **Tension with maximally concrete tagged rules.** Tagged rules name
  types and methods to anchor attention; this remains compatible —
  the type is the bounded artifact, the rule states the intent the
  type expresses.
