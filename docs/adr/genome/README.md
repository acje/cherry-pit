# Genome Domain — Architecture Decision Records

Binary serialization format: wire layout, schema hashing, zero-copy deserialization, compression, security limits (DoS protection, decompression bombs), type validation, and forward compatibility.

Governed by [GOVERNANCE.md](../GOVERNANCE.md).

## Browse

- Index, tier, status, parent edges: `cargo run -p adr-fmt -- --tree GEN`
- Per-ADR neighborhood: `cargo run -p adr-fmt -- --critique GEN-NNNN`
- Crate-scoped rule extraction: `cargo run -p adr-fmt -- --context <crate>`
- Reference graph: [`../adr-references.svg`](../adr-references.svg)
