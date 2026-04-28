# GEN-0034. Structured Fuzzing and Property-Based Testing for Codec Integrity

Date: 2026-04-28
Last-reviewed: 2026-04-28
Tier: B
Status: Proposed

## Related

References: GEN-0006, GEN-0011, GEN-0013, GEN-0032

## Context

Every major binary format has been fuzzed: Protobuf (OSS-Fuzz),
FlatBuffers (AFL), Cap'n Proto (Sandstorm). Genome's `forbid(unsafe_code)`
eliminates memory corruption but not logic bugs: incorrect offset
arithmetic, integer overflow, panics on crafted input. GEN-0011 catalogs
20 checks; GEN-0032 provides verify_roundtrip. Neither addresses
adversarial input. Pardosa processes events from NATS — a network
boundary requiring codec robustness.

1. **Structured fuzzing + proptest** — continuous adversarial coverage.
2. **Manual test vectors** — curated, incomplete by construction.
3. **Verification checks only** — untested against adversarial craft.

Option 1 chosen: automated fuzzing catches bugs manual testing cannot.

## Decision

Maintain continuous fuzz targets and property-based roundtrip tests
for both decode paths (bare message and file format).

R1 [5]: Run cargo-fuzz targets against decode_bare and decode_file
  with arbitrary byte slices, targeting zero panics on any input
R2 [5]: Use proptest to verify roundtrip identity: for all T where
  T implements GenomeSafe, decode(encode(t)) equals t
R3 [6]: Fuzz targets cover all PageClass configurations from
  GEN-0013 to exercise resource limit enforcement under adversarial
  input

## Consequences

- **Catches logic bugs unreachable by manual testing.** Offset
  arithmetic errors, overflow conditions, and edge-case panics.
- **CI cost.** Fuzz runs are time-bounded (e.g., 5 minutes per
  target per CI run). Longer runs scheduled periodically.
- **Corpus maintenance.** Fuzz corpus grows over time and should
  be committed for regression coverage.
- **Complements GEN-0011.** Verification checks are the specification;
  fuzzing validates the implementation matches the specification.
