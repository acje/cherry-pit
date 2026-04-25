# 18. Synchronous Domain Logic, Asynchronous Infrastructure

Date: 2026-04-25
Last-reviewed: 2026-04-25

## Status

Accepted

## Related

- Depends on: ADR 0008, ADR 0025
- Referenced by: ADR 0020

## Context

Cherry-pit defines two categories of traits:

- **Domain traits** — `Aggregate::apply`, `HandleCommand::handle`,
  `Policy::react`, `Projection::apply`. These express business
  logic: state reconstruction, invariant checking, event reactions,
  read-model building.
- **Infrastructure ports** — `EventStore`, `EventBus`, `CommandBus`,
  `CommandGateway`. These perform I/O: disk reads, network calls,
  message publishing.

The question is where the sync/async boundary falls.

ADR 0008 establishes that command handling is pure (no I/O, no side
effects). ADR 0025 establishes that infrastructure ports use RPITIT
(`impl Future + Send`). Neither explicitly states the boundary rule:
*all domain logic is synchronous; all infrastructure is asynchronous*.

Two design approaches:

1. **Async everywhere** — `async fn handle`, `async fn apply`. Allows
   domain logic to call external services during command handling.
   Requires async runtime in tests. Mixes I/O with business logic.
2. **Sync domain, async infrastructure** — domain traits return plain
   data. Infrastructure traits return futures. The boundary is at the
   port trait signatures.

## Decision

Domain traits are synchronous. Infrastructure ports are asynchronous.
The boundary is at the port trait signatures.

**Synchronous (no futures, no runtime dependency):**

```rust
// Aggregate::apply
fn apply(&mut self, event: &Self::Event);

// HandleCommand::handle
fn handle(&self, cmd: C) -> Result<Vec<Self::Event>, Self::Error>;

// Policy::react
fn react(&self, event: &EventEnvelope<Self::Event>) -> Vec<Self::Output>;

// Projection::apply
fn apply(&mut self, event: &EventEnvelope<Self::Event>);
```

**Asynchronous (returns `impl Future<...> + Send`):**

```rust
// EventStore::load, create, append
fn load(&self, id: AggregateId) -> impl Future<Output = ...> + Send;

// EventBus::publish
fn publish(&self, events: &[EventEnvelope<Self::Event>]) -> impl Future<Output = ...> + Send;

// CommandBus::create, dispatch
fn create<C>(&self, cmd: C) -> impl Future<Output = ...> + Send;

// CommandGateway::create, send
fn create<C>(&self, cmd: C) -> impl Future<Output = ...> + Send;
```

**Dependency consequence:** `pit-core` has zero dependency on any
async runtime (no tokio, no async-std). It depends only on `serde`,
`uuid`, and `jiff`. The domain crate is runtime-agnostic.

## Consequences

- Domain logic is testable without an async runtime. Aggregate tests
  are plain `#[test]` functions — no `#[tokio::test]`, no executor
  setup.
- Aggregates cannot call external services during command handling.
  If an aggregate needs external data, the caller must provide it in
  the command or a preceding query. This is consistent with ADR 0008
  (pure handling).
- Policies cannot perform async I/O. If a policy needs to call an
  external service, it returns a command (its `Output` type) that
  an infrastructure layer dispatches asynchronously.
- Projections cannot query databases during apply. Read models are
  built purely from events. If a projection needs enriched data, a
  separate async process handles it.
- `pit-core` has no async runtime dependency — it depends only on
  `serde`, `uuid`, and `jiff`. Any async runtime (tokio, async-std,
  etc.) can drive the infrastructure ports. Infrastructure crates
  (`pit-gateway`) bring the runtime dependency.
- The boundary is easy to audit: any trait method returning
  `impl Future` is infrastructure; anything returning plain data is
  domain. No grey area.
