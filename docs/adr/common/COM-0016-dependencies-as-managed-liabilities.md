# COM-0016. Dependencies as Managed Liabilities

Date: 2026-04-27
Last-reviewed: 2026-04-27
Tier: A

## Status

Accepted

## Related

- References: COM-0001

## Context

Every external dependency is a complexity cost imported wholesale
into the project. Unlike internal complexity — which the team
controls, reviews, and can refactor — external complexity has a
unique risk profile:

1. **External rate of change.** The dependency's maintainers set
   the release cadence, API evolution strategy, and deprecation
   timeline. The consuming project has no control over when
   breaking changes arrive or how long security patches take.

2. **Transitive depth.** A single direct dependency may pull in
   dozens of transitive dependencies. Each transitive dependency
   inherits the same risks (security, maintenance, compatibility)
   but receives less scrutiny because it was never explicitly
   chosen.

3. **Security attack surface.** Each dependency is a supply chain
   entry point. Malicious code injection, abandoned crates with
   known vulnerabilities, and typosquatting all scale with
   dependency count. The attack surface is multiplicative, not
   additive.

4. **Build and CI cost.** Dependencies increase compile times,
   CI duration, and the surface area of version resolution
   conflicts. These costs are paid on every build, by every
   contributor.

5. **License entanglement.** Each dependency introduces license
   obligations that may conflict with the project's own license
   or distribution model.

COM-0001 (complexity budget) establishes that complexity requires
justification. Dependencies are a specific category where the
justification must account for costs that are invisible at
adoption time but compound over the project's lifetime. Endler
(Corrode, "Long-term Rust Project Maintenance") frames this as
"every dependency should be seen as a liability." The Prossimo
project's sudo-rs case study demonstrated that deliberate
dependency reduction improved auditability and security posture.

This principle is distinct from COM-0012 (dependency rule), which
governs the *direction* of knowledge flow between architectural
layers. COM-0016 governs the *existence* of dependencies on
external code — whether to take a dependency at all.

Cherry-pit already practices dependency minimization:
`default-features = false` on `axum` and `reqwest`, workspace-level
dependency declarations for version consistency, and `Cargo.lock`
committed for reproducibility. What is missing is a citable
principle that makes dependency justification mandatory.

## Decision

Every external dependency is a liability that must be justified
against the complexity budget. Dependencies are not free — they
trade development convenience for long-term maintenance cost,
security exposure, and coupling to external change schedules.

### Rules

1. **Justify before adopting.** Before adding a new dependency,
   demonstrate that the functionality cannot be reasonably
   implemented in-house or sourced from the standard library.
   "It saves a few lines" is not sufficient justification for a
   dependency that brings transitive depth or maintenance risk.

2. **Minimize what you import.** Disable default features. Enable
   only the feature flags the project actually uses. Every unused
   feature is dead code that still participates in compilation,
   version resolution, and security scanning.

3. **Prefer the standard library.** When the standard library
   provides functionality that is adequate (even if not optimal),
   prefer it over an external crate. Standard library APIs have
   stronger stability guarantees and zero transitive cost.

4. **Evaluate the full cost.** When assessing a dependency, consider:
   transitive dependency count, maintenance activity, security
   track record, license compatibility, and the dependency's own
   MSRV and edition compatibility. A crate with 50 transitive
   dependencies is a different cost than one with zero.

5. **Audit and prune periodically.** Dependencies that were
   justified at adoption time may become unjustified as the
   standard library evolves, requirements change, or better
   alternatives emerge. Periodic review prevents dependency
   accumulation.

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
