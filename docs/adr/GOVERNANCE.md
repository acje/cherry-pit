# ADR Governance

Last-updated: 2026-04-25

This document is the single source of truth for Architecture Decision Record
management across the cherry-pit workspace. Changes to this document require
a pull request with explicit review. Breaking changes to numbering, template,
or domain taxonomy require a new ADR documenting the rationale.

---

## 1. Purpose and Scope

ADRs record significant architectural decisions — choices that affect the
structure, behaviour, or constraints of the codebase. They exist so that
future contributors can understand *why* the code is shaped the way it is,
not just *what* it does.

**Scope**: All crates in the cherry-pit Cargo workspace:

| Crate | Domain |
|-------|--------|
| `cherry-pit-core` | Cherry, Common |
| `cherry-pit-gateway` | Cherry, Common |
| `pardosa` | Pardosa, Common |
| `pardosa-genome` | Genome, Common |
| `pardosa-genome-derive` | Genome, Common |
| `cherry-pit-web` (planned) | Cherry, Common |
| `cherry-pit-projection` (planned) | Cherry, Common |
| `cherry-pit-agent` (planned) | Cherry, Common |

---

## 2. Domain Taxonomy (MECE)

Every ADR belongs to exactly one domain. Cross-domain decisions get a
primary home and cross-references to affected domains. A decision that
spans two domains at equal weight triggers a scoping discussion and may
result in a boundary-delineating ADR with cross-references.

| Domain | Prefix | Directory | Scope |
|--------|--------|-----------|-------|
| **Common** | `COM` | `docs/adr/common/` | Cross-cutting software design principles, informed by Ousterhout's "A Philosophy of Software Design." Technology-agnostic guidance on module depth, complexity management, error design, and abstraction layering. Distinct from Cherry's crate-specific architecture decisions. |
| **Cherry** | `CHE` | `docs/adr/cherry/` | Design philosophy, EDA/DDD/hexagonal architecture, domain model traits (aggregates, commands, events, policies), infrastructure ports, concurrency, delivery, storage backends, workspace tooling, testing strategy, build configuration |
| **Pardosa** | `PAR` | `docs/adr/pardosa/` | EDA storage layer: fiber semantics, stream management, NATS/JetStream transport, migration model, backpressure, single-writer fencing at transport level |
| **Genome** | `GEN` | `docs/adr/genome/` | Binary serialization format: wire layout, schema hashing, zero-copy deserialization, compression, security limits (DoS protection, decompression bombs), type validation, forward compatibility |

### MECE Validation

Before accepting any ADR, validate:

1. The ADR belongs to exactly one domain
2. No existing ADR in the same domain covers the same decision space
3. If the decision affects another domain, cross-references are added
   using the relationship vocabulary (§5)
4. All `References` targets exist and are `Accepted` (or `Amended`)
5. No circular dependencies are introduced
6. A tier (§4) is assigned with justification

---

## 3. Numbering

Format: `{PREFIX}-{NNNN}`

Examples: `CHE-0001`, `PAR-0006`, `GEN-0015`

### Rules

- Each domain maintains an independent monotonically increasing sequence
- Numbers are never reused, even after deprecation or supersession
- Current ranges:
  - Common: `COM-0001` through `COM-0006` (6 accepted)
  - Cherry: `CHE-0001` through `CHE-0045` (45 accepted/proposed)
  - Pardosa: `PAR-0001` through `PAR-0014` (14 accepted, pending migration)
  - Genome: `GEN-0001` through `GEN-0033` (33 accepted, pending migration)
- New ADRs append to their domain's sequence: next Common ADR is
  `COM-0007`, next Cherry ADR is
  `CHE-0046`, next Pardosa is `PAR-0015`, next Genome is `GEN-0034`
- File naming: `{PREFIX}-{NNNN}-kebab-case-slug.md`
  - Example: `CHE-0001-design-priority-ordering.md`

### Cross-Domain References

Use the full prefix in text references:

```markdown
- References: CHE-0001
- References: GEN-0006
```

Use relative paths for Markdown links:

```markdown
See [CHE-0006](../cherry/CHE-0006-single-writer-assumption.md)
See [GEN-0015](../genome/GEN-0015-forward-compatibility-contract.md)
```

Within the same domain, relative paths omit the parent traversal:

```markdown
See [CHE-0002](CHE-0002-illegal-states-unrepresentable.md)
```

---

## 4. Tier System

Tiers classify ADRs by architectural significance and stability
expectations. Every ADR must be assigned a tier.

| Tier | Name | Criteria | Stability |
|------|------|----------|-----------|
| **S** | Foundational | Design philosophy or architecture pattern — changing reverberates through every crate and every downstream consumer | Immutable post-1.0 |
| **A** | Core | Core trait design or invariant — changing requires major refactoring across multiple crates | Near-immutable; changes require RFC-level discussion |
| **B** | Behavioural | Behavioural contracts and API semantics — changing requires coordinated updates across call sites | Stable; changes documented via Amended status |
| **C** | Tooling | Tooling, DX, and build decisions — changing is localized to configuration or test infrastructure | Flexible; changes append monotonically |
| **D** | Detail | Implementation detail — changing affects one crate's internals | Mutable; may be superseded freely |

### Assignment Guidelines

- S-tier: "If this changed, would we need to rewrite the framework?"
  → Yes = S-tier
- A-tier: "If this changed, would trait signatures or type bounds change?"
  → Yes = A-tier
- B-tier: "If this changed, would call sites or runtime behaviour change?"
  → Yes = B-tier
- C-tier: "If this changed, would only CI, lints, or test setup change?"
  → Yes = C-tier
- D-tier: "If this changed, would only one crate's internal implementation
  change?" → Yes = D-tier

---

## 5. Relationship Vocabulary

ADRs link **toward the root** of the dependency graph using three
permitted verbs. Reverse links (parent listing children) are not
stored; use `adr-fmt --report` to compute a children index on demand.

| Verb | Meaning | Direction |
|------|---------|-----------|
| **Root** | Self-reference marking this ADR as a tree root (`- Root: CHE-0001` in CHE-0001's file) | Self |
| **References** | This ADR cites the target in context or consequences | Citing → Cited |
| **Supersedes** | Replaces target entirely; target becomes Deprecated/Superseded | Newer → Older |

### Usage Rules

1. Every ADR must have at least one relationship — no orphans, no
   placeholder dashes. Tree roots use `- Root: {OWN-ID}`.
2. `Root` and `References` may not coexist — a root ADR does not
   reference another ADR (it *is* the starting point). `Root` and
   `Supersedes` may coexist (a root can supersede an older root).
3. `Supersedes` requires setting the target's status to
   `Superseded by {PREFIX}-{NNNN}`.
4. Multiple roots per domain are permitted (multiple independent trees).
5. Cross-domain references are encouraged — they make the
   architecture's cross-cutting concerns explicit.

### Legacy Verbs (L006)

The following verbs were part of the original 12-verb vocabulary and
are now deprecated. `adr-fmt` emits L006 warnings for these:

| Legacy Verb | Migration |
|-------------|-----------|
| `Depends on` | → `References` |
| `Extends` | → `References` |
| `Illustrates` | → `References` |
| `Contrasts with` | → `References` |
| `Scoped by` | → `References` |
| `Informs` | Remove (reverse verb) |
| `Extended by` | Remove (reverse verb) |
| `Illustrated by` | Remove (reverse verb) |
| `Referenced by` | Remove (reverse verb) |
| `Superseded by` | Remove (reverse verb) |
| `Scopes` | Remove (reverse verb) |

See `docs/adr/cleanup.md` for migration scripts.

---

## 6. Lifecycle

```
Draft → Proposed → Accepted → Amended (with date log)
            ↓                      ↓
         Rejected           Deprecated → (optional) Superseded by X
```

| State | Meaning |
|-------|---------|
| **Draft** | Under development, not yet proposed for review. May be incomplete. |
| **Proposed** | Ready for review. All required fields present. |
| **Accepted** | Decision is binding. Implementation may be pending. |
| **Amended** | Accepted with recorded modifications. The amendment date and summary are appended to the Status section. Previous text is preserved, not deleted. |
| **Rejected** | Decision was proposed but deliberately not adopted. Remains in record for context. Requires a `## Rejection Rationale` section. |
| **Deprecated** | No longer applicable but preserved for historical context. Reason documented. |
| **Superseded** | Replaced by another ADR. Status reads: `Superseded by {PREFIX}-{NNNN}`. |

### Terminal States

`Rejected`, `Deprecated`, and `Superseded` are terminal states. ADRs in
these states are moved to `docs/adr/stale/` and require a
`## Retirement` section (≥10 words) explaining why the ADR left active
service. Active ADRs must not have a Retirement section.

### Amendment Format

When amending an accepted ADR, append to the Status section:

```markdown
## Status

Accepted

Amended 2026-04-25 — added fencing requirement (previously documented only)
```

Do not delete original text. Add new content inline with clear markers
or append a new section.

### Date Semantics

- `Date:` is the formal authorship date — the date the ADR was first
  written or accepted.
- `Last-reviewed:` is the most recent review or audit date.
- Amendment dates (in the Status section) must be ≥ `Date:`. An
  amendment cannot predate the ADR's creation. `adr-fmt` enforces this
  via rule T012.

### Title and Intent Immutability

Once an ADR reaches `Accepted` status:

1. **Title**: The title line (`# {PREFIX}-{NNNN}. Title`) is immutable.
   Renaming requires a new Superseding ADR.
2. **Decision intent**: The core decision (the "what we chose" in the
   Decision section) cannot be reversed or materially altered via
   amendment. Amendments may add detail, clarify scope, or document
   implementation refinements — but not change the fundamental choice.
3. **Reversing a decision**: Requires a new ADR with `Supersedes:
   {PREFIX}-{NNNN}` in its Related section and the original ADR's
   status set to `Superseded by {PREFIX}-{NNNN}`.

---

## 7. ADR Template

```markdown
# {PREFIX}-{NNNN}. Title

Date: YYYY-MM-DD
Last-reviewed: YYYY-MM-DD
Tier: S | A | B | C | D

## Status

Draft | Proposed | Accepted | Amended | Rejected | Deprecated | Superseded by {PREFIX}-{NNNN}

## Related

- Root: {OWN-PREFIX}-{OWN-NNNN}
- References: {PREFIX}-{NNNN}
- Supersedes: {PREFIX}-{NNNN}

## Context

What is the issue? Why does a decision need to be made?
Include alternatives considered and why they were rejected.

## Decision

What is the change being proposed or decided?
Be specific — name types, traits, crates, configuration.

## Consequences

What becomes easier or harder? Trade-offs and risks.
Include both positive and negative consequences.
```

### Stale ADR Template Addition

ADRs in `stale/` must append a Retirement section:

```markdown
## Retirement

Why this ADR left active service. Must be ≥10 words.
```

### Required Fields

All ADRs must have: Title, Date, Last-reviewed, Tier, Status,
Related (with ≥1 relationship — no empty placeholder), Context,
Decision, Consequences. Stale ADRs additionally require Retirement.

### Section Ordering

Sections must appear in canonical order: Status → Related → Context →
Decision → Consequences (→ Retirement for stale). `adr-fmt` rule T014
warns when sections are misordered.

### Minimum Word Count

Prose sections (Context, Decision, Consequences, Retirement) must each
contain ≥10 words. The threshold is configurable via `adr-fmt.toml`
rule T015 params.

### Optional Fields

- `Alternatives Considered` — may be a subsection of Context or a separate
  section
- `References` — links to design docs, issues, or external resources

### Code Block Guidance

Decision sections should use type signatures, trait bounds, or pseudocode.
Reference source files for full implementations. Code blocks exceeding 20
lines indicate implementation detail leaking into the ADR — `adr-fmt`
emits a T011 warning for these.

Acceptable: trait definitions, struct signatures, error enums.
Avoid: full method implementations, accessor methods, test code.

---

## 8. Cross-Domain Referencing

### Within ADR Text

Use the prefixed identifier inline:

```markdown
Cherry-pit's single-writer assumption (CHE-0006) is enforced at the
transport level by pardosa's NATS sequence fencing (PAR-0004).
```

### In Related Sections

Use the full prefix:

```markdown
## Related

- References: CHE-0004
- References: CHE-0006
```

### Markdown Links

Use relative paths from the ADR file:

```markdown
<!-- From docs/adr/pardosa/PAR-0004-... to a Cherry domain ADR -->
See [CHE-0006](../cherry/CHE-0006-single-writer-assumption.md)

<!-- From docs/adr/cherry/CHE-0022-... to a genome ADR -->
See [GEN-0002](../genome/GEN-0002-no-schema-evolution-fixed-layout.md)

<!-- Within the same domain -->
See [CHE-0002](CHE-0002-illegal-states-unrepresentable.md)
```

### Reference Docs

Design documents in `docs/` are referenced from ADRs using relative paths:

```markdown
See [pardosa design](../plans/pardosa-design.md)
See [genome design](../plans/genome.md)
```

---

## 9. When to Write an ADR

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

## 10. Review Process

1. **Author** creates a Draft ADR in the appropriate domain directory
2. **Author** validates the MECE checklist (§2)
3. **Author** opens a PR moving the ADR from Draft to Proposed
4. **Reviewer** verifies:
   - Correct domain assignment
   - Tier assignment with justification
   - Relationship vocabulary uses only permitted verbs (§5)
   - Every ADR has ≥1 relationship (Root, References, or Supersedes)
   - Template conformance (section order, word counts)
   - MECE compliance
5. On approval, status changes to Accepted and the PR is merged
6. Domain README index is updated in the same PR

---

## 11. Overlap Resolution

When pardosa or genome ADRs cover the same concern as a Cherry domain ADR at
a different abstraction level, the resolution is cross-referencing — not
merging:

- The Cherry domain ADR is the **principle** (abstract, crate-agnostic)
- The pardosa/genome ADR is the **implementation** (concrete,
  crate-specific)
- Both remain standalone with `References` links from the concrete
  ADR to the abstract one

Example: CHE-0006 (single-writer assumption) is the Cherry domain principle.
PAR-0004 (single-writer per stream via NATS fencing) illustrates it
at the transport level.

Merging is reserved for cases where two ADRs in the **same domain**
genuinely cover the same decision space — then the newer ADR supersedes
the older one.

---

## 12. Index Structure

Each domain directory contains a `README.md` with:

1. **Domain description** — one-paragraph scope statement
2. **Index table** — columns: `#`, `Title`, `Tier`, `Status`, `References`
3. **Dependency graph** — textual tree and/or Graphviz DOT
4. **Cross-domain references** — links to related ADRs in other domains

The top-level `docs/adr/README.md` is the hub linking to all domain
READMEs and GOVERNANCE.md.
