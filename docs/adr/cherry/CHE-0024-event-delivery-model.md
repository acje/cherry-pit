# CHE-0024. Event Delivery Model

Date: 2026-04-25
Last-reviewed: 2026-09-19
Tier: C
Status: Accepted

## Related

References: CHE-0004, COM-0025, GND-0001, GND-0005

## Context

EventBus is notification, not commitment. Recovery depends on the selected source retaining committed events and supporting replay with stable identity. Persistence certainty, durability, and notification success are distinct facts; an interrupted response can leave commitment unknown even for an atomic backend.

## Decision

Persist-then-publish with non-fatal delivery:

R1 [7]: Publish committed-event notifications only after persistence
  reports known commitment; retain commit knowledge separately from
  notification failure so failed delivery cannot reclassify committed
  events as rejected
R2 [7]: No subscribe method on the EventBus port trait; subscription
  is implementation-specific
R3 [7]: Recover missed notifications by replay from a retained
  authoritative event source using stable event identity and consumer
  progress; state retention and durability prerequisites for each
  adapter and deployment
R4 [8]: Consumer checkpoints record aggregate_id, last sequence, and
  handler identity after side effects complete successfully
R5 [8]: Failed policy outputs are routed to a dead-letter workflow
  containing event_id, correlation_id, causation_id, and error category
R6 [5]: Persistence ports MUST distinguish known committed, known not
  committed, and indeterminate outcomes; domain rejection is separate,
  and indeterminate operations require reconciliation before retry or
  committed-event publication
R7 [5]: EventStore batch atomicity MUST be an explicit implemented
  capability; adapters lacking all-or-none batch persistence MUST
  reject unsupported operations before writing or expose a separate
  weaker contract

Composition owns handler registration and application policy. Port-only
orchestration may sequence persistence and notification without
selecting backend or retry policy. Notification failure preserves
commit knowledge; recovery and checkpoint guarantees apply only within
declared source retention, durability, and effect-idempotency
capabilities.

## Consequences

+ becomes easier: truthful commit and delivery reporting across
  adapters.
− becomes harder: composition must supply recovery capabilities and
  bounded retry/dead-letter policy.
risks/migration: qualify existing persistence promises before adding
  weaker adapters; no universal exactly-once claim follows.
