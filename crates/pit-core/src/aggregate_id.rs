use std::fmt;

use serde::{Deserialize, Serialize};

/// Validated aggregate instance identifier — the stream partition key.
///
/// Identifies a specific aggregate instance within an event store.
/// Each aggregate's event stream is keyed by its `AggregateId`.
/// The `(AggregateId, sequence)` tuple is the globally unique
/// coordinate for any single event.
///
/// # ID assignment
///
/// Aggregate IDs are assigned by the [`EventStore`](crate::EventStore)
/// via its `create` method. The store auto-increments from 1. Callers
/// never invent IDs — they receive them from the store on creation and
/// pass them back on subsequent commands.
///
/// # Single-writer assumption
///
/// Cherry-pit assumes single-writer aggregates: each aggregate instance
/// is owned by exactly one process. This makes sequential `u64` IDs
/// safe without distributed coordination.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct AggregateId(u64);

impl AggregateId {
    /// Create an aggregate ID from a raw `u64`.
    ///
    /// All `u64` values are valid. In practice, IDs are assigned by
    /// the event store starting from 1 — `AggregateId(0)` is never
    /// assigned but is not rejected at the type level.
    #[must_use]
    pub const fn new(id: u64) -> Self {
        Self(id)
    }

    /// Extract the inner `u64` value.
    #[must_use]
    pub const fn into_inner(self) -> u64 {
        self.0
    }
}

impl fmt::Display for AggregateId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<u64> for AggregateId {
    fn from(id: u64) -> Self {
        Self(id)
    }
}

impl From<AggregateId> for u64 {
    fn from(id: AggregateId) -> Self {
        id.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display_outputs_inner_u64() {
        assert_eq!(AggregateId::new(42).to_string(), "42");
        assert_eq!(AggregateId::new(0).to_string(), "0");
        assert_eq!(AggregateId::new(u64::MAX).to_string(), u64::MAX.to_string());
    }

    #[test]
    fn copy_semantics() {
        let id = AggregateId::new(1);
        let copy = id; // Copy, not move
        assert_eq!(id, copy); // original still usable
    }

    #[test]
    fn from_u64() {
        let id: AggregateId = 7u64.into();
        assert_eq!(id.into_inner(), 7);
    }

    #[test]
    fn into_u64() {
        let id = AggregateId::new(99);
        let raw: u64 = id.into();
        assert_eq!(raw, 99);
    }

    #[test]
    fn ord_matches_u64_ordering() {
        let a = AggregateId::new(1);
        let b = AggregateId::new(2);
        let c = AggregateId::new(2);
        assert!(a < b);
        assert_eq!(b, c);
    }

    #[test]
    fn serde_json_roundtrip() {
        let id = AggregateId::new(42);
        let json = serde_json::to_string(&id).unwrap();
        assert_eq!(json, "42");
        let back: AggregateId = serde_json::from_str(&json).unwrap();
        assert_eq!(back, id);
    }

    #[test]
    fn serde_msgpack_roundtrip() {
        let id = AggregateId::new(42);
        let bytes = rmp_serde::to_vec(&id).unwrap();
        let back: AggregateId = rmp_serde::from_slice(&bytes).unwrap();
        assert_eq!(back, id);
    }

    #[test]
    fn hash_consistent() {
        use std::collections::HashSet;
        let mut set = HashSet::new();
        set.insert(AggregateId::new(1));
        set.insert(AggregateId::new(1));
        set.insert(AggregateId::new(2));
        assert_eq!(set.len(), 2);
    }

    #[test]
    fn const_new() {
        // Verify new() is usable in const context.
        const ID: AggregateId = AggregateId::new(1);
        assert_eq!(ID.into_inner(), 1);
    }
}
