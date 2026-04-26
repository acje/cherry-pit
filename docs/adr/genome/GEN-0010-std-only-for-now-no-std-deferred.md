# GEN-0010. std-Only for Now — no_std Deferred

Date: 2026-04-25
Last-reviewed: 2026-04-25
Tier: C

## Status

Accepted

Amended 2026-04-25 — no_std claims removed

## Related

- References: GEN-0008

## Context

The original design specified `core` → `alloc` → `std` tiered feature flags for
`no_std` support. However, the implementation used `std::error::Error`,
`std::collections`, and `std::sync` unconditionally. No `#![no_std]` attribute existed.
The `alloc` feature flag was defined in `Cargo.toml` but had no effect — enabling it
did not enable any `no_std` functionality.

No concrete `no_std` consumer existed or was planned.

## Decision

Remove the non-functional `alloc` feature from `Cargo.toml`. Document the crate as
`std`-only. Retain the `std` feature flag (always required) for forward compatibility.
Document the deferred `no_std` tiered model in [genome.md](../../plans/genome.md) §Future:
no_std Support for when an actual consumer exists.

**Previous state (removed):**
```toml
alloc = []  # had no effect — removed
```

**Current state:**
```toml
[features]
default = ["std", "derive"]
std = []
derive = ["dep:pardosa-genome-derive"]
zstd = ["std"]  # Phase 3: will add dep:zstd
```

## Consequences

- **Positive:** No misleading `no_std` claims. Users and dependents get accurate
  capability information.
- **Positive:** Reduces maintenance surface — no need to maintain untested `no_std`
  code paths.
- **Positive:** Design for `no_std` is documented and ready to implement when needed.
- **Negative:** Cannot be used in `no_std` environments until implemented.
- **Migration path:** When a `no_std` consumer exists: add `#![no_std]` attribute,
  gate `std::error::Error` impls, gate collections behind `alloc`, feature-gate
  `String` vs `&'static str` in error types.
