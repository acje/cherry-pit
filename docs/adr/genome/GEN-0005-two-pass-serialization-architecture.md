# GEN-0005. Two-Pass Serialization Architecture

Date: 2026-04-25
Last-reviewed: 2026-04-28
Tier: B
Status: Accepted

## Related

References: GEN-0001

## Context

Fixed-layout formats must write inline scalar data and variable-length heap data into a single contiguous buffer. Three strategies were evaluated: an intermediate AST (O(n) memory, simple), back-patching à la FlatBuffers (single pass but complex mutable offset fixups, incompatible with streaming), and two-pass sizing-then-writing (O(1) memory beyond the output buffer, requires traversing input twice). The two-pass approach trades a second traversal for zero reallocation and no intermediate allocations.

## Decision

Serialization uses two `serde::Serializer` implementations that share the same
structural logic:

1. **Pass 1 — `SizingSerializer`**: Walks the type tree computing total buffer size
   (inline + heap). Cost: one serde traversal, `O(depth)` stack frames, zero heap
   allocation.
2. **Pass 2 — `WritingSerializer`**: Pre-allocates the exact buffer. Writes inline data
   from position 0, appends heap items in breadth-first order.

A sizing mismatch between passes produces `SerError::InternalSizingMismatch` in all
build profiles, with an additional `debug_assert!` in debug builds.

R1 [5]: Serialization uses two passes — SizingSerializer computes exact
  buffer size, WritingSerializer writes with zero reallocation
R2 [6]: A sizing mismatch between passes produces
  SerError::InternalSizingMismatch in all build profiles
R3 [5]: WritingSerializer pre-allocates the exact buffer and writes
  inline data from position 0 with heap items in breadth-first order

## Consequences

- **Positive:** Peak memory ≈ 1× final message size. No intermediate tree, no
  back-patching, no reallocation.
- **Positive:** Breadth-first heap ordering ensures all offsets are forward-pointing.
- **Negative:** `value.serialize()` is called twice. Safe for any correct `Serialize`
  impl (immutable observation), but types with interior mutability during serialization
  (extremely rare) could produce inconsistent sizing. The sizing mismatch check catches
  this.
- **Negative:** Write throughput is lower than a single-pass approach due to two
  traversals; not yet benchmarked. Acceptable tradeoff for read-optimized format.
