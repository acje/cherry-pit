# CHE-0031. MessagePack with Named Encoding for Persistence

Date: 2026-04-25
Last-reviewed: 2026-04-25
Tier: D
Status: Accepted

## Related

- References: CHE-0045

## Context

`MsgpackFileStore` persists aggregate event streams to disk. The
serialization format must balance compactness, performance, and
forward compatibility (adding new fields to `EventEnvelope` without
breaking existing data).

Formats considered:

1. **JSON** — human-readable, widely supported. Verbose (string keys
   repeated per object, whitespace). Slower to parse than binary
   formats. Good schema evolution via `#[serde(default)]`.
2. **Bincode** — compact binary, fastest serialization. Positional
   encoding — adding or reordering fields breaks deserialization of
   existing data. No schema evolution without versioning.
3. **MessagePack (positional/array)** — compact binary. Same problem
   as bincode: field order matters.
4. **MessagePack (named/map)** — binary format with string keys.
   Slightly larger than positional msgpack. Supports
   `#[serde(default)]` for new fields — existing data deserializes
   with defaults for absent keys.

## Decision

`MsgpackFileStore` uses `rmp_serde::encode::to_vec_named` (map
encoding with string keys) for all writes. Deserialization uses
`rmp_serde::from_slice`.

This was validated when `correlation_id` and `causation_id` were added
to `EventEnvelope` with `#[serde(default)]` — existing data without
these fields deserializes with `None` values. A dedicated test
(`deserializes_old_format_without_correlation_fields`) proves this.

## Consequences

- New `Option` fields on `EventEnvelope` with `#[serde(default)]` can
  be added without migrating existing data files. This is the primary
  reason for choosing named encoding over positional.
- Wire size is larger than positional msgpack (field names are stored
  as strings). For a development/small-deployment store, this tradeoff
  is acceptable.
- The format is implementation-specific to `MsgpackFileStore`, not a
  trait-level requirement. Other `EventStore` implementations (e.g.,
  PostgreSQL-backed) can use different formats.
- Switching away from msgpack requires a migration tool that reads old
  format and writes new format — the store cannot hot-swap formats.
- The entire aggregate history is stored as a single msgpack value
  (`Vec<EventEnvelope<E>>`), not as a stream of individual records.
  This simplifies atomic writes but means large aggregates load the
  full history into memory.
