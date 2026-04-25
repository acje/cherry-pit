# CHE-0034. jiff::Timestamp as Temporal Foundation

Date: 2026-04-25
Last-reviewed: 2026-04-25
Tier: D

## Status

Accepted

## Related

- Informs: CHE-0033
- Referenced by: CHE-0033, CHE-0042

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

All temporal values use `jiff::Timestamp`.

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

- **Lossless serde roundtrips** — `jiff::Timestamp` serializes to
  RFC 9557 / RFC 3339 format with full precision. No information
  loss across JSON, MessagePack, or any serde-compatible format.
  Historical events deserialize to the exact same instant they were
  created with.
- **DST-safe arithmetic** — jiff's arithmetic operations account for
  daylight saving time transitions by default. Policies and
  projections that compute time intervals (e.g., "events in the last
  24 hours") get correct results without manual DST handling.
- **UTC instants only** — `Timestamp` is a UTC instant, not a
  datetime with timezone. Events record *when* something happened in
  absolute time. Display formatting with timezone conversion is a
  presentation concern, not a persistence concern.
- **Single timestamp per batch** — `build_envelopes` calls
  `Timestamp::now()` once per batch. All events in a create or
  append batch share the same timestamp, reflecting that the batch
  is atomic (CHE-0036).
- **Library maturity** — jiff is authored by BurntSushi, whose
  libraries (`regex`, `ripgrep`, `memchr`) are known for correctness
  and performance. The library is actively maintained.
- **Migration cost** — `jiff::Timestamp` is embedded in every
  serialized `EventEnvelope`. Switching to `chrono` or `time` would
  require migrating all persisted events. The choice is permanent
  for any data already written.
- **Version coupling** — jiff 0.2 is a pre-1.0 library. Breaking
  changes in jiff's serde format would require careful handling.
  The `serde` feature's stability is the critical dependency, not
  the arithmetic API.
- **Serde stability mitigation** — a golden-file regression test
  (CHE-0038) serializes a deterministic `EventEnvelope` with a
  fixed `jiff::Timestamp` and compares against a committed fixture.
  Bumping jiff's version will trigger a test failure if the
  serialized format changes, providing an early warning before any
  data is written with an incompatible format. No version pinning
  is required — the test catches the breakage.
