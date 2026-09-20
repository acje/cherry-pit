# CPP-0002. Preserve indeterminate outcomes through neutral consumers

Date: 2026-09-20
Last-reviewed: 2026-09-20
Tier: B
Status: Accepted

## Related

References: CPP-0001

## Context

Accepted narrow decision authority: gh-report `ghr-hxyqs.66.8`, informed by
`ghr-fl0i3` and `ghr-h50lm`. Implementation acceptance requires verification
and independent review. The consumer need in `ghr-hxyqs.66.11.1` extends the
same typed-knowledge contract to storage-native errors under commander authority.
Source-qualified rules mean Mattilsynet/gh-report
at `c8507377b2748a015148751ce288be2bad9ec708`: CHE-0046:R1/R2/R5/R7 and
CHE-0024:R1/R4/R5. Their historical text remains unchanged.

## Decision

R1 [5]: Neutral StoreError, DispatchError, ProjectionError and storage-native
  PersistenceError expose an explicit
  Indeterminate diagnostic source, classified ReconciliationRequired. Unknown
  completion MUST NOT imply safe retry or terminal dead-letter completion.
  Storage uses its own RetryClass without a cherry-pit-core dependency. All
  existing non-indeterminate storage classifications remain unchanged.

R2 [5]: Conversions preserve unknown outcomes. The sequential policy consumer
  stops dependent dispatch and App::run observes its result without requiring
  the shutdown signal first. Ordinary terminal routing and retryable logging
  retain their existing policy. HTTP returns opaque 500/indeterminate without
  Retry-After or replay advice. An accepted merger command whose reply is lost
  is indeterminate; rejection before channel acceptance remains distinguishable.

R3 [5]: This is a narrow producer amendment to source CHE-0046's binary guidance
  and CHE-0024's dead-letter routing. Established persistence still precedes
  publication and completed effects precede checkpointing. CPP-0001:R4's ban on
  invented atomic snapshot/checkpoint guarantees remains in force. There is no
  automatic reconciler, receipt API, new queue, or delivery guarantee.

## Consequences

Closed-enum additions require atomic consumer migration. Existing queue capacity
and sequential worker count remain unchanged; stopping releases buffered work,
not a durable catch-up guarantee. Category is response guidance, not evidence of
uncommitted writes. Outer Pardosa adaptation and known partial-effect carriers
remain the next separately reviewed increment with the original pin retained here.
