# COM-0010. Code Should Be Obvious

Date: 2026-04-26
Last-reviewed: 2026-04-26
Tier: B

## Status

Accepted

## Related

- References: COM-0006

## Context

Ousterhout (Ch. 18, "Code Should Be Obvious") addresses the
complement to documentation: code itself should minimize the need
for explanation. Obvious code can be read quickly and accurately,
with confidence that the reader's first impression is correct.
Non-obvious code requires the reader to consult other files,
documentation, or history to understand what it does.

**Obviousness is reader-relative.** What is obvious to the author
(who has full context) may be obscure to a reader encountering the
code for the first time. The test is not "can the author explain
this?" but "can a qualified reader understand this without
asking?"

**Techniques for obviousness:**

- **Judicious naming** — names that describe the abstraction, not
  the implementation. `correlation_id` is obvious; `cid` requires
  lookup. `aggregate_version` is obvious; `v` is not.

- **Structural clarity** — code that reads in the order things
  happen. Guard clauses before happy paths. Pattern matching that
  exhausts cases visibly. Return values that match the function's
  stated purpose.

- **Avoiding cleverness** — a clever one-liner that saves three
  lines but requires mental unpacking is a net complexity increase.
  The three-line version is often more obvious and thus cheaper for
  the project's total cognitive budget.

- **Type-level communication** — Rust's type system makes many
  invariants obvious: `NonZeroU64` communicates "never zero" without
  documentation. `()` return communicates infallibility without a
  comment. `#[non_exhaustive]` communicates "more variants may
  come."

Cherry-pit leverages type-level obviousness extensively:

- `AggregateId(NonZeroU64)` — the type name and inner type
  together communicate identity semantics and the zero exclusion.
- `apply(&mut self, event: &E)` returning `()` — infallibility
  is obvious from the signature (CHE-0009).
- `#[non_exhaustive]` on error types — future extensibility is
  visible in the definition (CHE-0021).
- Exhaustive `match` on domain events — the compiler enforces
  that all cases are handled, making the event-handling logic
  visibly complete.

## Decision

Code should be understandable on first reading by a qualified
developer who has not seen it before. Prefer clarity over brevity,
explicitness over cleverness.

### Rules

1. **Name for the reader.** Choose names that a reader unfamiliar
   with the implementation would understand. Full words over
   abbreviations. Domain terms over generic names. If a name
   requires a comment to explain it, the name is wrong.

2. **Let types speak.** Use Rust's type system to make invariants
   visible: newtypes for domain concepts, `NonZero*` for exclusion
   constraints, `()` for infallible operations, `#[non_exhaustive]`
   for extensibility contracts. A well-typed signature reduces the
   need for prose documentation.

3. **Avoid reader surprises.** Code that does something unexpected
   — a function with side effects not implied by its name, a match
   arm that handles a case differently from the pattern, an early
   return buried in a long function — violates the reader's
   expectations. Structure code so the first impression is accurate.

4. **Prefer straightforward over clever.** If two implementations
   produce the same result but one requires the reader to mentally
   simulate complex control flow, choose the straightforward one.
   The optimization is not worth the cognitive cost unless profiling
   justifies it.

5. **Red flags for non-obvious code:**
   - A function that requires reading its callers to understand
     its purpose
   - A variable whose meaning changes over its lifetime
   - A conditional whose branches are not obviously different
   - Magic numbers or string literals without named constants
   - Implicit ordering dependencies between statements

## Consequences

- Type-driven design (CHE-0002: make illegal states
  unrepresentable, CHE-0003: compile-time error preference) is
  a direct application: the type system makes constraints
  obvious to both the compiler and the reader.
- Naming conventions established in COM-0009 (consistency) and
  COM-0006 (documentation) are reinforced: consistent names are
  obvious names; good documentation explains what names cannot.
- Code review can cite COM-0010 when a change introduces
  non-obvious constructions. The bar is: "Would a new team member
  understand this on first reading?"
- Tension with conciseness: Rust idioms like iterator chains and
  pattern destructuring are concise and idiomatic but may not be
  obvious to developers from other languages. The resolution: Rust
  idioms are obvious to the target audience (qualified Rust
  developers). Non-idiomatic cleverness is not.
- Tension with performance: obvious code is not always the fastest.
  When performance requires non-obvious code, the COM-0006 rule
  applies: document why the non-obvious approach is necessary.
