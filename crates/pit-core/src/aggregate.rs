use std::error::Error;

use crate::command::Command;
use crate::event::DomainEvent;

/// The aggregate root — the consistency and transactional boundary.
///
/// An aggregate reconstructs its state by replaying events. It is the
/// only place where business invariants are enforced. The aggregate
/// itself only knows how to apply events — command handling is added
/// via the [`HandleCommand`] trait.
///
/// # Single-writer design
///
/// Cherry-pit assumes single-writer aggregates: each aggregate
/// instance is owned by exactly one process. No distributed
/// coordination is needed — the owning process serializes commands
/// internally. Optimistic concurrency (`expected_sequence` on
/// [`EventStore::append`](crate::EventStore::append)) serves as
/// defense-in-depth within the single writer.
///
/// # Design rationale
///
/// - `Default` — the aggregate starts as a blank slate. State is built
///   entirely by replaying events through `apply`.
/// - No `id()` method — aggregate identity is managed by the
///   infrastructure layer (event store, repository). The store assigns
///   [`AggregateId`](crate::AggregateId) values on creation. If the
///   domain logic needs its own ID, it stores it as a field set during
///   the first event's `apply`.
pub trait Aggregate: Default + Send + Sync + 'static {
    /// Events this aggregate produces and is reconstructed from.
    type Event: DomainEvent;

    /// Apply an event to update internal state.
    ///
    /// Must be deterministic and total — it must never fail. This
    /// method is called during state reconstruction (replaying history)
    /// and after handling new commands. If apply could fail, the
    /// aggregate's history would become unloadable.
    fn apply(&mut self, event: &Self::Event);
}

/// Command handling is a separate trait so each command→aggregate
/// pair is verified at compile time.
///
/// An aggregate implements `HandleCommand` once per command type it
/// accepts. The compiler guarantees exhaustive handling — you cannot
/// forget to implement a command. No runtime downcasting, no
/// match-arm gaps.
///
/// # Design rationale
///
/// - `handle` takes `self` by shared reference — the aggregate inspects
///   its current state but does not mutate directly. State changes
///   happen only through events returned by `handle`, then applied
///   via `apply`.
/// - `handle` takes ownership of the command — a command represents
///   one-time intent. After handling, it is consumed.
/// - `Error` is an associated type on `HandleCommand`, not on
///   `Aggregate` — different commands may have different error types.
pub trait HandleCommand<C: Command>: Aggregate {
    /// Domain-specific error for invariant violations.
    type Error: Error + Send + Sync;

    /// Handle a command against the current state.
    ///
    /// Returns zero or more events on success. Zero events means the
    /// command was accepted but no state change occurred (idempotent).
    /// Must be pure — no I/O, no side effects.
    ///
    /// # Errors
    ///
    /// Returns the domain-specific error type when a business invariant
    /// is violated and the command must be rejected.
    fn handle(&self, cmd: C) -> Result<Vec<Self::Event>, Self::Error>;
}
