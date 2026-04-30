# adr-fmt Domain — Architecture Decision Records

ADR governance tooling: CLI design, MADR template enforcement, markdown parsing strategy, diagnostic output, safe-write semantics, naming conventions, relationship vocabulary, tier classification, and domain management. Covers the adr-fmt binary's own architectural decisions as a Rust CLI tool.

Governed by [GOVERNANCE.md](../GOVERNANCE.md).

## Browse

- Index, tier, status, parent edges: `cargo run -p adr-fmt -- --tree AFM`
- Per-ADR neighborhood: `cargo run -p adr-fmt -- --critique AFM-NNNN`
- Crate-scoped rule extraction: `cargo run -p adr-fmt -- --context <crate>`
- Reference graph: [`../adr-references.svg`](../adr-references.svg)
