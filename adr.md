# Plan: Introduce Common ADR Domain (COM-NNNN)

## Overview

Introduce a fourth ADR domain **Common** (`COM-NNNN`, `docs/adr/common/`) for
cross-cutting software design principles grounded in Ousterhout's *A Philosophy
of Software Design*. Six ADRs, governance updates, adr-fmt tool updates, and
bidirectional retrofit of 11 existing CHE ADRs. ~23 file changes, single atomic
commit.

---

## Analysis

### Why a separate domain?

The six principles (deep modules, pull complexity down, layer abstraction, error
elimination, complexity budget, docs-first) are **technology-agnostic** — they
apply equally to `pit-core`, `pardosa`, and `pardosa-genome`. Placing them in
the Framework domain (`CHE`) would violate MECE: they'd scope-creep beyond the
framework into binary format and transport concerns. A dedicated domain
preserves the clean taxonomy.

### MECE boundary

Framework's existing scope includes "Design philosophy." The distinction:

- **COM**: Universal software design principles. Technology-agnostic. Not
  specific to Rust, EDA, or DDD. Sourced from Ousterhout and similar literature.
- **CHE**: How cherry-pit *applies* principles to its specific architecture —
  EDA/DDD/hexagonal, trait design, Rust-specific choices.

Example: COM-0005 says "redefine operations so errors aren't needed." CHE-0019
says "`EventStore::load` returns empty vec, not `NotFound`" — a concrete
Rust-specific application.

### adr-fmt impact analysis

The tool is prefix-agnostic in most code paths. Only **two hardcoded locations**
need updating:

1. `main.rs:117-121` — `discover_domains()` known-domain array
2. `rules/links.rs:14` — `KNOWN_PREFIXES` const array

Everything else (naming regex `[A-Z]{2,4}`, parser, template rules, index rules)
already supports arbitrary prefixes via the `DomainDir` abstraction.

### Atomicity requirement

All ~23 files must be committed together. A partial commit where COM ADRs exist
but CHE backlinks are missing (or vice versa) will cause L002 failures. Plan
enforces single-commit delivery.

---

## Steps

### Phase 1: Governance & Tooling Infrastructure

| # | File | Action | Reason |
|---|------|--------|--------|
| 1 | `docs/adr/GOVERNANCE.md` | Add "Common" row to §1 scope table — Common applies to all crates | Common applies to all crates |
| 2 | `docs/adr/GOVERNANCE.md` | Add "Common" row to §2 domain taxonomy table: `Common` \| `COM` \| `docs/adr/common/` \| scope text below | MECE boundary with explicit Ousterhout attribution |
| 3 | `docs/adr/GOVERNANCE.md` | Add to §3 numbering ranges: `Common: COM-0001 through COM-0006 (6 accepted)`, next = `COM-0007` | Track sequence |
| 4 | `crates/adr-fmt/src/main.rs` | Add `("common", "COM")` to `known` array in `discover_domains()` (line ~117) | Tool discovers COM ADRs |
| 5 | `crates/adr-fmt/src/rules/links.rs` | Add `"COM"` to `KNOWN_PREFIXES` const (line 14) | Cross-domain references to COM are validated |
| 6 | `crates/adr-fmt/src/rules/links.rs` | Add 2 new tests: (a) COM↔CHE bidirectional `Illustrates`/`Illustrated by` passes clean; (b) COM→PAR unmigrated reference produces L004 warning | Test gap — no existing test exercises COM prefix |

**Scope text for §2 taxonomy:**

> Cross-cutting software design principles, informed by Ousterhout's
> "A Philosophy of Software Design." Technology-agnostic guidance on module
> depth, complexity management, error design, and abstraction layering. Distinct
> from Framework's crate-specific architecture decisions.

### Phase 2: Common Domain ADRs

| # | File | Tier | Depends on | Illustrated by |
|---|------|------|------------|----------------|
| 7 | `COM-0001-complexity-budget-strategic-investment.md` | S | — | CHE-0001, CHE-0038 |
| 8 | `COM-0002-deep-modules-over-shallow-abstractions.md` | S | COM-0001 | CHE-0005, CHE-0030 |
| 9 | `COM-0003-pull-complexity-downward.md` | A | COM-0002 | CHE-0016, CHE-0020, CHE-0035, CHE-0043 |
| 10 | `COM-0004-different-layer-different-abstraction.md` | A | COM-0002 | CHE-0019 |
| 11 | `COM-0005-define-errors-out-of-existence.md` | A | COM-0002 | CHE-0009, CHE-0019, CHE-0041 |
| 12 | `COM-0006-interface-documentation-before-implementation.md` | C | COM-0001 | — |
| 13 | `docs/adr/common/README.md` | — | — | Domain README with index table, dependency graph, cross-domain references |

**All COM ADRs**: Status = `Accepted`, Date = today, Last-reviewed = today
(required for S/A-tier).

#### Content outline for each COM ADR

- **COM-0001 (Complexity Budget — Strategic Investment)**: Strategic vs. tactical
  programming. Zero tolerance for incremental complexity. 10-20% design
  investment. Ousterhout Ch. 3. Context explains why the ADR system itself
  exists. Red flags: "I'll clean it up later", interface shortcuts, unreviewed
  complexity additions.

- **COM-0002 (Deep Modules Over Shallow Abstractions)**: Simple interface,
  powerful implementation. Measured by interface-to-implementation ratio. Red
  flags: traits with many methods that each do trivial work, classitis, wrapper
  types that mirror what they wrap. Ousterhout Ch. 4. Context explains the Unix
  file I/O exemplar; Decision codifies the ratio test.

- **COM-0003 (Pull Complexity Downward)**: Infrastructure absorbs complexity;
  callers pass minimal information. Configuration parameters require
  justification — sensible defaults are mandatory. Ousterhout Ch. 8. Context
  explains the "many users, few developers" asymmetry.

- **COM-0004 (Different Layer, Different Abstraction)**: Adjacent layers must
  provide distinct abstractions. Pass-through methods and pass-through variables
  are red flags. If a method's signature mirrors the method it calls, question
  why that layer exists. Ousterhout Ch. 7.

- **COM-0005 (Define Errors Out of Existence)**: Before adding an error variant,
  demonstrate why the operation cannot be redefined to succeed. Prefer idempotent
  semantics, exception masking, and aggregation. Ousterhout Ch. 10. Context
  explains the disproportionate complexity cost of exceptions.

- **COM-0006 (Interface Documentation Before Implementation)**: Interface doc
  comments written before implementation, describing the abstraction ("what" and
  "why"), not the code ("how"). If a comment restates the function name, rewrite
  it. Ousterhout Ch. 13, 15.

### Phase 3: Retrofit Existing CHE ADRs

Each amended CHE ADR gets two changes: (a) add `Illustrates: COM-NNNN` to
`## Related`, (b) update Status to
`Amended {date} — added COM cross-reference`.

| # | File | Add to Related |
|---|------|---------------|
| 14 | `CHE-0001-design-priority-ordering.md` | `Illustrates: COM-0001` |
| 15 | `CHE-0005-single-aggregate-design.md` | `Illustrates: COM-0002` |
| 16 | `CHE-0009-infallible-apply.md` | `Illustrates: COM-0005` |
| 17 | `CHE-0016-store-created-envelopes.md` | `Illustrates: COM-0003` |
| 18 | `CHE-0019-load-returns-empty-not-error.md` | `Illustrates: COM-0004, COM-0005` |
| 19 | `CHE-0020-infrastructure-owned-identity.md` | `Illustrates: COM-0003` |
| 20 | `CHE-0030-flat-public-api.md` | `Illustrates: COM-0002` |
| 21 | `CHE-0035-two-level-concurrency.md` | `Illustrates: COM-0003` |
| 22 | `CHE-0038-testing-strategy.md` | `Illustrates: COM-0001` |
| 23 | `CHE-0041-idempotency-strategy.md` | `Illustrates: COM-0005` |
| 24 | `CHE-0043-process-level-file-fencing.md` | `Illustrates: COM-0003` |

### Phase 4: Index Updates

| # | File | Action |
|---|------|--------|
| 25 | `docs/adr/framework/README.md` | Update Status column to "Amended" for all 11 CHE ADRs; add cross-domain reference section for COM links |
| 26 | `docs/adr/README.md` | Add Common domain row to taxonomy table; update total (92 → 98); add Common to cross-domain overview; add link to `common/README.md` |

### Phase 5: Verification

| # | Action |
|---|--------|
| 27 | `cargo test -p adr-fmt` — all existing + 2 new tests pass |
| 28 | `cargo run -p adr-fmt` — zero errors; only expected L004 warnings for unmigrated PAR/GEN refs |
| 29 | Manual check: every `Illustrated by: CHE-NNNN` in a COM ADR has a matching `Illustrates: COM-NNNN` in the CHE ADR (bidirectional integrity) |

---

## Dependency Graph (Common Domain)

```
COM-0001 Complexity Budget (S)
├── COM-0002 Deep Modules (S)
│     ├── COM-0003 Pull Complexity Downward (A)
│     ├── COM-0004 Different Layer, Different Abstraction (A)
│     └── COM-0005 Define Errors Out of Existence (A)
└── COM-0006 Interface Documentation Before Implementation (C)
```

---

## Decisions (locked)

1. **COM-0001 = Complexity Budget** (meta-principle first), COM-0002 = Deep
   Modules. Reordered from initial proposal so the foundational "why" comes
   before the primary "how."
2. **CHE-0001 gets `Illustrates: COM-0001`** — bidirectional link enforced by
   adr-fmt. The framework's most important ADR formally links to the Common
   domain.
3. **CHE-0019 dual-illustrates COM-0004 + COM-0005** — valid per governance;
   the only dual-link case.

---

## Ousterhout Principle Mapping

| COM ADR | Ousterhout Principle | Book Chapter | Cherry-pit Examples |
|---------|---------------------|--------------|---------------------|
| COM-0001 | Strategic vs. Tactical Programming | Ch. 3 "Working Code Isn't Enough" | 92 ADRs before most code exists; compile-fail tests for type contracts |
| COM-0002 | Deep Modules vs. Shallow Modules | Ch. 4 "Modules Should Be Deep" | `EventStore`: 3 methods hiding file I/O, serialization, concurrency, fencing |
| COM-0003 | Pull Complexity Downwards | Ch. 8 "Pull Complexity Downwards" | Store creates envelopes (CHE-0016), store assigns IDs (CHE-0020) |
| COM-0004 | Different Layer, Different Abstraction | Ch. 7 "Different Layer, Different Abstraction" | Store sees "event streams", Bus sees "aggregate lifecycle" (CHE-0019) |
| COM-0005 | Define Errors Out of Existence | Ch. 10 "Define Errors Out of Existence" | `load()` returns empty vec (CHE-0019), `apply()` is infallible (CHE-0009) |
| COM-0006 | Write The Comments First | Ch. 13, 15 | 685-line `pit-core.md` trait design doc written before implementation |

---

## Risk Mitigations

| Risk | Mitigation |
|------|-----------|
| Partial commit causes L002 failures | All ~23 files committed atomically in single commit |
| `docs/adr/common/` not created before validation | Directory + ADRs created before running `cargo run -p adr-fmt` |
| MECE overlap with Framework "Design philosophy" | Explicit boundary text in GOVERNANCE.md §2 distinguishes technology-agnostic (COM) from crate-specific (CHE) |
| L004 false-negative for COM→PAR/GEN | COM added to `KNOWN_PREFIXES`; unmigrated PAR/GEN refs correctly warn via L004 |
| Missing test coverage for COM prefix | 2 new tests in `links.rs` for COM↔CHE and COM→PAR cross-domain |
