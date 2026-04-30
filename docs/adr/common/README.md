# Common Domain — Architecture Decision Records

Cross-cutting software design principles informed by Ousterhout's *A Philosophy of Software Design* and complementary works on software architecture, evolutionary design, and organizational alignment. Technology-agnostic guidance on module depth, complexity management, error design, abstraction layering, trade-off analysis, and structural communication. Distinct from Cherry's crate-specific architecture decisions.

Governed by [GOVERNANCE.md](../GOVERNANCE.md).

## Browse

- Index, tier, status, parent edges: `cargo run -p adr-fmt -- --tree COM`
- Per-ADR neighborhood: `cargo run -p adr-fmt -- --critique COM-NNNN`
- Crate-scoped rule extraction: `cargo run -p adr-fmt -- --context <crate>`
- Reference graph: [`../adr-references.svg`](../adr-references.svg)
