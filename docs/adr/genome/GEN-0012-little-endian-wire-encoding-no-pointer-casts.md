# GEN-0012. Little-Endian Wire Encoding — No Pointer Casts

Date: 2026-04-25
Last-reviewed: 2026-04-25
Tier: B
Status: Accepted

## Related

References: GEN-0001, GEN-0006, GEN-0007, GEN-0024

## Context

Binary formats must choose an endianness convention and a byte-reading strategy.
The choice affects every scalar read/write operation across every platform.

Two approaches exist for reading multi-byte values from buffers:

1. **Pointer casting** (`*(buf.as_ptr() as *const u32)`): requires aligned buffers,
   produces UB on misaligned access (some architectures), requires `unsafe`.
2. **Byte copying** (`u32::from_le_bytes([buf[0], buf[1], buf[2], buf[3]])`): works
   on any buffer alignment, no `unsafe`, compiler optimizes to single load on LE
   targets.

FlatBuffers uses pointer casting with alignment requirements. rkyv uses pointer
casting with `unsafe`. pardosa-genome uses `#![forbid(unsafe_code)]` (GEN-0006).

GEN-0007 defines the offset-based binary layout. This ADR defines the wire encoding
contract: how multi-byte values are read and written.

## Decision

All multi-byte fields in pardosa-genome are little-endian. All scalar reads use
`from_le_bytes` on byte slices. No pointer casts. No alignment requirements on
the input buffer.

Specifically:

- **Integers** (`u16`–`u128`, `i16`–`i128`): LE via `from_le_bytes`.
- **Floats** (`f32`, `f64`): LE IEEE 754 via `from_le_bytes`. Exact NaN bit
  patterns preserved (no canonicalization — see GEN-0024).
- **Char**: 4-byte LE `u32` via `from_le_bytes`, validated as Unicode scalar.
- **Offsets**: 4-byte LE `u32` via `from_le_bytes`.
- **Header fields**: All LE (format version, schema hash, flags, sizes).

The input buffer (`&[u8]`) has no alignment requirements. Safe to use with:
- Unaligned `mmap` mappings
- Network receive buffers
- Arbitrary byte slices from any source

On big-endian targets, `from_le_bytes` performs a byte swap per read. This is
the explicit tradeoff: universal safety over big-endian performance.

R1 [5]: All multi-byte fields use little-endian encoding on the wire
  regardless of host architecture
R2 [5]: All scalar reads use from_le_bytes on byte slices — no pointer
  casts and no alignment requirements on the input buffer
R3 [6]: Floats use LE IEEE 754 via from_le_bytes with exact NaN bit
  patterns preserved

## Consequences

- **Positive:** Zero `unsafe` code in the entire read path. Compatible with
  `#![forbid(unsafe_code)]`.
- **Positive:** No alignment requirements — any `&[u8]` is a valid input,
  regardless of its memory address.
- **Positive:** On x86/ARM64 (LE targets), the compiler optimizes `from_le_bytes`
  to a single unaligned load instruction — zero overhead.
- **Negative:** Big-endian targets pay a per-read byte-swap cost. Acceptable:
  pardosa-genome's primary targets (x86-64, AArch64) are little-endian.
- **Negative:** Cannot use SIMD-aligned bulk reads without an alignment copy step.
