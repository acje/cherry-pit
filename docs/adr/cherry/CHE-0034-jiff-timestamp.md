# CHE-0034. jiff::Timestamp as Temporal Foundation

Date: 2026-04-25
Last-reviewed: 2026-04-25
Tier: D
Status: Accepted

## Related

References: CHE-0001, CHE-0016

## Context

`EventEnvelope` stores a `timestamp` field — the instant an event was
created. This type appears in every persisted event, every projection
apply, every policy reaction. The choice of temporal library and type
is load-bearing: it affects serialization format, precision, timezone
handling, and arithmetic correctness.

Candidates:

1. **`chrono::DateTime<Utc>`** — the historical Rust standard. Large
   API surface. Requires `chrono-tz` for IANA timezone support. Some
   arithmetic operations are fallible in surprising ways (DST
   transitions).
2. **`time::OffsetDateTime`** — lightweight alternative. Good serde
   support. Less mature ecosystem for timezone arithmetic.
3. **`std::time::SystemTime`** — no serde support. No human-readable
   formatting. No timezone awareness.
4. **`jiff::Timestamp`** — modern Rust datetime library by BurntSushi
   (author of `regex`, `ripgrep`). `Timestamp` is a UTC instant.
   Lossless RFC 9557/RFC 3339 serde roundtrips. DST-safe arithmetic
   by default. Built-in IANA timezone support without a separate
   crate.

## Decision

All temporal values use `jiff::Timestamp`, providing UTC-instant semantics
with lossless RFC 9557/RFC 3339 serde roundtrips, DST-safe arithmetic, and
built-in IANA timezone support without a separate crate.

R1 [10]: Use jiff::Timestamp for all temporal values in the framework
R2 [10]: Call Timestamp::now() once per batch so all events in an
  atomic batch share the same timestamp

```rust
// EventEnvelope field
pub timestamp: jiff::Timestamp,

// Generated in MsgpackFileStore::build_envelopes
let timestamp = jiff::Timestamp::now();
```

Workspace dependency:

```toml
jiff = { version = "0.2", features = ["serde"] }
```

## Consequences

- **Lossless serde roundtrips** — serializes to RFC 9557/RFC 3339
  with full precision. No information loss across JSON, MessagePack,
  or any serde-compatible format.
- **DST-safe arithmetic** — jiff accounts for daylight saving time
  transitions by default, giving correct results for time-interval
  computations without manual handling.
- **UTC instants only** — `Timestamp` records absolute time. Timezone
  conversion is a presentation concern, not a persistence concern.
- **Single timestamp per batch** — `build_envelopes` calls
  `Timestamp::now()` once; all events in an atomic batch share the
  same timestamp (CHE-0036).
- **Migration cost** — `jiff::Timestamp` is embedded in every
  serialized `EventEnvelope`. Switching libraries would require
  migrating all persisted events.
- **Version coupling** — jiff 0.2 is pre-1.0. A golden-file
  regression test (CHE-0038) compares a deterministic envelope
  against a committed fixture, catching serde format changes before
  incompatible data is written.
