# ADR Tree Model — Implementation Plan

Status: Draft
Last-updated: 2026-04-29

This document captures the agreed design decisions and implementation plan
for restructuring the ADR corpus into explicit parent-edge trees. It serves
as the build contract: each step has a rationale, scope, and acceptance
criteria.

---

## 1. Concept

ADRs form trees within each domain. `Root:` marks foundational tree roots.
Every non-root ADR declares a single structural parent via the **first
`References:` target**. Remaining `References:` targets are secondary
citations — ADRs that directly constrain, justify, or bound this ADR's
decision. The tree structure is expressed through parent edges, not by
every ADR referencing the root.

---

## 2. Agreed Design Decisions

### 2.1 First Reference Is the Parent Edge

The first target in `References:` becomes the structural parent edge for
every non-root ADR. Later targets are secondary direct-constraint
citations. No new verb is introduced; `References` retains its minimal
vocabulary but gains positional semantics.

### 2.2 Same-Domain Parent Default

The first `References:` target must point to an ADR in the same domain.
Cross-domain parent edges are prohibited unless an explicit exception
rationale is documented. Secondary citations may freely cross domains.

### 2.3 Multiple Roots Per Domain Allowed

A domain may have multiple `Root:` ADRs when each root defines a distinct
foundational decision tree. Rationale for multiple roots belongs at the
domain level (in `adr-fmt.toml` or the domain README), not inside each
root ADR.

### 2.4 Parent Graph Must Be Acyclic

Cycles in the first-reference parent graph are forbidden — the parent
edges must form a forest. Secondary citation cycles are permitted; they
represent mutual influence between decisions and are not structural.

### 2.5 Root Citations Are Not Automatic

Non-root ADRs should cite the domain root in secondary references only
when the root directly constrains the ADR beyond ancestry. Ancestry
already implies inheritance; routine root citations add noise and flatten
the tree into a star.

### 2.6 Secondary References Are Direct Constraints Only

Each secondary `References:` target must directly constrain, justify, or
bound this ADR's decision. Broad thematic citations or related reading
do not qualify. This keeps reference load meaningful and lintable.

### 2.7 Advisory Enforcement With Severity

All lint diagnostics remain advisory (exit 0). Diagnostics are
categorized by severity:

- **Structural warnings** — invalid tree structure: missing parent,
  cross-domain parent, parent cycle, unreachable non-root ADR, parent
  not `Accepted`.
- **Heuristic warnings** — suspicious ordering: root first with later
  local candidates, parent lower-tier than child.

### 2.8 Parent Tier Ordering

The parent ADR should be same or higher leverage tier than its child.
A parent lower than its child triggers an advisory warning. Roots may
be any tier — RST-0001 is B-tier and forcing all roots to S would
distort tier semantics.

### 2.9 Parent Must Be Accepted

First-reference parent targets must be active `Accepted` ADRs.
Stale or superseded ADRs may appear as secondary historical citations
but not as structural parents.

### 2.10 Tree Display Shows Parent Edges Only

`adr-fmt --tree` renders parent-edge trees. Secondary references
appear as `also references:` annotations, not as tree edges. Full
citation graphs remain available via `--critique`.

### 2.11 One-Pass Corpus Migration

The existing corpus migrates in one intentional pass after docs and
tooling define the new semantics. Partial migration creates mixed
semantics where `References:` order means different things in different
files.

### 2.12 Parent Choice Uses Invalidation Test

When an ADR has several direct constraints, the author chooses the
same-domain ADR whose removal would most invalidate this ADR's
decision. If two candidates are equally structural parents, the ADR
is probably mixing concerns and should be split.

---

## 3. Implementation Steps

### Step 1: Update Governance Documentation

**Scope:** `docs/adr/GOVERNANCE.md`

**Changes:**
- Section 5 (Reference Ordering and Root Assignment): replace delegation
  to `adr-fmt` generated output with explicit policy:
  - First `References:` target is the structural parent edge.
  - Parent must be same-domain, `Accepted`, same-or-higher tier.
  - Later targets are secondary direct constraints.
  - Invalidation test for parent choice.
  - Root citations only when directly foundational beyond ancestry.
- Section 2 (Tier System / Tree Metaphor): clarify that roots may be
  any tier, not necessarily S-tier. Update the metaphor to reflect
  parent-edge trees rather than tier-as-depth.
- Section 4 (Overlap Resolution): confirm cross-domain references are
  secondary citations, never parent edges.

**Acceptance:** Governance doc is self-contained for the tree model;
no semantic delegation to generated CLI output for relationship rules.

### Step 2: Update Template Documentation

**Scope:** `docs/adr/TEMPLATE.md`

**Changes:**
- Replace "first referenced root wins subtree assignment" with
  parent-chain semantics.
- Add guidance: "List the structural parent first. List secondary
  direct constraints after. Do not cite the root unless it directly
  constrains beyond ancestry."
- Add invalidation test description for parent choice.
- Update migration checklist to include relationship reordering.

**Acceptance:** Template guidance matches governance; new authors can
write correct `References:` without consulting code.

### Step 3: Update CLI Guidelines Output

**Scope:** `crates/adr-fmt/src/guidelines.rs`

**Changes:**
- RELATIONSHIPS section: replace "first-root-referenced" algorithm
  description with parent-edge semantics.
- Add parent choice guidance (invalidation test).
- Add secondary reference scope guidance (direct constraints only).
- Add same-domain parent default.

**Acceptance:** `cargo run -p adr-fmt` output matches governance and
template documentation.

### Step 4: Add Parent-Edge Projection to Navigation

**Scope:** `crates/adr-fmt/src/nav.rs` (or new module)

**Changes:**
- Add a `compute_parent_edges(records) -> HashMap<AdrId, AdrId>`
  function that extracts the first `References:` target per non-root
  ADR as the parent edge.
- Add `compute_parent_children(records) -> HashMap<AdrId, Vec<AdrId>>`
  that inverts parent edges into a children map for tree rendering.
- Existing `compute_children` (which inverts all forward references)
  remains available for `--critique` full-graph traversal.

**Acceptance:** Parent-edge projection produces a forest (no cycles)
for a valid corpus. Unit tests cover: single parent, root self-parent,
secondary citations excluded, cross-domain parent detected.

### Step 5: Update Context Grouping

**Scope:** `crates/adr-fmt/src/context.rs`

**Changes:**
- Replace "first root referenced" assignment (pass 1) with parent-chain
  traversal: walk parent edges upward until a root is reached; assign
  to that root.
- Replace BFS fallback (pass 2) with parent-chain fallback: if parent
  chain does not reach a root (broken chain), assign to unclaimed group.
- Secondary citations no longer affect subtree assignment or BFS depth.

**Acceptance:** `--context` groups ADRs by parent-chain root, not by
first root in references. Tests cover: non-root first parent with root
citation later (assigned to parent's root, not the cited root);
secondary citation to another root does not reassign.

### Step 6: Update Tree Output

**Scope:** `crates/adr-fmt/src/output.rs`

**Changes:**
- `--tree` renders parent-edge trees using `compute_parent_children`.
- Each ADR node shows `also references: X, Y` for secondary citations.
- Orphan ADRs (parent chain broken) appear in a separate section.
- Indentation reflects tree depth.

**Acceptance:** `--tree` output is a forest; no secondary citation
appears as a tree edge. Visual output matches parent-edge semantics.

### Step 7: Add Lint Diagnostics

**Scope:** `crates/adr-fmt/src/rules/links.rs` (and/or new rule module)

**New diagnostics:**

| ID | Category | Description |
|----|----------|-------------|
| L010 | Structural | Non-root ADR has no `References:` (missing parent) |
| L011 | Structural | First `References:` target is in a different domain |
| L012 | Structural | First `References:` target is not `Accepted` |
| L013 | Structural | Parent-edge graph contains a cycle |
| L014 | Structural | Non-root ADR unreachable from any root via parent chain |
| L015 | Heuristic | First reference is a root while later references include same-domain non-root candidates |
| L016 | Heuristic | Parent tier is lower leverage than child tier |
| L017 | Structural | First `References:` target is `Superseded by` another ADR (precedence over L012) |

**Registration:** New rules must be registered in `crates/adr-fmt/src/rules/mod.rs`
alongside existing link rules.

**Acceptance:** Each diagnostic has unit tests. All diagnostics are
advisory (exit 0). Structural diagnostics are clearly labeled. Existing
link diagnostics (L001, L003, L006–L009) remain unchanged.

### Step 8: Add Tests

**Scope:** `crates/adr-fmt/tests/integration.rs` and unit tests in
changed modules.

**Test cases:**

1. First `References:` target selected as parent even when second
   target is a root.
2. Secondary citation to a root does not reassign subtree in `--context`.
3. Parent cycle detected across 2, 3, and N ADRs.
4. Secondary citation cycle does NOT trigger parent-cycle diagnostic.
5. Cross-domain first reference triggers L011.
6. Stale/superseded first-reference target triggers L012.
7. Lower-tier parent triggers L016.
8. Multi-root domain produces correct separate trees.
9. Root with `Supersedes:` still valid (no regression).
10. `--tree` renders parent edges only; secondary refs as annotations.
11. All new diagnostics exit 0 (advisory).
12. Parser preserves `References:` target order (first target stable).
13. L015 does not false-positive when root is the genuine structural
    parent (no same-domain non-root candidates exist).
14. Multi-root domain renders separate trees in `--tree` output.

**Acceptance:** All tests pass. No existing tests broken.

### Step 9: Corpus Migration

**Scope:** All ADR files under `docs/adr/`

> **Implementation note:** during planning a `--tree --proposed`
> preview command was considered as a migration helper. It was
> rejected as scope creep: the manual `--lint` → grep L015 →
> reorder → re-lint loop is short, has no false-positives, and
> any tooling proposal would still require human invalidation-test
> judgment per ADR. See GOVERNANCE.md §5.7 for the canonical
> manual workflow.

**Changes per ADR:**
1. Identify the correct structural parent using the invalidation test:
   which same-domain ADR's removal would most invalidate this decision?
2. Move that ADR to first position in `References:`.
3. Remove routine root citations where ancestry already implies the
   relationship.
4. Remove broad thematic citations that are not direct constraints.
5. Verify no cross-domain first references remain (unless explicitly
   justified).

**Domain-specific notes:**

- **Common (COM):** COM-0001 is the single root. Many ADRs currently
  first-reference COM-0001 despite having more specific local parents
  (e.g., COM-0003, COM-0017, COM-0020). Reorder to express branches.
- **Cherry (CHE):** Two roots: CHE-0001, CHE-0004. Many ADRs
  first-reference CHE-0001 when CHE-0004 or a mid-tier ADR is the
  true structural parent. Reorder to build subtrees under both roots.
- **Pardosa (PAR):** Two roots: PAR-0001, PAR-0004. Already shows
  some branch structure; verify and correct.
- **Genome (GEN):** GEN-0001 is the single root. Almost all ADRs
  first-reference GEN-0001 (star shape). Reorder to build branches
  through GEN-0007, GEN-0004, GEN-0005, GEN-0032, etc.
- **Rust (RST):** RST-0001 is the single root. Small domain; likely
  minimal reordering needed.
- **Security (SEC):** SEC-0001 is the single root. Check for branch
  opportunities through SEC-0002.
- **adr-fmt (AFM):** AFM-0001 is the single root. Check for branches
  through AFM-0003, AFM-0011.

**Acceptance:** `cargo run -p adr-fmt -- --lint` produces no structural
warnings after migration. `--tree` shows a meaningful forest, not stars.

### Step 10: Refresh Generated Artifacts

**Scope:** Generated READMEs, dot/SVG/PNG graphs under `docs/adr/`

**Changes:**
- Regenerate all domain README indexes.
- Regenerate `adr-references.dot` and any derived images.
- Verify generated output matches the new parent-edge tree model.

**Acceptance:** Generated artifacts are consistent with the migrated
corpus. No stale root listings or phantom ADRs in indexes.

---

## 4. Execution Order

```
Step 1 (Governance)  ─┐
Step 2 (Template)     ├─ Policy definition (no code changes)
Step 3 (Guidelines)  ─┘
                       │
Step 4 (Nav)         ─┐
Step 5 (Context)      ├─ Tooling changes (code + tests)
Step 6 (Tree output)  │
Step 7 (Lint rules)   │
Step 8 (Tests)       ─┘
                       │
Step 9 (Migration)   ── Corpus rewrite (requires tooling)
                       │
Step 10 (Artifacts)  ── Generated output refresh
```

Steps 1–3 are documentation-only and can be done in parallel.
Steps 4–8 are code changes with test coverage.
Step 9 depends on steps 4–8 (tooling must validate migration).
Step 10 depends on step 9 (artifacts reflect migrated corpus).

---

## 5. Risks and Mitigations

| Risk | Mitigation |
|------|------------|
| Mixed semantics during partial migration | Gate: define policy and tooling before touching corpus |
| Secondary citations appearing as tree children | Parent-edge projection separates structural from citation edges |
| Existing `--context` output changes | Document as intentional; verify with `--context` tests |
| Parent cycles in migrated corpus | L013 diagnostic catches cycles before commit |
| Stale generated artifacts after migration | Step 10 explicitly regenerates all artifacts |
| Root supersession breaks parent chains | Root + Supersedes already coexists; superseding ADR becomes new root |
| Multi-root domain without rationale | Document rationale in `adr-fmt.toml` or domain README |
| Bridge/boundary ADRs needing cross-domain parent | Allow exceptions with explicit rationale; L011 warns by default |
