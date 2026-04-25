# Genome Domain — Architecture Decision Records

ADRs for pardosa-genome, the binary serialization format: wire layout,
schema hashing, zero-copy deserialization, compression, security limits
(DoS protection, decompression bombs), type validation, and forward
compatibility.

Governed by [GOVERNANCE.md](../GOVERNANCE.md).

## Index

| # | Title | Tier | Status |
|---|-------|:----:|--------|
| [GEN-0001](GEN-0001-serde-native-serialization-with-genomesafe-marker-trait.md) | Serde-native serialization with GenomeSafe marker | S | Accepted |
| [GEN-0002](GEN-0002-no-schema-evolution-fixed-layout.md) | No schema evolution — fixed layout | S | Accepted |
| [GEN-0003](GEN-0003-compile-time-xxhash64-schema-hashing.md) | Compile-time xxHash64 schema hashing | A | Amended 2025-04-01 |
| [GEN-0004](GEN-0004-reject-non-deterministic-types-and-serde-attrs.md) | Reject non-deterministic types and serde attrs | A | Amended 2025-04-01, 2026-04-01 |
| [GEN-0005](GEN-0005-two-pass-serialization-architecture.md) | Two-pass serialization architecture | A | Accepted |
| [GEN-0006](GEN-0006-zero-copy-deserialization-with-forbid-unsafe-code.md) | Zero-copy deserialization with forbid(unsafe_code) | A | Accepted |
| [GEN-0007](GEN-0007-flatbuffers-style-offset-based-binary-layout.md) | FlatBuffers-style offset-based binary layout | S | Accepted |
| [GEN-0008](GEN-0008-transport-agnostic-core-with-companion-crate-separation.md) | Transport-agnostic core with companion crate separation | B | Accepted |
| [GEN-0009](GEN-0009-one-schema-per-file-with-embedded-schema-source.md) | One schema per file with embedded schema source | B | Accepted |
| [GEN-0010](GEN-0010-std-only-for-now-no-std-deferred.md) | std-only for now — no_std deferred | C | Amended 2025-04-01 |
| [GEN-0011](GEN-0011-inline-verification-check-catalog.md) | Inline verification check catalog | A | Accepted |
| [GEN-0012](GEN-0012-little-endian-wire-encoding-no-pointer-casts.md) | Little-endian wire encoding — no pointer casts | D | Accepted |
| [GEN-0013](GEN-0013-page-class-resource-limits-for-dos-protection.md) | Page-class resource limits for DoS protection | B | Accepted |
| [GEN-0014](GEN-0014-multi-layered-decompression-bomb-mitigation.md) | Multi-layered decompression bomb mitigation | B | Accepted |
| [GEN-0015](GEN-0015-forward-compatibility-contract.md) | Forward compatibility contract | B | Accepted |
| [GEN-0016](GEN-0016-xxhash64-for-file-integrity-checksums.md) | xxHash64 for file integrity checksums | B | Accepted |
| [GEN-0017](GEN-0017-4-gib-per-message-limit-u32-offsets.md) | 4 GiB per-message limit — u32 offsets | B | Accepted |
| [GEN-0018](GEN-0018-non-zero-padding-is-hard-error.md) | Non-zero padding is hard error | B | Accepted |
| [GEN-0019](GEN-0019-box-arc-transparency-rc-exclusion.md) | Box/Arc transparency — Rc exclusion | D | Accepted |
| [GEN-0020](GEN-0020-empty-containers-always-allocate-heap-entries.md) | Empty containers always allocate heap entries | B | Accepted |
| [GEN-0021](GEN-0021-breadth-first-heap-ordering.md) | Breadth-first heap ordering | B | Accepted |
| [GEN-0022](GEN-0022-externally-tagged-enums-discriminant-offset-encoding.md) | Externally tagged enums — discriminant offset encoding | B | Accepted |
| [GEN-0023](GEN-0023-i128-u128-alignment-capped-at-8-bytes.md) | i128/u128 alignment capped at 8 bytes | D | Accepted |
| [GEN-0024](GEN-0024-nan-bit-pattern-preservation-no-canonicalization.md) | NaN bit-pattern preservation — no canonicalization | D | Accepted |
| [GEN-0025](GEN-0025-bare-messages-structural-validation-only.md) | Bare messages — structural validation only | B | Accepted |
| [GEN-0026](GEN-0026-no-format-auto-detection-bare-vs-file.md) | No format auto-detection — bare vs file | D | Accepted |
| [GEN-0027](GEN-0027-full-serde-data-model-ron-algebraic-types.md) | Full serde data model — RON algebraic types | B | Accepted |
| [GEN-0028](GEN-0028-tuple-struct-tuple-wire-equivalence.md) | Tuple struct / tuple wire equivalence | D | Accepted |
| [GEN-0029](GEN-0029-reject-serde-default-at-compile-time.md) | Reject #[serde(default)] at compile time | B | Accepted |
| [GEN-0030](GEN-0030-zstd-only-compression-in-v1.md) | Zstd-only compression in v1 | D | Accepted |
| [GEN-0031](GEN-0031-rust-only-cross-language-read-deferred.md) | Rust-only — cross-language read deferred | D | Accepted |
| [GEN-0032](GEN-0032-canonical-encoding-contract.md) | Canonical encoding contract | S | Accepted |
| [GEN-0033](GEN-0033-genome-ord-marker-trait-for-map-keys.md) | GenomeOrd marker trait for map keys | D | Accepted |

**Tier distribution:** 4S · 5A · 14B · 1C · 9D

## Cross-Domain References

| Genome ADR | Framework ADR | Relationship |
|------------|---------------|--------------|
| GEN-0002 | CHE-0022 (Schema Evolution) | References |
| GEN-0006 | CHE-0007 (Forbid Unsafe) | Illustrates |

| Genome ADR | Pardosa ADR | Relationship |
|------------|-------------|--------------|
| GEN-0001 | PAR-0003 (Event Immutability) | Referenced by |
| GEN-0001 | PAR-0006 (Genome as Primary) | Referenced by |
| GEN-0002 | PAR-0002 (Index::NONE Sentinel) | References / Referenced by |
| GEN-0006 | PAR-0006 (Genome as Primary) | Referenced by |
| GEN-0007 | PAR-0002 (Index::NONE Sentinel) | Referenced by |
| GEN-0008 | PAR-0006 (Genome as Primary) | Referenced by |
| GEN-0012 | PAR-0006 (Genome as Primary) | Referenced by |
| GEN-0031 | PAR-0006 (Genome as Primary) | Referenced by |

## Reference Documents

- [genome.md](../../genome.md) — genome binary format design document
