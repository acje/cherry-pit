//! # cherry-pit-gateway
//!
//! Infrastructure implementations for cherry-pit port traits.
//!
//! This crate provides concrete implementations of the ports defined
//! in `cherry-pit-core`. Event stores are consumed via `pardosa`
//! (`.pgno`, default backend); the crate's own file-based `MessagePack`
//! event store was retired per CHE-0100 (msgpack-removal-2).
//!
//! ## Governing ADRs
//!
//! - [CHE-0006](https://github.com/Mattilsynet/gh-report/blob/c8507377b2748a015148751ce288be2bad9ec708/docs/adr/cherry/CHE-0006-single-writer-assumption.md) — single-writer assumption
//! - [CHE-0038](https://github.com/Mattilsynet/gh-report/blob/c8507377b2748a015148751ce288be2bad9ec708/docs/adr/cherry/CHE-0038-testing-strategy.md) — testing strategy
//! - [CHE-0043](https://github.com/Mattilsynet/gh-report/blob/c8507377b2748a015148751ce288be2bad9ec708/docs/adr/cherry/CHE-0043-process-level-file-fencing.md) — process-level file fencing
//! - [CHE-0047](https://github.com/Mattilsynet/gh-report/blob/c8507377b2748a015148751ce288be2bad9ec708/docs/adr/cherry/CHE-0047-operational-recovery-runbooks.md) — operational recovery runbooks

#![forbid(unsafe_code)]

mod recovery;

pub use recovery::{StaleLockEvidence, stale_lock_evidence};
