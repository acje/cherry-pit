# CHE-0021. Closed Public Error Enums

Date: 2026-04-25
Last-reviewed: 2026-09-19
Tier: B
Status: Accepted

## Related

References: CHE-0015

## Context

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
