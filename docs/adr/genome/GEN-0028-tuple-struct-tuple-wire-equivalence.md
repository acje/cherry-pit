# GEN-0028. Tuple Struct / Tuple Wire Equivalence

Date: 2026-04-25
Last-reviewed: 2026-04-25
Tier: D
Status: Accepted

## Related

References: GEN-0001, GEN-0003, GEN-0011

## Context

Serde treats tuple structs (e.g., `struct Point(f64, f64)`) and plain tuples
(e.g., `(f64, f64)`) identically at the data model level — both call
`serialize_tuple` with the same element count. This means the serialized
bytes for the inner data are identical regardless of whether the outer type
is a named tuple struct or an anonymous tuple.

pardosa-genome's schema hash (GEN-0003) includes the root type name in the
hash input (`"struct:Point"` vs `"tuple2"`), so the schema hashes differ.
But the wire bytes after the header (where the schema hash lives) are the
same.

## Decision

Tuple structs and plain tuples produce **identical wire bytes** for their
data payload. They are distinguished **only by schema hash**.

- `struct Point(f64, f64)` and `(f64, f64)` serialize to the same data bytes.
- Their schema hashes differ: `hash("struct:Point") ⊕ ...` vs
  `hash("tuple2") ⊕ ...`.
- In bare messages, the schema hash at bytes 2–9 catches substitution.
- In file format, the file header schema hash catches substitution.
- If schema verification is bypassed (future `decode_unchecked` — currently
  does not exist), deserializing one from the other's bytes succeeds silently.

This is **intentional**. The format does not store type names in the binary
payload — only the schema hash provides type identity. This matches serde's
own data model equivalence between tuple structs and tuples.

R1 [9]: Tuple structs and plain tuples produce identical wire bytes
  for their data payload
R2 [9]: They are distinguished only by schema hash — the format does
  not store type names in the binary payload
R3 [9]: Schema hash is the sole mechanism providing type identity for
  structurally equivalent types

## Consequences

- **Positive:** Consistent with serde's data model — no special-case logic needed.
- **Positive:** Schema hash catches accidental type substitution in normal operation (GEN-0011).
- **Negative:** Structurally identical types (`Meters(f64)` vs `Seconds(f64)`) produce identical payload bytes. Schema hash is the only defense; a future `decode_unchecked` bypassing the hash would allow silent type confusion.
- **Negative:** Users expecting structural typing must understand pardosa-genome uses nominal typing via schema hash.
