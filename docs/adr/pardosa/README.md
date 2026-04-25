# Pardosa Domain — Architecture Decision Records

This directory will contain ADRs for the pardosa EDA storage layer:
fiber semantics, stream management, NATS/JetStream transport,
migration model, backpressure, and single-writer fencing at transport
level.

Governed by [GOVERNANCE.md](../GOVERNANCE.md).

## Migration Status

**Pending.** 14 ADRs (PAR-0001 through PAR-0014) are awaiting
migration from `quicksilver/crates/pardosa/adr/`. The migration will:

1. Reformat each ADR to the [governance template](../GOVERNANCE.md#7-adr-template)
2. Assign tiers (S → D)
3. Add Date / Last-reviewed / Migration-Origin fields
4. Rewrite relative paths to cherry-pit locations
5. Add cross-domain links to Framework ADRs (7 overlap pairs
   documented in [framework/README.md](../framework/README.md#cross-domain-references))

## Planned Index

| #        | Title                                          | Status  |
|----------|------------------------------------------------|---------|
| PAR-0001 | Fiber state machine as inspectable data table  | Pending |
| PAR-0002 | Index::NONE sentinel replacing Option\<Index\> | Pending |
| PAR-0003 | Event immutability — private fields + non_exhaustive | Pending |
| PAR-0004 | Single-writer per stream                       | Pending |
| PAR-0005 | New-stream migration model                     | Pending |
| PAR-0006 | Genome as primary serialization                | Pending |
| PAR-0007 | Monotonic event_id for idempotent publish      | Pending |
| PAR-0008 | Publish-then-apply with durable-first semantics | Pending |
| PAR-0009 | LockedRescuePolicy enum replacing bool         | Pending |
| PAR-0010 | Fallible constructors replacing debug_assert   | Pending |
| PAR-0011 | 64-bit target requirement                      | Pending |
| PAR-0012 | Precursor chain verification on startup        | Pending |
| PAR-0013 | NATS KV registry for atomic stream discovery   | Pending |
| PAR-0014 | Backpressure and circuit breaker               | Pending |

## Cross-Domain References (Planned)

| Pardosa ADR | Framework ADR | Relationship |
|-------------|---------------|--------------|
| PAR-0004 | CHE-0006 (Single-Writer) | Illustrates |
| PAR-0005 | CHE-0022 (Schema Evolution) | Extends |
| PAR-0007 | CHE-0041 (Idempotency) | Illustrates |
| PAR-0003 | CHE-0022 (Schema Evolution) | References |
| PAR-0004 | CHE-0043 (File Fencing) | Contrasts with |
| PAR-0006 | CHE-0045 (Serialization Scope) | Scoped by |

## Reference Documents

- [pardosa-design.md](../../pardosa-design.md) — original design document
- [pardosa-next.md](../../pardosa-next.md) — revised design with amendments
- [automerge-ideas.md](../../automerge-ideas.md) — CRDT exploration notes
