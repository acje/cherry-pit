# RST-0003. Workspace Lint and Format Governance

Date: 2026-04-27
Last-reviewed: 2026-04-27
Tier: C

## Status

Accepted

## Related

- References: RST-0001, COM-0017

## Context

Rust provides two built-in code quality tools: `clippy` (static
analysis with 800+ lints) and `rustfmt` (canonical formatting).
Both are components of the Rust toolchain and evolve with each
stable release.

**Lint governance has three dimensions:**

1. **What to enforce.** Clippy organizes lints into groups:
   `clippy::correctness` (always errors), `clippy::style`,
   `clippy::complexity`, `clippy::perf`, `clippy::pedantic`,
   `clippy::restriction`, `clippy::nursery`, and `clippy::cargo`.
   Each group represents a different cost-benefit trade-off
   between strictness and noise.

2. **Where to configure.** Lints can be set per-crate in
   `lib.rs`/`main.rs`, per-workspace in `Cargo.toml`, or in a
   `clippy.toml` configuration file. Workspace-level
   configuration ensures all crates share the same standard
   (COM-0009: consistency as complexity reducer).

3. **How to enforce.** Local development, CI, or both. CI
   enforcement with `-D warnings` (deny all warnings) makes
   lint compliance a merge gate — no violation reaches the main
   branch.

Cherry-pit already configures `clippy::pedantic` at workspace
level in `Cargo.toml` (CHE-0026). What is missing is the
platform-level rationale for centralized lint governance, a
`clippy.toml` for MSRV-aware lint configuration, a `rustfmt.toml`
for formatting consistency, and explicit CI gate expectations.

COM-0017 (Mechanized Invariant Enforcement) establishes that
machine-enforced rules do not degrade. Linters and formatters
are the primary mechanized enforcement tools for code style and
correctness patterns.

Endler (Corrode, "Long-term Rust Project Maintenance") recommends
pedantic clippy settings for production projects. Borsos
(Swatinem) notes that enforcing lints combined with a pinned
toolchain (RST-0001) prevents surprise failures — lints only
change when the toolchain is deliberately bumped (RST-0002).

## Decision

Lint and formatting rules are configured at workspace level,
enforced in CI as merge gates, and evolved intentionally with
toolchain updates.

### Configuration

**`Cargo.toml` (workspace lints):**

```toml
[workspace.lints.clippy]
pedantic = { level = "warn", priority = -1 }
```

**`clippy.toml` (workspace root):**

```toml
msrv = "1.95"
```

**`rustfmt.toml` (workspace root):**

```toml
edition = "2024"
```

### Rules

1. **Workspace-level lint configuration.** All clippy lint levels
   are set in `[workspace.lints.clippy]` in the root
   `Cargo.toml`. Individual crates inherit via
   `[lints] workspace = true`. No per-crate lint level
   overrides except targeted `#[allow(...)]` with a justification
   comment.

2. **MSRV-aware clippy.** The `clippy.toml` file at the workspace
   root sets `msrv` to match the project's MSRV. This suppresses
   clippy suggestions for APIs unavailable at the supported Rust
   version.

3. **Canonical formatting.** `rustfmt.toml` at the workspace root
   defines the formatting standard. All code is formatted with
   `cargo fmt` before commit. Only stable `rustfmt` options are
   used — no nightly-only formatting configuration.

4. **CI gate.** CI runs `cargo clippy -- -D warnings` and
   `cargo fmt --check`. Both must pass for a PR to merge.
   Violations are not manually waived — they are fixed or
   explicitly allowed with justification.

5. **Per-site overrides require justification.** When a specific
   lint must be suppressed at a call site, use
   `#[allow(clippy::lint_name)]` with a comment explaining why
   the lint does not apply. Blanket `#[allow]` at module or
   crate level is not permitted except for lints that are
   genuinely inapplicable project-wide.

## Consequences

- All crates in the workspace compile under the same lint
  standard. Contributors moving between crates do not encounter
  different lint expectations (COM-0009).
- Clippy's MSRV awareness prevents false suggestions: clippy
  will not recommend APIs that require a newer Rust version than
  the project supports.
- `rustfmt` eliminates formatting debates. The formatter's
  output is canonical — there is no "preferred style," only the
  formatted style.
- CI enforcement means lint and format compliance is guaranteed
  on the main branch. No gradual degradation.
- Tension with developer experience: strict linting produces
  many warnings during rapid prototyping. The mitigation is
  `cargo clippy --fix` for auto-fixable lints and targeted
  `#[allow]` for justified exceptions.
- New clippy lints arrive with toolchain bumps (RST-0002). The
  dedicated-PR workflow ensures these are addressed
  systematically, not scattered across feature branches.
