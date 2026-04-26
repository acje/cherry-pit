# ADR Governance

Last-updated: 2026-04-26

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

### Foundation Domain

The Common (COM) domain is a **foundation domain**. It contains
cross-cutting software design principles informed by Ousterhout's
"A Philosophy of Software Design" and complementary works on
software architecture, evolutionary design, and organizational
alignment (Martin, Ford, Evans, Skelton, Read, et al.). All
COM principles are technology-agnostic.

When querying ADRs for a specific domain (e.g., Cherry), the foundation
domain's ADRs are always included — they provide the design principles
that all other domains build upon. When querying COM directly, only
COM ADRs are returned.

### MECE Rationale

The four-domain split reflects a clean architectural boundary:

- **Common** — *why* we design the way we do (principles)
- **Cherry** — *what* the framework's architecture looks like (structure)
- **Pardosa** — *how* events are stored and transported (infrastructure)
- **Genome** — *how* data is serialized on the wire (format)

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

## 4. When to Write an ADR

Write an ADR when a decision:

1. **Constrains future implementation** — the choice limits what can be
   built or how it can be built
2. **Has trade-offs** — reasonable alternatives exist and the choice is
   not obvious
3. **Crosses crate boundaries** — the decision affects more than one
   crate's API or behaviour
4. **Is hard to reverse** — undoing the decision would require
   significant refactoring
5. **Was debated** — multiple contributors or design sessions discussed
   alternatives

Do **not** write an ADR for:

- Trivial implementation choices within a single function
- Dependency version bumps (unless they change API surface)
- Formatting or style decisions (covered by lints/rustfmt)

---

## 5. Review Process

1. **Author** creates a Draft ADR in the appropriate domain directory
2. **Author** validates MECE compliance:
   - The ADR belongs to exactly one domain
   - No existing ADR in the same domain covers the same decision space
   - If the decision affects another domain, cross-references are added
   - All relationship targets exist and are Accepted or Amended
   - No circular dependencies are introduced
   - A tier is assigned
3. **Author** opens a PR moving the ADR from Draft to Proposed
4. **Reviewer** verifies:
   - Correct domain assignment
   - Tier assignment with justification
   - Template and vocabulary conformance (`cargo run -p adr-fmt`)
   - MECE compliance
5. On approval, status changes to Accepted and the PR is merged
6. Run `cargo run -p adr-fmt` to regenerate README indexes; commit
   the ADR and any regenerated files together

---

## 6. Overlap Resolution

When pardosa or genome ADRs cover the same concern as a Cherry domain ADR at
a different abstraction level, the resolution is cross-referencing — not
merging:

- The Cherry domain ADR is the **principle** (abstract, crate-agnostic)
- The pardosa/genome ADR is the **implementation** (concrete,
  crate-specific)
- Both remain standalone with `References` links from the concrete
  ADR to the abstract one

Example: CHE-0006 (single-writer assumption) is the Cherry domain principle.
PAR-0004 (single-writer per stream via NATS fencing) references it as the
concrete transport-level implementation.

Merging is reserved for cases where two ADRs in the **same domain**
genuinely cover the same decision space — then the newer ADR supersedes
the older one.
