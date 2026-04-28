# CHE-0028. Compile-Fail Tests as Type Safety Contracts

Date: 2026-04-25
Last-reviewed: 2026-04-28
Tier: B
Status: Accepted

## Related

- References: CHE-0002, CHE-0003, CHE-0005

## Context

Cherry-pit's type system enforces several invariants at compile time
(CHE-0005):

- A `CommandGateway` rejects commands the bound aggregate does not
  handle
- An `EventStore` typed for one event cannot persist a different
  event type
- `HandleCommand<C>` cannot be implemented for a type that does not
  implement `Command`

These guarantees are architectural — they are the reason the framework
uses associated types and trait bounds instead of runtime routing.
But compile-time guarantees are invisible in tests unless explicitly
verified. A refactoring could silently weaken a bound, and no test
would catch it.

Two approaches:

1. **Trust the type system** — if the code compiles, the guarantees
   hold. No explicit verification. Risk: accidental trait bound
   relaxation goes undetected.
2. **Compile-fail tests** — write code that should NOT compile, and
   verify that the compiler rejects it. The `trybuild` crate runs
   these as normal `#[test]` functions. If a refactoring weakens a
   bound, the previously-failing code starts compiling and the test
   fails.

## Decision

Architectural type safety guarantees are verified by `trybuild`
compile-fail tests. Each test is a `.rs` file in
`tests/compile_fail/` that contains code violating one specific
invariant.

Current contracts:

| File | Invariant tested |
|------|------------------|
| `gateway_rejects_unhandled_command.rs` | `CommandGateway::create(DeleteOrder)` fails when `MyAggregate` does not implement `HandleCommand<DeleteOrder>` |
| `wrong_event_store_type.rs` | An `EventStore` typed for `OrderEvent` cannot produce `Vec<EventEnvelope<UserEvent>>` |
| `handle_non_command.rs` | `HandleCommand<NotACommand>` fails when `NotACommand` does not implement `Command` |

Test harness:

```rust
#[test]
fn compile_fail() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/compile_fail/*.rs");
}
```

Dev dependency:

```toml
[dev-dependencies]
trybuild = { workspace = true }
```

## Consequences

- **Invariant regression detection** — if a refactoring accidentally
  removes a trait bound (e.g., dropping `C: Command` from
  `HandleCommand<C>`), the compile-fail test for that bound starts
  passing (the code compiles when it should not), and the test suite
  fails. This catches regressions that no runtime test can detect.
- **Living documentation** — each compile-fail test file documents
  exactly what the type system prevents. New contributors can read
  `tests/compile_fail/` to understand the framework's safety
  guarantees without reading trait definitions.
- **Compiler-version sensitivity** — compile-fail tests match on
  error messages. Compiler upgrades may change error wording, causing
  spurious test failures. `trybuild` handles this reasonably well
  by comparing stderr output, but major Rust version bumps may
  require updating expected error files.
- **One invariant per file** — each compile-fail test verifies exactly
  one type safety contract. This keeps failures precise: a failing
  test identifies exactly which invariant was broken.
- **New safety guarantees should add compile-fail tests** — when a
  new trait bound or associated type constraint is introduced, a
  corresponding compile-fail test should be added. The test suite
  grows with the type safety surface area.
- **These are architectural tests, not unit tests** — they verify
  the framework's compile-time contract with users, not runtime
  behavior. They belong alongside the traits they guard, in
  `cherry-pit-core`'s test suite.
