# CHE-0021. Closed Public Error Enums

Date: 2026-04-25
Last-reviewed: 2026-09-19
Tier: B
Status: Proposed

## Related

References: CHE-0015

## Context

Current behavior does not yet implement this decision: the public error
enums in cherry-pit-core carry #[non_exhaustive] (crates/cherry-pit-core/
src/error.rs lines 47, 111, 194, 234), and CHE-0015 remains the governing
accepted decision until a conformance change lands. The rules below are
therefore Proposed.

Library callers need exhaustive recovery decisions. Complete public
enum variants make an unhandled recovery state a compile-time error;
variant evolution is an explicit breaking API change rather than a
wildcard fallback.

## Decision

R1 [5]: Public error enums in cherry-pit-core MUST be closed
  enumerations without #[non_exhaustive], allowing downstream callers
  to match every represented failure state.
R2 [5]: Treat public error variant additions, removals, and semantic
  changes as breaking API changes with explicit consumer integration.
R3 [5]: Keep ErrorCategory as diagnostic classification, separate from
  commit knowledge; retry decisions MUST account for the operation
  outcome rather than category alone.

## Consequences

+ becomes easier: downstream exhaustiveness checks expose unhandled
  error states.
− becomes harder: variant changes require coordinated consumer updates.
risks/migration: replace wildcard-only recovery paths and review
  affected consumers; this enum rule does not change opaque struct
  construction guarantees.
