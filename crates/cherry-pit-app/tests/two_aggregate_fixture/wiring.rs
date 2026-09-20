//! Single `assemble()` entry point that wires the 2-aggregate fixture
//! into a runnable [`App`]. Load-bearing for the ergonomic LOC
//! benchmark — keep this file lean (no helpers, no domain reshaping).

use std::sync::Arc;

use cherry_pit_app::{App, InProcessEventBus, TracingDeadLetterSink};
use cherry_pit_core::CommandGateway;
use cherry_pit_core::testing::InMemoryEventStore;

use super::domain::{BarEvent, FooEvent, FooToBarOutput, FooToBarPolicy};
use super::infra::{BarGateway, FooGateway};

fn policy_error<E: std::error::Error + Send + Sync + 'static>(
    error: cherry_pit_core::DispatchError<E>,
) -> cherry_pit_app::AgentError {
    match error {
        cherry_pit_core::DispatchError::Indeterminate(source) => {
            cherry_pit_app::AgentError::Store(cherry_pit_core::StoreError::Indeterminate(source))
        }
        other => cherry_pit_app::AgentError::Policy(Box::new(other)),
    }
}

#[test]
fn indeterminate_policy_mapping_preserves_category_and_source() {
    use std::error::Error;
    let error = policy_error::<std::convert::Infallible>(
        cherry_pit_core::DispatchError::Indeterminate(std::io::Error::other("diagnostic").into()),
    );
    assert_eq!(
        error.category(),
        cherry_pit_core::ErrorCategory::ReconciliationRequired
    );
    assert_eq!(
        error.source().unwrap().source().unwrap().to_string(),
        "diagnostic"
    );
}

pub struct Assembled {
    pub app: App<
        FooGateway,
        InMemoryEventStore<FooEvent>,
        InProcessEventBus<FooEvent>,
        (),
        TracingDeadLetterSink,
    >,
    pub foo_gateway: Arc<FooGateway>,
    pub bar_gateway: Arc<BarGateway>,
}

pub fn assemble() -> Assembled {
    let foo_store = Arc::new(InMemoryEventStore::<FooEvent>::new());
    let bar_store = Arc::new(InMemoryEventStore::<BarEvent>::new());
    let foo_gateway = Arc::new(FooGateway::new(Arc::clone(&foo_store)));
    let bar_gateway = Arc::new(BarGateway::new(Arc::clone(&bar_store)));
    let mut app = App::new(
        FooGateway::new(Arc::clone(&foo_store)),
        InMemoryEventStore::<FooEvent>::new(),
        InProcessEventBus::<FooEvent>::new(),
        (),
        TracingDeadLetterSink::new(),
    );
    let bar_for_policy = Arc::clone(&bar_gateway);
    app.register_policy(
        FooToBarPolicy,
        move |out: FooToBarOutput, _gw: &FooGateway, ctx| {
            let bar = Arc::clone(&bar_for_policy);
            async move {
                let FooToBarOutput::Ping(cmd) = out;
                bar.create(cmd, ctx).await.map(|_| ()).map_err(policy_error)
            }
        },
        "FooToBarPolicy",
        "FooToBarOutput",
    );
    Assembled {
        app,
        foo_gateway,
        bar_gateway,
    }
}
