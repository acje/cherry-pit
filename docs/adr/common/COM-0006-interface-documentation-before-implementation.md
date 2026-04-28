# COM-0006. Interface Documentation Before Implementation

Date: 2026-04-25
Last-reviewed: 2026-04-28
Tier: B
Status: Accepted

## Related

References: COM-0001, COM-0002

## Context

Ousterhout (Ch. 13, "Comments Should Describe Things That Aren't Obvious from the Code"; Ch. 15, "Write The Comments First") argues that documentation is a design tool, not a post-hoc annotation. Writing interface comments before implementation forces the author to think about the abstraction — what and why — before getting lost in how. If you cannot describe the abstraction clearly, the abstraction itself is likely unclear. Comments written after implementation tend to restate function names and describe code rather than purpose. Reviewers can evaluate interface design from comments alone.

Cherry-pit's 685-line `cherry-pit-core.md` trait design document was written before implementations existed, describing what each trait does, why it exists, and how traits relate — information not inferable from method signatures.

Red flags include comments restating function names (`/// Returns the aggregate ID` on `fn aggregate_id()`), comments describing implementation details rather than abstractions, and absent comments on public APIs where the explanation of why the interface exists is missing.

## Decision

Interface documentation is written before implementation. Comments
describe the abstraction ("what" and "why"), not the code ("how").

R1 [5]: Write doc comments for public functions, traits, and types
  before implementing them — the comment is a design artifact
R2 [5]: Interface comments answer purpose, context, guarantees,
  requirements, and motivation — never implementation details
R3 [6]: Comments that restate the function name or describe code
  rather than the abstraction must be rewritten
R4 [5]: For complex subsystems, a design document describing
  abstractions and relationships is written before any code

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
