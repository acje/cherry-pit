use std::error::Error;
use std::fmt;
use std::path::PathBuf;

use crate::aggregate::{Aggregate, HandleCommand};
use crate::aggregate_id::AggregateId;
use crate::event::EventEnvelope;

/// Stable retry guidance for framework errors.
///
/// This category is intentionally coarse. Callers use it to choose a
/// first response strategy without matching every concrete error variant:
/// retry after reloading/backing off, or stop and surface the condition
/// for domain/operator action.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorCategory {
    /// Repeating the operation may succeed after backoff, reload, or
    /// infrastructure recovery.
    Retryable,

    /// Repeating the same operation against the same state is expected
    /// to fail until input, domain state, or stored data is repaired.
    Terminal,
}

impl ErrorCategory {
    /// Returns true for errors where retry is a valid first response.
    #[must_use]
    pub const fn is_retryable(self) -> bool {
        matches!(self, Self::Retryable)
    }

    /// Returns true for errors requiring caller/operator action before retry.
    #[must_use]
    pub const fn is_terminal(self) -> bool {
        matches!(self, Self::Terminal)
    }
}

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

impl<E: Error + Send + Sync> DispatchError<E> {
    /// Classify the dispatch failure as retryable or terminal.
    #[must_use]
    pub const fn category(&self) -> ErrorCategory {
        match self {
            Self::ConcurrencyConflict { .. } | Self::Infrastructure(_) => ErrorCategory::Retryable,
            Self::Rejected(_) | Self::AggregateNotFound { .. } => ErrorCategory::Terminal,
        }
    }
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

    /// The store directory is locked by another process.
    ///
    /// Returned when a file-based store cannot acquire an exclusive
    /// advisory lock on its directory. This indicates another process
    /// is already using the same store directory, which violates the
    /// single-writer assumption (CHE-0006).
    StoreLocked {
        /// The path to the store directory that is locked.
        path: PathBuf,
    },

    /// Persisted data failed structural or semantic validation.
    ///
    /// This includes malformed bytes, invalid envelopes, aggregate ID
    /// mismatches, sequence gaps, duplicates, and out-of-order events.
    /// Retrying the same read is not expected to succeed until the store
    /// is repaired or restored from backup.
    CorruptData(Box<dyn Error + Send + Sync>),

    /// Infrastructure failure (disk I/O, network, serialization).
    Infrastructure(Box<dyn Error + Send + Sync>),
}

impl StoreError {
    /// Classify the store failure as retryable or terminal.
    #[must_use]
    pub const fn category(&self) -> ErrorCategory {
        match self {
            Self::ConcurrencyConflict { .. }
            | Self::StoreLocked { .. }
            | Self::Infrastructure(_) => ErrorCategory::Retryable,
            Self::CorruptData(_) => ErrorCategory::Terminal,
        }
    }
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
            Self::StoreLocked { path } => write!(
                f,
                "store directory is locked by another process: {}",
                path.display()
            ),
            Self::CorruptData(e) => write!(f, "store corrupt data: {e}"),
            Self::Infrastructure(e) => write!(f, "store infrastructure error: {e}"),
        }
    }
}

impl Error for StoreError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::CorruptData(e) | Self::Infrastructure(e) => Some(e.as_ref()),
            Self::ConcurrencyConflict { .. } | Self::StoreLocked { .. } => None,
        }
    }
}

/// Error from event bus publication.
///
/// Intentionally simple — publication errors are infrastructure-level.
/// The `CommandBus` may log this error but does not propagate it as a
/// `DispatchError` — the command already succeeded (events are persisted).
#[derive(Debug)]
#[non_exhaustive]
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

    /// Classify event publication failure as retryable.
    #[must_use]
    pub const fn category(&self) -> ErrorCategory {
        ErrorCategory::Retryable
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

/// Errors from `EventEnvelope` construction or validation.
///
/// Only one variant: `NilEventId`. Sequence validity is guaranteed
/// by `NonZeroU64` — the type system eliminates zero sequences at
/// compile time, and serde rejects zero on deserialization.
#[derive(Debug)]
#[non_exhaustive]
pub enum EnvelopeError {
    /// The `event_id` is nil (`Uuid::nil()`), which indicates a
    /// missing or corrupted event identifier.
    NilEventId,

    /// The envelope belongs to a different aggregate stream than the
    /// file or store partition being loaded.
    AggregateIdMismatch {
        /// Aggregate ID expected from the stream key.
        expected: AggregateId,
        /// Aggregate ID found in the envelope.
        actual: AggregateId,
    },

    /// The stream sequence is not exactly contiguous from 1..=N.
    SequenceGap {
        /// Sequence required at this stream position.
        expected_sequence: u64,
        /// Sequence found in the envelope.
        actual_sequence: u64,
    },
}

impl EnvelopeError {
    /// Envelope validation failures indicate corrupt or malformed data.
    #[must_use]
    pub const fn category(&self) -> ErrorCategory {
        ErrorCategory::Terminal
    }
}

impl fmt::Display for EnvelopeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NilEventId => write!(f, "event_id must not be nil"),
            Self::AggregateIdMismatch { expected, actual } => write!(
                f,
                "event aggregate_id mismatch: expected {expected}, actual {actual}"
            ),
            Self::SequenceGap {
                expected_sequence,
                actual_sequence,
            } => write!(
                f,
                "event sequence gap: expected sequence {expected_sequence}, actual {actual_sequence}"
            ),
        }
    }
}

impl Error for EnvelopeError {}

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
