# RST-0002. Intentional Toolchain and Dependency Evolution

Date: 2026-04-27
Last-reviewed: 2026-04-27
Tier: C
Status: Accepted

## Related

- References: RST-0001, COM-0013

## Context

RST-0001 pins the Rust toolchain to a specific stable version.
COM-0013 (Evolutionary Design Over Predictive Architecture)
establishes that evolution should be guided and intentional, not
ad hoc. Together, these principles create a question: how and
when should the pinned toolchain and locked dependencies be
updated?

**Toolchain updates introduce change in three dimensions:**

1. **New lints.** Each Rust release ships new clippy lints. A
   toolchain bump may surface dozens of new warnings across the
   workspace. If these land in the same PR as feature work, the
   diff becomes unreadable and blame history is polluted.

2. **New language features.** Stabilized APIs and syntax become
   available. Adopting them immediately risks using features that
   contributors on the previous pinned version cannot compile.
   With RST-0001, this is solved — but the update PR itself must
   handle the transition cleanly.

3. **Dependency compatibility.** New dependency versions may
   require a newer MSRV. `cargo update` may pull in versions
   that require a toolchain bump, creating a coupling between
   dependency updates and toolchain updates.

**Dependency updates introduce parallel risks:**

- A patch version may include behavioral changes or new
  transitive dependencies.
- Mixing dependency updates with feature work makes regression
  bisection harder.
- Stale dependencies accumulate known vulnerabilities.

Endler (Corrode, "Long-term Rust Project Maintenance")
recommends regular `rustup update` and `cargo update` but as
deliberate maintenance activities, not mixed with feature work.
Borsos (Swatinem) argues that "updating the toolchain and fixing
those lints intentionally in a dedicated PR" prevents friction.

## Decision

Toolchain updates and dependency updates are deliberate,
atomic, reviewed changes — never ambient drift or side effects
of feature work.

### Rules

1. **Toolchain bumps are dedicated PRs.** Update
   `rust-toolchain.toml`, `rust-version` in `Cargo.toml`, fix
   all new lint warnings, and verify CI passes — all in one
   branch. No feature work in the same PR.

2. **Dependency updates are dedicated PRs.** Run `cargo update`,
   review the diff of `Cargo.lock`, run the full test suite,
   and submit as a standalone PR. If a dependency update
   requires code changes, those changes belong in the same PR
   for atomicity.

3. **One-release buffer for new features.** Recently stabilized
   Rust APIs are not adopted in the same PR that bumps the
   toolchain. Allow one release cycle for stabilization
   maturity before introducing calls to newly available APIs.
   This prevents churn if a stabilized API receives follow-up
   fixes.

4. **Edition migrations are dedicated PRs.** Run
   `cargo fix --edition`, review all automated changes for
   semantic correctness, and submit as a standalone PR. Edition
   migrations change language semantics and deserve focused
   review.

5. **Periodic cadence.** Toolchain and dependency updates are
   performed on a regular cadence rather than reactively. The
   cadence is not fixed by this ADR — it depends on project
   activity — but the practice of regular, scheduled updates
   prevents staleness from compounding into a large, risky
   migration.

## Consequences

- Blame history remains clean: lint fixes, dependency updates,
  and feature work are in separate commits and PRs. `git bisect`
  works across all three dimensions.
- Contributors never encounter "CI broke overnight" due to an
  unintentional toolchain drift. Updates are announced,
  reviewed, and merged deliberately.
- The one-release buffer for new APIs means the project trails
  the cutting edge by approximately six weeks. This is an
  intentional trade-off: stability over novelty (COM-0013 rule 4:
  prefer reversible decisions).
- Dependency update PRs may reveal version conflicts or breaking
  changes that would otherwise be discovered in the middle of
  feature work, where they cause more disruption.
- Risk of staleness: if the cadence slips, updates accumulate
  and become individually larger and riskier. The mitigation is
  treating the cadence as a maintenance commitment, not a
  best-effort aspiration.
