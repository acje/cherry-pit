# PAR-0002. Index::NONE Sentinel Replacing Option\<Index\>

Date: 2026-04-25
Last-reviewed: 2026-04-25
Tier: D

## Status

Accepted

## Related

- References: GEN-0002, GEN-0007

## Context

The original design used `Option<Index>` for event precursors — `None` for the
first event in a fiber, `Some(Index)` for subsequent events. In genome's
fixed-layout binary format, `Option<T>` requires a 4-byte offset into the heap
region plus the heap entry itself. Every event pays this cost for the precursor
field, even though the overwhelming majority of events have a valid precursor.

## Decision

Replace `Option<Index>` with a sentinel value:

```rust
impl Index {
    pub const NONE: Index = Index(u64::MAX);
}
```

`u64::MAX` is permanently reserved. A line with `u64::MAX` events would require
~147 exabytes of storage, so this value can never be a valid line position.

Guards:
- `Index::new(v)` panics if `v == u64::MAX` — prevents accidental sentinel
  construction from application code.
- `Index::checked_next()` caps at `u64::MAX - 1` — no valid index arithmetic
  can produce the sentinel.
- `is_none()` / `is_some()` methods for ergonomic checking.

## Consequences

- **Positive:** Saves 4 bytes inline + heap indirection per event in genome
  encoding. For a line with 1M events, this is ~4 MiB saved.
- **Positive:** `Index` is `Copy` — no heap allocation, no option branching
  on the read path.
- **Positive:** Sentinel semantics are explicit in the type's API (`is_none()`,
  `NONE` constant).
- **Negative:** Sentinel-based APIs are less type-safe than `Option`. A caller
  could forget to check `is_none()` before using the value.
- **Negative:** `u64::MAX` is permanently consumed from the value space. No
  runtime impact given physical constraints.
- **Cross-crate:** Genome's wire format must not assign structural meaning to
  `u64::MAX` for `Index`-typed fields. See
  [GEN-0002](../genome/GEN-0002-no-schema-evolution-fixed-layout.md)
  and
  [GEN-0007](../genome/GEN-0007-flatbuffers-style-offset-based-binary-layout.md).
