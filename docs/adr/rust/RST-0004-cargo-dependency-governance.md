# RST-0004. Cargo Dependency Governance

Date: 2026-04-27
Last-reviewed: 2026-04-27
Tier: C

## Status

Accepted

## Related

- References: RST-0001, COM-0016

## Context

COM-0016 (Dependencies as Managed Liabilities) establishes the
technology-agnostic principle that every external dependency is a
complexity cost requiring justification. RST-0004 implements this
principle for the Cargo ecosystem with specific tooling and
operational practices.

**Cargo's dependency model creates specific risks:**

1. **Feature flag proliferation.** Many crates enable features by
   default that the consumer does not need. Tokio's default
   features include `io-util`, `fs`, `process`, `signal`, and
   others. Each unused feature adds code to compile, test, and
   audit — with no benefit.

2. **Transitive depth.** Cargo resolves transitive dependencies
   automatically. A single `cargo add` can introduce dozens of
   crates the developer never explicitly chose. `cargo tree`
   reveals this depth; without inspection, it grows silently.

3. **Known vulnerabilities.** The RustSec advisory database tracks
   security vulnerabilities in published crates. Without
   automated checking, known-vulnerable dependencies persist
   undetected.

4. **License compliance.** Crates on crates.io use varied
   licenses. Mixing copyleft (GPL, AGPL) with permissive (MIT,
   Apache-2.0) licenses can create distribution obligations that
   are incompatible with the project's own license.

5. **Duplicate crates.** Multiple versions of the same crate in
   the dependency tree increase binary size, compile time, and
   the risk of version-specific bugs.

Cherry-pit already practices several mitigations:
`[workspace.dependencies]` centralizes version declarations,
`default-features = false` on `axum` and `reqwest` minimizes
feature surface, and `Cargo.lock` is committed for
reproducibility. What is missing is automated enforcement via
`cargo-audit` (vulnerability scanning) and `cargo-deny` (license,
duplicate, and advisory policy enforcement).

## Decision

Cargo dependencies are managed through workspace-level
declarations, minimal feature flags, and automated CI checks for
vulnerabilities, license compliance, and duplicate crates.

### Configuration

**`deny.toml` (workspace root):**

```toml
[advisories]
ignore = []
[licenses]
allow = [
    "MIT", "Apache-2.0", "BSD-2-Clause", "BSD-3-Clause",
    "BSL-1.0", "ISC", "Unicode-3.0", "Zlib",
]
confidence-threshold = 0.8

[licenses.private]
ignore = true

[bans]
multiple-versions = "warn"
wildcards = "warn"

[sources]
unknown-registry = "deny"
unknown-git = "deny"
allow-registry = ["https://github.com/rust-lang/crates.io-index"]
```

### Rules

1. **Workspace-level declarations.** All dependencies are declared
   in `[workspace.dependencies]` in the root `Cargo.toml`.
   Individual crates reference them via
   `dep = { workspace = true }`. This provides a single inventory
   of all external liabilities.

2. **Default features disabled.** New dependencies are added with
   `default-features = false` and only the required features
   explicitly enabled. This is the default posture, not an
   optimization applied after the fact.

3. **Vulnerability scanning in CI.** `cargo audit` runs in CI and
   fails the build on known vulnerabilities. The RustSec advisory
   database is the authoritative source.

4. **Policy enforcement in CI.** `cargo deny check` runs in CI and
   enforces license allowlist, bans on duplicate crate versions,
   wildcard version requirements, and unknown registries.

5. **New dependency approval.** Adding a new entry to
   `[workspace.dependencies]` requires justification in the PR
   description: what the dependency provides, why it cannot be
   implemented in-house or sourced from `std`, and what its
   transitive depth is (`cargo tree -p <crate>` output).

6. **Periodic pruning.** Dependencies are periodically reviewed
   for continued necessity. Tools like `cargo-udeps` or
   `cargo machete` identify unused dependencies. Dependencies
   whose functionality has been absorbed by `std` or superseded
   by a lighter alternative are candidates for removal.

## Consequences

- `cargo audit` catches known vulnerabilities before they reach
  production. The CI gate ensures no merge introduces a crate
  with an active advisory.
- `cargo deny` enforces license compliance automatically. Adding
  a GPL-licensed transitive dependency is caught at CI time, not
  during a legal review months later.
- The `default-features = false` posture reduces compile times
  and binary size. It also reduces the attack surface by
  excluding code paths the project does not use.
- The new-dependency approval process adds review friction. This
  friction is intentional — it aligns with COM-0016's principle
  that dependencies are liabilities. The friction is proportional
  to the cost: trivial internal implementations avoid the
  friction entirely.
- `deny.toml` configuration will evolve as the project's license
  posture and dependency landscape change. The initial allowlist
  covers common permissive licenses; additional licenses are
  added when justified by a specific dependency.
- Tension with rapid prototyping: strict dependency governance
  slows experimentation. The mitigation is that workspace-level
  declarations make it easy to add a dependency when justified
  and equally easy to audit and remove it when not.
