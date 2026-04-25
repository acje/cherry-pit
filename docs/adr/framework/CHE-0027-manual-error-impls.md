# CHE-0027. Manual Error Trait Implementations in pit-core

Date: 2026-04-25
Last-reviewed: 2026-04-25

## Status

Accepted

## Related

- Depends on: CHE-0001, CHE-0015

## Context

`pit-core` defines three error types: `DispatchError<E>`, `StoreError`,
and `BusError`. Each requires `Display` and `Error` trait
implementations. Two approaches:

1. **`thiserror`** — derive macro that generates `Display` and `Error`
   impls from attributes. Reduces boilerplate. Adds a proc-macro
   dependency. The `thiserror` crate is well-maintained (by dtolnay)
   and widely used.
2. **Manual impls** — hand-written `Display::fmt` and `Error::source`
   implementations. More code, but zero additional dependencies. Full
   control over formatting and error chain behavior.

`pit-core` is the foundation crate — every other cherry-pit crate and
every user crate depends on it. Its dependency tree is a multiplier:
every dependency of `pit-core` becomes a transitive dependency of the
entire ecosystem.

Current `pit-core` dependencies: `serde`, `uuid`, `jiff`. All three
are load-bearing (events must serialize, have IDs, and have
timestamps). `thiserror` would be a fourth dependency added purely for
convenience.

## Decision

`pit-core` uses manual `Display` and `Error` implementations. No
`thiserror` dependency.

```rust
// Example: manual Display for DispatchError<E>
impl<E: Error + Send + Sync> fmt::Display for DispatchError<E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Rejected(e) => write!(f, "command rejected: {e}"),
            Self::AggregateNotFound { aggregate_id } => {
                write!(f, "aggregate not found: {aggregate_id}")
            }
            Self::ConcurrencyConflict { aggregate_id, expected_sequence, actual_sequence } => {
                write!(f, "concurrency conflict on {aggregate_id}: expected sequence {expected_sequence}, actual {actual_sequence}")
            }
            Self::Infrastructure(e) => write!(f, "infrastructure error: {e}"),
        }
    }
}

// Example: manual Error for DispatchError<E>
impl<E: Error + Send + Sync + 'static> Error for DispatchError<E> {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Rejected(e) => Some(e),
            Self::Infrastructure(e) => Some(e.as_ref()),
            Self::AggregateNotFound { .. } | Self::ConcurrencyConflict { .. } => None,
        }
    }
}
```

`thiserror` **is** available in the workspace for infrastructure
crates (`pit-gateway`, `pardosa`, etc.) where dependency count is
less critical:

```toml
# workspace Cargo.toml
thiserror = "2"
```

## Consequences

- **Minimal dependency tree** — `pit-core` depends on exactly three
  external crates: `serde`, `uuid`, `jiff`. Each is load-bearing.
  No convenience-only dependencies.
- **Full control over error formatting** — message strings can be
  tuned without working around derive macro limitations. The
  `DispatchError::ConcurrencyConflict` message includes structured
  data (aggregate_id, expected, actual) in a controlled format.
- **Full control over error chains** — `source()` returns are
  explicit. `DispatchError::Rejected(e)` chains to the domain error;
  `Infrastructure(e)` chains to the boxed error. No derive-macro
  magic deciding which field is the source.
- **More boilerplate** — ~60 lines of manual impls in `error.rs` that
  `thiserror` would reduce to ~20 lines of attributes. Maintenance
  burden increases if new error variants are added.
- **Supply-chain risk reduction** — fewer dependencies means fewer
  potential supply-chain attack vectors. For a foundation crate that
  every user depends on, this is a P2 (security) concern.
- **Infrastructure crates are free to use thiserror** — this decision
  applies only to `pit-core`. `pit-gateway`, `pardosa`, and other
  infrastructure crates can use `thiserror` because their dependency
  trees are less critical (users depend on them directly, not
  transitively through every other crate).
