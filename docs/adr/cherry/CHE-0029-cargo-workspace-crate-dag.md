# CHE-0029. Cargo Workspace with Layered Crate DAG

Date: 2026-04-24
Last-reviewed: 2026-09-19
Tier: B
Status: Accepted

## Related

References: CHE-0001, CHE-0004, COM-0014

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

Cherry-pit owns the neutral framework contract. Repository ownership
and compile-time dependencies are distinct: the framework remains an
acyclic Cargo workspace, while its Pardosa adapter is owned and
released outside that workspace. Existing co-located legacy packages
do not establish dependency or release authority for the adopted seam.

R1 [5]: Organize the canonical acje/cherry-pit framework crates as a
  Cargo workspace with an acyclic crate dependency graph
R2 [5]: Share dependency versions at workspace level via
  [workspace.dependencies]
R3 [5]: Commit Cargo.lock to version control for reproducible
  dependency resolution across all environments
R4 [5]: Restrict cherry-pit-core/Cargo.toml [dependencies] to the
  pure-domain set serde, uuid, jiff so the crate stays a leaf with
  zero transport, runtime, or filesystem dependencies
R5 [5]: Keep async runtimes (tokio), web frameworks (axum), transport
  clients (async-nats), and observability stacks (tracing) in adapter
  crates such as cherry-pit-gateway, cherry-pit-web, and pardosa
R6 [5]: Verify cherry-pit-core's transitive dependency closure in CI
  via cargo tree -p cherry-pit-core, asserting no tokio, axum,
  async-nats, or tracing crate appears in the resolved graph
R7 [5]: Own and release neutral consumer-shaped ports, contract
  conformance, and justified port-only reusable orchestration in
  acje/cherry-pit; retain application domain rules, policy, and
  composition in gh-report
R8 [5]: Own and release the separate outer adapter package in
  acje/pardosa; implement Cherry ports through the Pardosa public
  facade, keeping the substrate independent of Cherry
R9 [5]: Keep the Cherry framework dependency closure free of Pardosa
  dependencies; confine the adapter-to-Cherry and adapter-to-Pardosa
  edges to the outer adapter package

```
A -> B means A depends on B

gh-report composition -> gh-report policy + Cherry ports/orchestration
                         + Pardosa outer adapter
Cherry port-only orchestration -> Cherry core
Pardosa outer adapter -> Cherry core + Pardosa public facade
Pardosa substrate -> no Cherry crate
```

Workspace-level configuration:
- `[workspace.dependencies]` for version consistency
- `[workspace.lints.clippy]` with pedantic warnings
- `[profile.release]` with LTO, strip, overflow-checks
- `Cargo.lock` is committed for reproducible builds

## Consequences

- Each crate has a single responsibility and minimal dependencies.
- Users depend on `cherry-pit-core` for domain work without pulling
  in infrastructure.
- Workspace-level versions prevent drift; independent crates compile
  concurrently.
- Risks/migration: reconcile existing library code before consumer pin
  changes; legacy package cleanup is separate work, not a prerequisite
  for this boundary. Conformance and release-pair evidence precede
  application integration.
- Review dependency metadata and consumer pins against R7-R9 before
  source integration; this amendment does not claim current source
  already satisfies the target graph.
- `Cargo.lock` commits ensure reproducible CI and the eventual binary.
- **De-scalability invariant.** Restricting `cherry-pit-core` to
  `serde`, `uuid`, `jiff` means domain code compiles and tests run
  even if every adapter crate breaks.
- **CI enforcement closes the gap.** A `cargo tree -p cherry-pit-core`
  check makes R4 a build error rather than a convention.
