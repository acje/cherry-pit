# CHE-0037. No Snapshot Support (Deliberate Deferral)

Date: 2026-04-25
Last-reviewed: 2026-04-25
Tier: D
Status: Accepted

## Related

- References: CHE-0009, CHE-0010, CHE-0040

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

Snapshots are not needed when:

- Aggregates are short-lived (bounded event count)
- Event streams are small (< 1,000 events per aggregate)
- The system is single-writer with local storage (low replay latency)

All three conditions hold for cherry-pit's current deployment model
(single-writer, file-based storage, development/small-scale).

## Consequences

- **Performance ceiling** — aggregate reconstruction time is O(n)
  where n = total events. An aggregate with 10,000 events replays
  all 10,000 on every command dispatch. For most domain models, this
  is sub-millisecond (apply is pure in-memory state mutation), but
  it becomes noticeable at scale.
- **No snapshot serialization requirement** — aggregates do not need
  to implement `Serialize`/`Deserialize` for their state. Only
  events need serde (CHE-0010). This keeps the aggregate trait
  bounds minimal: `Default + Send + Sync + 'static`.
- **Simpler `EventStore` trait** — no `load_from_sequence`,
  `save_snapshot`, or snapshot interval configuration. The trait
  surface stays small.
- **Simpler `CommandBus` implementations** — always load + replay.
  No snapshot lookup, no fallback logic, no snapshot staleness
  handling.
- **Full history always available** — debugging and auditing can
  replay the exact sequence of events without worrying about snapshot
  boundaries or stale snapshots.

### Revisit criteria

Add snapshot support when any of these conditions are met:

1. An aggregate type routinely exceeds 10,000 events
2. Command dispatch latency exceeds acceptable SLA (measured, not
   estimated)
3. A database-backed event store is deployed where full-stream reads
   have non-trivial I/O cost
4. Multi-process deployment where replay latency affects failover
   time

When revisiting, the snapshot design should:

- Add a `SnapshotStore` trait in `cherry-pit-core` (optional port)
- Require `Serialize + DeserializeOwned` on aggregate state (breaking
  change to `Aggregate` trait bounds)
- Support configurable snapshot intervals
- Handle snapshot-event consistency (what if snapshot is stale?)
