# COM-0006. Interface Documentation Before Implementation

Date: 2026-04-25
Last-reviewed: 2026-04-25
Tier: C

## Status

Accepted

## Related

- Depends on: COM-0001

## Context

Ousterhout (Ch. 13, "Comments Should Describe Things That Aren't
Obvious from the Code"; Ch. 15, "Write The Comments First") argues
that documentation is a design tool, not a post-hoc annotation.
Writing interface comments before implementation forces the author
to think about the abstraction — what the module does and why —
before getting lost in how it does it.

**Documentation-first benefits:**

1. **Design clarity** — if you cannot describe the abstraction in a
   clear comment, the abstraction is likely unclear. Writing the
   comment first surfaces design problems before code is written.

2. **Interface stability** — comments written after implementation
   tend to describe the code rather than the abstraction. They
   restate the function name, describe parameters by type rather than
   purpose, and omit the "why."

3. **Review quality** — reviewers can evaluate the interface design
   from the comments alone, without reading the implementation. If
   the comments are unclear, the interface is unclear.

Cherry-pit's 685-line `cherry-pit-core.md` trait design document was written
before the trait implementations existed. It describes what each
trait does, why it exists, and how traits relate — information that
cannot be inferred from method signatures.

**Red flags for bad documentation:**

- Comment restates the function name: `/// Returns the aggregate ID`
  on `fn aggregate_id() -> AggregateId`. This adds no information.
- Comment describes the implementation: `/// Uses a HashMap to look
  up...`. This belongs in the implementation, not the interface.
- Comment is absent on a public API. If the interface is
  self-documenting, the comment should explain why it exists, not
  what it does.

## Decision

Interface documentation is written before implementation. Comments
describe the abstraction ("what" and "why"), not the code ("how").

### Rules

1. **Write interface comments first.** Before implementing a public
   function, trait, or type, write the doc comment describing its
   purpose, preconditions, postconditions, and semantics. The
   comment is a design artifact, not an afterthought.

2. **Comments describe abstractions.** Interface comments answer:
   - What does this do? (purpose)
   - When should it be used? (context)
   - What are the guarantees? (postconditions)
   - What are the requirements? (preconditions)
   - Why does it exist? (motivation)

3. **Comments do not describe code.** Implementation details belong
   in inline comments within the function body, not in interface
   documentation. If an interface comment mentions data structures,
   algorithms, or internal state, it is leaking the abstraction.

4. **Restate-the-name comments must be rewritten.** If a doc comment
   could be mechanically generated from the function signature, it
   adds no value. Rewrite it to describe the abstraction.

5. **Design documents precede implementation.** For complex
   subsystems (trait hierarchies, infrastructure ports), a design
   document describing the abstractions and their relationships is
   written before any code. The document is the design; the code is
   the implementation.

## Consequences

- Public APIs without doc comments are flagged in review. The
  missing comment indicates either a missing design step or an
  interface that has not been thought through.
- Design documents like `cherry-pit-core.md` are first-class artifacts, not
  optional supplements. They capture design intent that source code
  cannot express.
- Comments that restate function names are refactored during review
  with a COM-0006 citation.
- This principle supports Rust's `#![warn(missing_docs)]` lint. The
  lint catches missing comments; COM-0006 ensures the comments that
  exist are meaningful.
- Over-documentation is a risk: comments that describe every
  implementation detail create maintenance burden without
  proportional value. The "abstraction, not code" rule mitigates
  this — implementation comments are inline and co-located, not part
  of the public interface.
