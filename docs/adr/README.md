# Architecture Decision Records

This directory contains all ADRs for the cherry-pit EDA framework
workspace, organised by domain. See [GOVERNANCE.md](GOVERNANCE.md) for
the format, numbering, tier system, relationship vocabulary, lifecycle
management, and MECE validation rules.

## Domain Taxonomy

| Domain | Prefix | Directory | ADRs | Scope |
|--------|--------|-----------|------|-------|
| [Common](common/README.md) | `COM` | `common/` | 6 | Cross-cutting software design principles (Ousterhout): module depth, complexity management, error design, abstraction layering |
| [Cherry](cherry/README.md) | `CHE` | `cherry/` | 45 | Design philosophy, EDA/DDD/hexagonal architecture, domain model traits, infrastructure ports, concurrency, storage, tooling, testing |
| [Pardosa](pardosa/README.md) | `PAR` | `pardosa/` | 14 | EDA storage layer: fiber semantics, stream management, NATS transport, migration, backpressure |
| [Genome](genome/README.md) | `GEN` | `genome/` | 33 | Binary serialization format: wire layout, schema hashing, zero-copy, compression, security limits |

**Total**: 98 ADRs

## Quick Reference

- **Numbering**: `{PREFIX}-{NNNN}` — e.g., `CHE-0001`, `PAR-0006`, `GEN-0015`
- **Tiers**: S (foundational) → A (core) → B (behavioural) → C (tooling) → D (detail)
- **Lifecycle**: Draft → Proposed → Accepted → Amended → Deprecated → Superseded
- **Template**: See [GOVERNANCE.md §7](GOVERNANCE.md#7-adr-template)

## Cross-Domain Overview

The four domains connect through shared principles, scoping decisions,
and concrete illustrations:

```
Common (COM) — Technology-agnostic principles
├── COM-0001 Complexity Budget ──── illustrated by ──── CHE-0001 Design Priority Ordering
│                              ──── illustrated by ──── CHE-0038 Testing Strategy
├── COM-0002 Deep Modules ───────── illustrated by ──── CHE-0005 Single Aggregate Design
│                              ──── illustrated by ──── CHE-0030 Flat Public API
├── COM-0003 Pull Complexity Down ── illustrated by ──── CHE-0016 Store Created Envelopes
│                              ──── illustrated by ──── CHE-0020 Infrastructure-Owned Identity
│                              ──── illustrated by ──── CHE-0035 Two-Level Concurrency
│                              ──── illustrated by ──── CHE-0043 Process-Level File Fencing
├── COM-0004 Different Layer ────── illustrated by ──── CHE-0019 Load Returns Empty
├── COM-0005 Define Errors Out ──── illustrated by ──── CHE-0009 Infallible Apply
│                              ──── illustrated by ──── CHE-0019 Load Returns Empty
│                              ──── illustrated by ──── CHE-0041 Idempotency Strategy
└── COM-0006 Docs Before Impl

Cherry (CHE)
├── CHE-0006 Single-Writer ──── illustrated by ──── PAR-0004 Single-Writer per Stream
├── CHE-0007 Forbid Unsafe ──── illustrated by ──── GEN-0006 Zero-Copy + Forbid Unsafe
├── CHE-0022 Schema Evolution ── extended by ────── PAR-0005 New-Stream Migration
│                            ── contrasts with ─── GEN-0002 No Schema Evolution (fixed layout)
├── CHE-0041 Idempotency ────── illustrated by ──── PAR-0007 Monotonic Event ID
├── CHE-0043 File Fencing ───── contrasts with ──── PAR-0004 NATS Sequence Fencing
└── CHE-0045 Serialization Scope
    ├── scopes ── CHE-0031 MsgPack (cherry-pit-gateway)
    └── scopes ── PAR-0006 Genome as Primary (pardosa)
```

See each domain README for full dependency graphs.
