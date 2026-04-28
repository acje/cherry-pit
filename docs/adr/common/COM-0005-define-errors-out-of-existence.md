# COM-0005. Define Errors Out of Existence

Date: 2026-04-25
Last-reviewed: 2026-04-28
Tier: B
Status: Accepted

## Related

References: COM-0001, COM-0002, COM-0003

## Context

Ousterhout (Ch. 10, "Define Errors Out of Existence") identifies exception handling as a major source of complexity. Every error variant adds interface complexity for callers, propagation decisions for each layer, dedicated test cases, and combinatorial state explosion. Before adding an error variant, the first question should be: "Can the operation be redefined so that this error does not exist?"

Techniques include redefining operations to succeed — `load` for an unknown aggregate returns `Vec::new()` instead of `NotFound` (CHE-0019); making operations infallible — `Aggregate::apply` returns `()` as a pure state transition (CHE-0009); idempotent semantics — duplicate commands produce `Ok(vec![])` rather than errors (CHE-0041); exception masking when the module can take a reasonable default action; and exception aggregation into single composite operations.

Cherry-pit applies this aggressively: `apply()` is infallible, `load()` returns empty instead of not-found, and idempotent command handling produces empty event vectors instead of duplicate-command errors.

## Decision

Before adding an error variant to any `Result` type, demonstrate
that the operation cannot be redefined to succeed. Error elimination
is preferred over error handling.

R1 [5]: Before writing a new error variant, ask whether the operation
  can be redefined so the condition becomes a success case
R2 [5]: Operations whose failure would be unrecoverable must be
  infallible; truly unrecoverable conditions use panic, not Result
R3 [5]: Repeated operations with the same input return Ok with no
  side effects rather than a duplicate-operation error
R4 [6]: Each error variant requires justification explaining why the
  caller cannot avoid it, why the module cannot handle it, and what
  the caller does with it
R5 [5]: When a module can handle an error with retry, fallback, or
  empty result, it masks the exception rather than propagating it

## Consequences

- `StoreError` has no `NotFound` variant — eliminated by redefining
  `load` to return empty vectors (CHE-0019). `NotFound` semantics
  live at the `DispatchError` level where they have meaning.
- `apply()` is infallible — eliminated by design. Corrupt data
  panics because it represents a bug, not a runtime condition
  (CHE-0009).
- Idempotent command handling returns `Ok(vec![])` — eliminated by
  treating duplicate commands as successful no-ops (CHE-0041).
- New error variants in PRs require COM-0005 justification: "Why
  can't this operation be redefined to succeed?"
- This principle creates tension with explicit error reporting
  (CHE-0015: error type per command). The resolution: COM-0005
  minimizes the *number* of error variants; CHE-0015 ensures the
  remaining variants are typed, not stringly-typed. Both reduce
  complexity from different angles.
- Overuse of error elimination can hide real problems. The safeguard
  is rule 2: truly unrecoverable conditions (data corruption, I/O
  failure) must still be reported. The principle targets *eliminable*
  errors, not all errors.
