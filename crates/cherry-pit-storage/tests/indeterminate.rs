use cherry_pit_storage::{PersistenceError, RetryClass};
use std::error::Error;

#[test]
fn backend_unavailable_preserves_original_source_chain() {
    let error = PersistenceError::BackendUnavailable {
        source: Box::new(PersistenceError::Io(std::io::Error::from(
            std::io::ErrorKind::ConnectionRefused,
        ))),
    };
    let source = error.source().expect("original backend diagnostic");
    assert!(matches!(
        source.downcast_ref::<PersistenceError>(),
        Some(PersistenceError::Io(_))
    ));
    let root = source.source().expect("original transport error");
    assert_eq!(
        root.downcast_ref::<std::io::Error>().unwrap().kind(),
        std::io::ErrorKind::ConnectionRefused
    );
    assert_eq!(error.retry_class(), RetryClass::Retryable);
    assert!(error.retry_class().is_retryable());
    assert!(!error.retry_class().is_terminal());
}

#[test]
fn backend_unavailable_accepts_string_only_diagnostics() {
    let error = PersistenceError::BackendUnavailable {
        source: "backend offline".into(),
    };
    assert_eq!(error.source().unwrap().to_string(), "backend offline");
    assert_eq!(error.to_string(), "backend unavailable: backend offline");
    assert_eq!(error.retry_class(), RetryClass::Retryable);
}

#[test]
fn consumer_can_match_reconciliation_required_without_core_dependency() {
    let error = PersistenceError::Indeterminate(std::io::Error::other("unknown completion").into());
    let guidance = match error.retry_class() {
        RetryClass::ReconciliationRequired => "reconcile",
        RetryClass::Retryable => "retry",
        RetryClass::Terminal => "terminal",
    };
    assert_eq!(guidance, "reconcile");
}
