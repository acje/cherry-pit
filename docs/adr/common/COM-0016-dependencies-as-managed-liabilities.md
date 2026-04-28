# COM-0016. Dependencies as Managed Liabilities

Date: 2026-04-27
Last-reviewed: 2026-04-28
Tier: B
Status: Accepted

## Related

References: COM-0001

## Context

Every external dependency is complexity imported wholesale with a unique risk profile: external maintainers control release cadence and breaking changes; transitive dependencies inherit security and maintenance risks without explicit selection; each dependency expands the supply chain attack surface; compile times and CI costs scale with dependency count; and license obligations may conflict with the project's distribution model.

COM-0001 (complexity budget) establishes that complexity requires justification. Dependencies are a category where costs are invisible at adoption but compound over the project's lifetime. Endler frames this as "every dependency should be seen as a liability." This is distinct from COM-0012 (dependency rule), which governs knowledge flow *direction* — COM-0016 governs whether to take a dependency at all.

Cherry-pit already practices dependency minimization: `default-features = false` on `axum` and `reqwest`, workspace-level dependency declarations for version consistency, and `Cargo.lock` committed for reproducibility. What is missing is a citable principle making dependency justification mandatory.

## Decision

Every external dependency is a liability that must be justified
against the complexity budget. Dependencies are not free — they
trade development convenience for long-term maintenance cost,
security exposure, and coupling to external change schedules.

R1 [5]: Before adding a dependency, demonstrate the functionality
  cannot be reasonably implemented in-house or sourced from the
  standard library
R2 [5]: Disable default features and enable only the feature flags
  the project actually uses
R3 [5]: Prefer the standard library when it provides adequate
  functionality, even if not optimal, due to stronger stability
  guarantees and zero transitive cost
R4 [6]: Evaluate the full cost of a dependency — transitive count,
  maintenance activity, security record, license compatibility,
  and MSRV compatibility
R5 [6]: Audit and prune dependencies periodically as the standard
  library evolves and requirements change

## Consequences

- New dependencies require explicit justification in the PR
  description. Reviewers can cite COM-0016 to challenge
  unjustified additions.
- The `default-features = false` pattern already used for `axum`
  and `reqwest` is validated as the standard posture, not an
  exception.
- Workspace-level dependency declarations (`[workspace.dependencies]`)
  are validated as a complexity management tool — they provide
  a single inventory of external liabilities.
- This principle creates tension with development velocity.
  Implementing functionality in-house costs time now; taking a
  dependency costs time later. The resolution aligns with
  COM-0001: strategic investment (10–20% more time now) for
  reduced long-term maintenance cost.
- Transitive dependency awareness is a new review dimension.
  `cargo tree` and dependency auditing tools become part of the
  evaluation process, not just post-hoc checks.
