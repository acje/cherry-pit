# CHE-0049. Closed Public Error Enums Proposal

Date: 2026-09-19
Last-reviewed: 2026-09-19
Tier: B
Status: Proposed

## Related

References: CHE-0021, CHE-0015, COM-0021

## Context

The accepted policy is the opposite of this proposal and remains in force: CHE-0021 makes the public error types in cherry-pit-core non_exhaustive, and COM-0021 R1 supplies the general accepted public-enum rule. Current code matches the accepted policy, with non_exhaustive on DispatchError, StoreError, BusError, and EnvelopeError; ErrorCategory is already closed. This document records the competing proposal so its ideas survive review without being emitted as current constraints, and it takes effect only if a future decision supersedes CHE-0021 and reconciles COM-0021 R1.

## Decision

Proposed only. Nothing below is current authority for any crate.

R1 [5]: Public error enums in cherry-pit-core would be closed enumerations without non_exhaustive, letting downstream callers match every represented failure state
R2 [5]: Public error variant additions, removals, and semantic changes would be treated as breaking API changes with explicit consumer integration
R3 [5]: ErrorCategory would stay diagnostic classification separate from commit knowledge, so retry decisions account for the operation outcome rather than category alone

## Consequences

+ becomes easier: downstream exhaustiveness checks would expose unhandled error states at compile time.
− becomes harder: every variant change would require coordinated consumer updates, and COM-0021 R1 would need an explicit carve-out or amendment.
risks/migration: adopting this requires superseding CHE-0021, repairing its inbound citations, removing the four non_exhaustive attributes, and reviewing wildcard-only recovery paths in consumers. The enum rule does not change opaque struct construction guarantees.
