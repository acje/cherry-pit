# GEN-0020. Empty Containers Always Allocate Heap Entries

Date: 2026-04-25
Last-reviewed: 2026-04-25
Tier: B
Status: Accepted

## Related

References: GEN-0001, GEN-0017, GEN-0014

## Context

Binary formats using offsets for variable-length data must define offset 0 semantics. Treating offset 0 as a sentinel ("empty") creates ambiguity with data at position 0. Treating offset 0 as data eliminates ambiguity — empty containers get normal heap entries with `len = 0`. pardosa-genome already uses `0xFFFFFFFF` as the `Option::None` sentinel (GEN-0017); adding a second sentinel doubles special-case logic in the deserializer.

## Decision

**Offset 0 always means "data at position 0."** It is never overloaded as a
sentinel. The only sentinel value in the entire format is `0xFFFFFFFF`
(`Option::None`).

Empty containers always allocate a heap entry:

- **Empty string (`""`):** Heap entry `[len:u32 = 0x00000000]`. Offset points
  to the 4-byte length prefix. Zero bytes of string data follow.
- **Empty byte slice:** Same as empty string.
- **Empty `Vec<T>`:** Heap entry `[count:u32 = 0x00000000]`. Offset points to
  the 4-byte count prefix. Zero elements follow.
- **Empty `BTreeMap<K, V>`:** Same as empty Vec — `[count:u32 = 0x00000000]`.

This means every string, byte slice, vec, and map — regardless of length —
produces a valid heap entry with a forward-pointing offset. The deserializer has
no special case for empty containers; it reads the length/count prefix and
proceeds with zero iterations.

**Edge case — `Option<()>` with `Some(())`:** The offset points to a valid
position in the buffer, but zero bytes are read (unit type has 0 inline size).
The offset may point to `buf.len()` — a 0-byte read at the end of the buffer
is valid.

R1 [5]: Offset 0 always means data at position 0 and is never
  overloaded as a sentinel
R2 [5]: The only sentinel value in the entire format is 0xFFFFFFFF
  for Option::None
R3 [5]: Empty containers always allocate a heap entry with len or
  count equal to zero

## Consequences

- **Positive:** Eliminates offset-0 ambiguity. Every non-`0xFFFFFFFF` offset is a real data pointer. Single sentinel for the entire format.
- **Positive:** Simplifies the deserializer — no special case for empty containers. All containers follow `read-offset → read-count → iterate`.
- **Negative:** Empty containers cost 4 bytes of heap (the length prefix). Negligible overhead; compression (GEN-0014) eliminates it in practice.
