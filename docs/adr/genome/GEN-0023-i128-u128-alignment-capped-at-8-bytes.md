# GEN-0023. i128/u128 Alignment Capped at 8 Bytes

Date: 2026-04-25
Last-reviewed: 2026-04-28
Tier: D
Status: Accepted

## Related

References: GEN-0007

## Context

Every type in pardosa-genome has a fixed natural alignment determining padding. For most types alignment matches byte width. For 128-bit types, 16-byte alignment maximizes SIMD friendliness but causes up to 15 bytes padding. 8-byte alignment caps padding at 7 bytes and matches Rust's actual i128/u128 alignment on x86-64.

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

- Consistent maximum alignment (8 bytes) across all types. Padding predictable without special-casing.
- Reduced waste: `u8` followed by `u128` wastes 7 bytes (not 15). Cross-platform consistency.
- SIMD consumers must copy i128/u128 to aligned storage before SIMD ops. Marginal cost given `from_le_bytes` usage.
- Frozen wire format — cannot change to 16-byte alignment without breaking existing data.
