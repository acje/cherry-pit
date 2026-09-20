use cherry_pit_storage::{PersistenceError, RetryClass};

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
