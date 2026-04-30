# GEN-0007. FlatBuffers-Style Offset-Based Binary Layout

Date: 2026-04-25
Last-reviewed: 2026-04-28
Tier: S
Status: Accepted

## Related

References: GEN-0001

## Context

Binary formats use one of three layout strategies: tag-length-value (protobuf,
postcard), vtable-based (FlatBuffers), or fixed inline + heap offset (Cap'n Proto).
pardosa-genome needs a layout that supports both zero-copy reads and fixed-layout
determinism without vtables (no schema evolution).

## Decision

Use a two-region layout: **inline region** (struct fields, scalars, enum discriminants)
and **heap region** (variable-length data: strings, vecs, maps, option payloads, enum
variant data). Inline fields reference heap data via 4-byte LE offsets (absolute
positions within the message buffer).

Key layout rules:
- Scalars: inline at natural alignment, LE encoding
- Strings/bytes: 4B offset inline → heap `[len:u32][data]`
- Option: 4B offset inline → heap inner value; `0xFFFFFFFF` = None sentinel
- Vec/Map: 4B offset inline → heap `[count:u32][elements]`
- Enum: `[discriminant:u32][offset:u32]` inline → heap variant data (unit: offset=0)
- Struct: fields inline in declaration order
- Tuple: elements inline with alignment
- Newtype: transparent (inner type's layout)
- Unit: 0 bytes

R1 [2]: Use a two-region layout with inline region for scalars and heap
  region for variable-length data referenced by 4-byte LE offsets
R2 [3]: Scalars are inline at natural alignment with LE encoding
R3 [3]: Strings and bytes use 4-byte offset inline pointing to heap
  len-prefixed data
R4 [3]: Enums use discriminant-u32 plus offset-u32 inline pointing to
  heap variant data
R5 [3]: All u32 offsets are absolute byte positions measured from the
  start of the message buffer data region

## Consequences

- **Positive:** O(1) field access for scalars — no scanning, no vtable lookup.
- **Positive:** 4-byte offsets keep inline size small while supporting messages up to
  4 GiB.
- **Positive:** LE encoding + `from_le_bytes` works on any platform without alignment.
- **Negative:** Message size capped at `u32::MAX` (~4 GiB). Split across multiple
  messages for larger datasets.
- **Negative:** No random-access to individual struct fields by name (must know the
  type layout). Not a self-describing format at the field level.
