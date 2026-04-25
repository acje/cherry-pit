# Agent guidance: customizing cherry-pit

Prescriptive guidance for agents and developers building domain
systems on cherry-pit. These rules derive from the framework's design
priorities (CHE-0001) and type-safety principles (CHE-0002, CHE-0003).

## Make illegal states unrepresentable

When defining domain types, encode invariants in the type system.

### Do

- **Newtypes for domain identifiers** — `struct OrderId(NonZeroU64)`,
  not raw `u64`. Prevents mixing order IDs with customer IDs at
  compile time.
- **Exhaustive enums for domain states** — `enum OrderStatus { Pending,
  Confirmed, Shipped, Delivered }`, not `String`. The compiler forces
  handling every state.
- **`NonZero*` for quantities** — `NonZeroU32` for line item counts.
  Zero items is unrepresentable.
- **Validated constructors** — `fn new(email: &str) -> Result<Email,
  InvalidEmail>`. Invalid emails cannot exist as `Email` values.
- **Separate types for lifecycle phases** — `DraftOrder` vs
  `ConfirmedOrder` when the available operations differ significantly.

### Don't

- Use `String` where an enum captures the domain vocabulary.
- Use `Option<T>` where absence is never valid after construction.
- Use `bool` flags where an enum with named variants is clearer
  (`enum Visibility { Public, Private }` over `is_public: bool`).
- Use `u64` directly where a newtype prevents confusion with other
  numeric values.
- Use `#[non_exhaustive]` on domain event enums — exhaustive matching
  in `apply` is required (CHE-0009, CHE-0022).

## Prefer compile-time errors

When defining APIs and constraints, make the compiler do the checking.

### Do

- **One `HandleCommand<C>` impl per command** — the compiler verifies
  each command-aggregate pair. No runtime dispatch table.
- **`where` clauses on public functions** — callers see constraints in
  the signature: `where A: HandleCommand<C>, C: Command`.
- **Associated types over generic parameters** — fix the relationship
  per instance, not per call. `EventStore::Event` is one type, not
  a generic parameter on every method.
- **Domain errors per command** (CHE-0015) — callers match on
  `ShipOrderError::NotConfirmed`, not `Box<dyn Error>`.

### Don't

- Use `Box<dyn Any>` or `Box<dyn Error>` where a concrete type is
  known.
- Use runtime `assert!` where a type constraint can prevent the
  condition.
- Use `match` with `_ =>` wildcard on domain types — handle every
  variant explicitly.
- Use `String` error messages where a typed error enum captures the
  failure modes.

## Decision checklist

When making a design choice during framework customization:

1. Does this type prevent construction of invalid values?
2. Does this API catch misuse at compile time?
3. Does this error surface in `cargo check`, not in production?
4. Can the compiler enforce this constraint, even if it adds verbosity?
5. Is the runtime check defense-in-depth, not primary enforcement?

If the answer to 1-4 is "no" and can be "yes" with reasonable effort,
redesign the type or API.
