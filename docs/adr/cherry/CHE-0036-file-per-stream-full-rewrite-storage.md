# CHE-0036. File-Per-Stream Full-Rewrite Storage Model

Date: 2026-04-25
Last-reviewed: 2026-04-25
Tier: D

## Status

Accepted

## Related

- Depends on: CHE-0031, CHE-0032

## Context

`MsgpackFileStore<E>` needs a storage topology (how events map to
files on disk) and a persistence strategy (how new events are written).

CHE-0031 covers the serialization format (MessagePack named encoding).
CHE-0032 covers the write mechanism (temp-file + rename for
atomicity). Neither addresses the structural decisions: how many files
per aggregate, what each file contains, and how appends work.

Three topology options:

1. **Single file for all aggregates** — one append-only log file.
   Simple append, but loading one aggregate requires scanning the
   entire log. Index needed for random access.
2. **File per aggregate instance** — one `.msgpack` file per aggregate
   ID. Loading is a single file read. No index needed.
3. **Directory per aggregate with segment files** — segment rotation
   for large aggregates. Complex, premature for current scale.

Three persistence strategies:

1. **Append-only** — new events appended to end of file. Efficient
   writes. Requires a format that supports incremental appending
   (MessagePack as `Vec<Envelope>` does not — the outer array length
   is fixed at write time).
2. **Full rewrite** — load entire history, extend, write all back.
   Simple. O(n) write cost where n = total events for the aggregate.
3. **WAL + compaction** — write-ahead log with periodic compaction.
   Complex, premature.

## Decision

One file per aggregate instance. Full rewrite on every append.

### File layout

```text
store/
  1.msgpack     # aggregate 1: Vec<EventEnvelope<E>>
  2.msgpack     # aggregate 2: Vec<EventEnvelope<E>>
  ...
```

Each file contains the complete event history for one aggregate,
serialized as a single `Vec<EventEnvelope<E>>` in MessagePack named
format. The filename is the aggregate ID (a `u64`).

### Append flow

```
append(id, expected_sequence, events):
  1. Acquire per-aggregate write lock
  2. Read entire file → Vec<EventEnvelope<E>>
  3. Check optimistic concurrency (expected_sequence)
  4. Build new envelopes (UUID v7, sequence, timestamp)
  5. Extend the vector with new envelopes
  6. Serialize entire vector to bytes
  7. Write to {id}.msgpack.tmp
  8. Rename to {id}.msgpack (atomic, CHE-0032)
```

### Load flow

```
load(id):
  1. Read entire file → bytes
  2. Deserialize → Vec<EventEnvelope<E>>
  3. Return (empty Vec if file not found)
```

## Consequences

- **O(n) write cost per append** — every append rewrites the entire
  history. For an aggregate with 1,000 events, appending one event
  serializes and writes all 1,001. This is acceptable for development
  and small deployments; production systems with long-lived
  aggregates should use a database-backed store.
- **O(1) file reads per load** — loading any aggregate is a single
  file read and a single deserialization. No index, no seeking, no
  scanning.
- **File count equals aggregate count** — a system with 100,000
  aggregates has 100,000 files. File systems handle this well up to
  ~1M files per directory; beyond that, sharding directories or
  switching storage backends is needed.
- **Filename is the aggregate ID** — `u64` IDs cannot cause path
  traversal. No validation or escaping needed.
- **No partial reads** — snapshots or partial loading are impossible
  with this model. The entire history must be loaded. This is
  consistent with CHE-0037 (no snapshot support).
- **MessagePack's `Vec` encoding** does not support append-in-place.
  The named-map format (CHE-0031) writes the array length first,
  making incremental appending structurally impossible without
  re-encoding. Full rewrite is the only correct strategy given the
  chosen serialization format.
- **Atomic rename (CHE-0032)** ensures readers never see a partially
  written file. The temp-file pattern is necessary because the full
  rewrite is not an atomic operation at the filesystem level.
- The storage model is a concrete implementation choice in
  `cherry-pit-gateway`, not a framework-level constraint. Other `EventStore`
  implementations (database-backed, Pardosa-backed) will have
  different topologies and persistence strategies.
