# COM-0010. Code Should Be Obvious

Date: 2026-04-26
Last-reviewed: 2026-04-26
Tier: B
Status: Accepted

## Related

References: COM-0001, COM-0006

## Context

Ousterhout (Ch. 18, "Code Should Be Obvious") addresses the complement to documentation: code itself should minimize the need for explanation. Obvious code can be read quickly with confidence that the reader's first impression is correct. The test is not "can the author explain this?" but "can a qualified reader understand this without asking?" — obviousness is reader-relative.

Techniques include judicious naming (abstractions, not implementations — `correlation_id` not `cid`), structural clarity (guard clauses before happy paths, exhaustive pattern matching), avoiding cleverness (a three-line version that reads clearly beats a clever one-liner), and type-level communication where Rust's type system makes invariants visible without documentation.

Cherry-pit leverages type-level obviousness: `AggregateId(NonZeroU64)` communicates identity semantics and zero exclusion; `apply` returning `()` makes infallibility obvious (CHE-0009); `#[non_exhaustive]` on error types signals future extensibility (CHE-0021); and exhaustive `match` on domain events makes handling visibly complete.

## Decision

Code should be understandable on first reading by a qualified
developer who has not seen it before. Prefer clarity over brevity,
explicitness over cleverness.

R1 [5]: Choose names a reader unfamiliar with the implementation
  would understand; full words over abbreviations, domain terms
  over generic names
R2 [5]: Use Rust's type system to make invariants visible — newtypes
  for domain concepts, NonZero for exclusion constraints, unit
  return for infallible operations
R3 [6]: Code must not surprise the reader; side effects not implied
  by a name, hidden early returns, and implicit ordering
  dependencies are prohibited
R4 [5]: When two implementations produce the same result, choose the
  straightforward one over the clever one unless profiling justifies
  the complexity

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
