# COM-0011. Trade-Off Analysis as First-Class Practice

Date: 2026-04-26
Last-reviewed: 2026-04-28
Tier: B
Status: Accepted

## Related

References: COM-0001, COM-0008

## Context

Richards and Ford (Fundamentals of Software Architecture, Ch. 2) state: "Everything in software architecture is a trade-off." This principle is distinct from COM-0001 (complexity budget), which addresses *how much* complexity is justified. Trade-off analysis addresses the prior question: *which qualities compete*, and *which side does this project choose*?

Cherry-pit's quality attribute trade-offs include correctness over performance (CHE-0028, CHE-0009, CHE-0032), simplicity over flexibility (CHE-0037, CHE-0040, GEN-0031), safety over ergonomics (`#[non_exhaustive]` per CHE-0021, `forbid(unsafe_code)` per CHE-0007), and consistency over local optimality (uniform error-per-command per CHE-0015, infallible apply per CHE-0009, with COM-0009 justifying the global benefit).

The ADR system is inherently a trade-off documentation system: Context presents competing forces, Decision resolves them, Consequences makes costs explicit. What is missing is a citable principle making trade-off analysis mandatory and providing vocabulary for it.

## Decision

Every architectural decision is a trade-off. The competing quality
attributes and the rationale for the chosen balance must be
explicit in the decision record.

R1 [5]: Reject decision framing that claims an approach is
  universally superior; reframe as "optimizes for X at cost of Y"
R2 [5]: Every ADR Context section must identify at least two quality
  attributes in tension
R3 [6]: Cherry-pit's priority ordering — correctness then safety
  then simplicity then performance then flexibility — is the
  standing resolution for common trade-offs
R4 [5]: The Consequences section must state what was given up, not
  just what was gained
R5 [6]: When project scale or requirements shift, review affected
  ADRs against their trade-off assumptions

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
