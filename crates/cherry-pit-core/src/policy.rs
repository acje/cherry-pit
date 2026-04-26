use crate::event::{DomainEvent, EventEnvelope};

/// A policy reacts to domain events by producing commands.
///
/// Policies are the mechanism for cross-aggregate and cross-context
/// coordination. They observe what happened (events) and decide what
/// should happen next (commands). Policies are eventually consistent
/// by nature.
///
/// # Design rationale
///
/// - `Output` is a static associated type, not `Box<dyn AnyCommand>`.
///   The agent defines an enum of possible command outputs, and the
///   compiler verifies exhaustive matching when the infrastructure
///   dispatches them.
/// - Policies receive `EventEnvelope`, not raw events — they often
///   need metadata (timestamp, `aggregate_id`) to construct correctly
///   targeted commands.
/// - Idempotency requirement — since event delivery may be
///   at-least-once (especially over NATS), policies must tolerate
///   replays.
pub trait Policy: Send + Sync + 'static {
    /// The event type this policy reacts to.
    type Event: DomainEvent;

    /// The output type — typically an enum of possible commands.
    type Output: Send + Sync + 'static;

    /// React to an event. Returns zero or more outputs to dispatch.
    ///
    /// An empty vec means this event is not relevant to this policy.
    /// Policies must be idempotent — reacting to the same event
    /// twice must produce the same outputs.
    #[must_use]
    fn react(&self, event: &EventEnvelope<Self::Event>) -> Vec<Self::Output>;
}
