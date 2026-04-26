# COM-0005. Define Errors Out of Existence

Date: 2026-04-25
Last-reviewed: 2026-04-25
Tier: A

## Status

Accepted

## Related

- References: COM-0002

## Context

Ousterhout (Ch. 10, "Define Errors Out of Existence") identifies
exception handling as a major source of software complexity.
Exceptions are disproportionately expensive:

1. **Interface complexity** — every error variant is part of the
   module's interface. Callers must understand and handle each
   variant.
2. **Propagation complexity** — errors propagate through call stacks,
   and each layer must decide: handle, translate, or propagate.
3. **Testing complexity** — error paths require their own test cases,
   often more than the happy path.
4. **Combinatorial explosion** — N operations with M error variants
   each produce N × M potential error states.

Before adding an error variant, the first question should be: "Can
the operation be redefined so that this error does not exist?"

**Techniques for eliminating errors:**

- **Redefine the operation to succeed.** `load` for an unknown
  aggregate returns `Vec::new()` instead of `NotFound` — the
  operation succeeds by defining an empty stream as a valid result
  (CHE-0019).

- **Make the operation infallible.** `Aggregate::apply` returns `()`
  — replay cannot fail because `apply` is a pure state transition,
  not a validation step (CHE-0009).

- **Idempotent semantics.** A duplicate command produces `Ok(vec![])`
  — zero events, not an error. The operation "succeeded" by doing
  nothing (CHE-0041).

- **Exception masking.** Handle the error internally when the module
  can take a reasonable default action. The caller never sees the
  error.

- **Exception aggregation.** Collect multiple potential errors into a
  single operation that either fully succeeds or reports one
  composite failure.

Cherry-pit applies this principle aggressively: `apply()` is
infallible, `load()` returns empty instead of not-found, and
idempotent command handling produces empty event vectors instead of
duplicate-command errors.

## Decision

Before adding an error variant to any `Result` type, demonstrate
that the operation cannot be redefined to succeed. Error elimination
is preferred over error handling.

### Rules

1. **Redefine before handling.** Before writing
   `Err(SomeError::NewVariant)`, ask: "Can this operation be
   redefined so that this condition is a success case?"

2. **Infallible operations where possible.** If an operation's
   failure would make the system unrecoverable (e.g., event replay
   fails), the operation should be infallible. Truly unrecoverable
   conditions use `panic`, not `Result`.

3. **Idempotent semantics over duplicate errors.** When an operation
   is repeated with the same input, returning `Ok` with no side
   effects is preferred over returning a `DuplicateOperation` error.

4. **Error variants require justification.** Each error variant is
   interface complexity. The justification must explain:
   - Why the caller cannot avoid this condition
   - Why the module cannot handle it internally
   - What the caller is expected to do with the error

5. **Exception masking for internal recovery.** When a module can
   handle an error with a reasonable default (retry, fallback, empty
   result), it should mask the exception rather than propagating it.

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
