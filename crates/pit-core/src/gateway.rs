use std::future::Future;

use crate::aggregate::HandleCommand;
use crate::aggregate_id::AggregateId;
use crate::command::Command;
use crate::correlation::CorrelationContext;
use crate::error::{CreateResult, DispatchResult};

/// The primary entry point for dispatching commands into the system.
///
/// Every primary adapter (webhook listener, REST API poller, scheduled
/// job) and every Policy dispatches commands through the gateway. The
/// gateway is the outermost port on the driving side of the hexagon.
///
/// The gateway adds cross-cutting concerns (interceptors, retry,
/// logging) on top of the [`CommandBus`](crate::CommandBus).
///
/// Bound to a single aggregate type. The compiler verifies that every
/// command dispatched through this gateway is accepted by the bound
/// aggregate — no runtime routing errors possible.
pub trait CommandGateway: Send + Sync + 'static {
    /// The single aggregate type this gateway dispatches to.
    type Aggregate: crate::aggregate::Aggregate;

    /// Create a new aggregate instance.
    ///
    /// The gateway:
    /// 1. Runs dispatch interceptors (logging, metadata, validation).
    /// 2. Delegates to the `CommandBus`.
    /// 3. Optionally retries on transient infrastructure failure.
    ///
    /// Returns the store-assigned [`AggregateId`] and the event
    /// envelopes produced by the aggregate on success.
    fn create<C>(
        &self,
        cmd: C,
        context: CorrelationContext,
    ) -> impl Future<Output = CreateResult<Self::Aggregate, C>> + Send
    where
        Self::Aggregate: HandleCommand<C>,
        C: Command;

    /// Dispatch a command targeting an existing aggregate instance.
    ///
    /// The gateway:
    /// 1. Runs dispatch interceptors (logging, metadata, validation).
    /// 2. Delegates to the `CommandBus`.
    /// 3. Optionally retries on transient infrastructure failure.
    ///
    /// Returns the event envelopes produced by the aggregate on success.
    fn send<C>(
        &self,
        id: AggregateId,
        cmd: C,
        context: CorrelationContext,
    ) -> impl Future<Output = DispatchResult<Self::Aggregate, C>> + Send
    where
        Self::Aggregate: HandleCommand<C>,
        C: Command;
}
