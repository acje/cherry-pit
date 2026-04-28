# GEN-0030. Zstd-Only Compression in v1

Date: 2026-04-25
Last-reviewed: 2026-04-25
Tier: D
Status: Accepted

## Related

References: GEN-0001, GEN-0014

## Context

pardosa-genome supports optional compression via the `compression_algo` field. GEN-0014 documents decompression bomb mitigation. Zstd dominates brotli for real-time workloads: ~4× faster decompression (~1500 vs ~400 MB/s), ~25× faster compression at comparable ratios. Brotli's marginally better compression ratio does not justify the speed penalty. The file header reserves 3 bits (7 values) for the algorithm, ensuring brotli can be added later without a format version bump.

## Decision

v1 supports **zstd only**. Brotli is deferred to a future version.

**Compression algorithm codes:**
- `0x00` — no compression
- `0x01` — zstd
- `0x02`–`0x07` — reserved (reader must reject with
  `FileError::UnsupportedCompression`)

**Default zstd level:** 3 (best speed/ratio tradeoff for real-time workloads).
Levels 1–22 are supported.

**Feature gating:** Zstd is behind the `zstd` Cargo feature flag (`default =
["std", "derive"]` — zstd is opt-in). When the `zstd` feature is not enabled,
files with `compression_algo = 0x01` produce
`FileError::CompressionNotAvailable`.

### Rationale

The primary use case is real-time network and MLOps pipelines where
decompression speed dominates. Zstd's ≈4× faster decompression and ≈25×
faster compression than brotli at comparable ratios makes it the clear choice
for this workload. Brotli's marginally better compression ratio does not
justify the decompression speed penalty for real-time consumers.

The file header reserves 3 bits for the compression algorithm (7 possible
values), ensuring brotli can be added in a future version without a format
version bump. v1 readers reject unknown algorithm codes, ensuring clean
forward compatibility.

R1 [9]: v1 supports zstd only — Brotli is deferred to a future version
R2 [9]: Zstd is behind the zstd Cargo feature flag and is opt-in —
  not included in default features
R3 [9]: Reserved algorithm codes 0x02 through 0x07 must be rejected
  with FileError::UnsupportedCompression

## Consequences

- **Positive:** Single compression dependency reduces testing surface and
  build complexity. Only one decompression code path to audit for security
  (GEN-0014).
- **Positive:** Zstd's dictionary support (reserved `dict_id` in file header)
  enables future 2–5× compression improvement on small messages without
  algorithm change.
- **Positive:** Reserved algorithm codes ensure brotli (or other algorithms)
  can be added without breaking the wire format.
- **Negative:** Cold storage workloads where write speed is unimportant
  cannot use brotli's marginally better ratios until a future version.
- **Negative:** `no_std` consumers who need decompression must use `ruzstd`
  (experimental) or implement their own decompressor.
