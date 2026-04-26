//! # cherry-pit-gateway
//!
//! Infrastructure implementations for cherry-pit port traits.
//!
//! This crate provides concrete implementations of the ports defined
//! in `cherry-pit-core`. For development and small deployments, the
//! [`MsgpackFileStore`] persists aggregate event streams as `MessagePack`
//! files on the local filesystem.
//!
//! ## Event store implementations
//!
//! - [`MsgpackFileStore`] — file-based, MessagePack-serialized, default
//!   directory `store/`

#![forbid(unsafe_code)]

mod event_store;

pub use event_store::MsgpackFileStore;
