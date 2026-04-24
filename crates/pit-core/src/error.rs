use std::error::Error;
use std::fmt;

use crate::aggregate::{Aggregate, HandleCommand};
use crate::aggregate_id::AggregateId;
use crate::event::EventEnvelope;

/// Errors that can occur during command dispatch.
///
/// Generic over `E` — the domain-specific error type from
/// `HandleCommand<C>::Error`. This preserves full type information
/// through the gateway and bus, allowing callers to match on
/// domain errors without downcasting.
#[derive(Debug)]
#[non_exhaustive]
pub enum DispatchError<E: Error + Send + Sync> {
    /// The aggregate rejected the command (business invariant violation).
    Rejected(E),

    /// No events exist for this aggregate — it has never been created.
    AggregateNotFound { aggregate_id: AggregateId },

    /// Another command was persisted against this aggregate between our
    /// load and our persist. The caller may retry.
    ConcurrencyConflict {
        aggregate_id: AggregateId,
        expected_sequence: u64,
        actual_sequence: u64,
    },

    /// Infrastructure failure (event store unavailable, serialization
    /// error, transport timeout, etc.).
    Infrastructure(Box<dyn Error + Send + Sync>),
}

impl<E: Error + Send + Sync> fmt::Display for DispatchError<E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Rejected(e) => write!(f, "command rejected: {e}"),
            Self::AggregateNotFound { aggregate_id } => {
                write!(f, "aggregate not found: {aggregate_id}")
            }
            Self::ConcurrencyConflict {
                aggregate_id,
                expected_sequence,
                actual_sequence,
            } => write!(
                f,
                "concurrency conflict on {aggregate_id}: expected sequence {expected_sequence}, actual {actual_sequence}"
            ),
            Self::Infrastructure(e) => write!(f, "infrastructure error: {e}"),
        }
    }
}

impl<E: Error + Send + Sync + 'static> Error for DispatchError<E> {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Rejected(e) => Some(e),
            Self::Infrastructure(e) => Some(e.as_ref()),
            Self::AggregateNotFound { .. } | Self::ConcurrencyConflict { .. } => None,
        }
    }
}

/// Errors from event store operations.
#[derive(Debug)]
#[non_exhaustive]
pub enum StoreError {
    /// Optimistic concurrency violation — another writer persisted
    /// events after our load.
    ConcurrencyConflict {
        aggregate_id: AggregateId,
        expected_sequence: u64,
        actual_sequence: u64,
    },

    /// Infrastructure failure (disk I/O, network, serialization).
    Infrastructure(Box<dyn Error + Send + Sync>),
}

impl fmt::Display for StoreError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ConcurrencyConflict {
                aggregate_id,
                expected_sequence,
                actual_sequence,
            } => write!(
                f,
                "concurrency conflict on {aggregate_id}: expected sequence {expected_sequence}, actual {actual_sequence}"
            ),
            Self::Infrastructure(e) => write!(f, "store infrastructure error: {e}"),
        }
    }
}

impl Error for StoreError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Infrastructure(e) => Some(e.as_ref()),
            Self::ConcurrencyConflict { .. } => None,
        }
    }
}

/// Error from event bus publication.
///
/// Intentionally simple — publication errors are infrastructure-level.
/// The `CommandBus` may log this error but does not propagate it as a
/// `DispatchError` — the command already succeeded (events are persisted).
#[derive(Debug)]
pub struct BusError(Box<dyn Error + Send + Sync>);

impl BusError {
    /// Wrap an infrastructure error as a bus error.
    pub fn new(source: impl Into<Box<dyn Error + Send + Sync>>) -> Self {
        Self(source.into())
    }

    /// Consume the error and return the underlying cause.
    #[must_use]
    pub fn into_inner(self) -> Box<dyn Error + Send + Sync> {
        self.0
    }
}

impl fmt::Display for BusError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "event bus error: {}", self.0)
    }
}

impl Error for BusError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        Some(self.0.as_ref())
    }
}

/// Result type for command dispatch through the bus or gateway.
///
/// Returns the event envelopes produced and persisted on success.
/// Envelopes contain full metadata: `event_id`, `aggregate_id`,
/// `sequence`, and `timestamp` alongside the domain event payload.
pub type DispatchResult<A, C> = Result<
    Vec<EventEnvelope<<A as Aggregate>::Event>>,
    DispatchError<<A as HandleCommand<C>>::Error>,
>;

/// Result type for aggregate creation through the bus or gateway.
///
/// Returns the store-assigned [`AggregateId`] and the event envelopes
/// produced by the aggregate on success.
pub type CreateResult<A, C> = Result<
    (AggregateId, Vec<EventEnvelope<<A as Aggregate>::Event>>),
    DispatchError<<A as HandleCommand<C>>::Error>,
>;
