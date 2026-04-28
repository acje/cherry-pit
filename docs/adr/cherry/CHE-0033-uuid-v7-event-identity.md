# CHE-0033. UUID v7 for Event Identity

Date: 2026-04-25
Last-reviewed: 2026-04-25
Tier: D
Status: Accepted

## Related

References: CHE-0001, CHE-0006, CHE-0016, CHE-0034

## Context

Every `EventEnvelope` carries an `event_id` — a globally unique
identifier for the event instance. The choice of ID scheme affects
ordering, uniqueness guarantees, timestamp derivability, storage
efficiency, and cross-system correlation.

Candidates:

1. **UUID v4** — random, 122 bits of entropy. No ordering. No
   embedded timestamp. Widely supported.
2. **UUID v7** — time-ordered (RFC 9562). Millisecond-precision
   Unix timestamp in the high bits, 62 bits of randomness in the
   low bits. Naturally sortable by creation time.
3. **ULID** — similar to UUID v7 (timestamp + randomness) but uses
   a non-standard encoding (Crockford Base32). Not a UUID — requires
   separate type or conversion.
4. **Auto-increment u64** — simple, compact, ordered. Not globally
   unique across aggregate types or processes. Already used for
   `sequence` within a stream; using it for event identity too would
   conflate two distinct concepts.

## Decision

`event_id` is a UUID v7, generated via `uuid::Uuid::now_v7()`.

R1 [10]: Generate event_id as UUID v7 via uuid::Uuid::now_v7()
R2 [10]: UUID v7 IDs are globally unique across all aggregate types
  and processes without coordination

```rust
// EventEnvelope field
pub event_id: uuid::Uuid,

// Generated in MsgpackFileStore::build_envelopes
event_id: uuid::Uuid::now_v7(),
```

Workspace dependency:
```toml
uuid = { version = "1", features = ["v7", "serde"] }
```

## Consequences

- **Natural chronological ordering** — sorting by `event_id`
  approximates creation-time ordering, useful for debugging and
  cross-aggregate correlation without timestamp comparison.
- **Embedded timestamp** — the high 48 bits contain millisecond Unix
  time, providing a secondary timestamp independent of the
  `timestamp` field for forensic analysis.
- **Global uniqueness** — unique across all aggregate types,
  processes, and deployments without coordination. Safe for use as
  correlation/causation IDs across bounded contexts.
- **16 bytes on the wire** — larger than u64 (8 bytes) but smaller
  than string representation (36 bytes). MessagePack encodes UUIDs
  as binary, so overhead is minimal.
- **Monotonicity within a millisecond** — the `uuid` crate's
  `now_v7()` uses RFC 9562 Section 6.2 counter-based ordering.
  Events in the same batch (CHE-0034) get strictly ordered IDs.
- **No coordination required** — consistent with the single-writer
  assumption (CHE-0006).
- **Migration constraint** — switching ID schemes would require
  migrating existing event data. The choice is permanent for
  persisted events.
