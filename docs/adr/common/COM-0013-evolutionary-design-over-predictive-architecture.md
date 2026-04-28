# COM-0013. Evolutionary Design Over Predictive Architecture

Date: 2026-04-26
Last-reviewed: 2026-04-26
Tier: A
Status: Accepted

## Related

- References: COM-0001

## Context

Ford, Parsons, and Kua (Building Evolutionary Architectures, 2nd
ed., Ch. 1) define evolutionary architecture as architecture that
"supports guided, incremental change across multiple dimensions."
Erder, Pureur, and Woods (Continuous Architecture in Practice,
Principle 3) state: "Delay design decisions until they are
absolutely necessary." Both contrast with predictive architecture,
which attempts to anticipate all future requirements and build for
them upfront.

**The core insight:** the cost of a wrong prediction compounds
over time. An abstraction built for a future that never arrives
is permanently maintained complexity. An abstraction built when
the need is concrete is informed by real constraints.

**Predictive architecture anti-patterns:**

- **Premature generalization** — making an interface generic for
  hypothetical consumers that may never exist. Each type parameter,
  trait bound, or configuration option is maintained regardless of
  whether the predicted use case materializes.

- **Speculative infrastructure** — building infrastructure for
  scale, performance, or features that are not yet required. The
  infrastructure costs development time now and maintenance time
  forever.

- **Over-abstraction** — introducing indirection layers "in case we
  need to swap implementations later." If the swap never happens,
  the indirection is pure cost (COM-0004: layers must justify
  their existence).

Cherry-pit practices evolutionary design through deliberate
deferrals — decisions explicitly documented as "not now, and here
is why":

- **CHE-0037** (no snapshot support) — deferred because full replay
  is fast enough at current scale. The ADR documents the trigger
  condition for revisiting.
- **CHE-0040** (no saga support) — deferred because the current
  single-aggregate model does not require cross-aggregate
  coordination.
- **GEN-0031** (Rust-only, cross-language deferred) — deferred
  because no non-Rust consumer exists.
- **GEN-0010** (std-only, no_std deferred) — deferred because no
  embedded target is planned.

Each deferral includes the condition under which the decision
should be revisited, making the evolution guided rather than
ad hoc.

## Decision

Design for the requirements that exist now. Defer design for
requirements that may exist later. Document the deferral and the
conditions that would trigger revisiting the decision.

### Rules

1. **Build for concrete needs.** Every abstraction, generic
   parameter, and infrastructure component must be justified by a
   current, concrete requirement. "It might be useful later" is not
   justification (COM-0001, rule 2).

2. **Defer explicitly.** When a future need is anticipated but not
   yet required, write a deliberate-deferral ADR. The ADR
   documents: the anticipated need, why current constraints do not
   justify building it, and the trigger condition for revisiting.

3. **Make evolution cheap.** Design decisions should minimize the
   cost of future change, even when the specific change is unknown.
   Deep modules (COM-0002), information hiding (COM-0007), and the
   dependency rule (COM-0012) all reduce the cost of change by
   isolating decisions behind stable interfaces.

4. **Prefer reversible decisions.** When two designs are comparable,
   choose the one that is easier to reverse or evolve. Reversibility
   reduces the cost of being wrong. The tier system (GOVERNANCE.md
   §2) reflects this: tier-D decisions are freely superseded;
   tier-S decisions are near-immutable.

5. **Review deferrals periodically.** Deliberate-deferral ADRs are
   not "closed" — they are "deferred." When the project's scale,
   audience, or requirements change, review deferred decisions
   against their trigger conditions.

## Consequences

- Deliberate-deferral ADRs (status: Accepted, title includes
  "Deliberate Deferral" or "Planned") are a recognized ADR
  category, not a workaround. They are first-class architecture
  decisions that say "not yet."
- The complexity budget (COM-0001) is preserved: speculative
  infrastructure is not built, so the budget is available for
  concrete needs.
- New feature proposals that require speculative infrastructure
  can be challenged with COM-0013: "What concrete requirement
  justifies this complexity today?"
- The trigger-condition pattern prevents deferrals from becoming
  forgotten decisions. Each deferral is actionable: when condition
  X is met, revisit decision Y.
- Risk of under-investment. Evolutionary design is not an excuse
  for tactical programming (COM-0001). The strategic investment
  is in making the current design evolvable, not in building less.
