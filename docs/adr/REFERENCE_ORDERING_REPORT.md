# ADR Reference-Ordering Optimization Report

**Goal:** rank `References:` in each ADR broad → narrow so the first reference is the most significant relation. This optimizes downstream LLM consumption — `--context <CRATE>` outputs and direct ADR reads benefit from primacy ordering when an agent has a token budget or attention bias toward early tokens.

## Method

For each non-root ADR, score every existing `References:` entry into a priority bucket and sort. Buckets (lower index = broader / placed earlier):

| # | Bucket |
|---|--------|
| 0 | Own-domain root |
| 1 | Foundation root (COM-0001 / RST-0001 / SEC-0001) |
| 2 | Own-domain S-tier non-root |
| 3 | Foundation S-tier non-root |
| 4 | Cross-domain (non-foundation) S-tier |
| 5 | Own-domain A-tier |
| 6 | Foundation A-tier |
| 7 | Cross-domain A-tier |
| 8 | Own-domain B-tier |
| 9 | Foundation B-tier |
| 10 | Cross-domain B-tier |
| 11 | Own-domain C-tier |
| 12 | Foundation C-tier |
| 13 | Cross-domain C-tier |
| 14 | Own-domain D-tier |
| 15 | Foundation D-tier |
| 16 | Cross-domain D-tier |

Ties within a bucket: ADR number ascending (stable).

**Rationale.** An ADR's own-domain S-tier paradigm is broader than a foundation B-tier mechanism. The Cherry-pit Cherry domain's "Make illegal states unrepresentable" (CHE-0002) is a paradigm choice that *frames* how its derivatives use foundation rules; a foundation B-tier such as COM-0017 ("mechanized invariant enforcement") is the tactical implementation. Paradigm precedes mechanism. Foundation roots come ahead of own-domain S-tier non-roots only because the foundation root is the explicit cross-cutting anchor — its single appearance signals the entire foundation domain applies.

**Add/remove candidates** are advisory only (Section 3); each requires reading the actual rule text.

---

## 1. Reordering Proposals (per domain)

## Domain: COM

| ADR | Tier | Current `References:` | Proposed | Δ |
|-----|------|------------------------|----------|---|
| COM-0001 | S | _root_ | _root_ | — |
| COM-0002 | S | COM-0001 | COM-0001 | ✓ |
| COM-0003 | B | COM-0001, COM-0002, COM-0019 | COM-0001, COM-0002, COM-0019 | ✓ |
| COM-0004 | B | COM-0001, COM-0002, COM-0003, COM-0012 | COM-0001, COM-0002, COM-0012, COM-0003 | **reorder** |
| COM-0005 | B | COM-0001, COM-0002, COM-0003 | COM-0001, COM-0002, COM-0003 | ✓ |
| COM-0006 | B | COM-0001, COM-0002 | COM-0001, COM-0002 | ✓ |
| COM-0007 | B | COM-0001, COM-0002, COM-0019 | COM-0001, COM-0002, COM-0019 | ✓ |
| COM-0008 | B | COM-0001, COM-0011 | COM-0001, COM-0011 | ✓ |
| COM-0009 | B | COM-0001 | COM-0001 | ✓ |
| COM-0010 | B | COM-0001, COM-0006 | COM-0001, COM-0006 | ✓ |
| COM-0011 | B | COM-0001, COM-0008 | COM-0001, COM-0008 | ✓ |
| COM-0012 | S | COM-0001, COM-0004 | COM-0001, COM-0004 | ✓ |
| COM-0013 | B | COM-0001, COM-0002 | COM-0001, COM-0002 | ✓ |
| COM-0014 | B | COM-0001, COM-0002, COM-0012 | COM-0001, COM-0002, COM-0012 | ✓ |
| COM-0015 | B | COM-0001, COM-0006 | COM-0001, COM-0006 | ✓ |
| COM-0016 | B | COM-0001, RST-0004 | COM-0001, RST-0004 | ✓ |
| COM-0017 | B | COM-0001, COM-0009 | COM-0001, COM-0009 | ✓ |
| COM-0018 | S | COM-0001, CHE-0006, PAR-0004 | COM-0001, PAR-0004, CHE-0006 | **reorder** |
| COM-0019 | A | COM-0001, COM-0003, COM-0005, COM-0007, COM-0010 | COM-0001, COM-0003, COM-0005, COM-0007, COM-0010 | ✓ |
| COM-0020 | B | COM-0003, COM-0005, COM-0017 | COM-0003, COM-0005, COM-0017 | ✓ |
| COM-0021 | B | COM-0013, COM-0007, COM-0017 | COM-0007, COM-0013, COM-0017 | **reorder** |
| COM-0022 | B | COM-0005, COM-0018, COM-0020 | COM-0018, COM-0005, COM-0020 | **reorder** |
| COM-0023 | B | COM-0017, COM-0020 | COM-0017, COM-0020 | ✓ |
| COM-0024 | B | COM-0012, COM-0017, COM-0002 | COM-0002, COM-0012, COM-0017 | **reorder** |

## Domain: RST

| ADR | Tier | Current `References:` | Proposed | Δ |
|-----|------|------------------------|----------|---|
| RST-0001 | B | _root_ | _root_ | — |
| RST-0002 | B | RST-0001, COM-0013 | RST-0001, COM-0013 | ✓ |
| RST-0003 | B | RST-0001, COM-0017 | RST-0001, COM-0017 | ✓ |
| RST-0004 | B | RST-0001, COM-0016 | RST-0001, COM-0016 | ✓ |
| RST-0005 | B | RST-0001, SEC-0004 | RST-0001, SEC-0004 | ✓ |

## Domain: SEC

| ADR | Tier | Current `References:` | Proposed | Δ |
|-----|------|------------------------|----------|---|
| SEC-0001 | S | _root_ | _root_ | — |
| SEC-0002 | B | SEC-0001 | SEC-0001 | ✓ |
| SEC-0003 | B | SEC-0001 | SEC-0001 | ✓ |
| SEC-0004 | B | SEC-0001 | SEC-0001 | ✓ |
| SEC-0005 | B | SEC-0001 | SEC-0001 | ✓ |
| SEC-0006 | A | SEC-0001, SEC-0002, SEC-0004 | SEC-0001, SEC-0002, SEC-0004 | ✓ |
| SEC-0007 | B | SEC-0001, SEC-0004, SEC-0005 | SEC-0001, SEC-0004, SEC-0005 | ✓ |
| SEC-0008 | B | SEC-0001, SEC-0002, SEC-0004, SEC-0005 | SEC-0001, SEC-0002, SEC-0004, SEC-0005 | ✓ |
| SEC-0009 | B | SEC-0001, SEC-0002, SEC-0004, RST-0004 | SEC-0001, SEC-0002, SEC-0004, RST-0004 | ✓ |
| SEC-0010 | B | SEC-0001, SEC-0002, SEC-0005, SEC-0007 | SEC-0001, SEC-0002, SEC-0005, SEC-0007 | ✓ |

## Domain: CHE

| ADR | Tier | Current `References:` | Proposed | Δ |
|-----|------|------------------------|----------|---|
| CHE-0001 | S | _root_ | _root_ | — |
| CHE-0002 | S | CHE-0001 | CHE-0001 | ✓ |
| CHE-0003 | S | CHE-0001, CHE-0002, CHE-0028 | CHE-0001, CHE-0002, CHE-0028 | ✓ |
| CHE-0004 | S | _root_ | _root_ | — |
| CHE-0005 | S | CHE-0001, CHE-0002, COM-0002 | CHE-0001, CHE-0002, COM-0002 | ✓ |
| CHE-0006 | S | CHE-0001, CHE-0004, PAR-0004 | CHE-0001, CHE-0004, PAR-0004 | ✓ |
| CHE-0007 | B | CHE-0001, COM-0017 | CHE-0001, COM-0017 | ✓ |
| CHE-0008 | A | CHE-0001, CHE-0004, CHE-0014 | CHE-0001, CHE-0004, CHE-0014 | ✓ |
| CHE-0009 | A | CHE-0001, CHE-0004, CHE-0008, COM-0005 | CHE-0001, CHE-0004, CHE-0008, COM-0005 | ✓ |
| CHE-0010 | A | CHE-0001, CHE-0004, CHE-0014, CHE-0022 | CHE-0001, CHE-0004, CHE-0014, CHE-0022 | ✓ |
| CHE-0011 | B | CHE-0001, CHE-0002, CHE-0006 | CHE-0001, CHE-0002, CHE-0006 | ✓ |
| CHE-0012 | A | CHE-0001, CHE-0008, CHE-0009, CHE-0013, CHE-0037 | CHE-0001, CHE-0008, CHE-0009, CHE-0013, CHE-0037 | ✓ |
| CHE-0013 | B | CHE-0001, CHE-0011 | CHE-0001, CHE-0011 | ✓ |
| CHE-0014 | B | CHE-0001, CHE-0004, CHE-0010 | CHE-0001, CHE-0004, CHE-0010 | ✓ |
| CHE-0015 | B | CHE-0001, CHE-0005 | CHE-0001, CHE-0005 | ✓ |
| CHE-0016 | B | CHE-0001, CHE-0004, CHE-0039, COM-0003 | CHE-0001, CHE-0004, CHE-0039, COM-0003 | ✓ |
| CHE-0017 | B | CHE-0001, CHE-0004, CHE-0005 | CHE-0001, CHE-0004, CHE-0005 | ✓ |
| CHE-0018 | B | CHE-0001, CHE-0008, CHE-0025 | CHE-0001, CHE-0008, CHE-0025 | ✓ |
| CHE-0019 | B | CHE-0001, CHE-0013, COM-0004, COM-0005 | CHE-0001, CHE-0013, COM-0004, COM-0005 | ✓ |
| CHE-0020 | B | CHE-0001, CHE-0006, CHE-0011, CHE-0013, CHE-0018, CHE-0033, COM-0003 | CHE-0001, CHE-0006, CHE-0011, CHE-0013, CHE-0018, COM-0003, CHE-0033 | **reorder** |
| CHE-0021 | B | CHE-0001, CHE-0015, CHE-0022 | CHE-0001, CHE-0015, CHE-0022 | ✓ |
| CHE-0022 | B | CHE-0001, CHE-0009, CHE-0010, CHE-0021, CHE-0031, GEN-0002 | CHE-0001, GEN-0002, CHE-0009, CHE-0010, CHE-0021, CHE-0031 | **reorder** |
| CHE-0023 | B | CHE-0001, CHE-0009, CHE-0013 | CHE-0001, CHE-0009, CHE-0013 | ✓ |
| CHE-0024 | C | CHE-0001, CHE-0004, CHE-0017, CHE-0041 | CHE-0001, CHE-0004, CHE-0017, CHE-0041 | ✓ |
| CHE-0025 | D | CHE-0001, CHE-0018 | CHE-0001, CHE-0018 | ✓ |
| CHE-0026 | D | CHE-0001, CHE-0007, RST-0003 | CHE-0001, CHE-0007, RST-0003 | ✓ |
| CHE-0027 | D | CHE-0001, CHE-0015 | CHE-0001, CHE-0015 | ✓ |
| CHE-0028 | B | CHE-0001, CHE-0002, CHE-0003, CHE-0005, COM-0017 | CHE-0001, CHE-0002, CHE-0003, CHE-0005, COM-0017 | ✓ |
| CHE-0029 | B | CHE-0001, CHE-0004, COM-0014 | CHE-0001, CHE-0004, COM-0014 | ✓ |
| CHE-0030 | B | CHE-0001, CHE-0029, COM-0002 | CHE-0001, COM-0002, CHE-0029 | **reorder** |
| CHE-0031 | D | CHE-0001, CHE-0022, CHE-0045 | CHE-0001, CHE-0022, CHE-0045 | ✓ |
| CHE-0032 | D | CHE-0001, CHE-0006 | CHE-0001, CHE-0006 | ✓ |
| CHE-0033 | D | CHE-0001, CHE-0006, CHE-0016, CHE-0034 | CHE-0001, CHE-0006, CHE-0016, CHE-0034 | ✓ |
| CHE-0034 | D | CHE-0001, CHE-0016 | CHE-0001, CHE-0016 | ✓ |
| CHE-0035 | D | CHE-0001, CHE-0006, CHE-0032, COM-0003 | CHE-0001, CHE-0006, COM-0003, CHE-0032 | **reorder** |
| CHE-0036 | D | CHE-0001, CHE-0031, CHE-0032 | CHE-0001, CHE-0031, CHE-0032 | ✓ |
| CHE-0037 | D | CHE-0001, CHE-0009, CHE-0010, CHE-0040 | CHE-0001, CHE-0009, CHE-0010, CHE-0040 | ✓ |
| CHE-0038 | B | CHE-0001, CHE-0003, CHE-0028, COM-0001 | CHE-0001, COM-0001, CHE-0003, CHE-0028 | **reorder** |
| CHE-0039 | A | CHE-0016, CHE-0004, CHE-0017 | CHE-0004, CHE-0016, CHE-0017 | **reorder** |
| CHE-0040 | B | CHE-0001, CHE-0017, CHE-0024, CHE-0037, CHE-0039 | CHE-0001, CHE-0039, CHE-0017, CHE-0024, CHE-0037 | **reorder** |
| CHE-0041 | B | CHE-0001, CHE-0006, CHE-0008, CHE-0017, CHE-0039, COM-0005 | CHE-0001, CHE-0006, CHE-0008, CHE-0039, CHE-0017, COM-0005 | **reorder** |
| CHE-0042 | A | CHE-0001, CHE-0002, CHE-0010, CHE-0016, CHE-0039 | CHE-0001, CHE-0002, CHE-0010, CHE-0039, CHE-0016 | **reorder** |
| CHE-0043 | D | CHE-0001, CHE-0006, CHE-0032, CHE-0035, COM-0003 | CHE-0001, CHE-0006, COM-0003, CHE-0032, CHE-0035 | **reorder** |
| CHE-0044 | D | CHE-0001, CHE-0006, CHE-0031, CHE-0032, CHE-0043 | CHE-0001, CHE-0006, CHE-0031, CHE-0032, CHE-0043 | ✓ |
| CHE-0045 | B | CHE-0001, CHE-0004, CHE-0022, CHE-0029 | CHE-0001, CHE-0004, CHE-0022, CHE-0029 | ✓ |

## Domain: PAR

| ADR | Tier | Current `References:` | Proposed | Δ |
|-----|------|------------------------|----------|---|
| PAR-0001 | B | _root_ | _root_ | — |
| PAR-0002 | D | PAR-0001, PAR-0007 | PAR-0001, PAR-0007 | ✓ |
| PAR-0003 | B | PAR-0001, PAR-0008 | PAR-0001, PAR-0008 | ✓ |
| PAR-0004 | S | _root_ | _root_ | — |
| PAR-0005 | B | PAR-0004, PAR-0013 | PAR-0004, PAR-0013 | ✓ |
| PAR-0006 | B | PAR-0003, PAR-0005, PAR-0007 | PAR-0003, PAR-0005, PAR-0007 | ✓ |
| PAR-0007 | B | PAR-0004, PAR-0008 | PAR-0004, PAR-0008 | ✓ |
| PAR-0008 | S | PAR-0004, PAR-0007, PAR-0014 | PAR-0004, PAR-0007, PAR-0014 | ✓ |
| PAR-0009 | D | PAR-0001, PAR-0005 | PAR-0001, PAR-0005 | ✓ |
| PAR-0010 | B | PAR-0001, PAR-0012 | PAR-0001, PAR-0012 | ✓ |
| PAR-0011 | D | PAR-0001 | PAR-0001 | ✓ |
| PAR-0012 | B | PAR-0004, PAR-0005, PAR-0008 | PAR-0004, PAR-0008, PAR-0005 | **reorder** |
| PAR-0013 | B | PAR-0004, PAR-0005, PAR-0007 | PAR-0004, PAR-0005, PAR-0007 | ✓ |
| PAR-0014 | C | PAR-0004, PAR-0008 | PAR-0004, PAR-0008 | ✓ |
| PAR-0015 | B | PAR-0004, PAR-0007, PAR-0008, PAR-0013 | PAR-0004, PAR-0008, PAR-0007, PAR-0013 | **reorder** |
| PAR-0016 | B | PAR-0004, PAR-0007, PAR-0008 | PAR-0004, PAR-0008, PAR-0007 | **reorder** |

## Domain: GEN

| ADR | Tier | Current `References:` | Proposed | Δ |
|-----|------|------------------------|----------|---|
| GEN-0001 | S | _root_ | _root_ | — |
| GEN-0002 | S | GEN-0001, CHE-0022, PAR-0002 | GEN-0001, CHE-0022, PAR-0002 | ✓ |
| GEN-0003 | B | GEN-0001 | GEN-0001 | ✓ |
| GEN-0004 | B | GEN-0001, GEN-0033 | GEN-0001, GEN-0033 | ✓ |
| GEN-0005 | B | GEN-0001 | GEN-0001 | ✓ |
| GEN-0006 | A | GEN-0001, CHE-0007 | GEN-0001, CHE-0007 | ✓ |
| GEN-0007 | S | GEN-0001, GEN-0021 | GEN-0001, GEN-0021 | ✓ |
| GEN-0008 | B | GEN-0001 | GEN-0001 | ✓ |
| GEN-0009 | B | GEN-0001, GEN-0003, GEN-0031 | GEN-0001, GEN-0003, GEN-0031 | ✓ |
| GEN-0010 | D | GEN-0001, GEN-0008 | GEN-0001, GEN-0008 | ✓ |
| GEN-0011 | B | GEN-0001, GEN-0006, GEN-0034 | GEN-0001, GEN-0006, GEN-0034 | ✓ |
| GEN-0012 | B | GEN-0001, GEN-0006, GEN-0007, GEN-0024 | GEN-0001, GEN-0007, GEN-0006, GEN-0024 | **reorder** |
| GEN-0013 | B | GEN-0001 | GEN-0001 | ✓ |
| GEN-0014 | B | GEN-0001, GEN-0013 | GEN-0001, GEN-0013 | ✓ |
| GEN-0015 | B | GEN-0001 | GEN-0001 | ✓ |
| GEN-0016 | D | GEN-0001, GEN-0003 | GEN-0001, GEN-0003 | ✓ |
| GEN-0017 | B | GEN-0001, GEN-0007 | GEN-0001, GEN-0007 | ✓ |
| GEN-0018 | B | GEN-0001, GEN-0011 | GEN-0001, GEN-0011 | ✓ |
| GEN-0019 | D | GEN-0001, GEN-0004 | GEN-0001, GEN-0004 | ✓ |
| GEN-0020 | B | GEN-0001, GEN-0017, GEN-0014 | GEN-0001, GEN-0014, GEN-0017 | **reorder** |
| GEN-0021 | B | GEN-0001, GEN-0005, GEN-0011, GEN-0032 | GEN-0001, GEN-0032, GEN-0005, GEN-0011 | **reorder** |
| GEN-0022 | B | GEN-0001, GEN-0004, GEN-0007, GEN-0018 | GEN-0001, GEN-0007, GEN-0004, GEN-0018 | **reorder** |
| GEN-0023 | D | GEN-0001, GEN-0007 | GEN-0001, GEN-0007 | ✓ |
| GEN-0024 | D | GEN-0001, GEN-0012 | GEN-0001, GEN-0012 | ✓ |
| GEN-0025 | B | GEN-0001, GEN-0011, GEN-0016 | GEN-0001, GEN-0011, GEN-0016 | ✓ |
| GEN-0026 | D | GEN-0001, GEN-0008 | GEN-0001, GEN-0008 | ✓ |
| GEN-0027 | B | GEN-0001, GEN-0004, GEN-0033 | GEN-0001, GEN-0033, GEN-0004 | **reorder** |
| GEN-0028 | D | GEN-0001, GEN-0003, GEN-0011 | GEN-0001, GEN-0003, GEN-0011 | ✓ |
| GEN-0029 | B | GEN-0001, GEN-0004 | GEN-0001, GEN-0004 | ✓ |
| GEN-0030 | D | GEN-0001, GEN-0014 | GEN-0001, GEN-0014 | ✓ |
| GEN-0031 | D | GEN-0001, GEN-0003, GEN-0009 | GEN-0001, GEN-0003, GEN-0009 | ✓ |
| GEN-0032 | S | GEN-0001, GEN-0004, GEN-0021 | GEN-0001, GEN-0004, GEN-0021 | ✓ |
| GEN-0033 | A | GEN-0001, GEN-0004, GEN-0032 | GEN-0001, GEN-0032, GEN-0004 | **reorder** |
| GEN-0034 | B | GEN-0006, GEN-0011, GEN-0013, GEN-0032 | GEN-0032, GEN-0006, GEN-0011, GEN-0013 | **reorder** |

## Domain: AFM

| ADR | Tier | Current `References:` | Proposed | Δ |
|-----|------|------------------------|----------|---|
| AFM-0001 | S | _root_ | _root_ | — |
| AFM-0003 | B | AFM-0001 | AFM-0001 | ✓ |
| AFM-0004 | A | AFM-0001 | AFM-0001 | ✓ |
| AFM-0006 | D | AFM-0004 | AFM-0004 | ✓ |
| AFM-0008 | S | AFM-0001 | AFM-0001 | ✓ |
| AFM-0009 | S | AFM-0001 | AFM-0001 | ✓ |
| AFM-0011 | S | AFM-0001 | AFM-0001 | ✓ |
| AFM-0012 | S | AFM-0011 | AFM-0011 | ✓ |
| AFM-0013 | D | AFM-0001 | AFM-0001 | ✓ |
| AFM-0014 | B | AFM-0003 | AFM-0003 | ✓ |
| AFM-0015 | B | AFM-0001, AFM-0008 | AFM-0001, AFM-0008 | ✓ |


**Summary:** 25 of 145 ADRs reordered (root/empty excluded).


---

## 2. Add Candidates — Missing Domain Roots

These ADRs reference siblings but omit their own domain root. Adding the root as the first reference both anchors the broad context and matches the convention used by the rest of the corpus.


- **AFM-0006** (D) — current refs: `AFM-0004` — consider adding domain root `AFM-0001`
- **AFM-0012** (S) — current refs: `AFM-0011` — consider adding domain root `AFM-0001`
- **AFM-0014** (B) — current refs: `AFM-0003` — consider adding domain root `AFM-0001`
- **COM-0020** (B) — current refs: `COM-0003, COM-0005, COM-0017` — consider adding domain root `COM-0001`
- **COM-0021** (B) — current refs: `COM-0013, COM-0007, COM-0017` — consider adding domain root `COM-0001`
- **COM-0022** (B) — current refs: `COM-0005, COM-0018, COM-0020` — consider adding domain root `COM-0001`
- **COM-0023** (B) — current refs: `COM-0017, COM-0020` — consider adding domain root `COM-0001`
- **COM-0024** (B) — current refs: `COM-0012, COM-0017, COM-0002` — consider adding domain root `COM-0001`
- **GEN-0034** (B) — current refs: `GEN-0006, GEN-0011, GEN-0013, GEN-0032` — consider adding domain root `GEN-0001`
- **PAR-0006** (B) — current refs: `PAR-0003, PAR-0005, PAR-0007` — consider adding domain root `PAR-0001`


### Recommended additions

| ADR | Current | Proposed |
|-----|---------|----------|
| AFM-0006 | `AFM-0004` | `AFM-0001, AFM-0004` |
| AFM-0012 | `AFM-0011` | `AFM-0001, AFM-0011` |
| AFM-0014 | `AFM-0003` | `AFM-0001, AFM-0003` |
| COM-0020 | `COM-0003, COM-0005, COM-0017` | `COM-0001, COM-0003, COM-0005, COM-0017` |
| COM-0021 | `COM-0007, COM-0013, COM-0017` (after reorder) | `COM-0001, COM-0007, COM-0013, COM-0017` |
| COM-0022 | `COM-0018, COM-0005, COM-0020` (after reorder) | `COM-0001, COM-0018, COM-0005, COM-0020` |
| COM-0023 | `COM-0017, COM-0020` | `COM-0001, COM-0017, COM-0020` |
| COM-0024 | `COM-0002, COM-0012, COM-0017` (after reorder) | `COM-0001, COM-0002, COM-0012, COM-0017` |
| GEN-0034 | `GEN-0032, GEN-0006, GEN-0011, GEN-0013` (after reorder) | `GEN-0001, GEN-0032, GEN-0006, GEN-0011, GEN-0013` |
| PAR-0006 | `PAR-0003, PAR-0005, PAR-0007` | `PAR-0001, PAR-0003, PAR-0005, PAR-0007` |

Each addition stays within the T020 reference cap for its tier (B-tier ≤ 7; D-tier ≤ 5).

---

## 3. Remove / Demote Candidates — Narrower References

These references point from a broader (lower tier letter) ADR *down* to a narrower one. The relationship is real (the broader ADR was likely revised after the narrower one was written) but the *reverse* direction — having the narrower ADR reference the broader one — usually carries the architectural weight. The narrower ADR already shows up in the focal's `--critique` fan-in.

**Recommendation:** keep most; consider removing only when the rule text of the narrower target adds nothing the focal does not already state.


- **CHE-0003** (S) → `CHE-0028` (B) — narrower reference; verify CHE-0028 genuinely informs CHE-0003 or move to fan-in only
- **CHE-0008** (A) → `CHE-0014` (B) — narrower reference; verify CHE-0014 genuinely informs CHE-0008 or move to fan-in only
- **CHE-0009** (A) → `COM-0005` (B) — narrower reference; verify COM-0005 genuinely informs CHE-0009 or move to fan-in only
- **CHE-0010** (A) → `CHE-0014` (B) — narrower reference; verify CHE-0014 genuinely informs CHE-0010 or move to fan-in only
- **CHE-0010** (A) → `CHE-0022` (B) — narrower reference; verify CHE-0022 genuinely informs CHE-0010 or move to fan-in only
- **CHE-0012** (A) → `CHE-0013` (B) — narrower reference; verify CHE-0013 genuinely informs CHE-0012 or move to fan-in only
- **CHE-0012** (A) → `CHE-0037` (D) — narrower reference; verify CHE-0037 genuinely informs CHE-0012 or move to fan-in only
- **CHE-0018** (B) → `CHE-0025` (D) — narrower reference; verify CHE-0025 genuinely informs CHE-0018 or move to fan-in only
- **CHE-0020** (B) → `CHE-0033` (D) — narrower reference; verify CHE-0033 genuinely informs CHE-0020 or move to fan-in only
- **CHE-0022** (B) → `CHE-0031` (D) — narrower reference; verify CHE-0031 genuinely informs CHE-0022 or move to fan-in only
- **CHE-0039** (A) → `CHE-0016` (B) — narrower reference; verify CHE-0016 genuinely informs CHE-0039 or move to fan-in only
- **CHE-0039** (A) → `CHE-0017` (B) — narrower reference; verify CHE-0017 genuinely informs CHE-0039 or move to fan-in only
- **CHE-0040** (B) → `CHE-0024` (C) — narrower reference; verify CHE-0024 genuinely informs CHE-0040 or move to fan-in only
- **CHE-0040** (B) → `CHE-0037` (D) — narrower reference; verify CHE-0037 genuinely informs CHE-0040 or move to fan-in only
- **CHE-0042** (A) → `CHE-0016` (B) — narrower reference; verify CHE-0016 genuinely informs CHE-0042 or move to fan-in only
- **COM-0012** (S) → `COM-0004` (B) — narrower reference; verify COM-0004 genuinely informs COM-0012 or move to fan-in only
- **COM-0019** (A) → `COM-0003` (B) — narrower reference; verify COM-0003 genuinely informs COM-0019 or move to fan-in only
- **COM-0019** (A) → `COM-0005` (B) — narrower reference; verify COM-0005 genuinely informs COM-0019 or move to fan-in only
- **COM-0019** (A) → `COM-0007` (B) — narrower reference; verify COM-0007 genuinely informs COM-0019 or move to fan-in only
- **COM-0019** (A) → `COM-0010` (B) — narrower reference; verify COM-0010 genuinely informs COM-0019 or move to fan-in only
- **GEN-0002** (S) → `CHE-0022` (B) — narrower reference; verify CHE-0022 genuinely informs GEN-0002 or move to fan-in only
- **GEN-0002** (S) → `PAR-0002` (D) — narrower reference; verify PAR-0002 genuinely informs GEN-0002 or move to fan-in only
- **GEN-0006** (A) → `CHE-0007` (B) — narrower reference; verify CHE-0007 genuinely informs GEN-0006 or move to fan-in only
- **GEN-0007** (S) → `GEN-0021` (B) — narrower reference; verify GEN-0021 genuinely informs GEN-0007 or move to fan-in only
- **GEN-0009** (B) → `GEN-0031` (D) — narrower reference; verify GEN-0031 genuinely informs GEN-0009 or move to fan-in only
- **GEN-0012** (B) → `GEN-0024` (D) — narrower reference; verify GEN-0024 genuinely informs GEN-0012 or move to fan-in only
- **GEN-0025** (B) → `GEN-0016` (D) — narrower reference; verify GEN-0016 genuinely informs GEN-0025 or move to fan-in only
- **GEN-0032** (S) → `GEN-0004` (B) — narrower reference; verify GEN-0004 genuinely informs GEN-0032 or move to fan-in only
- **GEN-0032** (S) → `GEN-0021` (B) — narrower reference; verify GEN-0021 genuinely informs GEN-0032 or move to fan-in only
- **GEN-0033** (A) → `GEN-0004` (B) — narrower reference; verify GEN-0004 genuinely informs GEN-0033 or move to fan-in only
- **PAR-0008** (S) → `PAR-0007` (B) — narrower reference; verify PAR-0007 genuinely informs PAR-0008 or move to fan-in only
- **PAR-0008** (S) → `PAR-0014` (C) — narrower reference; verify PAR-0014 genuinely informs PAR-0008 or move to fan-in only
- **SEC-0006** (A) → `SEC-0002` (B) — narrower reference; verify SEC-0002 genuinely informs SEC-0006 or move to fan-in only
- **SEC-0006** (A) → `SEC-0004` (B) — narrower reference; verify SEC-0004 genuinely informs SEC-0006 or move to fan-in only

### Per-case judgment for the strongest demotion candidates

| ADR | Narrow ref | Verdict | Reason |
|-----|------------|---------|--------|
| CHE-0003 → CHE-0028 | keep | CHE-0028 (compile-fail tests) is the *enforcement* mechanism for CHE-0003's preference; the link is load-bearing |
| CHE-0010 → CHE-0022 | keep | CHE-0022 schema evolution constrains the supertrait bounds |
| CHE-0012 → CHE-0037 | keep | Default-construction makes sense only because no-snapshot-support makes zero-state replay the path |
| CHE-0040 → CHE-0037 | **demote / remove** | Saga deferral and snapshot deferral are independent decisions; cite once in Consequences instead |
| CHE-0040 → CHE-0024 | keep | Delivery model directly affects compensation timing |
| GEN-0009 → GEN-0031 | **demote / remove** | One-schema-per-file is independent of the cross-language defer; the link is incidental |
| GEN-0012 → GEN-0024 | keep | NaN bit-pattern preservation depends on the no-pointer-cast wire encoding |
| GEN-0025 → GEN-0016 | keep | Bare-message validation contract uses the file-checksum scheme as contrast |
| GEN-0032 → GEN-0021 | keep | Canonical encoding requires breadth-first heap ordering |
| GEN-0032 → GEN-0004 | keep | Canonical encoding requires deterministic types |
| PAR-0008 → PAR-0014 | keep | Backpressure is the durable-first failure mode |
| SEC-0006 → SEC-0002 | keep | TOCTOU elimination *is* a trust-boundary integrity rule |

For all other narrower-pointing references, recommend keeping — the structural relationship is genuine and a reader benefits from the forward link.

---

## 4. Summary

- **149 ADRs** analysed, 7 are roots (skipped), 1 has no refs (skipped).
- **25 ADRs** require reordering only.
- **10 ADRs** should add their missing domain root.
- **2 ADRs** have a clear demotion candidate (CHE-0040 → CHE-0037, GEN-0009 → GEN-0031); the rest of the "narrower reference" flags are kept.

After applying these changes, every non-root ADR will lead with its broadest, most paradigm-shaping reference. `--context <CRATE>` output is already tier-sorted globally; this change additionally orders refs *within* each ADR so an agent reading a single ADR file gets the same broad-to-narrow flow.

## 5. Application Plan (proposed, not yet executed)

1. Apply the 25 reorderings from Section 1.
2. Apply the 10 root additions from Section 2.
3. Apply the 2 demotions from Section 3.
4. Run `cargo run -p adr-fmt -- --lint` — expect 0 new warnings (T020 caps respected).
5. Spot-check 5 random reordered ADRs with `--critique` to confirm fan-in/fan-out still parse.

Confirm before edits begin.
