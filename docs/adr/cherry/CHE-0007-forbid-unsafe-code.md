# CHE-0007. Forbid Unsafe Code

Date: 2026-04-25
Last-reviewed: 2026-04-28
Tier: B
Status: Accepted

## Related

References: CHE-0001, COM-0017

## Context

Cherry-pit's P1 correctness and P2 security priorities (CHE-0001) demand memory safety. Rust's borrow checker provides this — but only when `unsafe` is absent. A single `unsafe` block can violate every compiler guarantee. Two options: allow unsafe where justified (requires auditing every block) or `#![forbid(unsafe_code)]` (structurally guaranteed, not audit-dependent). Cherry-pit has no FFI, SIMD, or platform abstractions requiring `unsafe`. Dependencies (tokio, serde, scc) handle low-level operations.

## Decision

Every cherry-pit crate uses `#![forbid(unsafe_code)]` at the crate
root. No `unsafe` blocks, no `unsafe impl`, no `unsafe fn`. This
applies to all current crates (`cherry-pit-core`, `cherry-pit-gateway`,
`pardosa`, `pardosa-genome`, `pardosa-genome-derive`, `adr-fmt`) and all
future crates in the workspace.

Dependencies may use `unsafe` internally — that is their
responsibility and their audit surface. Cherry-pit's own code is
structurally memory-safe.

R1 [5]: Every cherry-pit crate uses #![forbid(unsafe_code)] at the
  crate root
R2 [5]: No unsafe blocks, unsafe impl, or unsafe fn in any
  cherry-pit crate
R3 [5]: Every new crate added to the workspace must include
  #![forbid(unsafe_code)]

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
