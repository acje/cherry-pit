# COM-0017. Mechanized Invariant Enforcement

Date: 2026-04-27
Last-reviewed: 2026-04-28
Tier: B
Status: Accepted

## Related

References: COM-0001, COM-0009

## Context

COM-0009 rule 3 frames mechanical enforcement as a fallback from convention. In practice, cherry-pit inverts this: mechanical enforcement is the *primary* strategy, and convention is the fallback for rules that cannot yet be mechanized. Human-enforced rules degrade over time — reviewers are inconsistent and fatigued. Machine-enforced rules are deterministic, catching every violation on every commit. Ousterhout (Ch. 17), Martin (Clean Code, Ch. 5), and Ford et al. (The Hard Parts, Ch. 6) all advocate automated enforcement through "fitness functions."

Cherry-pit's enforcement hierarchy proceeds from strongest to weakest: the type system (`AggregateId(NonZeroU64)` prevents zero IDs at compile time), compiler lints (`#[non_exhaustive]` per CHE-0021), static analysis (`clippy::pedantic` per CHE-0026), compile-fail tests (`trybuild` per CHE-0028), custom tooling (`adr-fmt` for ADR conformance), and code review as the fallback for invariants not yet mechanized. This principle elevates the pattern to a citable design principle: when a rule can be mechanized, it should be.

## Decision

When an invariant can be checked by machine — compiler, linter,
formatter, type system, or test harness — it must be. Human
enforcement through code review is the fallback for rules that
cannot yet be mechanized, not the primary strategy.

R1 [5]: Prefer compile-time constraints over runtime checks;
  a type-level constraint that prevents invalid states is stronger
  than a runtime assertion that detects them
R2 [5]: Prefer a linter rule over a reviewer convention; when a
  recurring review comment identifies a mechanical pattern,
  investigate whether a lint can enforce it
R3 [6]: CI gates that block merge on violation are more reliable
  than local conventions; mechanical enforcement in CI guarantees
  no violation reaches the main branch
R4 [5]: Every ADR that establishes an invariant must state its
  enforcement mechanism — type system, lint, CI check, compile-fail
  test, or code review
R5 [6]: Enforcement escalation ladder from strongest to weakest:
  type system, compiler error, compiler lint, CI gate, code review,
  documentation — choose the strongest feasible mechanism

## Consequences

- The ADR template gains an implicit review question: "How is
  this invariant enforced?" ADRs that establish rules without
  enforcement mechanisms are incomplete under COM-0017.
- Compile-fail tests (CHE-0028) are validated as a first-class
  enforcement mechanism, not a testing novelty. They sit between
  "compiler lint" and "CI gate" on the escalation ladder.
- `adr-fmt` is validated as an architectural enforcement tool,
  not just a formatter. It mechanizes invariants (template
  conformance, link integrity, naming conventions) that would
  otherwise require manual review.
- Clippy pedantic (CHE-0026) is validated as an enforcement
  strategy, not just a code quality preference.
- New invariants proposed in ADRs will be challenged to identify
  their enforcement mechanism. "We'll catch it in review" is
  the weakest acceptable answer and signals a mechanization
  opportunity.
- Risk of over-mechanization: not every guideline benefits from
  rigid enforcement. Taste, naming quality, and architectural
  judgment resist mechanization. The escalation ladder
  acknowledges this — code review remains a valid mechanism for
  subjective invariants.
