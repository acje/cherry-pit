# Architecture Decision Records

This directory contains all ADRs for the cherry-pit EDA framework
workspace, organised by domain. See [GOVERNANCE.md](GOVERNANCE.md) for
the format, numbering, tier system, relationship vocabulary, lifecycle
management, and MECE validation rules.

## Domain Taxonomy

| Domain | Prefix | Directory | ADRs | Scope |
|--------|--------|-----------|------|-------|
| [Framework](framework/README.md) | `CHE` | `framework/` | 45 | Design philosophy, EDA/DDD/hexagonal architecture, domain model traits, infrastructure ports, concurrency, storage, tooling, testing |
| [Pardosa](pardosa/README.md) | `PAR` | `pardosa/` | 14 (pending migration) | EDA storage layer: fiber semantics, stream management, NATS transport, migration, backpressure |
| [Genome](genome/README.md) | `GEN` | `genome/` | 33 (pending migration) | Binary serialization format: wire layout, schema hashing, zero-copy, compression, security limits |

**Total**: 92 ADRs (45 migrated, 47 pending migration)

## Quick Reference

- **Numbering**: `{PREFIX}-{NNNN}` — e.g., `CHE-0001`, `PAR-0006`, `GEN-0015`
- **Tiers**: S (foundational) → A (core) → B (behavioural) → C (tooling) → D (detail)
- **Lifecycle**: Draft → Proposed → Accepted → Amended → Deprecated → Superseded
- **Template**: See [GOVERNANCE.md §7](GOVERNANCE.md#7-adr-template)

## Cross-Domain Overview

The three domains connect through shared principles and scoping
decisions:

```
Framework (CHE)
├── CHE-0006 Single-Writer ──── illustrated by ──── PAR-0004 Single-Writer per Stream
├── CHE-0007 Forbid Unsafe ──── illustrated by ──── GEN-0006 Zero-Copy + Forbid Unsafe
├── CHE-0022 Schema Evolution ── extended by ────── PAR-0005 New-Stream Migration
│                            ── contrasts with ─── GEN-0002 No Schema Evolution (fixed layout)
├── CHE-0041 Idempotency ────── illustrated by ──── PAR-0007 Monotonic Event ID
├── CHE-0043 File Fencing ───── contrasts with ──── PAR-0004 NATS Sequence Fencing
└── CHE-0045 Serialization Scope
    ├── scopes ── CHE-0031 MsgPack (pit-gateway)
    └── scopes ── PAR-0006 Genome as Primary (pardosa)
```

See each domain README for full dependency graphs.
