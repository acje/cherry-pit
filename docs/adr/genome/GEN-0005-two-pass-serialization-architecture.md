# GEN-0005. Two-Pass Serialization Architecture

Date: 2026-04-25
Last-reviewed: 2026-04-28
Tier: B
Status: Accepted

## Related

- References: GEN-0001

## Context

Fixed-layout formats need to write inline scalar data and variable-length
heap data (strings, vecs, maps, option payloads) into a single contiguous
buffer. Three serialization strategies were evaluated:

1. **Intermediate AST.** Build a tree of nodes representing the
   serialized structure, then flatten to bytes. Memory cost: O(n)
   proportional to the data. Used by JSON serializers. Simple
   implementation but high peak memory.

2. **Back-patching (FlatBuffers approach).** Write data in-order,
   patching offsets retroactively as later data resolves positions.
   Single pass, but complex cursor management and mutable offset
   fixups. Requires mutable offsets — incompatible with streaming
   writes.

3. **Two-pass (sizing then writing).** First pass computes exact
   buffer size. Second pass writes with zero reallocation. Memory
   cost: O(1) beyond the output buffer. Requires the input to be
   traversed twice — safe for immutable serde `Serialize` impls.

| Strategy | Passes | Peak memory | Complexity | Streaming |
|----------|--------|-------------|------------|-----------|
| AST | 1 | O(n) | Low | No |
| Back-patching | 1 | O(1) | High | No |
| Two-pass | 2 | O(1) | Medium | No |

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
