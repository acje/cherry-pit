use serde::de::DeserializeOwned;
use serde::Serialize;

use crate::aggregate_id::AggregateId;

/// Marker trait for domain events.
///
/// Events are immutable facts — something that happened. They are the
/// source of truth in an event-sourced system. Every event must be
/// serializable (for persistence/transport) and cloneable (for fan-out
/// to multiple consumers).
pub trait DomainEvent: Serialize + DeserializeOwned + Clone + Send + Sync + 'static {
    /// A stable string identifier for this event type.
    ///
    /// Used for routing, schema registry, and deserialization dispatch.
    /// Must not change once events of this type exist in a log.
    fn event_type(&self) -> &'static str;
}

/// Infrastructure wrapper around a domain event.
///
/// Provided by pit-core, not implemented by the agent. This is what
/// gets persisted and transported. The domain event is the payload;
/// the envelope adds the metadata needed for ordering, routing, and
/// idempotency.
///
/// Envelopes are created by the [`EventStore`](crate::EventStore)
/// during `create` and `append` — callers pass raw domain events,
/// the store stamps on the metadata.
#[derive(Debug, Clone, Serialize, serde::Deserialize)]
#[serde(bound(
    serialize = "E: Serialize",
    deserialize = "E: DeserializeOwned"
))]
pub struct EventEnvelope<E: DomainEvent> {
    /// Unique identifier for this event instance (UUID v7, time-ordered).
    pub event_id: uuid::Uuid,

    /// The aggregate instance this event belongs to (stream partition key).
    pub aggregate_id: AggregateId,

    /// Monotonically increasing sequence within the aggregate's stream.
    /// Enables optimistic concurrency and ordered replay.
    pub sequence: u64,

    /// When this event was created (UTC instant).
    pub timestamp: jiff::Timestamp,

    /// The domain event payload.
    pub payload: E,
}
