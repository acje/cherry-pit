# ADR Governance

Last-updated: 2026-04-27

This document is the root of authority for Architecture Decision Record
management across the cherry-pit workspace. It covers rationale, process,
and judgment-based guidance. All invariant rules are enforced by `adr-fmt`
and documented via `cargo run -p adr-fmt -- --guidelines`.

**Single source of truth architecture:**

- **`adr-fmt` (code)** — invariant rules: template requirements, naming
  conventions, relationship vocabulary, lifecycle states, link integrity
- **`adr-fmt.toml`** — configurable aspects: domain definitions, crate
  mappings, rule parameters, stale directory path
- **`--guidelines` output** — generated complete reference combining
  code invariants and configuration
- **ADRs** — architectural decisions, validated by `adr-fmt`
- **This document** — rationale, process, judgment

Changes to this document require a pull request with explicit review.

---

## 1. Domain Taxonomy

Every ADR belongs to exactly one domain. Domains are mutually exclusive
and collectively exhaustive (MECE) across the architectural decision
space of the cherry-pit workspace.

Domain definitions, prefixes, directories, and crate mappings are
configured in `adr-fmt.toml`.

### Foundation Domains

The workspace has two **foundation domains**:

1. **Common (COM)** contains cross-cutting software design principles
   informed by Ousterhout's "A Philosophy of Software Design" and
   complementary works on software architecture, evolutionary design,
   and organizational alignment (Martin, Ford, Evans, Skelton, Read,
   et al.). All COM principles are technology-agnostic.

2. **Rust (RST)** contains Rust language and toolchain governance
   decisions: toolchain pinning, MSRV policy, lint configuration,
   dependency management, and platform evolution strategy. RST
   decisions are Rust-specific but cross-cutting — they apply to all
   Rust crates in the workspace. The separation from COM preserves
   COM's technology-agnostic property while acknowledging that
   platform-specific decisions deserve their own domain. If the
   workspace ever includes non-Rust codebases, RST applies only to
   Rust crates.

When querying ADRs for a specific domain (e.g., Cherry), both
foundation domains' ADRs are included — COM provides design
principles, RST provides platform governance. When querying a
foundation domain directly, only that domain's ADRs are returned.

### MECE Rationale

The domain split reflects clean architectural boundaries:

- **Common** — *why* we design the way we do (principles)
- **Rust** — *how* we use the Rust platform (toolchain)
- **Cherry** — *what* the framework's architecture looks like (structure)
- **Pardosa** — *how* events are stored and transported (infrastructure)
- **Genome** — *how* data is serialized on the wire (format)
- **AFM** — *how* ADR governance is enforced (tooling)

Each domain has a distinct rate of change, audience, and abstraction
level. A decision that spans two domains at equal weight triggers a
scoping discussion and may result in a boundary-delineating ADR with
cross-references.

---

## 2. Tier System

Tiers classify ADRs by architectural significance and stability
expectations. Every ADR must be assigned a tier.

Tier values, descriptions, and stability expectations are documented
in `cargo run -p adr-fmt -- --guidelines`. The assignment guidelines
below help determine the correct tier:

- **S** — "If this changed, would we need to rewrite the framework?"
- **A** — "If this changed, would trait signatures or type bounds change?"
- **B** — "If this changed, would call sites or runtime behaviour change?"
- **C** — "If this changed, would only CI, lints, or test setup change?"
- **D** — "If this changed, would only one crate's internal
  implementation change?"

Answer "Yes" to assign that tier. Start from S and work down.

---

## 3. Lifecycle

ADR lifecycle states and their meanings are documented in
`cargo run -p adr-fmt -- --guidelines`.

Terminal states (Rejected, Deprecated, Superseded) require moving the
ADR to the stale directory and adding a `## Retirement` section
explaining why the ADR left active service.

---

## 4. Overlap Resolution

When a domain-specific ADR covers the same concern as a foundation
domain ADR (COM or RST) at a different abstraction level, the
resolution is cross-referencing — not merging:

- The foundation ADR is the **principle** (COM) or **platform rule**
  (RST) — abstract and cross-cutting
- The domain ADR is the **implementation** (concrete, crate-specific)
- Both remain standalone with `References` links from the concrete
  ADR to the abstract one

Example: COM-0016 (Dependencies as Managed Liabilities) is the
principle. RST-0004 (Cargo Dependency Governance) references it as the
Rust-specific implementation. CHE-0026 (Correctness-first Build Config)
references RST-0003 as the workspace-level lint governance it inherits.

When pardosa or genome ADRs cover the same concern as a Cherry domain
ADR at a different abstraction level, the same cross-referencing
pattern applies:

Example: CHE-0006 (single-writer assumption) is the Cherry domain principle.
PAR-0004 (single-writer per stream via NATS fencing) references it as the
concrete transport-level implementation.

Merging is reserved for cases where two ADRs in the **same domain**
genuinely cover the same decision space — then the newer ADR supersedes
the older one.
