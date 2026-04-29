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

Every ADR belongs to exactly one domain. Domain definitions, prefixes,
directories, and crate mappings are configured in `adr-fmt.toml`.
Canonical domain list: `cargo run -p adr-fmt`.

Foundation domains (marked in `adr-fmt.toml`) are included when
querying any non-foundation domain via `--context`. When querying a
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

Tiers classify ADRs by systemic leverage, derived from Donella
Meadows' twelve leverage points. Canonical tier table:
`cargo run -p adr-fmt`.

### Theoretical Foundation

Meadows' twelve leverage points form a spectrum from shallow
(parameter adjustments) to deep (paradigm shifts). This workspace
groups them into five tiers by splitting at the self-organization
boundary: Meadows level 4 (the power to create new structures) is a
distinct leverage type from levels 5–6 (rules and information flows
governing existing structures). In software: "EventStore is a trait"
(enables new backends — level 4) is a different kind of decision from
"EventStore::load returns `Result<Vec<EventEnvelope>, StoreError>`"
(constrains the contract — level 5).

### Tree Metaphor

The tiers form a tree: S-tier decisions are roots — few in number,
deepest leverage, most abstract. D-tier decisions are leaves — many
in number, most concrete, greatest surface area with real-world code.

Higher tiers appear first in `--context` output — the agent sees
foundational constraints before implementation details. This exploits
primacy bias: LLMs attend more strongly to rules at the start of
context.

### Assignment Protocol

Questions use **system-characteristic framing** — they classify by
what the decision *is*, not by what would change if the decision
changed. **First-yes-wins:** Start at S and work down.

### Tier-Assignment Guidance

- **Self-organization vs. Design (A vs. B):** A-tier is about *what
  can be extended* — trait definitions, generic type parameters, plugin
  boundaries. B-tier is about *how existing things must behave* —
  specific trait signatures, type bounds, public API contracts.

- **CI, lints, and tests:** B-tier when they enforce architectural
  boundaries. D-tier when they configure tooling internals.

- **Meta-decisions** about the decision process itself are S-tier:
  Meadows level 1.

---

## 3. Lifecycle

Lifecycle states and terminal requirements: `cargo run -p adr-fmt`.

### Format Migration (2026-04-28)

The template format has two changes:

1. **Status as metadata field.** Status moves from a `## Status`
   section to a preamble field (`Status: Accepted`). Legacy `## Status`
   is still recognized as fallback. If both are present, the metadata
   field takes precedence (T005b warning).

2. **Pipe-separated Related.** The `## Related` section uses
   pipe-separated format: `Verb: targets | Verb: targets`.
   The old bullet format is no longer parsed.

Migrate existing ADRs by tier (S first) when touching them for
other reasons — no urgent batch conversion required.

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

---

## 5. Reference Ordering and Root Assignment

Reference ordering mechanics and root assignment algorithm:
`cargo run -p adr-fmt` — see RELATIONSHIPS section.

