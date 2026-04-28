# COM-0017. Mechanized Invariant Enforcement

Date: 2026-04-27
Last-reviewed: 2026-04-27
Tier: A
Status: Accepted

## Related

- References: COM-0009

## Context

COM-0009 (Consistency as Complexity Reducer) rule 3 states:
"Enforce through convention, then tooling." This frames
mechanical enforcement as a fallback escalation from convention.
In practice, cherry-pit inverts this hierarchy: mechanical
enforcement is the *primary* strategy, and convention is the
fallback for rules that cannot yet be mechanized.

**The core insight:** human-enforced rules degrade over time.
Code review catches violations probabilistically — reviewers
are inconsistent, fatigued, and unfamiliar with every rule.
Machine-enforced rules are deterministic: they catch every
violation, on every commit, without fatigue or knowledge gaps.

Ousterhout (Ch. 17) argues that the most effective consistency
mechanisms are those enforced automatically. Martin (Clean Code,
Ch. 5) advocates for formatting rules enforced by tools rather
than conventions. Ford, Richards, Sadalage, and Dehghani
(Software Architecture: The Hard Parts, Ch. 6) describe
"fitness functions" — automated checks that verify
architectural properties continuously.

**Cherry-pit's enforcement hierarchy in practice:**

- **Type system** — `AggregateId(NonZeroU64)` prevents zero IDs
  at compile time. No runtime check needed. No review needed.
  The compiler is the enforcer.

- **Compiler lints** — `#[non_exhaustive]` on error enums
  (CHE-0021) forces downstream callers to handle unknown
  variants. Enforced by `rustc`, not by convention.

- **Static analysis** — `clippy::pedantic` at workspace level
  (CHE-0026) catches implicit conversions, missing docs,
  shadowing, and other subtle issues. Enforced in CI.

- **Compile-fail tests** — `trybuild` tests (CHE-0028) verify
  that invalid type compositions produce compile errors. The
  test suite enforces type contracts that documentation alone
  cannot guarantee.

- **Custom tooling** — `adr-fmt` validates ADR template
  conformance, relationship integrity, naming conventions,
  and section ordering. Enforced on every run.

- **Code review** — the fallback for invariants not yet
  mechanized. Review catches design intent violations,
  naming consistency, and architectural alignment that no
  existing tool checks.

This principle elevates the pattern from an implementation
detail to a citable design principle: when a rule can be
mechanized, it should be.

## Decision

When an invariant can be checked by machine — compiler, linter,
formatter, type system, or test harness — it must be. Human
enforcement through code review is the fallback for rules that
cannot yet be mechanized, not the primary strategy.

### Rules

1. **Prefer compile-time over runtime.** A type-level constraint
   that prevents invalid states is stronger than a runtime check
   that detects them. `NonZeroU64` is better than
   `assert!(id != 0)`. A sealed trait is better than a doc
   comment saying "do not implement."

2. **Prefer linter over reviewer.** A clippy lint that flags a
   pattern is more reliable than a review comment citing a
   convention. When a recurring review comment identifies a
   mechanical pattern, investigate whether a lint can enforce it.

3. **Prefer CI gate over local convention.** A CI check that
   blocks merge on violation is more reliable than a development
   guide that asks developers to run a tool locally. Mechanical
   enforcement in CI guarantees that no violation reaches the
   main branch.

4. **Document the enforcement mechanism.** Every ADR that
   establishes an invariant should state how the invariant is
   enforced: type system, lint, CI check, compile-fail test,
   or code review. If the answer is "code review only," the
   invariant is a candidate for mechanization.

5. **Escalation ladder.** Enforcement mechanisms from strongest
   to weakest: type system → compiler error → compiler lint →
   CI gate (test/lint/format) → code review → documentation.
   Choose the strongest feasible mechanism. Move enforcement
   up the ladder as tooling improves.

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
