# CHE-0012. Aggregate Default for Zero-State Construction

Date: 2026-04-25
Last-reviewed: 2026-04-25

## Status

Accepted

## Related

- Depends on: CHE-0009
- Referenced by: CHE-0020

## Context

Every aggregate must have an initial state before any events are
applied. The framework needs to construct this initial state during
command handling (for `create`) and during replay (for `dispatch`).

Three approaches:

1. **`Default` trait bound** — `Aggregate: Default`. The aggregate
   provides a `default()` method returning a blank-slate instance.
   State is built entirely by replaying events through `apply`.
2. **Constructor method** — `Aggregate::initial_state() -> Self` or
   `Aggregate::new() -> Self`. Custom name, same semantics. Adds a
   method to the trait.
3. **Builder pattern** — aggregate constructed from configuration or
   initial parameters. Adds complexity; parameters would need to be
   part of the first command.

## Decision

`Aggregate` requires `Default`. Aggregates start as blank slates.

```rust
pub trait Aggregate: Default + Send + Sync + 'static {
    type Event: DomainEvent;
    fn apply(&mut self, event: &Self::Event);
}
```

The `Default` instance represents "no events applied yet." It is not
a valid domain state — it is a construction artifact that becomes
valid after the first event is applied. The `CommandBus::create` flow
is:

1. `A::default()` — blank aggregate
2. `aggregate.handle(cmd)` — pure handler returns events (CHE-0008)
3. Events applied via `aggregate.apply(event)` for each event
4. Events persisted via `EventStore::create` (CHE-0013)

For `dispatch`, the flow adds replay:

1. `A::default()` — blank aggregate
2. `EventStore::load(id)` — load all events (CHE-0037: no snapshots)
3. For each loaded event: `aggregate.apply(event)` — rebuild state
4. `aggregate.handle(cmd)` — handle against rebuilt state
5. New events persisted via `EventStore::append`

## Consequences

- **No constructor arguments** — aggregates cannot take configuration
  or initial parameters in their constructor. Any initial data must
  arrive via the first command and first event.
- **Default is not a valid domain state** — `A::default()` may have
  fields set to `None`, empty strings, or zero values that violate
  domain invariants. This is acceptable because no command is ever
  dispatched against a default-only aggregate: `create` always
  produces at least one event (CHE-0013), and `dispatch` loads
  history first.
- **Idiomatic Rust** — `Default` is a standard trait. Users can
  `#[derive(Default)]` in most cases. No custom trait method to
  learn.
- **Testability** — creating an aggregate in a specific state for
  testing is done by constructing `default()` then applying setup
  events. This is the standard event-sourcing test pattern and
  requires no mocks.
- **Infallible construction** — `Default::default()` cannot fail.
  Combined with infallible `apply` (CHE-0009), aggregate
  reconstruction is guaranteed to succeed for any valid event
  history.
- **Related to CHE-0009** — infallible `apply` assumes a total
  function from any state. `Default` provides the guaranteed
  starting point. Together, they ensure that `default() + replay`
  always produces a valid aggregate.
