# GEN-0001. Serde-Native Serialization with GenomeSafe Marker Trait

Date: 2026-04-25
Last-reviewed: 2026-04-27
Tier: S
Status: Accepted

## Related

Root: GEN-0001

## Context

pardosa-genome must integrate with an existing Rust codebase where
every data type already derives `Serialize` and `Deserialize`. Two
forces are in tension: zero-copy read performance (the event replay
hot path deserializes millions of events — per-field allocating
formats impose a throughput ceiling) and adoption friction
(introducing a parallel derive ecosystem like rkyv or FlatBuffers
doubles maintenance surface with mirror types throughout the
codebase).

rkyv achieves full struct-level zero-copy but requires an
`ArchivedFoo` for every `Foo`. FlatBuffers requires external `.fbs`
schema files and code generation. Both sever the serde ecosystem —
types cannot be used with JSON, TOML, or any other serde format
without dual serialization implementations. A serde-native custom
binary format avoids mirror types entirely while achieving partial
zero-copy (str, bytes). PAR-0006 contains the detailed per-library
alternatives analysis.

## Decision

Use standard `serde::Serializer` / `serde::Deserializer<'de>` as the serialization
interface. Introduce a separate `GenomeSafe` marker trait with no methods — only two
associated constants (`SCHEMA_HASH: u64`, `SCHEMA_SOURCE: &'static str`). A
`#[derive(GenomeSafe)]` proc-macro generates both constants and performs compile-time
validation.

Types only need `#[derive(Serialize, Deserialize, GenomeSafe)]`. No mirror types,
no external schema files, no code generation beyond these three derives.

R1 [2]: All serializable types use serde Serialize and Deserialize as
  the sole serialization interface — no parallel trait hierarchies
R2 [2]: GenomeSafe is a marker trait with no methods, carrying only
  SCHEMA_HASH and SCHEMA_SOURCE associated constants
R3 [2]: No mirror types, no external schema files, no code generation
  beyond derive(Serialize, Deserialize, GenomeSafe)

## Consequences

- **Positive:** Near-zero adoption friction for existing serde users. Any type that
  already derives `Serialize + Deserialize` only needs one additional derive.
- **Positive:** Full access to serde's ecosystem (custom serializers, derives, testing).
- **Negative:** Serde's data model constrains format design. Some serde attributes
  (`flatten`, `tag`, `untagged`, `skip_serializing_if`) are incompatible with
  fixed-layout binary serialization — must be rejected at compile time.
- **Negative:** Manual `Serialize` impls can bypass `GenomeSafe` validation. Runtime
  detection and `verify_roundtrip` CI checks are defense-in-depth.
- **Risk:** Serde major version changes could require format migration. Mitigated by
  serde's strong backward compatibility record.
