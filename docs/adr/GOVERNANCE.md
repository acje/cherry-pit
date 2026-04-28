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

Tiers classify ADRs by systemic leverage, derived from Abson et al.
(2017) and Donella Meadows' twelve leverage points. Higher tiers
represent deeper intervention in the system — changing them reshapes
everything below. Every ADR must be assigned a tier.

Canonical tier metadata (name, description, stability) is output by
`cargo run -p adr-fmt -- --guidelines`.

### Theoretical Foundation

Abson et al. group Meadows' twelve leverage points into four system
characteristics: Intent, Design, Feedbacks, and Parameters. This
workspace splits Abson's "Design" grouping at the self-organization
boundary because Meadows level 4 (the power to create new structures)
is a distinct leverage type from levels 5–6 (rules and information
flows governing existing structures). In software: "EventStore is a
trait" (enables new backends — level 4) is a different kind of
decision from "EventStore::load returns `Result<Vec<EventEnvelope>,
StoreError>`" (constrains the contract — level 5).

### Tier Table

| Tier | System characteristic | Meadows levels | Classification question |
|------|----------------------|----------------|------------------------|
| S | **Intent** — paradigm, goals, governance | 1–3 | Does this decision define the system's paradigm, system-wide architectural pattern, or decision governance? |
| A | **Self-organization** — capacity to evolve structure | 4 | Does this decision introduce or remove trait definitions, generic type parameters, or plugin boundaries that enable new implementations? |
| B | **Design** — rules, information flows | 5–6 | Does this decision prescribe a structural rule or establish an information flow — a type contract, API boundary, visibility constraint, enforcement gate, or observability requirement? |
| C | **Feedbacks** — reinforcing and balancing loops | 7–8 | Does this decision define how components observe, notify, retry, or react to each other at runtime? |
| D | **Parameters** — constants, stocks, flows, delays | 9–12 | Is this only a crate-internal implementation detail or tooling configuration value? |

### Tree Metaphor

The tiers form a tree: S-tier decisions are roots — few in number,
deepest leverage, most abstract. D-tier decisions are leaves — many
in number, most concrete, greatest surface area with real-world code.

Each deeper tier constrains everything shallower: Intent defines what
the system is for, Self-organization determines how it can evolve,
Design sets the rules and feedback structures it must follow,
Feedbacks govern runtime dynamics, Parameters fill in the remaining
degrees of freedom.

Higher tiers appear first in `--context` output — the agent sees
foundational constraints before implementation details. This exploits
primacy bias: LLMs attend more strongly to rules at the start of
context.

### Assignment Protocol

Questions use **system-characteristic framing** — they classify by
what the decision *is*, not by what would change if the decision
changed. This aligns with Meadows' leverage hierarchy: a parameter
change can have large blast radius, but it is still a parameter.

**First-yes-wins:** Start at S and work down. The first question
answered "Yes" determines the tier.

### Tier-Assignment Guidance

- **Self-organization vs. Design (A vs. B):** A-tier is about *what
  can be extended* — trait definitions, generic type parameters, plugin
  boundaries. B-tier is about *how existing things must behave* —
  specific trait signatures, type bounds, public API contracts. Example:
  "EventStore is a trait" → A. "EventStore::load returns
  `Result<Vec<EventEnvelope>, StoreError>`" → B.

- **CI, lints, and tests:** B-tier (Design) when they enforce
  architectural boundaries or encode structural invariants — module-
  boundary lints, property-based test suites, architectural CI gates.
  D-tier (Parameters) when they configure tooling internals — cache
  paths, lint versions, timeout values.

- **Meta-decisions** about the decision process itself (TEMPLATE.md,
  GOVERNANCE.md, ADR workflow) are S-tier: Meadows level 1 (power to
  transcend paradigms).

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
