# CHE-0037. No Snapshot Support (Deliberate Deferral)

Date: 2026-04-25
Last-reviewed: 2026-04-25
Tier: D
Status: Accepted

## Related

References: CHE-0001, CHE-0009, CHE-0010, CHE-0040

## Context

Event-sourced systems reconstruct aggregate state by replaying all
events from the beginning of the stream. As aggregates accumulate
events, replay time grows linearly. Snapshots are the standard
mitigation: periodically persist the aggregate's materialized state,
then replay only events after the snapshot.

Cherry-pit's `EventStore` trait provides:

```rust
fn load(&self, id: AggregateId)
    -> impl Future<Output = Result<Vec<EventEnvelope<Self::Event>>, StoreError>> + Send;
```

`load` returns the **complete** event history. There is no
`load_from_snapshot`, no `SnapshotStore` trait, no snapshot types
anywhere in `cherry-pit-core` or `cherry-pit-gateway`.

The `CommandBus` describes its lifecycle as "load the aggregate from
the event store (replay via apply)" — always full replay.

Two approaches:

1. **Add snapshot support now** — define `SnapshotStore` trait,
   snapshot interval configuration, snapshot serialization. Adds
   complexity to every aggregate (must be snapshot-serializable),
   every store implementation (must manage snapshots alongside
   events), and the bus (must load snapshot + remaining events).
2. **Defer snapshots** — full replay always. Simpler system. Accept
   the performance ceiling. Add snapshots when empirical evidence
   shows they are needed.

## Decision

No snapshot support. Full event replay for every aggregate
reconstruction. Deliberate deferral, not oversight.

R1 [11]: Full event replay for every aggregate reconstruction with no
  snapshot support
R2 [11]: Aggregates do not need Serialize or Deserialize bounds on
  their state type

Snapshots are not needed when:

- Aggregates are short-lived (bounded event count)
- Event streams are small (< 1,000 events per aggregate)
- The system is single-writer with local storage (low replay latency)

All three conditions hold for cherry-pit's current deployment model
(single-writer, file-based storage, development/small-scale).

## Consequences

- **Performance ceiling** — reconstruction is O(n) where n = total
  events. For most domain models this is sub-millisecond (pure
  in-memory mutation), but becomes noticeable at scale.
- **No snapshot serialization requirement** — aggregates do not need
  `Serialize`/`Deserialize` for their state; only events need serde
  (CHE-0010). Trait bounds stay minimal: `Default + Send + Sync`.
- **Simpler traits and bus** — no `load_from_sequence`,
  `save_snapshot`, snapshot interval, or staleness handling.
- **Full history always available** — debugging and auditing replay
  the exact event sequence without snapshot boundaries.

### Revisit criteria

Add snapshot support when: an aggregate routinely exceeds 10,000
events, dispatch latency exceeds SLA (measured), a database-backed
store makes full-stream reads expensive, or multi-process deployment
makes replay latency affect failover. When revisiting, add a
`SnapshotStore` trait in `cherry-pit-core`, require
`Serialize + DeserializeOwned` on aggregate state, support
configurable intervals, and handle snapshot-event consistency.
