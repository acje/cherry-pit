# Rust Domain — Architecture Decision Records

Rust language and toolchain governance: toolchain pinning, MSRV policy, edition strategy, workspace lint and format configuration, dependency management, supply chain security, and conservative feature adoption. Platform-specific decisions that apply to all Rust crates in the workspace.

Governed by [GOVERNANCE.md](../GOVERNANCE.md).

## Browse

- Index, tier, status, parent edges: `cargo run -p adr-fmt -- --tree RST`
- Per-ADR neighborhood: `cargo run -p adr-fmt -- --critique RST-NNNN`
- Crate-scoped rule extraction: `cargo run -p adr-fmt -- --context <crate>`
- Reference graph: [`../adr-references.svg`](../adr-references.svg)
