# Pardosa Domain — Architecture Decision Records

ADRs for the pardosa EDA storage layer: fiber semantics, stream
management, NATS/JetStream transport, migration model, backpressure,
and single-writer fencing at transport level.

Governed by [GOVERNANCE.md](../GOVERNANCE.md).

## Index

| # | Title | Tier | Status | Depends on |
|---|-------|------|--------|------------|
| [PAR-0001](PAR-0001-fiber-state-machine-as-inspectable-data-table.md) | Fiber state machine as inspectable data table | B | Accepted | — |
| [PAR-0002](PAR-0002-index-none-sentinel-replacing-option.md) | Index::NONE sentinel replacing Option\<Index\> | B | Accepted | — |
| [PAR-0003](PAR-0003-event-immutability-private-fields-non-exhaustive.md) | Event immutability — private fields + non_exhaustive | A | Accepted | — |
| [PAR-0004](PAR-0004-single-writer-per-stream.md) | Single-writer per stream | S | Amended | — |
| [PAR-0005](PAR-0005-new-stream-migration-model.md) | New-stream migration model | B | Accepted | — |
| [PAR-0006](PAR-0006-genome-as-primary-serialization.md) | Genome as primary serialization | A | Amended | — |
| [PAR-0007](PAR-0007-monotonic-event-id-for-idempotent-publish.md) | Monotonic event_id for idempotent publish | B | Accepted | PAR-0004 |
| [PAR-0008](PAR-0008-publish-then-apply-durable-first.md) | Publish-then-apply with durable-first semantics | S | Amended | PAR-0004, PAR-0007 |
| [PAR-0009](PAR-0009-locked-rescue-policy-enum-replacing-bool.md) | LockedRescuePolicy enum replacing bool | B | Accepted | — |
| [PAR-0010](PAR-0010-fallible-constructors-replacing-debug-assert.md) | Fallible constructors replacing debug_assert | B | Accepted | — |
| [PAR-0011](PAR-0011-64-bit-target-requirement.md) | 64-bit target requirement | D | Accepted | — |
| [PAR-0012](PAR-0012-precursor-chain-verification-on-startup.md) | Precursor chain verification on startup | D | Accepted | — |
| [PAR-0013](PAR-0013-nats-kv-registry-for-atomic-stream-discovery.md) | NATS KV registry for atomic stream discovery | C | Amended | — |
| [PAR-0014](PAR-0014-backpressure-and-circuit-breaker.md) | Backpressure and circuit breaker | D | Accepted | PAR-0008 |

**Tier distribution:** 2S · 2A · 6B · 1C · 3D

## Dependency Graph

```
PAR-0004 Single-Writer per Stream (S)
├── PAR-0007 Monotonic event_id (B)
│     └── PAR-0008 Publish-then-Apply (S)
│           └── PAR-0014 Backpressure (D)
└── PAR-0008 (also depends on PAR-0004 directly)

Independent (no Depends-on):
  PAR-0001 Fiber State Machine (B)
  PAR-0002 Index::NONE Sentinel (B)
  PAR-0003 Event Immutability (A)
  PAR-0005 New-Stream Migration (B)
  PAR-0006 Genome as Primary Serialization (A)
  PAR-0009 LockedRescuePolicy (B)
  PAR-0010 Fallible Constructors (B)
  PAR-0011 64-bit Target (D)
  PAR-0012 Precursor Chain (D)
  PAR-0013 NATS KV Registry (C)
```

## Cross-Domain References

| Pardosa ADR | Framework ADR | Relationship |
|-------------|---------------|--------------|
| PAR-0003 | CHE-0022 (Schema Evolution) | References |
| PAR-0004 | CHE-0006 (Single-Writer) | Illustrates |
| PAR-0004 | CHE-0043 (File Fencing) | Contrasts with |
| PAR-0005 | CHE-0022 (Schema Evolution) | Extends |
| PAR-0006 | CHE-0045 (Serialization Scope) | Scoped by |
| PAR-0007 | CHE-0041 (Idempotency) | Illustrates |

| Pardosa ADR | Genome ADR | Relationship |
|-------------|------------|--------------|
| PAR-0002 | GEN-0002, GEN-0007 | References |
| PAR-0006 | GEN-0001, GEN-0006, GEN-0008, GEN-0012, GEN-0031 | References |

## Reference Documents

- [pardosa-design.md](../../pardosa-design.md) — original design document
- [pardosa-next.md](../../pardosa-next.md) — revised design with amendments
- [automerge-ideas.md](../../automerge-ideas.md) — CRDT exploration notes
