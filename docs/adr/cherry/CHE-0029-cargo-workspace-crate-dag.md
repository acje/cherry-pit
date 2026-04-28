# CHE-0029. Cargo Workspace with Layered Crate DAG

Date: 2026-04-24
Last-reviewed: 2026-04-28
Tier: B
Status: Accepted

## Related

References: CHE-0001, CHE-0004

## Context

Cherry-pit provides multiple concerns: core traits, infrastructure
adapters, serialization/transport, web serving, projections, and
agent composition. These must be organized into crates with clear
dependency boundaries.

Options:
1. **Single crate with feature flags** — simpler but couples
   everything. Users pull in dependencies they don't need.
2. **Cargo workspace (monorepo)** — each crate has a single
   responsibility. Users depend only on what they need. Clean
   dependency DAG with build parallelism.
3. **Separate repositories** — maximum isolation but coordination
   overhead for cross-crate changes.

## Decision

Cherry-pit is a Cargo workspace monorepo with 8 planned crates (2
currently active). Dependencies are shared at workspace level via
`[workspace.dependencies]`. The DAG is acyclic:

R1 [5]: Organize cherry-pit as a Cargo workspace monorepo with an
  acyclic crate dependency graph
R2 [5]: Share dependency versions at workspace level via
  [workspace.dependencies]
R3 [5]: Commit Cargo.lock to version control for reproducible
  dependency resolution across all environments

```
cherry-pit-core (leaf — no cherry-pit dependencies)
├── cherry-pit-gateway
├── pardosa-genome → pardosa-genome-derive → pardosa
├── cherry-pit-projection
└── cherry-pit-web
    └── cherry-pit-agent (root — depends on everything)
```

Workspace-level configuration:
- `[workspace.dependencies]` for version consistency
- `[workspace.lints.clippy]` with pedantic warnings
- `[profile.release]` with LTO, strip, overflow-checks
- `Cargo.lock` is committed for reproducible builds

## Consequences

- Each crate has a single responsibility and minimal dependencies.
- Users can depend on just `cherry-pit-core` for domain work without pulling
  in infrastructure dependencies.
- Workspace-level dependency versions prevent version drift across
  crates.
- Independent crates compile concurrently, improving build times.
- 6 planned crates are designed on paper but unvalidated — trait
  coherence and feature flag interactions won't surface until built.
- `cherry-pit-agent` (binary crate, planned) will be the integration point
  where cross-crate mismatches surface.
- `Cargo.lock` is committed to ensure reproducible CI/test runs and
  prepare for the eventual binary crate.
