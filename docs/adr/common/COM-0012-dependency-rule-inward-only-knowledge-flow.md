# COM-0012. Dependency Rule — Inward-Only Knowledge Flow

Date: 2026-04-26
Last-reviewed: 2026-04-26
Tier: S
Status: Accepted

## Related

References: COM-0001, COM-0004

## Context

Martin (Clean Architecture, Ch. 22) and Cockburn's Hexagonal Architecture formalize the same principle: source code dependencies must point inward toward higher-level policy. Inner layers define abstractions; outer layers implement them. Nothing in an inner layer may reference anything in an outer layer. This is distinct from COM-0004 (different layer, different abstraction), which addresses *what* each layer abstracts; the dependency rule addresses *which direction* knowledge flows.

Cherry-pit's crate DAG embodies this rule. `cherry-pit-core` defines traits (`Aggregate`, `EventStore`, `CommandGateway`, `DomainEvent`) knowing nothing about MessagePack, file I/O, or HTTP. `cherry-pit-gateway` and `cherry-pit-web` depend on core, never the reverse. `pardosa` implements the `EventStore` port depending only on core traits. The Cargo workspace (CHE-0029) enforces this physically — circular dependencies are compile errors.

Violation patterns include inner modules importing concrete outer types, domain types carrying serialization annotations, and core trait signatures referencing adapter library types.

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
R4 [3]: The Cargo crate DAG makes dependency violations a compile
  error; logical layering within a crate uses module visibility
R5 [2]: When an inner layer needs an outer capability, define a
  trait in the inner layer and implement it in the outer layer

## Consequences

- The crate DAG is the primary enforcement mechanism. Adding a
  dependency from `cherry-pit-core` to `pardosa` or
  `cherry-pit-web` is a compile error and an architectural
  violation.
- Domain types remain free of serialization annotations. GEN-0001
  (GenomeSafe marker trait) and CHE-0045 (serialization scope)
  ensure that serde derives are confined to infrastructure crates
  where they belong.
- New infrastructure ports (CHE-0044: object store) can be added
  as outer-layer implementations without modifying inner-layer
  traits — the dependency rule guarantees this.
- COM-0004 (different layer, different abstraction) and COM-0012
  are complementary: COM-0004 ensures each layer adds value;
  COM-0012 ensures dependencies flow in one direction. Together
  they prevent both shallow layers and circular dependencies.
- The rule creates friction when inner-layer abstractions need to
  evolve: changing a core trait affects all outer implementations.
  This friction is intentional — it makes the cost of abstraction
  change visible and encourages stable, well-considered interfaces
  (COM-0002: deep modules).
