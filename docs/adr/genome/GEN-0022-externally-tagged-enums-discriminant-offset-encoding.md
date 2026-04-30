# GEN-0022. Externally Tagged Enums — Discriminant Offset Encoding

Date: 2026-04-25
Last-reviewed: 2026-04-25
Tier: B
Status: Accepted

## Related

References: GEN-0007

## Context

Enum encoding determines inline size, heap layout, and supported serde representations. GEN-0004 rejects incompatible serde attributes; GEN-0007 lists the layout as `[discriminant:u32][offset:u32]` without documenting the rationale. Four strategies were evaluated: varint discriminant (breaks fixed-layout), tag+length (complicates random access), union table with vtable (rejected by GEN-0002), and fixed `[discriminant:u32][offset:u32]` (constant 8-byte inline, deterministic, simple).

## Decision

All enums use a fixed 8-byte inline encoding:

```
[discriminant:u32 LE][offset:u32 LE]
```

- **Discriminant:** 0-indexed variant position in the Rust enum declaration
  order. Validated on deserialization — unknown discriminants produce
  `DeError::InvalidDiscriminant`.
- **Offset (data variants):** Points to the variant's payload in the heap
  region. Payload layout is the variant's field layout (struct variant: fields
  in declaration order; tuple variant: elements inline; newtype variant:
  inner type).
- **Offset (unit variants):** Must be `0x00000000`. Treated as padding — the
  backward offset check does not apply. Non-zero offset on a unit variant
  produces `DeError::NonZeroPadding` (GEN-0018).

Only serde's **externally tagged** representation (the default, no attribute)
is supported. Internally tagged (`#[serde(tag)]`), adjacently tagged
(`#[serde(tag, content)]`), and untagged (`#[serde(untagged)]`) are rejected
at compile time by `#[derive(GenomeSafe)]` (GEN-0004).

R1 [5]: All enums use a fixed 8-byte inline encoding of discriminant-u32
  plus offset-u32
R2 [5]: Discriminant is the 0-indexed variant position in Rust enum
  declaration order validated on deserialization
R3 [5]: Unit variant offset must be 0x00000000 treated as padding
R4 [5]: Only serde externally tagged representation is supported —
  internally tagged, adjacently tagged, and untagged are rejected

## Consequences

- **Positive:** Fixed 8-byte inline size per enum — struct layout fully determined at compile time.
- **Positive:** Simple read path: read discriminant, check bounds, read offset, jump to heap.
- **Negative:** Unit variants waste 4 bytes (offset is padding). Many-variant enums pay proportionally.
- **Negative:** Only externally tagged enums supported; users of `#[serde(tag)]` must restructure.
- **Negative:** Inserting enum variants mid-declaration changes discriminants, breaking existing data. Append-only addition preserves compatibility.
