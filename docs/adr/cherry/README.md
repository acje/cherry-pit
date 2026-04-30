# Cherry Domain — Architecture Decision Records

Design philosophy, EDA/DDD/hexagonal architecture, domain model traits (aggregates, commands, events, policies), infrastructure ports, concurrency, delivery, storage backends, workspace tooling, testing strategy, and build configuration.

Governed by [GOVERNANCE.md](../GOVERNANCE.md).

## Browse

- Index, tier, status, parent edges: `cargo run -p adr-fmt -- --tree CHE`
- Per-ADR neighborhood: `cargo run -p adr-fmt -- --critique CHE-NNNN`
- Crate-scoped rule extraction: `cargo run -p adr-fmt -- --context <crate>`
- Reference graph: [`../adr-references.svg`](../adr-references.svg)
