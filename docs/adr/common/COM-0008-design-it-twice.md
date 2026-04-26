# COM-0008. Design It Twice

Date: 2026-04-26
Last-reviewed: 2026-04-26
Tier: B

## Status

Accepted

## Related

- References: COM-0001

## Context

Ousterhout (Ch. 11, "Design It Twice") observes that the first
design that comes to mind is unlikely to be the best. Considering
multiple alternatives before committing — even briefly — exposes
trade-offs that a single-pass design misses.

**The practice:** for any significant design decision, sketch at
least two fundamentally different approaches. Compare them against
the relevant quality attributes (simplicity, performance, safety,
evolvability). Pick the one that best fits the context — or
synthesize a hybrid that combines strengths.

**Why developers skip it:** the first design feels "obvious" and
exploring alternatives feels like wasted effort. But the cost of
exploring a second design is minutes; the cost of discovering the
first design was wrong is days or weeks of refactoring.

**Scope:** "Design It Twice" applies to module interfaces, data
representations, algorithm choices, and API boundaries — not to
every function body. The investment scales with the decision's
reversibility (COM-0001: tier assignment heuristic).

Cherry-pit's ADR system institutionalizes this practice: the
Context section requires describing the problem space and
alternatives considered, making multi-design thinking a structural
requirement rather than a personal discipline. Specific examples:

- **CHE-0011** (AggregateId as NonZeroU64) explicitly evaluated
  `u64`, `Uuid`, and `NonZeroU64` before selecting the newtype.
- **CHE-0031** (MessagePack) compared JSON, bincode, CBOR, and
  MessagePack with named encoding.
- **GEN-0007** (FlatBuffers-style layout) evaluated Cap'n Proto,
  FlatBuffers, and a custom layout before converging on an
  offset-based binary design.

## Decision

For any design decision at tier B or above, consider at least two
fundamentally different approaches before committing. Document the
alternatives and the rationale for selection.

### Rules

1. **Two designs minimum.** Before committing to a module interface,
   data representation, or architectural boundary, sketch at least
   two approaches that differ in structure, not just in detail.
   Varying a parameter is not a second design; changing the
   abstraction is.

2. **Compare on quality attributes.** Evaluate each design against
   the attributes that matter for the context: interface simplicity
   (COM-0002), information hiding (COM-0007), consistency
   (COM-0009), performance, safety, evolvability. No design wins on
   all axes — the comparison reveals trade-offs.

3. **Time-box the exploration.** The goal is exposure to
   alternatives, not exhaustive analysis. For a tier-B decision, 15
   minutes of sketching two approaches is sufficient. For tier-S,
   the exploration may warrant a design document.

4. **ADRs encode the outcome.** The Context section captures the
   alternatives considered. The Decision section captures the
   selection rationale. This is the durable record that "Design It
   Twice" happened.

5. **Tier scoping.** Tier D decisions (single-crate internals) may
   skip formal multi-design analysis. Tier C and above should apply
   this principle. Tier S decisions should always have a written
   comparison.

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
