# COM-0012. Dependency Rule — Inward-Only Knowledge Flow

Date: 2026-04-26
Last-reviewed: 2026-04-28
Tier: S
Status: Accepted

## Related

References: COM-0001

## Context

Martin (Clean Architecture, Ch. 22) and Cockburn's Hexagonal Architecture formalize the same principle: source code dependencies must point inward toward higher-level policy. Inner layers define abstractions; outer layers implement them. This is distinct from COM-0004 (what each layer abstracts) — the dependency rule addresses which direction knowledge flows.

Cherry-pit's crate DAG embodies this: `cherry-pit-core` defines traits knowing nothing about MessagePack or file I/O. Gateway and web crates depend on core, never the reverse. The Cargo workspace (CHE-0029) enforces this physically — circular dependencies are compile errors. Violation patterns include domain types carrying serialization annotations.

## Decision

Source code dependencies point inward. Inner layers define
abstractions; outer layers implement them. No inner layer may
reference, import, or depend on an outer layer.

R1 [2]: Traits, domain types, and error types are defined in the
  innermost layer that uses them; outer layers depend on these
  definitions, never the reverse
R2 [2]: Data crossing boundaries is converted to the inner layer's
  types at the boundary; inner layers never handle serialization
  formats or transport types
R3 [2]: Domain logic must not reference infrastructure concerns —
  serialization libraries, database clients, network protocols,
  or file system operations
R4 [3]: The Cargo crate DAG and each crate's declared dependency set
  bound inter-crate dependencies; an inward dependency added to an
  inner crate is a reviewable violation, not an automatic compile
  error; logical layering within a crate uses module visibility
R5 [2]: When an inner layer needs an outer capability, define a
  trait in the inner layer and implement it in the outer layer

## Consequences

The crate DAG is the primary enforcement — `cherry-pit-core` is declared a leaf whose dependency set is restricted by CHE-0029 R4, so an inward dependency on `cherry-pit-web` or an adapter crate is a reviewable violation of that rule. Serialization concerns are confined by CHE-0045: the core exposes serde bounds on `DomainEvent` but no encoding choice. New infrastructure ports (CHE-0044) can be added without modifying inner-layer traits. COM-0004 and COM-0012 are complementary: one ensures layers add value, the other ensures unidirectional flow. The rule creates intentional friction when core traits evolve, encouraging stable interfaces (COM-0002).
