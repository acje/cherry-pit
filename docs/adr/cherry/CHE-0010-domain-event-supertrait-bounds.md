# CHE-0010. DomainEvent Supertrait Bounds

Date: 2026-04-25
Last-reviewed: 2026-04-25
Tier: A

## Status

Accepted

## Related

- References: CHE-0001, CHE-0004, CHE-0014

## Context

`DomainEvent` is the marker trait for all events in the system. Every
event type must implement it. The supertrait bounds on `DomainEvent`
constrain every event type in every cherry-pit system — they are
among the most load-bearing API decisions in the framework.

Bounds considered:

- **`Serialize + DeserializeOwned`** — events cross process
  boundaries (file storage, NATS transport, Pardosa logs). Without
  serde bounds, the `EventStore` trait cannot persist events and the
  `EventBus` trait cannot transport them.
- **`Clone`** — events fan out to multiple consumers (projections,
  policies, integrations). Without `Clone`, the infrastructure
  cannot deliver the same event to multiple handlers without shared
  ownership (`Arc`).
- **`Send + Sync + 'static`** — events cross thread boundaries in
  async runtimes. Without these, events cannot be stored in `Vec`,
  returned from async functions, or passed to spawned tasks.
- **`Debug`** — useful for logging but adds a derive requirement on
  every event type. Not included to keep the bound minimal.
- **`PartialEq`** — useful for test assertions but adds a constraint
  on every event type. Not included.

## Decision

```rust
pub trait DomainEvent:
    Serialize + DeserializeOwned + Clone + Send + Sync + 'static
{
    fn event_type(&self) -> &'static str;
}
```

Every bound is load-bearing:

| Bound | Required by |
|-------|-------------|
| `Serialize` | `EventStore::create`, `EventStore::append`, `EventBus::publish`, Pardosa logs |
| `DeserializeOwned` | `EventStore::load`, Pardosa consumer, NATS subscriber |
| `Clone` | `EventBus::publish` fan-out, `EventEnvelope` derives `Clone` |
| `Send` | Async task spawning, cross-thread event delivery |
| `Sync` | Shared references to events across threads |
| `'static` | Storage in `Vec`, `Box`, and async futures |

`event_type() -> &'static str` is a stable string discriminator used
for routing, schema registry, and deserialization dispatch. It must
never change once events of this type exist in a log.

## Consequences

- Every domain event type must derive (or implement) `Serialize`,
  `Deserialize`, and `Clone`. This is the entry cost of using
  cherry-pit — users cannot define events that are not serializable.
- `DeserializeOwned` (not `Deserialize<'de>`) means deserialization
  produces owned values. Zero-copy deserialization of borrowed data
  is not possible for events. This simplifies the type system at the
  cost of one allocation per deserialized event.
- No `Debug` bound means the framework cannot log events by default.
  Users add `#[derive(Debug)]` themselves (most do). A future ADR
  could add `Debug` to the bound if framework-level event logging
  is needed.
- No `PartialEq` bound means test assertions on events require
  either user-derived `PartialEq` or field-by-field comparison.
- The `event_type()` method creates a contract: the string must be
  stable forever. Renaming an event type without preserving the
  string breaks deserialization of historical data.
- Contrast with `Command` (CHE-0014): commands have minimal bounds
  (`Send + Sync + 'static`) because they stay in-process by default.
  Events have maximal bounds because they cross every boundary.
