# COM-0014. Modular Boundaries Aligned to Rate of Change

Date: 2026-04-26
Last-reviewed: 2026-04-28
Tier: B
Status: Accepted

## Related

References: COM-0001, COM-0002, COM-0012

## Context

Evans (Domain-Driven Design, Ch. 14) introduces bounded contexts for managing model complexity. Skelton and Pais (Team Topologies, Ch. 6) extend this: team and software boundaries should reflect rate of change. Kaiser (Architecture for Flow, Ch. 3) synthesizes these ideas via Wardley Mapping — module boundaries should separate components with different rates of change so evolution in one area does not force coordinated changes in another. When a boundary cuts across a rate-of-change boundary, every fast-moving change forces coordinated changes in the slow-moving area.

Cherry-pit's domain taxonomy applies this principle (see GOVERNANCE.md §1, MECE Rationale — COM-0014 elevates that governance text into a citable design principle). Common (COM) changes rarely (years). Cherry (CHE) changes occasionally (months). Pardosa (PAR) changes with deployment topology (weeks to months). Genome (GEN) changes with serialization requirements (months, constrained by backward compatibility). Each domain has a distinct rate of change, audience, and abstraction level. The crate DAG (CHE-0029) physically enforces these boundaries.

## Decision

Module, crate, and domain boundaries should align with rate-of-change
boundaries. Components that change for the same reason belong
together; components that change independently should be separable.

R1 [5]: Group concepts by reason for change; if two concepts always
  change together they belong in the same module
R2 [5]: Separate by audience — different audiences imply different
  rates of change and different review needs
R3 [6]: Components that must be deployable independently require
  separate crates; co-deployed components use modules within a crate
R4 [5]: The interface between components with different rates of
  change must be stable; the slow-changing component defines the
  interface
R5 [6]: ADRs spanning two domains at equal weight signal a boundary
  misalignment requiring adjustment or splitting

## Consequences

- The four-domain taxonomy is validated as a rate-of-change
  alignment, not an arbitrary grouping. Each domain's audience,
  abstraction level, and change frequency are distinct.
- The crate DAG (CHE-0029) is the physical enforcement. Crate
  boundaries align with domain boundaries, which align with
  rate-of-change boundaries.
- New modules proposed for cherry-pit are evaluated for boundary
  alignment: "Does this change at the same rate as the crate it
  lives in? If not, should it be a separate crate?"
- Cross-domain ADRs (an ADR that affects two domains equally) are
  flagged as potential boundary misalignments. The resolution is
  either adjusting the boundary or splitting into two ADRs with
  cross-references (GOVERNANCE.md §6).
- This principle creates tension with consolidation (COM-0002: deep
  modules). The resolution: consolidate within a rate-of-change
  boundary; separate across boundaries. A deep module that spans
  two rates of change is a coupling risk.
