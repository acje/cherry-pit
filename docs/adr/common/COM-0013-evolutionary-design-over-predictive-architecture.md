# COM-0013. Evolutionary Design Over Predictive Architecture

Date: 2026-04-26
Last-reviewed: 2026-04-28
Tier: B
Status: Accepted

## Related

References: COM-0001, COM-0002

## Context

Ford, Parsons, and Kua (Building Evolutionary Architectures, 2nd ed., Ch. 1) define evolutionary architecture as supporting "guided, incremental change across multiple dimensions." Erder, Pureur, and Woods (Continuous Architecture in Practice, Principle 3) add: "Delay design decisions until they are absolutely necessary." The cost of a wrong prediction compounds — an abstraction built for a future that never arrives is permanently maintained complexity.

Predictive anti-patterns include premature generalization (generic interfaces for hypothetical consumers), speculative infrastructure (building for unneeded scale), and over-abstraction (indirection layers "in case" — COM-0004: layers must justify their existence).

Cherry-pit practices evolutionary design through deliberate deferrals with documented trigger conditions: CHE-0037 (no snapshots — replay is fast enough), CHE-0040 (no sagas — single-aggregate model suffices), GEN-0031 (Rust-only — no non-Rust consumer exists), and GEN-0010 (std-only — no embedded target planned). Each deferral makes the evolution guided rather than ad hoc.

## Decision

Design for the requirements that exist now. Defer design for
requirements that may exist later. Document the deferral and the
conditions that would trigger revisiting the decision.

R1 [5]: Every abstraction, generic parameter, and infrastructure
  component must be justified by a current concrete requirement
R2 [5]: When a future need is anticipated but not yet required,
  write a deliberate-deferral ADR documenting the need, why current
  constraints do not justify building it, and the trigger condition
R3 [6]: Design decisions should minimize the cost of future change
  through deep modules, information hiding, and the dependency rule
R4 [5]: When two designs are comparable, choose the one easier to
  reverse or evolve; reversibility reduces the cost of being wrong
R5 [6]: Deliberate-deferral ADRs are reviewed periodically against
  their trigger conditions when scale or requirements change

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
