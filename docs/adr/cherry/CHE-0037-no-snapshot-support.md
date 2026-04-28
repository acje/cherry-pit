# CHE-0037. No Snapshot Support (Deliberate Deferral)

Date: 2026-04-25
Last-reviewed: 2026-04-25
Tier: D
Status: Accepted

## Related

References: CHE-0001, CHE-0009, CHE-0010, CHE-0040

## Context

Event-sourced systems reconstruct aggregate state by replaying all events. As events accumulate, replay time grows linearly. Snapshots mitigate this by persisting materialized state. Cherry-pit's `EventStore::load` returns the complete event history — no `load_from_snapshot`, no `SnapshotStore` trait exists. Adding snapshot support now would require aggregates to be snapshot-serializable, stores to manage snapshots alongside events, and the bus to load snapshot + remaining events. All three deferral conditions hold: aggregates are short-lived, event streams are small, and the system is single-writer with local storage.

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

- **Performance ceiling** — reconstruction is O(n) where n = total events. Sub-millisecond for most domain models but noticeable at scale.
- **No snapshot serialization requirement** — aggregates need only `Default + Send + Sync`, not `Serialize`/`Deserialize` (CHE-0010).
- **Simpler traits and bus** — no `load_from_sequence`, `save_snapshot`, or staleness handling.
- **Full history always available** — debugging replays the exact event sequence without snapshot boundaries.
- Revisit when: aggregates exceed 10,000 events, dispatch latency exceeds SLA, or multi-process deployment makes replay latency affect failover.
