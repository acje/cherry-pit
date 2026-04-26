# CHE-0007. Forbid Unsafe Code

Date: 2026-04-25
Last-reviewed: 2026-04-25
Tier: A

## Status

Accepted

## Related

- Depends on: CHE-0001
- Illustrates: CHE-0001

## Context

Cherry-pit's design priorities place correctness first and security
second. Rust's type system and borrow checker provide memory safety
without garbage collection — but only when `unsafe` blocks are absent.
A single `unsafe` block can violate every guarantee the compiler
provides.

Options:

1. **Allow unsafe where justified** — developers use `unsafe` for FFI,
   performance-critical hot paths, or platform-specific operations.
   Requires auditing every `unsafe` block.
2. **`#![forbid(unsafe_code)]`** — the compiler rejects all `unsafe`
   blocks in the crate. Memory safety is structurally guaranteed, not
   audit-dependent.

Cherry-pit is a framework providing "undifferentiated heavy lifting."
There are no FFI requirements, no SIMD hot paths, and no platform
abstractions that require `unsafe`. Dependencies (tokio, serde, scc)
handle low-level operations.

## Decision

Every cherry-pit crate uses `#![forbid(unsafe_code)]` at the crate
root. No `unsafe` blocks, no `unsafe impl`, no `unsafe fn`. This
applies to all current crates (`cherry-pit-core`, `cherry-pit-gateway`) and all
future crates in the workspace.

Dependencies may use `unsafe` internally — that is their
responsibility and their audit surface. Cherry-pit's own code is
structurally memory-safe.

## Consequences

- Memory safety in cherry-pit code is guaranteed by the compiler, not
  by code review. `forbid` (not `warn` or `deny`) means it cannot be
  overridden with `#[allow]` in inner scopes.
- Performance-sensitive operations that would benefit from `unsafe`
  (e.g., unchecked indexing, `MaybeUninit`) are unavailable. The
  framework relies on the compiler's optimizer and safe abstractions
  (e.g., `scc::HashMap` for lock-free concurrency).
- Future crates added to the workspace must include
  `#![forbid(unsafe_code)]`. This is enforced by convention — Cargo
  does not propagate `forbid` across crate boundaries.
- Third-party dependencies are not covered. Users should audit their
  dependency tree separately (e.g., via `cargo-geiger`).
