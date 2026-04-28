# COM-0008. Design It Twice

Date: 2026-04-26
Last-reviewed: 2026-04-26
Tier: B
Status: Accepted

## Related

References: COM-0001

## Context

Ousterhout (Ch. 11, "Design It Twice") observes that the first design that comes to mind is unlikely to be the best. For any significant decision, sketching at least two fundamentally different approaches exposes trade-offs a single-pass design misses. The cost of exploring a second design is minutes; the cost of discovering the first was wrong is days of refactoring. This applies to module interfaces, data representations, and API boundaries — not every function body. The investment scales with the decision's reversibility (COM-0001: tier assignment heuristic).

Cherry-pit's ADR system institutionalizes this: the Context section requires describing alternatives considered, making multi-design thinking a structural requirement. CHE-0011 (AggregateId) evaluated `u64`, `Uuid`, and `NonZeroU64`. CHE-0031 (MessagePack) compared JSON, bincode, CBOR, and MessagePack. GEN-0007 (FlatBuffers-style layout) evaluated Cap'n Proto, FlatBuffers, and a custom offset-based design.

## Decision

For any design decision at tier B or above, consider at least two
fundamentally different approaches before committing. Document the
alternatives and the rationale for selection.

R1 [5]: Before committing to a module interface or architectural
  boundary, sketch at least two approaches that differ in structure,
  not just in detail
R2 [6]: Evaluate each design against quality attributes — interface
  simplicity, information hiding, consistency, performance, safety,
  evolvability — to reveal trade-offs
R3 [5]: Time-box the exploration proportionally to tier; fifteen
  minutes for tier-B, a design document for tier-S
R4 [5]: The ADR Context section captures alternatives considered;
  the Decision section captures selection rationale

## Consequences

- The ADR template's Context section is the structural enforcement
  mechanism. An ADR that presents only one approach with no
  alternatives considered is incomplete.
- Code review for new traits and infrastructure ports can cite
  COM-0008 to request alternative designs before acceptance.
- The 10–20% strategic investment budget (COM-0001) explicitly
  covers time spent on multi-design exploration. It is not
  additional overhead — it is part of the investment.
- Deliberate deferrals (CHE-0037: no snapshots, CHE-0040: no sagas)
  are a specific application: the second design was "implement now,"
  evaluated and rejected in favor of "defer until needed."
- Risk of analysis paralysis. The time-box rule (rule 3) is the
  mitigation: the practice is exploratory, not exhaustive.
