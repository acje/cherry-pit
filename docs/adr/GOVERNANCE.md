# ADR Governance

Last-updated: 2026-04-29 (Section 5 added: Parent-Edge Tree Model)

This document is the root of authority for Architecture Decision Record
management across the cherry-pit workspace. It covers rationale, process,
and judgment-based guidance. All invariant rules are enforced by `adr-fmt`
and documented via `cargo run -p adr-fmt` (the default mode).

**Single source of truth architecture:**

- **`adr-fmt` (code)** — invariant rules: template requirements, naming
  conventions, relationship vocabulary, lifecycle states, link integrity
- **`adr-fmt.toml`** — configurable aspects: domain definitions, crate
  mappings, rule parameters, stale directory path
- **Default-mode output** — generated complete reference combining
  code invariants and configuration (printed by `cargo run -p adr-fmt`
  with no flags)
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

### Tier as Leverage Stratification

Tiers are a **stratification** of decisions by systemic leverage,
distinct from the ADR **parent-edge tree** described in Section 5.
Tiers answer "how deep does this decision cut?"; the parent tree
answers "which decision does this one specialize?".

S-tier decisions cut deepest — few in number, most abstract, often
Root ADRs in their domain. D-tier decisions cut shallowest — many
in number, most concrete. A typical parent edge crosses no more
than one tier boundary at a time (B-tier child under an A-tier
parent is normal; D-tier child directly under an S-tier parent is
suspicious and triggers **L016**).

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

The leverage-tier mapping is operationalised by two enforcement rules:
**T019** (rule-tier tension — rule's Meadows layer should imply a tier
within ±1 of the ADR tier) and **T020** (reference load — `References:`
count is capped per tier). When either fires, the tier classification
or rule layering is likely off.

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

By the parent-edge model (Section 5), the concrete ADR's structural
parent is its **first** `References:` target. When the concrete ADR
is part of a same-domain subtree, list the same-domain parent first
and the foundation ADR after — that keeps the ADR rooted in its own
domain. When the concrete ADR has no same-domain parent and the
foundation ADR is the natural parent, list the foundation ADR first
and add `Parent-cross-domain: PREFIX-NNNN — <reason>` to suppress
L011.

Example: COM-0016 (Dependencies as Managed Liabilities) is the
principle. RST-0004 (Cargo Dependency Governance) references it as the
Rust-specific implementation. CHE-0026 (Correctness-first Build Config)
references RST-0003 as the workspace-level lint governance it inherits.

When pardosa or genome ADRs cover the same concern as a Cherry domain
or Common-domain ADR at a different abstraction level, the same
cross-referencing pattern applies:

Example: COM-0025 (Distributed Failure Model) is the foundation
principle. PAR-0016 (Timestamp Policy and Cross-Stream Ordering)
implements time semantics consistent with that failure model. PAR-0016's
`References:` lists the same-domain root `PAR-0004` first, then
`COM-0025` later — keeping PAR-0016 rooted in its own domain while
preserving the foundation citation.

Merging is reserved for cases where two ADRs in the **same domain**
genuinely cover the same decision space — then the newer ADR supersedes
the older one.

---

## 5. Parent-Edge Tree Model

Every ADR (other than a Root) has exactly one **structural parent**:
the **first** target listed in its `References:` field. Other forward
links — additional `References:`, `Refines:`, `Supersedes:` — are
**secondary citations** that contribute to argument and history but
do not place the ADR in the tree.

This rule is mechanical, deterministic, and enforced by `adr-fmt`. It
exists so that every ADR has an unambiguous answer to the question
"under which decision does this one live?" — without that answer, the
`--context` resolution cannot scope rules to a crate, the `--tree`
view collapses into a flat list, and reviewers cannot tell at a glance
which decisions a new ADR builds on versus merely cites.

### 5.1 Roots

A **Root ADR** declares itself the apex of a domain subtree by listing
its own ID in `Root:`:

```
Root: COM-0001
```

A Root has no structural parent. Roots are typically S-tier and
articulate a domain's organizing principle (e.g., COM-0001 *Complexity
Budget*, CHE-0001 *Design Priority Ordering*). A domain should have
**exactly one Root**. Multi-root domains are permitted only when the
domain genuinely splits into independent concerns; the rationale should
be recorded in `adr-fmt.toml` under the domain's `multi_root_rationale`
field. (Today the field is parsed but no warning fires yet — the
enforcement check is a planned follow-up. Cherry and Pardosa currently
have two roots each; rationales should be filled in when the warning
is wired.)

### 5.2 Structural Parent vs. Secondary Citations

The structural parent is selected by the **first References target,
in document order**. Specifically:

- `Supersedes:`, `Refines:`, and reverse verbs (`Cited-by`,
  `Refined-by`, etc.) **do not** create parent edges.
- `Root:` self-reference does not create a parent edge.
- Only `References:` targets create parent edges, and only the first
  one in the field's value list.

Reorder a `References:` line and you have re-parented the ADR. This
is intentional: the tooling treats the first reference as the
load-bearing claim and ranks it accordingly. Specialized parents
should appear first, foundational citations after.

**Worked example.** An ADR refining single-writer concurrency would
reference its specialized parent first, then the principle:

```
References: CHE-0006, COM-0018      ← CHE-0006 is the structural parent
Supersedes: CHE-0027                ← does not affect parent edge
```

If you flip the order:

```
References: COM-0018, CHE-0006      ← COM-0018 becomes parent (cross-domain)
```

`adr-fmt` will emit **L011** (cross-domain parent) and the ADR will
appear under COM in the tree view, not CHE.

### 5.3 Cross-Domain Parents

A structural parent in a different domain is permitted, but it must
be explicitly justified via the preamble:

```
Parent-cross-domain: COM-0018 — concurrency model is a workspace-wide
                                principle, no Cherry-domain analog exists yet
```

Without this field, `adr-fmt` emits **L011**. The reason text is free
prose and is preserved through linting. If the field is present but
points to a different ADR than the actual first References target,
L011 still fires — the suppression must match exactly.

### 5.4 Non-Accepted Parents

If an ADR's structural parent is `Draft` or `Proposed`, `adr-fmt`
emits **L012** as a warning, but the parent chain still flows
through that ADR. This is the *advisory waypoint* policy: an ADR
under a Draft parent is itself usable for `--context` resolution,
but readers are warned that the parent is not yet stable and the
relationship may need to be re-rooted when the parent is rejected
or superseded.

If the parent is `Superseded by`, `adr-fmt` emits **L017** (which
takes precedence over L012). A superseded parent is a structural
defect that should be repaired by re-pointing References to the
successor.

### 5.5 The Tree View

`cargo run -p adr-fmt -- --tree` renders each domain's parent-edge
forest using box-drawing. Each line shows
`ID Title [Tier] STATUS [also: Verb Target, …]` where the `also`
list contains every forward link other than the structural parent.
This makes secondary citations visible without conflating them with
the tree shape.

ADRs that are unreachable from any Root via parent edges land in a
per-domain orphan section, annotated with the reason:

- `(no References — parent missing)` — non-Root ADR without any
  `References:` field. Triggers **L010**.
- `(chain ends at non-root)` — chain terminates at an ADR that is
  not itself a Root. Triggers **L014**.
- `(cycle)` — chain forms a cycle. Triggers **L013**.

### 5.6 Diagnostics Reference

| ID   | Severity | Trigger |
|------|----------|---------|
| L010 | warning  | Non-Root ADR has no `References:` (no parent) |
| L011 | warning  | First `References:` target is in a different domain (suppress with `Parent-cross-domain`) |
| L012 | warning  | First `References:` target is `Draft` or `Proposed` (advisory; chain still flows) |
| L013 | warning  | Parent-edge graph contains a cycle |
| L014 | warning  | Non-Root ADR's parent chain does not terminate at any Root |
| L015 | warning  | First reference is a Root while later References include same-domain non-Root candidates — consider promoting one |
| L016 | warning  | Structural parent's tier is *lower* leverage than child's (e.g., a B-tier ADR parented under a D-tier) |
| L017 | warning  | First `References:` target is `Superseded by` another ADR (precedence over L012) |

L015 and L016 are **heuristics**: they encode preferences about
parent-shape, not strict invariants. Suppressing one is a judgment
call, typically made via reordering the references rather than
adding a config exception.

### 5.7 Migration

The parent-edge model treats the existing corpus's "References: ROOT
first, specialized siblings after" convention as suboptimal and
enforces the inverted ordering via L015 (see §5.6 and TEMPLATE.md
§Reference Ordering). Migration is per-domain and manual.

The migration loop for each domain:

1. Run `cargo run -p adr-fmt -- --lint` and grep for `L015` warnings
   in that domain.
2. For each L015 finding, identify the most-specialized same-domain
   Accepted ADR among the References. Apply the invalidation test:
   "if I removed this ADR, would the child decision still hold?" The
   strongest "no" is the structural parent.
3. Reorder the `References:` lines so the structural parent is in
   first position; the previous first reference (typically a Root)
   becomes a later citation, or is removed entirely if the relationship
   is already implied by ancestry.
4. Re-run `--lint`. Confirm L015 falls to zero for the domain. If
   reordering produces a new orphan (chain ends at non-root, see
   §5.5), treat that orphan ADR as the next L015 target. Commit
   the domain's migration as one PR.

This sequencing prevents partial migrations from polluting the tree
mid-flight. Foundation domains (COM, RST, SEC) are migrated last
because they are referenced from many other domains; their reordering
can shift the structural parent of every dependent ADR.

`--tree` (without `--lint`) renders the current parent-edge view at
any time, which is useful for sanity-checking the result of a
migration step. ADRs whose parent chain does not terminate at a root
appear in a per-domain orphan section, categorized as "no References",
"chain ends at non-root", or "cycle".

### 5.8 Known L015 Exceptions

Some ADRs legitimately list a Root in first position with same-domain
non-Root co-citations. L015 fires but is accepted because both
references are body-prose direct constraints (§5.6) and the Root is
genuinely the structural parent.

| ADR | Root parent | Co-citation(s) | Rationale |
|-----|-------------|----------------|-----------|
| AFM-0014 | AFM-0001 | AFM-0003 | Stderr seam (R4) is constrained by AFM-0003's exit-code semantics; AFM-0001 is the SSOT root |
| COM-0026 | COM-0001 | COM-0013 | Subtractive design *implements* the COM-0001 complexity budget; COM-0013 is contrasted, not parented |
| COM-0032 | COM-0001 | COM-0013, COM-0011 | Requirement interrogation gates COM-0001 budget spend; COM-0013/COM-0011 cite contrasting practices |

New L015 hits are not exceptions by default — they require body-prose
justification and listing here. The lint warning persists; the table
records why suppression-by-action (reordering) is not appropriate for
these cases.


