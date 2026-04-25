# Genome Domain — Architecture Decision Records

This directory will contain ADRs for pardosa-genome, the binary
serialization format: wire layout, schema hashing, zero-copy
deserialization, compression, security limits (DoS protection,
decompression bombs), type validation, and forward compatibility.

Governed by [GOVERNANCE.md](../GOVERNANCE.md).

## Migration Status

**Pending.** 33 ADRs (GEN-0001 through GEN-0033) are awaiting
migration from `quicksilver/crates/pardosa-genome/adr/`. The
migration will:

1. Reformat each ADR to the [governance template](../GOVERNANCE.md#7-adr-template)
2. Assign tiers (S → D)
3. Add Date / Last-reviewed / Migration-Origin fields
4. Rewrite relative paths to cherry-pit locations
5. Add cross-domain links to Framework and Pardosa ADRs

## Planned Index

| #        | Title                                                | Status  |
|----------|------------------------------------------------------|---------|
| GEN-0001 | Serde-native serialization with GenomeSafe marker    | Pending |
| GEN-0002 | No schema evolution — fixed layout                   | Pending |
| GEN-0003 | Compile-time xxHash64 schema hashing                 | Pending |
| GEN-0004 | Reject non-deterministic types and serde attrs       | Pending |
| GEN-0005 | Two-pass serialization architecture                  | Pending |
| GEN-0006 | Zero-copy deserialization with forbid(unsafe_code)   | Pending |
| GEN-0007 | FlatBuffers-style offset-based binary layout         | Pending |
| GEN-0008 | Transport-agnostic core with companion crate separation | Pending |
| GEN-0009 | One schema per file with embedded schema source      | Pending |
| GEN-0010 | std-only for now — no_std deferred                   | Pending |
| GEN-0011 | Inline verification check catalog                    | Pending |
| GEN-0012 | Little-endian wire encoding — no pointer casts       | Pending |
| GEN-0013 | Page-class resource limits for DoS protection        | Pending |
| GEN-0014 | Multi-layered decompression bomb mitigation          | Pending |
| GEN-0015 | Forward compatibility contract                       | Pending |
| GEN-0016 | xxHash64 for file integrity checksums                | Pending |
| GEN-0017 | 4 GiB per-message limit — u32 offsets                | Pending |
| GEN-0018 | Non-zero padding is hard error                       | Pending |
| GEN-0019 | Box/Arc transparency — Rc exclusion                  | Pending |
| GEN-0020 | Empty containers always allocate heap entries        | Pending |
| GEN-0021 | Breadth-first heap ordering                          | Pending |
| GEN-0022 | Externally tagged enums — discriminant offset encoding | Pending |
| GEN-0023 | i128/u128 alignment capped at 8 bytes                | Pending |
| GEN-0024 | NaN bit-pattern preservation — no canonicalization   | Pending |
| GEN-0025 | Bare messages — structural validation only           | Pending |
| GEN-0026 | No format auto-detection — bare vs file              | Pending |
| GEN-0027 | Full serde data model — RON algebraic types          | Pending |
| GEN-0028 | Tuple struct / tuple wire equivalence                | Pending |
| GEN-0029 | Reject #[serde(default)] at compile time             | Pending |
| GEN-0030 | Zstd-only compression in v1                          | Pending |
| GEN-0031 | Rust-only — cross-language read deferred             | Pending |
| GEN-0032 | Canonical encoding contract                          | Pending |
| GEN-0033 | GenomeOrd marker trait for map keys                  | Pending |

## Cross-Domain References (Planned)

| Genome ADR | Framework ADR | Relationship |
|------------|---------------|--------------|
| GEN-0006 | CHE-0007 (Forbid Unsafe) | Illustrates |
| GEN-0002 | CHE-0022 (Schema Evolution) | Contrasts with |

| Genome ADR | Pardosa ADR | Relationship |
|------------|-------------|--------------|
| GEN-0001 | PAR-0006 (Genome as Primary) | Referenced by |
| GEN-0008 | PAR-0006 (Genome as Primary) | Referenced by |

## Reference Documents

- [genome.md](../../genome.md) — genome binary format design document
