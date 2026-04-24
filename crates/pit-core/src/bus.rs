use std::future::Future;

use crate::aggregate::HandleCommand;
use crate::aggregate_id::AggregateId;
use crate::command::Command;
use crate::error::{BusError, CreateResult, DispatchResult};
use crate::event::{DomainEvent, EventEnvelope};

/// The internal command routing and execution mechanism.
///
/// The bus performs the actual work:
/// 1. Load the aggregate from the event store (replay via apply).
/// 2. Call `HandleCommand::handle()` with the command.
/// 3. Persist the produced events (store creates envelopes).
/// 4. Publish envelopes to the event bus for fan-out.
///
/// Bound to a single aggregate type via the `Aggregate` associated
/// type. The compiler proves that commands, events, store, and bus
/// all agree on the same aggregate — no cross-aggregate ID/type
/// mismatches are possible.
///
/// The [`CommandGateway`](crate::CommandGateway) wraps the bus and adds
/// cross-cutting middleware.
pub trait CommandBus: Send + Sync + 'static {
    /// The single aggregate type this bus manages.
    type Aggregate: crate::aggregate::Aggregate;

    /// Create a new aggregate — full lifecycle without a known ID.
    ///
    /// The bus:
    /// 1. Creates a `Default` aggregate.
    /// 2. Handles the command (producing events).
    /// 3. Persists via `EventStore::create` (store assigns the ID).
    /// 4. Publishes envelopes to the event bus.
    ///
    /// Returns the store-assigned [`AggregateId`] and produced envelopes.
    fn create<C>(
        &self,
        cmd: C,
    ) -> impl Future<Output = CreateResult<Self::Aggregate, C>> + Send
    where
        Self::Aggregate: HandleCommand<C>,
        C: Command;

    /// Load, handle, persist, publish — the full command lifecycle.
    ///
    /// Implementors manage the unit of work: if event persistence
    /// fails, no events are published. If optimistic concurrency
    /// is violated, a `ConcurrencyConflict` error is returned.
    fn dispatch<C>(
        &self,
        id: AggregateId,
        cmd: C,
    ) -> impl Future<Output = DispatchResult<Self::Aggregate, C>> + Send
    where
        Self::Aggregate: HandleCommand<C>,
        C: Command;
}

/// Port for publishing events to downstream consumers.
///
/// After the `CommandBus` persists new events via the
/// [`EventStore`](crate::EventStore), it publishes them through the
/// `EventBus` for fan-out to Policies, Projections, and external
/// integrations.
///
/// Each bus instance is bound to a single domain event type. In a
/// distributed system, each bounded context has its own `EventBus`
/// publishing its aggregate's events (e.g. to a dedicated NATS
/// subject). Cross-context consumption uses separate subscriptions
/// typed to the foreign event type.
///
/// This is a secondary (driven) port. Concrete implementations
/// (in-memory synchronous fan-out, NATS-backed, channel-based) live
/// in infrastructure crates.
pub trait EventBus: Send + Sync + 'static {
    /// The single domain event type this bus publishes.
    type Event: DomainEvent;

    /// Publish events to all registered consumers.
    ///
    /// Called by the `CommandBus` after events are successfully
    /// persisted. Because events are already safely stored, publication
    /// failure is non-fatal — tracking-style processors can catch up
    /// on missed publications.
    fn publish(
        &self,
        events: &[EventEnvelope<Self::Event>],
    ) -> impl Future<Output = Result<(), BusError>> + Send;
}
