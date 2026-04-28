# COM-0012. Dependency Rule — Inward-Only Knowledge Flow

Date: 2026-04-26
Last-reviewed: 2026-04-26
Tier: S
Status: Accepted

## Related

- References: COM-0004

## Context

Martin (Clean Architecture, Ch. 22, "The Clean Architecture")
formalizes a principle present across multiple architectural styles:
source code dependencies must point inward toward higher-level
policy. Inner layers define abstractions; outer layers provide
implementations. Nothing in an inner layer may know about, name,
import, or reference anything in an outer layer.

Cockburn's Hexagonal Architecture (Ports and Adapters) expresses
the same principle geometrically: the application core defines
ports (interfaces); adapters in the outer ring implement them.
The core never references an adapter. Data crossing boundaries
is always in the form defined by the inner layer.

This principle is distinct from COM-0004 (different layer,
different abstraction), which addresses *what* each layer
abstracts. The dependency rule addresses *which direction*
knowledge flows between layers.

**Cherry-pit's architecture is structured around this rule:**

- **cherry-pit-core** defines traits (`Aggregate`, `EventStore`,
  `CommandGateway`, `DomainEvent`) and knows nothing about
  MessagePack, file I/O, NATS, or HTTP.

- **cherry-pit-gateway** implements `CommandGateway` by composing
  an `EventStore` — it depends on core, never the reverse.

- **cherry-pit-web** provides HTTP adapters — it depends on
  gateway and core, never the reverse.

- **pardosa** implements the `EventStore` port — it depends on core
  traits, never on gateway or web.

The Cargo workspace's crate DAG (CHE-0029) is the physical
enforcement: circular dependencies between crates are a compile
error.

**Violation patterns:**

- An inner module imports a concrete type from an outer module
  instead of defining a trait
- A domain type includes serialization annotations (`#[derive(Serialize)]`)
  that couple it to an infrastructure concern
- A core trait method signature references a library type from an
  adapter crate (e.g., `nats::Message` in a core trait)

## Decision

Source code dependencies point inward. Inner layers define
abstractions; outer layers implement them. No inner layer may
reference, import, or depend on an outer layer.

### Rules

1. **Inner layers own the abstractions.** Traits, domain types, and
   error types are defined in the innermost layer that uses them.
   Outer layers depend on these definitions; inner layers never
   depend on outer implementations.

2. **Data crosses boundaries in inner-layer types.** When data moves
   from an outer layer to an inner layer, it is converted to the
   inner layer's types at the boundary. The inner layer never
   handles serialization formats, transport types, or framework
   types.

3. **No infrastructure in domain.** Domain logic (aggregates,
   commands, events, policies) must not reference infrastructure
   concerns: serialization libraries, database clients, network
   protocols, file system operations. CHE-0045 (serialization
   scope per crate) enforces this for serde specifically.

4. **Enforce physically, not just logically.** The Cargo crate DAG
   (CHE-0029) makes dependency violations a compile error. Logical
   layering within a single crate is maintained through module
   visibility (`pub(crate)`, private modules).

5. **Dependency inversion for runtime flexibility.** When an inner
   layer needs a capability provided by an outer layer, define a
   trait in the inner layer and implement it in the outer layer.
   The classic port-and-adapter pattern. The trait belongs to the
   domain; the implementation belongs to infrastructure.

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
