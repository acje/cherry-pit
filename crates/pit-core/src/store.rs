use std::future::Future;

use crate::aggregate_id::AggregateId;
use crate::error::StoreError;
use crate::event::{DomainEvent, EventEnvelope};

/// Port for loading and persisting a single aggregate's event streams.
///
/// The event store is the single source of truth for aggregate state
/// in an event-sourced system. Every aggregate's history is an ordered
/// sequence of `EventEnvelope`s keyed by `(AggregateId, sequence)`.
///
/// Each event store instance is bound to exactly one domain event type
/// via the `Event` associated type. This gives compile-time proof that
/// every load/append operates on the correct event type — the caller
/// cannot accidentally deserialize one aggregate's events as another's.
///
/// # Envelope construction
///
/// The store creates [`EventEnvelope`]s — callers pass raw domain
/// events. The store assigns `event_id` (UUID v7), `aggregate_id`,
/// `sequence`, and `timestamp`. This eliminates redundancy and makes
/// malformed envelopes impossible by construction.
///
/// # ID assignment
///
/// New aggregates get their ID from [`create`](Self::create), which
/// auto-increments a `u64` counter. Callers never invent IDs.
///
/// # Single-writer assumption
///
/// Cherry-pit assumes single-writer aggregates. Optimistic concurrency
/// (`expected_sequence` on `append`) serves as defense-in-depth within
/// the single writer process.
///
/// This is a secondary (driven) port — the domain tells infrastructure
/// when to load and persist. Concrete implementations (in-memory for
/// testing, Pardosa-backed, PostgreSQL-backed) live in infrastructure
/// crates.
pub trait EventStore: Send + Sync + 'static {
    /// The single domain event type this store persists.
    type Event: DomainEvent;

    /// Load all events for an aggregate, ordered by sequence.
    ///
    /// Returns an empty `Vec` if no events exist for this aggregate.
    /// This is not an error — it means the aggregate has never been
    /// created.
    fn load(
        &self,
        id: AggregateId,
    ) -> impl Future<Output = Result<Vec<EventEnvelope<Self::Event>>, StoreError>> + Send;

    /// Create a new aggregate — the store assigns the next ID.
    ///
    /// The store auto-increments a `u64` counter to assign the ID,
    /// creates [`EventEnvelope`]s from the raw domain events (assigning
    /// `event_id`, `sequence`, and `timestamp`), and persists them.
    ///
    /// Returns the assigned [`AggregateId`] and the created envelopes.
    ///
    /// # Errors
    ///
    /// Returns `StoreError::Infrastructure` if `events` is empty —
    /// an aggregate cannot exist without at least one event.
    #[allow(clippy::type_complexity)]
    fn create(
        &self,
        events: Vec<Self::Event>,
    ) -> impl Future<Output = Result<(AggregateId, Vec<EventEnvelope<Self::Event>>), StoreError>>
           + Send;

    /// Append new events to an existing aggregate's stream.
    ///
    /// The store creates [`EventEnvelope`]s from the raw domain events
    /// (assigning `event_id`, `sequence`, and `timestamp`) and persists
    /// them. Returns the created envelopes.
    ///
    /// `expected_sequence` is the sequence number of the last event
    /// the caller loaded (0 if the aggregate has no events yet — though
    /// this should not happen since `create` always produces ≥1 event).
    /// If the store's actual last sequence does not match, the append
    /// is rejected with `StoreError::ConcurrencyConflict`.
    ///
    /// Empty `events` is a no-op — returns `Ok(vec![])`.
    ///
    /// Atomic — either all events persist, or none do.
    fn append(
        &self,
        id: AggregateId,
        expected_sequence: u64,
        events: Vec<Self::Event>,
    ) -> impl Future<Output = Result<Vec<EventEnvelope<Self::Event>>, StoreError>> + Send;
}
