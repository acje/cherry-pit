# Documentation Consistency Audit

Generated: 2026-04-27
Updated: 2026-04-27 (SSOT/MECE review + verification review)

Internal inconsistencies between README, plan docs, code comments, and
actual crate implementations. Organized by severity and actionability.

---

## SSOT Hierarchy

Authority chain for resolving drift between documentation sources.
Established during verification review (2026-04-27). When two sources
disagree, the higher-ranked source is presumed correct unless evidence
shows it is stale.

1. **ADRs** — Accepted architectural decisions. Highest-quality written
   source. Changing an ADR requires the lifecycle process defined in
   GOVERNANCE.md (amend or supersede). Plan docs and code that contradict
   an accepted ADR are the ones that drifted — unless the ADR itself is
   identified as stale, in which case it must be amended/superseded
   *before* downstream docs are corrected.
2. **Code** (Cargo.toml, lib.rs, actual source) — Runtime truth. If code
   diverges from a plan doc but aligns with ADRs, the plan doc is stale.
   If code diverges from an ADR, investigate whether the ADR was
   intentionally bypassed (missing amendment) or the code drifted.
3. **Plan docs** (build-plan.md, pardosa-next.md, genome.md, etc.) —
   Aspirational/historical. Lowest authority when they conflict with ADRs
   or code. Useful as intent records but must not be treated as current
   truth without cross-referencing.

**Key implication:** Correcting downstream docs (genome.md, plan docs)
that follow an accepted ADR requires amending the ADR first. Editorial
deletion of ADR-aligned content without a governance-tracked ADR change
violates this hierarchy.

**Scope:** This hierarchy governs conflicts between documentation sources
for implementation truth. ADR governance processes (lifecycle, template,
naming) remain governed by GOVERNANCE.md §SSOT architecture.

---

## SSOT/MECE Review Findings

Cross-document audit focusing on Single Source of Truth violations and
Mutually Exclusive, Collectively Exhaustive gaps. Findings supplement
sections A–D below.

### SSOT Violations

**S1. Exact-duplicate files (CRITICAL)**

`crates/pardosa/pardosa.md` is byte-identical to `docs/plans/pardosa.md`.
`crates/pardosa/pardosa-design.md` is byte-identical to
`docs/plans/pardosa-design.md`. Verified via `diff` — zero output. Any
future edit to one copy silently diverges the other.

Timestamp analysis: crate-level copies are older (Apr 23/25), docs/plans/
copies are newer (Apr 26). This is consistent with crate-level files
being the originals that were copied outward. Neither file is a symlink.

Status: **🔍 noted for future grilling** — crate-level docs vs plan docs
ownership model needs a design conversation before acting.

**S2. pardosa-next.md G1–G9 stale genome spec changes (HIGH → DELETE)**

These sections were proposed amendments to an earlier genome.md draft.
genome.md has since evolved past them in incompatible ways:

| Detail | pardosa-next.md | genome.md (SSOT) |
|---|---|---|
| Bare message header | 6 bytes (no schema_hash, no algo) | 15 bytes (version + schema_hash + algo + size) |
| Index entry | 20 bytes, CRC32 | 24 bytes, xxHash64 (per GEN-0016) |
| Header flags | bit 0=compressed, bits 1-3=algo (000=brotli), bits 4-8=quality_hint | bits 0-2=compression_algo (000=none, 001=zstd), bits 3-15=reserved |
| File header @20 | 12 bytes reserved | page_class:u8 + schema_size:u32 + reserved:7B |

GEN-0016 (xxHash64), GEN-0030 (zstd-only v1) confirm genome.md is
authoritative. The G1–G9 proposals were absorbed and evolved; the stale
text actively misleads.

**Resolution: delete G1–G9 sections entirely.** Replace with a single line:
"Genome spec changes originally proposed here have been incorporated into
genome.md. See genome.md §Binary Format Specification for the authoritative
spec."

**S3. genome.md NATS references — misnaming + misplacement (HIGH)**

GEN-0008 ADR correctly separates concerns: genome = `bytes ↔ types`,
transport = separate crate. The architectural decision is sound. However:

1. **Misnaming:** GEN-0008 names the companion crate `pardosa-genome-nats`.
   This implies genome owns transport integration. Transport is pardosa's
   domain — the correct name is `pardosa-nats`. GEN-0008 must be amended
   before downstream references are corrected (per SSOT Hierarchy above).
2. **Misplacement:** genome.md contains NATS transport documentation that
   belongs in pardosa docs. Genome is a binary serialization format; NATS
   integration APIs (NatsPublisher, NatsConsumer, stream discovery) are
   pardosa's responsibility.
3. **Scope:** genome.md references NATS/`pardosa-genome-nats` at 12+ lines,
   not just 1100–1104 as originally noted: lines 1098, 1100-1104, 1663,
   1667, 2362, 2570, 2842, 2864, 2875, 2878 (includes a Smithy spec file
   reference at `spec/smithy/pardosa-genome-nats.smithy`).

No `pardosa-genome-nats` crate exists in the workspace.

**Resolution (ordered — governance before editorial):**
1. Amend GEN-0008 ADR: rename `pardosa-genome-nats` → `pardosa-nats`,
   reassign transport ownership to pardosa domain. Follow GOVERNANCE.md
   lifecycle process.
2. Remove all `pardosa-genome-nats` references from genome.md (12+ lines).
3. Move NATS integration API documentation to pardosa docs or a future
   `pardosa-nats` crate plan.
4. Update README components table if applicable.

**S4. Superseded content lacks deprecation markers (MEDIUM)**

pardosa-next.md line 7 states "Supersedes pardosa.md Phase 2–5." Neither
pardosa.md nor pardosa-design.md carry any deprecation marker. A reader
encountering them first follows stale plans.

**Resolution:** Add supersedence banner to pardosa.md and
pardosa-design.md (after S1 ownership question is resolved).

**S5. pardosa-next.md async_trait usage (LOW → INVESTIGATE LATER)**

`PersistenceAdapter` trait uses `#[async_trait]` and the Cargo.toml
section lists `async-trait = "0.1"`. This contradicts CHE-0025 (RPITIT
over async_trait) and cherry-pit-core's approach. May be intentional for
object-safety reasons in this specific trait, or may be an oversight.

Status: **🔎 investigate later** — determine whether pardosa's
PersistenceAdapter requires object safety (dyn dispatch) or can use RPITIT.

**S6. GEN-0008 ADR uses stale companion crate name (HIGH — BLOCKS S3)**

GEN-0008 (Transport-Agnostic Core with Companion Crate Separation) is an
accepted ADR that names the companion crate `pardosa-genome-nats`. Per the
SSOT Hierarchy, this is currently the authoritative name. However, the
naming conflates genome (serialization) with NATS (transport). Transport
is pardosa's domain, not genome's.

The ADR's architectural decision (separate transport from serialization)
is correct. The crate name is wrong. This is ADR drift — the decision
was made before the domain ownership boundaries were fully established.

**Resolution:** Amend or supersede GEN-0008 (per Q7 resolution) via
GOVERNANCE.md lifecycle process:
- Rename `pardosa-genome-nats` → `pardosa-nats` throughout the ADR
- Reassign transport ownership explicitly to pardosa domain
- This amendment unblocks S3 (genome.md NATS cleanup) and M2 below

### MECE Gaps

**M1. Pardosa design spread across 7 locations (HIGH)**

No single document is authoritative for any pardosa topic. The complete
current plan requires reading pardosa.md + pardosa-design.md +
pardosa-next.md + mentally applying the amendments. Additionally, the
crate-level duplicates add confusion about which copy to edit.

Locations:
1. `docs/plans/pardosa.md` — 25-line overview
2. `docs/plans/pardosa-design.md` — 429-line design (phases 2-5 superseded)
3. `docs/plans/pardosa-next.md` — 1150-line amendments
4. `docs/plans/automerge-ideas.md` — future patterns
5. `crates/pardosa/pardosa.md` — exact duplicate of (1)
6. `crates/pardosa/pardosa-design.md` — exact duplicate of (2)
7. `crates/pardosa/README.md` — overlapping state machine description

**Resolution:** Deferred until S1 crate-level docs ownership is resolved.
The consolidation strategy depends on whether crate-level docs are kept,
replaced with pointers, or eliminated.

**M2. Glossary not collectively exhaustive (MEDIUM)**

`docs/plans/glossary.md` defines 8 cherry-pit-core terms. Missing:

- Pardosa: Fiber, Dragline, Line, Index, DomainId, Generation, ECST,
  FiberState, MigrationPolicy, LockedRescuePolicy
- Genome: GenomeSafe, GenomeOrd, bare message, genome file, schema hash,
  page class
- Cross-cutting: single-writer assumption, publish-then-apply

Header says "Terms used throughout cherry-pit documentation" but coverage
is cherry-pit-core only.

**Resolution:** Extend glossary with pardosa and genome terms.

**M3. README Documentation table incomplete (LOW)**

Lists 5 plan docs but omits `pardosa-design.md`, `pardosa-next.md`,
`genome.md`, and `automerge-ideas.md`. Pardosa link text says "Event
serialization, append-only log format, schema evolution" — stale
description.

**Resolution:** Update table after pardosa doc consolidation.

---

## A. Crate Identity Confusion: pardosa-genome

See also: S3 above (genome.md NATS misnaming/misplacement).

The most pervasive inconsistency. Three documents describe pardosa-genome
differently:

| Source | Description |
|--------|-------------|
| **Cargo.toml + lib.rs (authoritative)** | "Binary serialization format with zero-copy reads and serde integration" |
| build-plan.md Phase 2 | "Append-only file format, log versioning, migration engine" |
| pardosa.md | "the append-only file format, log versioning, and migration engine" |

The build-plan.md and pardosa.md descriptions match the original vision
before genome evolved into a serialization format. The append-only log
functionality belongs to pardosa itself, not pardosa-genome.

## B. build-plan.md Drift from Implementation

build-plan.md was written as the initial plan but was not updated as
crates were built. Nearly every detail about Phase 2 (pardosa-genome) and
Phase 3 (pardosa) is now wrong:

- **Module layouts** — planned files (`log.rs`, `entry.rs`, `version.rs`)
  bear no relation to actual files (`config.rs`, `format.rs`,
  `genome_safe.rs`)
- **Dependencies** — planned deps (sha2, bytes, jiff) replaced by actual
  deps (serde, xxhash-rust, pardosa-genome-derive)
- **Rust version** — says "we target 1.85+" in one place, "1.95" in
  another
- **rmp-serde version** — Phase 4 table says "0.15" (doesn't exist),
  workspace Cargo.toml says "1"
- **Missing modules** — `aggregate_id.rs`, `correlation.rs` exist but
  aren't listed
- **Missing types** — `CorrelationContext`, `EnvelopeError`, `CreateResult`
  in code but not in "Contents (implemented)" list

## C. pardosa-next.md Staleness

See also: S2 above (G1–G9 marked for deletion).

pardosa-next.md proposes genome spec changes (G1–G9) that were partially
adopted but the document was never updated to reflect outcomes. **Decision:
delete G1–G9 entirely** — genome.md is the SSOT for all genome spec
details.

| Proposal | Status | Issue |
|----------|--------|-------|
| G1: 2-byte version prefix for bare messages | Adopted (genome.md has it plus schema_hash + algo byte) | pardosa-next.md shows simpler format |
| G2: CRC32 in 20-byte index entries | **Not adopted** — genome.md uses xxHash64 in 24-byte entries (per GEN-0016) | Still presented as proposed |
| G3: dict_id in header | Adopted | pardosa-next.md still shows 12-byte reserved vs genome.md's actual layout |
| G4: 5-bit quality_hint in flags | **Not adopted** — genome.md has simpler 3-bit compression_algo | Still presented as proposed |
| G5-G6: NatsPublisher/NatsConsumer | Decided by GEN-0008 (needs amendment) — delegates to `pardosa-genome-nats` companion crate that doesn't exist | Still presented as genome types |
| G8: CRC32 trailer API | **Not adopted** — genome.md uses xxHash64 | Still presented as proposed |
| G9.7: max_total_elements "default 16M" | **Wrong** — genome.md shows default is PageClass::Page0 = 256 | Active misinformation |

Additionally, pardosa-next.md lists `async-nats = "0.40"` (workspace:
"0.47") and uses `async_trait` (contradicts build-plan.md's explicit "No
async_trait" decision and cherry-pit-core's RPITIT approach).

## D. Miscellaneous

- **infrastructure.md** references `pit-*` namespace; actual crates use
  `cherry-pit-*`
- **GOVERNANCE.md** MECE rationale lists 5 domains, omits AFM (adr-fmt) —
  6th domain exists with 10 ADRs
- **genome.md** references `pardosa-genome-nats` companion crate —
  misnaming + misplacement (see S3, S6). GEN-0008 ADR must be amended
  before genome.md references are removed. Transport integration belongs
  in pardosa or a future `pardosa-nats` crate
- **genome.md** crate structure section mixes existing Phase 1 files with
  future Phase 2-3 files without clear demarcation
- **genome.md** dependencies section shows removed `crc` and planned
  `bolero` not in actual Cargo.toml
- **README.md line 70** pardosa link text says "Event serialization,
  append-only log format, schema evolution" — reflects old pardosa-genome
  description
- **pardosa** is at version `0.2.0` while all other crates are `0.1.0` —
  no rationale documented

---

## Fixes

### HIGH priority — factual contradictions

**H1. build-plan.md Phase 2 pardosa-genome description**

Update from "Append-only file format, log versioning, migration engine" to
"Binary serialization format with zero-copy reads and serde integration".
Update module layout and dependencies to match actual crate state.

Planned module layout says:

```
pardosa-genome/
└── src/
    ├── lib.rs
    ├── log.rs
    ├── entry.rs
    ├── version.rs
    ├── migration.rs
    ├── integrity.rs
    └── error.rs
```

Actual:

```
pardosa-genome/
└── src/
    ├── lib.rs
    ├── config.rs
    ├── error.rs
    ├── format.rs
    └── genome_safe.rs
```

Planned dependencies: `serde`, `serde_json`, `sha2`, `bytes`, `thiserror`,
`jiff`. Actual: `serde`, `xxhash-rust` (const_xxh64), `pardosa-genome-derive`
(optional).

**H2. build-plan.md Phase 3 pardosa dependencies**

Listed: `cherry-pit-core`, `pardosa-genome`, `serde`, `serde_json`, `tokio`,
`async-nats`, `bytes`, `tracing`, `thiserror`. Actual: `serde`,
`serde_json`, `thiserror` only. No cherry-pit-core, no pardosa-genome, no
tokio, no async-nats.

**H3. build-plan.md Rust version contradiction**

Section 4 (line 50-51): "Requires Rust 1.75+ (we target 1.85+)". Header
(line 4): "rust-version: 1.95". Workspace Cargo.toml: `rust-version =
"1.95"`. rust-toolchain.toml: `channel = "1.95"`. README: "rust 1.95+".

Fix: change "we target 1.85+" to "we target 1.95+".

**H4. build-plan.md rmp-serde version**

Phase 4 dependencies table (line 224): `rmp-serde | 0.15`. Workspace
Cargo.toml: `rmp-serde = "1"`. Line 109 of the same file correctly shows
version "1". Internal contradiction within build-plan.md itself.

Fix: change "0.15" to "1".

**H5. build-plan.md cherry-pit-core module layout and contents**

Module layout (lines 131-145) omits `aggregate_id.rs` and `correlation.rs`,
both present in actual `crates/cherry-pit-core/src/`.

"Contents (implemented)" list (lines 111-128) omits `CorrelationContext`,
`EnvelopeError`, and `CreateResult`, all present in actual code and
re-exported from `lib.rs`.

**H6. pardosa.md pardosa-genome description**

Line 24-25: "pardosa-genome — the append-only file format, log versioning,
and migration engine". Should match actual: "Binary serialization format
with zero-copy reads and serde integration".

Note: pardosa.md has a byte-identical copy at `crates/pardosa/pardosa.md`
(see S1) — both need updating or the duplicate needs removing first.

**H7. pardosa-next.md G1–G9 — DELETE**

~~Add banner at top marking Genome Spec Changes (G1-G9) as partially
superseded by genome.md.~~ **Decision: delete G1–G9 sections entirely.**
genome.md is the SSOT. The stale proposals actively mislead — they
describe formats (CRC32, brotli, 20-byte index entries) that were never
adopted. Replace with a one-line pointer to genome.md.

**H8. pardosa-next.md version and trait contradictions**

- `async-nats` version: "0.40" → should be "0.47"
- Uses `async_trait` in `PersistenceAdapter` trait and Cargo.toml —
  contradicts build-plan.md "No async_trait" decision and cherry-pit-core's
  RPITIT approach. **Status: investigate later** — may be intentional for
  object safety in this specific trait.

**H9. infrastructure.md namespace**

Line 4: "narrow, purpose-built crates under the `pit-*` namespace". Actual
crate names use `cherry-pit-*` namespace.

**H10. README.md pardosa link text**

Line 70: pardosa link described as "Event serialization, append-only log
format, schema evolution". This reflects the old pardosa-genome description
rather than pardosa's actual scope as an EDA storage layer.

### MEDIUM priority — stale/outdated information

**M1. GOVERNANCE.md MECE rationale**

Lines 63-67 list 5 domains (Common, Rust, Cherry, Pardosa, Genome). AFM
(adr-fmt) domain exists with 10 ADRs and is configured in `adr-fmt.toml`.
GOVERNANCE.md is incomplete.

**M2. genome.md NATS references — REMOVE (blocked by S6)**

~~Lines 1100-1104 reference `pardosa-genome-nats` companion crate without
marking it as planned/non-existent.~~ **Decision: remove entirely.** Genome
is a file format. NATS transport is pardosa's responsibility.

This fix requires two distinct actions in order:

1. **Governance action (S6):** Amend GEN-0008 ADR to rename
   `pardosa-genome-nats` → `pardosa-nats` and reassign transport ownership
   to pardosa. Must follow GOVERNANCE.md lifecycle process.
2. **Editorial action:** After GEN-0008 is amended, remove all
   `pardosa-genome-nats` references from genome.md (12+ lines — see S3
   for full scope), README components table, and any other plan docs.

Transport APIs (NatsPublisher, NatsConsumer, stream discovery) belong in
a future `pardosa-nats` crate plan.

**M3. genome.md crate structure**

Lines 1341-1368 mix existing Phase 1 files (`lib.rs`, `genome_safe.rs`,
`format.rs`, `config.rs`, `error.rs`) with future Phase 2-3 files
(`sizing_ser.rs`, `writing_ser.rs`, `de.rs`, `compress.rs`, `reader.rs`,
`writer.rs`, `bin/genome-dump.rs`) without clear demarcation. Could mislead
about current implementation state.

**M4. genome.md dependencies section**

Lines 1197-1209 show `crc` as "REMOVED" (still listed) and `bolero = "0.11"`
as a dev-dep. Neither is in actual Cargo.toml. `bolero` is planned tooling,
not yet added.

**M5. pardosa-next.md max_total_elements**

G9 item 7 (line 289): "Default 16M elements". genome.md shows
`DecodeOptions::default()` uses `PageClass::Page0.max_elements()` = 256.
Off by ~5 orders of magnitude (62,500×).

### LOW priority — cosmetic

**L1. build-plan.md test count**

Phase 4 module layout comment says "20 tests" for `MsgpackFileStore`.
Actual file has 30+ test functions. Remove hard-coded count or generalize.

**L2. pardosa version**

`pardosa` is at version `0.2.0` while all other crates are `0.1.0`. No
changelog or rationale documented. Document the version bump reason or
reset if unintentional.

**L3. README pardosa-genome-nats**

~~If `pardosa-genome-nats` is intended as a future workspace member,
consider adding it to the Components table as "planned".~~
**Resolved: `pardosa-genome-nats` is a misnaming — remove, do not
add.** If a NATS transport crate is created, it should be `pardosa-nats`
(pardosa owns transport, genome owns serialization). See S3 and S6.

---

## Open Questions

1. ~~**pardosa-next.md G1-G9**: Annotate each proposal individually with
   resolution status, or single banner disclaimer?~~ **Resolved: delete
   G1–G9 entirely.** genome.md is the SSOT.

2. **pardosa-next.md async approach**: Should `PersistenceAdapter` switch
   to RPITIT for consistency with cherry-pit-core, or does pardosa
   deliberately diverge? **Status: investigate later.**

3. **pardosa version 0.2.0**: Intentional (reflecting fallible-constructor
   transition)? Document it, or reset?

4. **genome.md crate structure**: Add "(planned)" per file, split into
   "Current"/"Planned" sections, or leave as-is since genome.md is a
   design plan?

5. **Crate-level pardosa docs**: `crates/pardosa/pardosa.md` and
   `crates/pardosa/pardosa-design.md` are byte-identical duplicates of
   `docs/plans/` copies. Delete? Symlink? Thin pointer? **Status: noted
   for future grilling** — needs design conversation about crate-level
   docs ownership model.

6. ~~**genome.md pardosa-genome-nats**: planned or misunderstanding?~~
   **Resolved: misnaming + misplacement.** GEN-0008 ADR architecture is
   correct (separate transport from serialization). Crate name is wrong
   (`pardosa-genome-nats` → `pardosa-nats`). Requires GEN-0008 amendment
   before genome.md cleanup. See S3, S6.

7. **GEN-0008 amendment process**: Should the companion crate rename be
   a full new ADR superseding GEN-0008, or an in-place amendment with
   changelog? GOVERNANCE.md lifecycle rules apply. The architectural
   decision (transport-agnostic core) is unchanged — only the crate name
   and domain ownership assignment need correction.

8. **SSOT hierarchy location**: The hierarchy documented above in this
   audit file should be formalized. Candidate locations: GOVERNANCE.md
   (extends existing ADR governance to cover all doc types), a new ADR,
   or a project-level CONTRIBUTING.md. Where does it belong?

---

## Verification Log

Added: 2026-04-27

All 27 factual claims in sections A–D and H1–H10 were verified against
source files. Zero false positives detected. Verification covered:

- S1: MD5 hash comparison confirmed byte-identical duplicates (not symlinks)
- S2: genome.md format specs confirmed (15-byte bare header, 24-byte
  xxHash64 index entries, 3-bit compression_algo, page_class layout).
  GEN-0016 and GEN-0030 ADRs cross-referenced.
- S3: 12+ NATS references found in genome.md (scope larger than original
  5-line estimate). No `pardosa-genome-nats` crate exists.
- S6: GEN-0008 ADR confirmed as accepted, naming `pardosa-genome-nats`.
  ADR drift identified — amendment required before downstream cleanup.
- B (Section B bullets): Module layouts, dependencies, Rust version,
  rmp-serde version, missing modules, missing types all confirmed against
  Cargo.toml and actual source files.
- H9: infrastructure.md `pit-*` namespace confirmed stale.
- M1: GOVERNANCE.md MECE list confirmed as 5 domains, AFM omitted.
- M5: 16M vs 256 default confirmed (~5 orders of magnitude, 62,500×).

Three interpretation errors corrected:
1. S3 reclassified from "misunderstanding" to "misnaming + misplacement"
2. S3 NATS scope expanded from 5 lines to 12+
3. G5-G6 table status corrected from "Reframed" to "Decided by GEN-0008"
