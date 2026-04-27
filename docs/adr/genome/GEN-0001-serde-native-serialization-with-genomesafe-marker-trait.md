# GEN-0001. Serde-Native Serialization with GenomeSafe Marker Trait

Date: 2026-04-25
Last-reviewed: 2026-04-27
Tier: S

## Status

Accepted

Amended 2026-04-27 — expanded Context with forces-in-tension and
  alternatives analysis

## Related

- Root: GEN-0001

## Context

pardosa-genome must integrate with an existing Rust codebase where
every data type already derives `Serialize` and `Deserialize`. Two
forces are in tension:

1. **Zero-copy read performance.** The event storage hot path
   deserializes millions of events during replay. Formats that
   allocate per-field (JSON, bincode, postcard) impose a throughput
   ceiling. Zero-copy formats (rkyv, FlatBuffers) avoid this but
   require their own trait hierarchies.

2. **Adoption friction.** Introducing a second derive ecosystem
   (rkyv's `Archive + Serialize + Deserialize`, FlatBuffers' codegen)
   doubles the maintenance surface. Every type change requires
   updates to both the serde representation and the zero-copy
   representation. Mirror types spread through the entire codebase.

Three approaches were evaluated:

| Approach | Zero-copy | Serde compat | Mirror types | Schema hash |
|----------|-----------|-------------|--------------|-------------|
| serde + custom binary format | Partial (str, bytes) | Full | None | Custom |
| rkyv | Full (struct-level) | None | Required | None |
| FlatBuffers + codegen | Full | None | Required | External |

rkyv achieves full struct-level zero-copy but at the cost of a
parallel type hierarchy (`ArchivedFoo` for every `Foo`) that spreads
through the entire codebase. FlatBuffers requires external `.fbs`
schema files and code generation. Both sever the serde ecosystem
connection — types cannot be used with JSON, TOML, or any other serde
format without maintaining two serialization implementations.
PAR-0006 contains the detailed per-library alternatives analysis that
informed this decision.

## Decision

Use standard `serde::Serializer` / `serde::Deserializer<'de>` as the serialization
interface. Introduce a separate `GenomeSafe` marker trait with no methods — only two
associated constants (`SCHEMA_HASH: u64`, `SCHEMA_SOURCE: &'static str`). A
`#[derive(GenomeSafe)]` proc-macro generates both constants and performs compile-time
validation.

Types only need `#[derive(Serialize, Deserialize, GenomeSafe)]`. No mirror types,
no external schema files, no code generation beyond these three derives.

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
