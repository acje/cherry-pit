use crate::event::{DomainEvent, EventEnvelope};

/// A projection folds events into a query-optimized read model.
///
/// Projections are the read side of CQRS. They consume events and
/// build denormalized views optimized for specific query patterns.
///
/// # Design rationale
///
/// - `Default` — projections can be rebuilt from scratch at any time
///   by replaying the full event history. No migration story needed.
/// - Receives `EventEnvelope` — projections often use metadata
///   (timestamp for time-based views, sequence for ordering).
/// - No error return — projection application must be total. If a
///   projection cannot handle an event, that is a bug, not a runtime
///   error.
pub trait Projection: Default + Send + Sync + 'static {
    /// The event type this projection consumes.
    type Event: DomainEvent;

    /// Apply an event to update the read model.
    ///
    /// Must be deterministic and total. A projection can always be
    /// rebuilt from scratch by replaying all events.
    fn apply(&mut self, event: &EventEnvelope<Self::Event>);
}
