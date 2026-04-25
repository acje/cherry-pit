//! pardosa-genome — Binary serialization with zero-copy reads and serde integration.
//!
//! Combines `FlatBuffers`' zero-copy read performance with RON's full algebraic
//! data model. Standard serde with a lightweight `GenomeSafe` marker derive.
//!
//! # Status
//!
//! Phase 1 implementation: crate scaffold, `GenomeSafe` trait, format constants,
//! config types, error catalog.

#![forbid(unsafe_code)]

pub mod config;
pub mod error;
pub mod format;
pub mod genome_safe;

// Re-exports
pub use config::{Compression, DecodeOptions, EncodeOptions, PageClass};
pub use error::{DeError, FileError, SerError};
pub use genome_safe::{GenomeOrd, GenomeSafe, schema_hash_bytes, schema_hash_combine};

// Re-export derive macro when the `derive` feature is enabled.
// Derive macros and traits live in different namespaces — both resolve
// correctly when imported via `use pardosa_genome::GenomeSafe`.
#[cfg(feature = "derive")]
pub use pardosa_genome_derive::GenomeSafe;
