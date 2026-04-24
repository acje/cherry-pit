//! # pit-core
//!
//! Foundational traits for cherry-pit: the narrow, typed ports that
//! agents program against. All domain logic lives behind these traits.
//! All infrastructure lives on the other side.
//!
//! ## Single-aggregate design
//!
//! Every infrastructure port (`EventStore`, `EventBus`, `CommandBus`,
//! `CommandGateway`) is bound to a single aggregate/event type via
//! associated types. The compiler enforces end-to-end type safety
//! from command dispatch through event persistence and publication.
//!
//! Multiple aggregates are supported at the system level by deploying
//! separate bounded contexts — each with its own typed infrastructure
//! stack. Cross-context communication happens through event
//! subscriptions (e.g. NATS subjects), not shared stores.
//!
//! ## Domain traits
//!
//! - [`DomainEvent`] — immutable facts (something that happened)
//! - [`Command`] — intent to change state
//! - [`Aggregate`] — consistency boundary, reconstructed from events
//! - [`HandleCommand`] — compile-time verified command→aggregate pairs
//! - [`Policy`] — reacts to events by producing commands
//! - [`Projection`] — folds events into read-optimized views
//!
//! ## Port traits (async, RPITIT)
//!
//! - [`CommandGateway`] — primary entry point for dispatching commands
//! - [`CommandBus`] — load → handle → persist → publish lifecycle
//! - [`EventStore`] — persistence of aggregate event streams
//! - [`EventBus`] — fan-out of persisted events
//!
//! ## Types
//!
//! - [`AggregateId`] — stream partition key (auto-assigned `u64`)
//! - [`EventEnvelope`] — infrastructure wrapper around domain events
//! - [`DispatchError`] — typed command dispatch errors
//! - [`DispatchResult`] — return type alias for bus/gateway dispatch
//! - [`CreateResult`] — return type alias for aggregate creation
//! - [`StoreError`] — event store operation errors
//! - [`BusError`] — event bus publication errors

#![forbid(unsafe_code)]

mod aggregate;
mod aggregate_id;
mod bus;
mod command;
mod error;
mod event;
mod gateway;
mod policy;
mod projection;
mod store;

pub use aggregate::{Aggregate, HandleCommand};
pub use aggregate_id::AggregateId;
pub use bus::{CommandBus, EventBus};
pub use command::Command;
pub use error::{BusError, CreateResult, DispatchError, DispatchResult, StoreError};
pub use event::{DomainEvent, EventEnvelope};
pub use gateway::CommandGateway;
pub use policy::Policy;
pub use projection::Projection;
pub use store::EventStore;
