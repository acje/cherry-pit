# COM-0014. Modular Boundaries Aligned to Rate of Change

Date: 2026-04-26
Last-reviewed: 2026-04-28
Tier: B
Status: Accepted

## Related

- References: COM-0002

## Context

Evans (Domain-Driven Design, Ch. 14, "Maintaining Model Integrity")
introduces bounded contexts as the primary tool for managing model
complexity at scale: each context has its own internally consistent
model, and the relationships between contexts are explicit.

Skelton and Pais (Team Topologies, Ch. 6) extend this to
organizational design: team boundaries should align with software
boundaries, and both should reflect the rate of change of the
underlying concerns. Components that change together should be
owned together; components that change independently should be
separable.

Kaiser (Architecture for Flow, Ch. 3) synthesizes these ideas:
Wardley Mapping reveals which components are evolving rapidly
(genesis/custom-built) versus which are stable (commodity/utility).
Module boundaries should separate components with different rates
of change so that evolution in one area does not force coordinated
changes in another.

**The principle:** module boundaries should reflect semantic
boundaries — groups of concepts that change for the same reason,
at the same rate, by the same people. When a boundary cuts across
a rate-of-change boundary, every change in the fast-moving area
forces a coordinated change in the slow-moving area.

Cherry-pit's domain taxonomy is an explicit application of this
principle (see also GOVERNANCE.md §1, MECE Rationale, which
establishes "distinct rate of change" as a domain boundary
criterion — COM-0014 elevates that governance text into a citable,
enforceable design principle):

- **Common (COM)** — principles that change rarely (rate: years).
  Foundational design philosophy.
- **Cherry (CHE)** — framework architecture that changes
  occasionally (rate: months). Trait design, lifecycle, semantics.
- **Pardosa (PAR)** — transport infrastructure that changes when
  deployment topology changes (rate: weeks to months).
- **Genome (GEN)** — wire format that changes when serialization
  requirements change (rate: months, constrained by backward
  compatibility).

Each domain has a distinct rate of change, audience, and
abstraction level (GOVERNANCE.md §1). The crate DAG (CHE-0029)
physically enforces these boundaries through separate compilation
units.

## Decision

Module, crate, and domain boundaries should align with rate-of-change
boundaries. Components that change for the same reason belong
together; components that change independently should be separable.

### Rules

1. **Group by reason for change.** When deciding where a concept
   belongs, ask: "When this concept changes, what else changes with
   it?" If two concepts always change together, they belong in the
   same module. If they change independently, they belong in
   different modules.

2. **Separate by audience.** Different audiences (domain experts vs.
   infrastructure engineers vs. format designers) imply different
   rates of change and different review needs. The domain taxonomy
   reflects this: COM serves all audiences; CHE serves framework
   users; PAR serves infrastructure operators; GEN serves format
   implementors.

3. **Physical boundaries for independent deployment.** When two
   components must be deployable independently, they require
   separate crates. Logical boundaries (modules within a crate)
   are sufficient for components that are always deployed together.

4. **Cross-boundary contracts are stable interfaces.** The interface
   between components with different rates of change must be stable.
   The slow-changing component defines the interface (COM-0012:
   dependency rule); the fast-changing component implements it. This
   prevents rapid changes in one area from destabilizing another.

5. **Domain taxonomy is a rate-of-change map.** The four-domain
   split (COM/CHE/PAR/GEN) encodes the project's rate-of-change
   topology. ADRs that span domains at equal weight signal a
   boundary misalignment (GOVERNANCE.md §1, MECE rationale).

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
