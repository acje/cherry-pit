# CHE-0021. Non-Exhaustive Error Types for Semver Safety

Date: 2026-04-25
Last-reviewed: 2026-04-25
Tier: B
Status: Accepted

## Related

References: CHE-0001, CHE-0015, CHE-0022

## Context

Cherry-pit is a framework. Downstream users `match` on error types
returned by the infrastructure ports. If a new error variant is added
to a public enum in a minor version, all downstream `match` statements
break — this is a semver-breaking change.

Rust provides `#[non_exhaustive]` to address this: it forces external
callers to include a wildcard arm (`_ =>`) in their `match`, allowing
new variants to be added without breaking compilation.

## Decision

All public error types in `cherry-pit-core` are `#[non_exhaustive]`:

- `DispatchError<E>` — enum with `Rejected`, `AggregateNotFound`,
  `ConcurrencyConflict`, `Infrastructure` variants.
- `StoreError` — enum with `ConcurrencyConflict`, `Infrastructure`,
  `StoreLocked` variants. (`StoreLocked` added by CHE-0043.)
- `BusError` — struct wrapping `Box<dyn Error>`. `#[non_exhaustive]`
  prevents external pattern matching on the struct fields.

New variants (e.g., `RateLimited`, `Timeout`, `SchemaVersionMismatch`)
can be added in minor versions.

R1 [5]: All public error types in cherry-pit-core are
  #[non_exhaustive]
R2 [5]: New error variants may be added in minor versions without
  breaking downstream callers

## Consequences

- Downstream callers must use wildcard arms in `match` statements on
  `DispatchError` and `StoreError`. This is slightly less ergonomic
  but enables safe API evolution.
- Within the `cherry-pit-core` crate, exhaustive matching is still allowed —
  `#[non_exhaustive]` only affects external callers.
- `BusError` is a newtype struct with a private field, so external
  construction was already impossible. `#[non_exhaustive]` adds the
  additional constraint that external code cannot destructure it in
  patterns, maintaining consistency with the enum error types.
- `#[derive(Debug)]` is the only derive on error types. `Clone` and
  `PartialEq` are not derived because `Box<dyn Error>` prevents them.
  Manual `Display` and `Error` impls are used instead of `thiserror`
  to avoid a dependency in the foundational crate.
