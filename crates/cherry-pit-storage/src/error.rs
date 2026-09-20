//! Persistence error types for crash-safe file operations.

use thiserror::Error;

/// Errors related to persistence (checkpoints, evidence, locks, publishing).
#[derive(Debug, Error)]
pub enum PersistenceError {
    /// Completion is unknown; reconcile before deciding whether to retry.
    #[error("persistence outcome indeterminate: {0}")]
    Indeterminate(#[source] Box<dyn std::error::Error + Send + Sync + 'static>),

    #[error("collection lock failed: {reason}")]
    LockFailed { reason: String },

    #[error("atomic write failed: {reason}")]
    AtomicWriteFailed { reason: String },

    /// Returned when a persistence file exists but cannot be parsed,
    /// has an unsupported schema version, or fails structural validation.
    /// Distinct from `Io` (which covers filesystem-level failures) and
    /// `AtomicWriteFailed` (which covers write-path errors).
    #[error("load failed: {reason}")]
    LoadFailed { reason: String },

    /// Returned when a torn `.pgno` append could not be recovered to
    /// the last durable manifest checkpoint.
    #[error("torn-write recovery failed: {source}")]
    TornWriteRecovery {
        #[source]
        source: Box<dyn std::error::Error + Send + Sync + 'static>,
    },

    #[error("single-writer fence conflict: {source}")]
    FencedConflict {
        /// Sequence the caller expected; threaded from the
        /// `pardosa-fiber-store` concurrency-conflict variant. `None` when
        /// the lower ring did not populate it.
        expected_seq: Option<u64>,
        /// Broker-observed current sequence; threaded from the
        /// `pardosa-fiber-store` concurrency-conflict variant. `None` when
        /// the lower ring could not extract it.
        actual_seq: Option<u64>,
        #[source]
        source: Box<dyn std::error::Error + Send + Sync + 'static>,
    },

    /// Returned when the underlying store backend (e.g. NATS) is
    /// unreachable or otherwise infrastructurally unavailable. Transient:
    /// a retry may succeed once the backend recovers.
    #[error("backend unavailable: {reason}")]
    BackendUnavailable { reason: String },

    /// Returned when a store-level invariant (one-fiber-per-key) is
    /// violated. Structural: not retryable, indicates a bug or corrupted
    /// state upstream of this conversion.
    #[error("store invariant violated: {reason}")]
    InvariantViolation { reason: String },

    /// Returned when the underlying store's in-process mutex was
    /// poisoned by a panicking holder. Unrecoverable: the process must
    /// not continue operating on this store instance.
    #[error("store state poisoned")]
    PoisonedState,

    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
}

/// Storage-native retry guidance for [`PersistenceError`].
///
/// Classification mirroring the intent of
/// `cherry_pit_core::ErrorCategory` and `DispatchError::category()`
/// without depending on `cherry-pit-core` (CHE-0053:R1/R8 forbid a
/// cherry-pit-core dependency in this crate).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RetryClass {
    /// Completion is unknown. Neither automatic retry nor terminal completion
    /// is justified until the caller reconciles the outcome.
    ReconciliationRequired,

    /// Repeating the operation may succeed after backoff or backend
    /// recovery.
    Retryable,

    /// Repeating the same operation against the same state is expected
    /// to fail until the underlying condition is repaired. Includes
    /// `FencedConflict`: the substrate cannot itself resync a fenced
    /// writer, so in-append retry is not offered here — convergence is
    /// the consumer's responsibility (CHE-0088; PGN-0016:R10).
    Terminal,
}

impl RetryClass {
    /// Returns true for classes where retry is a valid first response.
    #[must_use]
    pub const fn is_retryable(self) -> bool {
        matches!(self, Self::Retryable)
    }

    /// Returns true for terminal failures. Unknown completion is not terminal.
    #[must_use]
    pub const fn is_terminal(self) -> bool {
        matches!(self, Self::Terminal)
    }
}

impl PersistenceError {
    /// Classify the persistence outcome as retryable, terminal, or requiring reconciliation.
    #[must_use]
    pub const fn retry_class(&self) -> RetryClass {
        match self {
            Self::Indeterminate(_) => RetryClass::ReconciliationRequired,
            Self::BackendUnavailable { .. } => RetryClass::Retryable,
            Self::LockFailed { .. }
            | Self::AtomicWriteFailed { .. }
            | Self::LoadFailed { .. }
            | Self::TornWriteRecovery { .. }
            | Self::FencedConflict { .. }
            | Self::InvariantViolation { .. }
            | Self::PoisonedState
            | Self::Io(_) => RetryClass::Terminal,
        }
    }
}

#[cfg(test)]
mod retry_class_tests {
    use super::{PersistenceError, RetryClass};
    use std::error::Error;

    #[test]
    fn indeterminate_requires_reconciliation_without_retry_or_terminal_guidance() {
        let error = PersistenceError::Indeterminate(Box::new(std::io::Error::other("unknown")));
        let class = error.retry_class();
        assert_eq!(class, RetryClass::ReconciliationRequired);
        assert!(!class.is_retryable());
        assert!(!class.is_terminal());
        assert!(RetryClass::Retryable.is_retryable());
        assert!(!RetryClass::Retryable.is_terminal());
        assert!(RetryClass::Terminal.is_terminal());
        assert!(!RetryClass::Terminal.is_retryable());
    }

    #[test]
    fn indeterminate_preserves_typed_diagnostic_chain() {
        let error = PersistenceError::Indeterminate(Box::new(PersistenceError::Io(
            std::io::Error::from(std::io::ErrorKind::TimedOut),
        )));
        let source = error.source().expect("boxed diagnostic source");
        assert!(matches!(
            source.downcast_ref::<PersistenceError>(),
            Some(PersistenceError::Io(_))
        ));
        let root = source.source().expect("underlying I/O source");
        assert_eq!(
            root.downcast_ref::<std::io::Error>().unwrap().kind(),
            std::io::ErrorKind::TimedOut
        );
        assert!(
            error
                .to_string()
                .starts_with("persistence outcome indeterminate:")
        );
    }

    #[test]
    fn retry_class_preserves_existing_variants() {
        assert_eq!(
            PersistenceError::BackendUnavailable {
                reason: "down".to_string()
            }
            .retry_class(),
            RetryClass::Retryable
        );

        assert_eq!(
            PersistenceError::LockFailed {
                reason: "held".to_string()
            }
            .retry_class(),
            RetryClass::Terminal
        );
        assert_eq!(
            PersistenceError::AtomicWriteFailed {
                reason: "rename failed".to_string()
            }
            .retry_class(),
            RetryClass::Terminal
        );
        assert_eq!(
            PersistenceError::LoadFailed {
                reason: "bad schema".to_string()
            }
            .retry_class(),
            RetryClass::Terminal
        );
        assert_eq!(
            PersistenceError::TornWriteRecovery {
                source: "torn".into(),
            }
            .retry_class(),
            RetryClass::Terminal
        );
        assert_eq!(
            PersistenceError::FencedConflict {
                expected_seq: Some(1),
                actual_seq: Some(2),
                source: "fenced".into(),
            }
            .retry_class(),
            RetryClass::Terminal,
            "FencedConflict must map Terminal per PGN-0016:R10 — the substrate must not offer in-append retry"
        );
        assert_eq!(
            PersistenceError::InvariantViolation {
                reason: "bug".to_string()
            }
            .retry_class(),
            RetryClass::Terminal
        );
        assert_eq!(
            PersistenceError::PoisonedState.retry_class(),
            RetryClass::Terminal
        );
        assert_eq!(
            PersistenceError::Io(std::io::Error::other("io")).retry_class(),
            RetryClass::Terminal
        );
    }
}
