# CHE-0025. RPITIT Over async_trait

Date: 2026-04-24
Last-reviewed: 2026-04-25
Tier: C

## Status

Accepted

## Related

- References: CHE-0001

## Context

Cherry-pit's port traits (EventStore, CommandBus, CommandGateway,
EventBus) are async. Two approaches exist for async trait methods in
Rust:

1. **`async_trait` proc macro** — wraps return types in
   `Box<dyn Future>`, causing a heap allocation per call. Widely
   used, supports object safety.
2. **RPITIT (Return Position Impl Trait in Traits)** — uses
   `-> impl Future<...> + Send` in trait return position. Zero-cost:
   the compiler monomorphizes each impl. No box, no vtable. Requires
   Rust 1.75+ (stabilized in late 2023).

Command dispatch is a hot path. A `Box<dyn Future>` allocation per
dispatch is measurable overhead for a framework.

## Decision

All async port traits use RPITIT (`impl Future` in return position)
instead of the `async_trait` proc macro. The minimum supported Rust
version is 1.95 (edition 2024).

## Consequences

- Zero heap allocation per command dispatch.
- Object safety is permanently sacrificed — no `dyn EventStore`,
  `dyn CommandBus`, etc. This is consistent with the single-aggregate
  design (concrete types everywhere).
- The `Send` bound on returned futures (`+ Send`) requires all state
  captured across `.await` points to be `Send`, constraining adapter
  implementations.
- Trait signatures use explicit `-> impl Future<...> + Send` rather
  than `async fn` sugar, which is more verbose but precise.
- MSRV of 1.95 excludes users on older toolchains. Acceptable for a
  pre-1.0 project.
