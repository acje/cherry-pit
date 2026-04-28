# COM-0011. Trade-Off Analysis as First-Class Practice

Date: 2026-04-26
Last-reviewed: 2026-04-28
Tier: B
Status: Accepted

## Related

- References: COM-0001

## Context

Richards and Ford (Fundamentals of Software Architecture, Ch. 2,
"Architectural Thinking") state the first law of software
architecture: "Everything in software architecture is a trade-off."
Ford, Richards, Sadalage, and Dehghani (Software Architecture: The
Hard Parts, Ch. 1) extend this: "If an architect thinks they have
discovered something that isn't a trade-off, more likely they just
haven't identified the trade-off yet."

This principle is distinct from COM-0001 (complexity budget), which
addresses *how much* complexity is justified. Trade-off analysis
addresses the prior question: *which qualities compete*, and *which
side of each trade-off does this project choose*?

**Quality attribute trade-offs in cherry-pit:**

- **Correctness vs. performance** — compile-fail tests (CHE-0028),
  infallible apply (CHE-0009), and atomic file writes (CHE-0032)
  all prioritize correctness. Performance is explicitly second
  (CHE-0001: design priority ordering).

- **Simplicity vs. flexibility** — no snapshot support (CHE-0037),
  no saga support (CHE-0040), and no cross-language support
  (GEN-0031) all choose simplicity now over flexibility later.

- **Safety vs. ergonomics** — `#[non_exhaustive]` on errors
  (CHE-0021) and `forbid(unsafe_code)` (CHE-0007, GEN-0006) trade
  caller convenience for long-term safety guarantees.

- **Consistency vs. local optimality** — uniform error-per-command
  (CHE-0015) and uniform infallible apply (CHE-0009) may not be
  locally optimal for every aggregate, but the global consistency
  benefit (COM-0009) outweighs local gains.

The ADR system is inherently a trade-off documentation system: the
Context section presents the competing forces, the Decision section
resolves them, and the Consequences section makes the costs
explicit. What is missing is a citable principle that makes
trade-off analysis mandatory and provides vocabulary for discussing
it.

## Decision

Every architectural decision is a trade-off. The competing quality
attributes and the rationale for the chosen balance must be
explicit in the decision record.

### Rules

1. **No "best" designs.** Reject decision framing that claims an
   approach is universally superior. Reframe as: "This approach
   optimizes for X at the cost of Y."

2. **Name the competing attributes.** Every ADR Context section
   must identify at least two quality attributes in tension.
   Common pairs: correctness vs. performance, simplicity vs.
   flexibility, safety vs. ergonomics, consistency vs. local
   optimality, coupling vs. autonomy.

3. **State the project's bias.** Cherry-pit's priority ordering
   (CHE-0001) is the standing resolution for common trade-offs:
   correctness > safety > simplicity > performance > flexibility.
   Decisions that align with this ordering need only cite it.
   Decisions that deviate must justify the deviation.

4. **Make costs explicit.** The Consequences section must state what
   was given up, not just what was gained. "We chose X" is
   incomplete without "at the cost of Y, mitigated by Z."

5. **Revisit when context changes.** A trade-off resolved under one
   set of constraints may reverse under different constraints. When
   the project's scale, audience, or requirements shift, affected
   ADRs should be reviewed for continued validity.

## Consequences

- The ADR template's Context and Consequences sections are the
  structural enforcement mechanisms. An ADR that presents only
  benefits without costs is incomplete under this principle.
- CHE-0001 (design priority ordering) becomes the project's
  standing trade-off resolution, reducing per-decision cognitive
  load for common quality attribute conflicts.
- Deliberate deferrals (CHE-0037, CHE-0040, GEN-0031) are
  validated as explicit trade-offs: flexibility deferred in
  exchange for simplicity now, with documented conditions under
  which the trade-off should be revisited.
- This principle makes COM-0008 (Design It Twice) more productive:
  comparing two designs is only useful if the comparison axes
  (quality attributes) are explicit. Trade-off analysis provides
  the vocabulary.
- Risk of over-analysis. The mitigation is COM-0008's time-box:
  trade-off analysis should be proportional to the decision's
  tier and reversibility. Tier-D decisions need a sentence;
  tier-S decisions need a section.
