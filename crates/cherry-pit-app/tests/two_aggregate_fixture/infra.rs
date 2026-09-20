//! In-fixture infrastructure: `InMemoryEventStore`-backed `CommandGateway`
//! generic over an aggregate type, plus aliases binding it to `Foo` /
//! `Bar`.
//!
//! Modelled on `cherry-pit-web/tests/integration_inmem.rs::InMemGateway`.
//! No canonical `CommandGateway` impl ships in the workspace today
//! (only test-local stubs); per S7 contract `abort_if #4` fallback we
//! provide one here. This fixture exercises higher wiring/policy logic
//! only (no file/recovery semantics under test), so an in-memory store
//! suffices (bd ghr-eac11b81 Target B triage).

use std::sync::Arc;

use cherry_pit_core::testing::InMemoryEventStore;
use cherry_pit_core::{
    Aggregate, AggregateId, Command, CommandGateway, CorrelationContext, CreateResult,
    DispatchError, DispatchResult, EventEnvelope, EventStore, HandleCommand, StoreError,
};

use super::domain::{Bar, Foo};

#[test]
fn indeterminate_store_mapping_preserves_category_and_source() {
    use std::error::Error;
    let error = dispatch_store_error::<std::convert::Infallible>(StoreError::Indeterminate(
        std::io::Error::other("diagnostic").into(),
    ));
    assert_eq!(
        error.category(),
        cherry_pit_core::ErrorCategory::ReconciliationRequired
    );
    assert_eq!(
        error.source().unwrap().source().unwrap().to_string(),
        "diagnostic"
    );
}

fn dispatch_store_error<E: std::error::Error + Send + Sync>(error: StoreError) -> DispatchError<E> {
    match error {
        unknown @ StoreError::Indeterminate(_) => DispatchError::Indeterminate(Box::new(unknown)),
        StoreError::ConcurrencyConflict {
            aggregate_id,
            expected_sequence,
            actual_sequence,
        } => DispatchError::ConcurrencyConflict {
            aggregate_id,
            expected_sequence,
            actual_sequence,
        },
        other => DispatchError::Infrastructure(Box::new(other)),
    }
}

/// Generic `InMemoryEventStore`-backed gateway parameterised over `A`.
///
/// Each instance is bound to one aggregate type per CHE-0005:R1.
pub struct FileStoreGateway<A: Aggregate> {
    store: Arc<InMemoryEventStore<<A as Aggregate>::Event>>,
}

impl<A: Aggregate> FileStoreGateway<A> {
    pub fn new(store: Arc<InMemoryEventStore<<A as Aggregate>::Event>>) -> Self {
        Self { store }
    }
}

impl<A> CommandGateway for FileStoreGateway<A>
where
    A: Aggregate,
{
    type Aggregate = A;

    async fn create<C>(
        &self,
        cmd: C,
        context: CorrelationContext,
    ) -> CreateResult<Self::Aggregate, C>
    where
        Self::Aggregate: HandleCommand<C>,
        C: Command,
    {
        let agg = A::default();
        let events = agg.handle(cmd).map_err(DispatchError::Rejected)?;
        let (id, envelopes) = self
            .store
            .create(events, context)
            .await
            .map_err(dispatch_store_error)?;
        Ok((id, envelopes))
    }

    async fn send<C>(
        &self,
        id: AggregateId,
        cmd: C,
        context: CorrelationContext,
    ) -> DispatchResult<Self::Aggregate, C>
    where
        Self::Aggregate: HandleCommand<C>,
        C: Command,
    {
        let history = self.store.load(id).await.map_err(dispatch_store_error)?;
        if history.is_empty() {
            return Err(DispatchError::AggregateNotFound { aggregate_id: id });
        }
        let mut agg = A::default();
        for env in &history {
            agg.apply(env.payload());
        }
        let last_seq = history.last().map(EventEnvelope::sequence).ok_or_else(|| {
            DispatchError::Infrastructure("loaded stream had zero sequence on tail event".into())
        })?;
        let new_events = agg.handle(cmd).map_err(DispatchError::Rejected)?;
        let envelopes = self
            .store
            .append(id, last_seq, new_events, context)
            .await
            .map_err(dispatch_store_error)?;
        Ok(envelopes)
    }
}

/// Concrete gateway for `Foo`.
pub type FooGateway = FileStoreGateway<Foo>;

/// Concrete gateway for `Bar`.
pub type BarGateway = FileStoreGateway<Bar>;
