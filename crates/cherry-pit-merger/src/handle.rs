//! [`MergerHandle`] — the public dispatch surface returned by
//! [`crate::Merger::spawn`].
//!
//! Wraps a [`mpsc::Sender<MergerCommand<A, Arm>>`] and exposes
//! [`MergerHandle::dispatch`], the single async call that crosses the
//! channel boundary, awaits the merger's [`oneshot`] reply, and
//! returns the arm's [`Result<(), Arm::Err>`] verbatim.
//!
//! The handle is [`Clone`] (cheap [`Arc`]-equivalent under
//! [`mpsc::Sender`]'s own clone semantics) so consumers can spread
//! it across services that all dispatch into the same merger
//! substrate per [CHE-0005:R1] single-aggregate-per-port.
//!
//! [`mpsc::Sender<MergerCommand<A, Arm>>`]: https://docs.rs/tokio/latest/tokio/sync/mpsc/struct.Sender.html
//! [`oneshot`]: https://docs.rs/tokio/latest/tokio/sync/oneshot/index.html
//! [CHE-0005:R1]: https://github.com/Mattilsynet/gh-report/blob/c8507377b2748a015148751ce288be2bad9ec708/docs/adr/cherry/CHE-0005-single-aggregate-design.md

use cherry_pit_core::{Aggregate, CorrelationContext, StoreError};
use tokio::sync::{mpsc, oneshot};

use crate::arm::MergerArm;
use crate::command::MergerCommand;

/// Public dispatch surface for a spawned [`crate::Merger`].
///
/// Cloning is cheap (clones the underlying [`mpsc::Sender`]); spread
/// the handle across services that all need to dispatch into the
/// same merger.
pub struct MergerHandle<A, Arm>
where
    A: Aggregate,
    Arm: MergerArm<A>,
{
    tx: mpsc::Sender<MergerCommand<A, Arm>>,
}

impl<A, Arm> Clone for MergerHandle<A, Arm>
where
    A: Aggregate,
    Arm: MergerArm<A>,
{
    fn clone(&self) -> Self {
        Self {
            tx: self.tx.clone(),
        }
    }
}

impl<A, Arm> std::fmt::Debug for MergerHandle<A, Arm>
where
    A: Aggregate,
    Arm: MergerArm<A>,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MergerHandle").finish_non_exhaustive()
    }
}

impl<A, Arm> MergerHandle<A, Arm>
where
    A: Aggregate,
    Arm: MergerArm<A>,
{
    pub(crate) fn new(tx: mpsc::Sender<MergerCommand<A, Arm>>) -> Self {
        Self { tx }
    }

    /// Dispatch `cmd` to the merger and await the triad result.
    ///
    /// The future resolves to:
    ///
    /// - `Ok(())` once the merger has loaded + folded + handled +
    ///   persisted + published (or absorbed the publish failure per
    ///   CHE-0024:R1).
    /// - `Err(Arm::Err)` on any failure surfaced by the arm
    ///   ([`MergerArm::handle`] domain error), the store
    ///   ([`StoreError`] lifted through [`MergerArm::Err`]'s
    ///   `From<StoreError>` bound), or the routing-index
    ///   ([`MergerArm::missing_key_error`] for
    ///   [`crate::PersistMode::AppendStrict`] misses).
    ///
    /// # Errors
    ///
    /// Returns [`MergerArm::Err`] for the failure cases above.
    /// Rejected channel sends are infrastructure failures before acceptance.
    /// Losing the reply after acceptance is [`StoreError::Indeterminate`]:
    /// callers must reconcile rather than retry an unknown outcome.
    pub async fn dispatch(&self, cmd: Arm::Cmd, ctx: CorrelationContext) -> Result<(), Arm::Err> {
        let (reply_tx, reply_rx) = oneshot::channel();
        let envelope = MergerCommand {
            cmd,
            ctx,
            reply: reply_tx,
        };
        self.tx.send(envelope).await.map_err(|_send_err| {
            Arm::Err::from(StoreError::Infrastructure(
                "MergerHandle::dispatch: merger task channel closed; \
                 the merger task has shut down and is no longer accepting commands"
                    .into(),
            ))
        })?;
        match reply_rx.await {
            Ok(result) => result,
            Err(recv_err) => Err(Arm::Err::from(StoreError::Indeterminate(Box::new(
                recv_err,
            )))),
        }
    }
}

#[cfg(test)]
mod uncertainty_tests {
    use super::*;
    use cherry_pit_core::DomainEvent;

    #[derive(Clone, Debug)]
    struct Event;
    impl DomainEvent for Event {
        fn event_type(&self) -> &'static str {
            "test"
        }
    }
    #[derive(Default)]
    struct State;
    impl Aggregate for State {
        type Event = Event;
        fn apply(&mut self, _: &Event) {}
    }
    struct Arm;
    impl MergerArm<State> for Arm {
        type Cmd = ();
        type Err = StoreError;
        fn persist_mode(&self, (): &()) -> crate::PersistMode {
            crate::PersistMode::Create
        }
        fn handle(&self, _: &State, (): ()) -> Result<Vec<Event>, StoreError> {
            Ok(vec![Event])
        }
        fn publish_label(&self, (): &()) -> &'static str {
            "test"
        }
    }

    struct UnknownStore;
    #[derive(Default)]
    struct CountingBus(std::sync::atomic::AtomicUsize);
    impl cherry_pit_core::EventBus for CountingBus {
        type Event = Event;
        fn publish(
            &self,
            _: &[cherry_pit_core::EventEnvelope<Event>],
        ) -> impl std::future::Future<Output = Result<(), cherry_pit_core::BusError>> + Send
        {
            self.0.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            async { Ok(()) }
        }
    }
    impl cherry_pit_core::EventStore for UnknownStore {
        type Event = Event;
        #[expect(
            clippy::unused_async_trait_impl,
            reason = "fault stub implements async store port without I/O"
        )]
        async fn load(
            &self,
            _: cherry_pit_core::AggregateId,
        ) -> Result<Vec<cherry_pit_core::EventEnvelope<Event>>, StoreError> {
            Ok(Vec::new())
        }
        #[expect(
            clippy::unused_async_trait_impl,
            reason = "fault stub implements async store port without I/O"
        )]
        async fn create(
            &self,
            _: Vec<Event>,
            _: CorrelationContext,
        ) -> cherry_pit_core::StoreCreateResult<Event> {
            Err(StoreError::Indeterminate(
                std::io::Error::other("persist unknown").into(),
            ))
        }
        #[expect(
            clippy::unused_async_trait_impl,
            reason = "fault stub implements async store port without I/O"
        )]
        async fn append(
            &self,
            _: cherry_pit_core::AggregateId,
            _: std::num::NonZeroU64,
            _: Vec<Event>,
            _: CorrelationContext,
        ) -> Result<Vec<cherry_pit_core::EventEnvelope<Event>>, StoreError> {
            Err(StoreError::Indeterminate("append unknown".into()))
        }
    }

    #[tokio::test]
    async fn indeterminate_persistence_never_publishes_or_indexes() {
        use std::sync::{Arc, Mutex};
        let bus = Arc::new(CountingBus::default());
        let index = Arc::new(Mutex::new(std::collections::HashMap::new()));
        let sequences = Arc::new(Mutex::new(std::collections::HashMap::new()));
        let (handle, task) = crate::Merger::<State, _, _, _>::spawn(
            Arm,
            Arc::new(UnknownStore),
            Arc::clone(&bus),
            Arc::clone(&index),
            Arc::clone(&sequences),
        );
        let result = handle.dispatch((), CorrelationContext::none()).await;
        assert!(matches!(result, Err(StoreError::Indeterminate(_))));
        assert_eq!(bus.0.load(std::sync::atomic::Ordering::SeqCst), 0);
        assert!(index.lock().unwrap().is_empty());
        assert!(sequences.lock().unwrap().is_empty());
        drop(handle);
        tokio::time::timeout(std::time::Duration::from_secs(2), task)
            .await
            .unwrap()
            .unwrap();
    }

    #[tokio::test]
    async fn admitted_reply_loss_is_indeterminate_but_rejected_send_is_not() {
        let (tx, mut rx) = mpsc::channel(1);
        let handle = MergerHandle::<State, Arm>::new(tx);
        let receiver = async {
            drop(rx.recv().await.unwrap());
        };
        let (result, ()) = tokio::join!(handle.dispatch((), CorrelationContext::none()), receiver);
        assert!(matches!(result, Err(StoreError::Indeterminate(_))));
        drop(rx);
        assert!(matches!(
            handle.dispatch((), CorrelationContext::none()).await,
            Err(StoreError::Infrastructure(_))
        ));
    }
}
