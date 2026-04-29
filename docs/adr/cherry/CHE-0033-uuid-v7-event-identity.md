# CHE-0033. UUID v7 for Event Identity

Date: 2026-04-25
Last-reviewed: 2026-04-28
Tier: D
Status: Accepted

## Related

References: CHE-0001, CHE-0006, CHE-0016, CHE-0034

## Context

Every `EventEnvelope` carries an `event_id`. UUID v4 is random with no ordering. UUID v7 (RFC 9562) embeds a millisecond timestamp with 62 bits of randomness — naturally sortable by creation time. Cherry-pit needs global uniqueness without coordination and chronological ordering for debugging.

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

- Natural chronological ordering — sorting by `event_id` approximates creation-time ordering.
- Global uniqueness without coordination — safe for correlation/causation IDs across bounded contexts.
- 16 bytes on the wire; MessagePack encodes as binary, minimal overhead.
- Monotonicity within a millisecond via RFC 9562 counter-based ordering.
- Switching ID schemes requires migrating all existing event data.
