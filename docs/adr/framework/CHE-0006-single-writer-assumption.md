# CHE-0006. Single-Writer Assumption Per Aggregate

Date: 2026-04-24
Last-reviewed: 2026-04-25
Tier: S

## Status

Accepted

## Related

- Depends on: CHE-0004

## Context

Event-sourced systems face a fundamental question: how many processes
can concurrently write to the same aggregate? Options:

1. **Multi-writer with distributed consensus** — Raft, Paxos, or
   similar protocol to coordinate writes. Complex, high latency.
2. **Multi-writer with optimistic concurrency** — all writers attempt,
   conflicts detected and retried. Requires a shared store with
   atomic compare-and-swap.
3. **Single-writer** — each aggregate instance is owned by exactly one
   process. No coordination needed.

Cherry-pit targets agent-first, single-process systems where
distributed coordination is overhead without benefit.

## Decision

Each aggregate instance is owned by exactly one OS process. No
distributed coordination protocol is used. Sequential auto-increment
IDs (NonZeroU64) are safe without distributed ID generation. Optimistic
concurrency (`expected_sequence` on `append`) serves as defense-in-depth
within the single writer, not as a distributed coordination mechanism.

## Consequences

- Eliminates distributed consensus complexity entirely.
- Sequential `NonZeroU64` IDs are simple, fast, and Copy.
- Horizontal scaling per aggregate is impossible — a hotspot aggregate
  cannot be sharded across processes.
- No fencing mechanism exists at the storage level. If two processes
  accidentally share a store directory, data corruption is possible.
  **Mitigated**: CHE-0043 adds process-level advisory file locking
  (`flock`) that detects and rejects a second writer on the same
  directory.
- Multi-node deployment requires an external mechanism (NATS subject
  partitioning, process registry) to route commands to the owning
  process — this is currently undesigned.
- The single-writer assumption is load-bearing. Changing it later
  requires significant rearchitecture.
