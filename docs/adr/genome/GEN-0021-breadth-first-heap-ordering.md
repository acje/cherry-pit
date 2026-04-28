# GEN-0021. Breadth-First Heap Ordering

Date: 2026-04-25
Last-reviewed: 2026-04-25
Tier: B
Status: Accepted

## Related

References: GEN-0001, GEN-0005, GEN-0011

## Context

pardosa-genome uses a two-region binary layout: an inline region (struct fields,
scalars, enum discriminants) followed by a heap region (variable-length data:
strings, vecs, maps, option payloads, enum variant data). GEN-0005 establishes
the two-pass serialization architecture and mentions breadth-first heap ordering
as a consequence. However, the heap ordering is a frozen wire format property
that determines binary equality — two conforming serializers must produce
identical bytes for the same value.

Three heap ordering strategies were considered:

- **Depth-first:** Write each heap item immediately when encountered. Simple to
  implement but produces interleaved inline/heap boundaries and backward
  offset references.
- **Breadth-first:** Write all items directly referenced by the inline region
  first, then items referenced by those heap items, and so on. Forward-pointing
  offsets guaranteed by construction.
- **Arbitrary/unspecified:** Let implementations choose. Breaks binary equality
  and content-addressable storage.

## Decision

Heap items are written in **breadth-first order**: all items directly referenced
by the inline region are written first, then items referenced by those heap
items, then items referenced by those items, and so on. Each level processes
nodes that are strictly closer to leaves than the previous level.

This ordering is a **frozen wire format property**. Changing it invalidates all
existing genome files and bare messages. Golden test vectors pin the exact byte
output for multi-level heap structures.

**Termination guarantee:** The type tree is finite by construction. Leaf nodes
(scalars, strings, bytes) produce no further heap items. Each breadth-first
level is strictly smaller than the previous, reaching an empty level in bounded
iterations.

R1 [5]: Heap items are written in breadth-first order — all items
  directly referenced by the inline region first, then items referenced
  by those heap items, and so on
R2 [5]: This ordering is a frozen wire format property — changing it
  invalidates all existing genome files and bare messages
R3 [6]: Golden test vectors pin the exact byte output for multi-level
  heap structures

## Consequences

- **Positive:** All offsets are forward-pointing into the heap region by
  construction. The backward offset check (GEN-0011, check #6) is trivially
  satisfied for correctly serialized data.
- **Positive:** Binary equality — identical values always produce identical
  bytes, enabling content-addressable storage, checksumming, and deduplication.
- **Positive:** Deterministic byte output enables `verify_roundtrip` to compare
  serialized bytes directly, not just deserialized values.
- **Negative:** Write path complexity. The `WritingSerializer` must track heap
  items by level and write them after the inline pass. More complex than
  depth-first (which writes heap items inline during traversal).
- **Negative:** Frozen forever. Any future serializer must produce the same
  ordering. Second implementations (other languages, `genome-dump` re-serializer)
  must replicate this exactly.
