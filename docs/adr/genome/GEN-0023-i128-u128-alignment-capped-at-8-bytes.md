# GEN-0023. i128/u128 Alignment Capped at 8 Bytes

Date: 2026-04-25
Last-reviewed: 2026-04-25
Tier: D
Status: Accepted

## Related

References: GEN-0001, GEN-0007

## Context

Every type in pardosa-genome has a fixed natural alignment determining padding insertion. For most types alignment matches byte width (u32→4, u64→8). For 128-bit types, 16-byte alignment maximizes SIMD friendliness but causes up to 15 bytes worst-case padding. 8-byte alignment caps padding at 7 bytes and matches Rust's actual i128/u128 alignment on x86-64. A wire format must choose a single cross-platform alignment.

## Decision

i128 and u128 align to **8 bytes** in the wire format, not 16.

This is consistent with the maximum alignment of any other supported type
(u64/i64/f64 all align to 8). It prevents excessive padding on 32-bit targets
where the next-lower-aligned field would leave up to 15 bytes of zeros.

The encoding is 16 bytes LE, written at an 8-byte-aligned cursor position.

R1 [9]: i128 and u128 align to 8 bytes in the wire format not 16
R2 [9]: The encoding is 16 bytes LE written at an 8-byte-aligned
  cursor position
R3 [9]: Maximum alignment of any supported type is 8 bytes across all
  platforms

## Consequences

- **Positive:** Consistent maximum alignment (8 bytes) across all types.
  Struct padding is predictable without special-casing 128-bit types.
- **Positive:** Reduced waste on 32-bit targets. A struct with `u8` followed
  by `u128` wastes 7 bytes of padding (not 15).
- **Positive:** Cross-platform consistency — the same alignment on all targets.
- **Negative:** SIMD consumers on platforms with 128-bit registers (AVX, NEON)
  must copy i128/u128 values to aligned storage before SIMD operations.
  Marginal cost given that pardosa-genome uses `from_le_bytes` (not pointer
  casts) for all reads.
- **Negative:** Frozen wire format decision. Cannot change to 16-byte alignment
  without breaking all existing data.
