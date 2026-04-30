# Pardosa Domain — Architecture Decision Records

EDA storage layer: fiber semantics, stream management, NATS/JetStream transport, migration model, backpressure, and single-writer fencing at transport level.

Governed by [GOVERNANCE.md](../GOVERNANCE.md).

## Browse

- Index, tier, status, parent edges: `cargo run -p adr-fmt -- --tree PAR`
- Per-ADR neighborhood: `cargo run -p adr-fmt -- --critique PAR-NNNN`
- Crate-scoped rule extraction: `cargo run -p adr-fmt -- --context <crate>`
- Reference graph: [`../adr-references.svg`](../adr-references.svg)
