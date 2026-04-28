# CHE-0034. jiff::Timestamp as Temporal Foundation

Date: 2026-04-25
Last-reviewed: 2026-04-25
Tier: D
Status: Accepted

## Related

References: CHE-0001, CHE-0016

## Context

`EventEnvelope` stores a `timestamp` field appearing in every persisted event, projection, and policy reaction. `chrono` has surprising fallible arithmetic at DST transitions. `time` has a less mature timezone ecosystem. `std::time::SystemTime` lacks serde and formatting. `jiff::Timestamp` (by BurntSushi) provides UTC-instant semantics, lossless RFC 9557/RFC 3339 serde roundtrips, DST-safe arithmetic, and built-in IANA timezone support without a separate crate.

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

- Lossless serde roundtrips via RFC 9557/RFC 3339 with full precision.
- DST-safe arithmetic by default.
- UTC instants only — timezone conversion is a presentation concern, not a persistence concern.
- Single timestamp per batch — `build_envelopes` calls `Timestamp::now()` once (CHE-0036).
- `jiff::Timestamp` is embedded in every serialized envelope. Switching libraries requires migrating all persisted events.
- jiff 0.2 is pre-1.0. A golden-file regression test (CHE-0038) catches serde format changes before incompatible data is written.
- In distributed deployments, clock skew between nodes makes timestamps unreliable for event ordering — sequence numbers (not timestamps) determine causal order within an aggregate stream.
