# CHE-0040. Saga and Compensation Patterns (Deliberate Deferral)

Date: 2026-04-25
Last-reviewed: 2026-04-25
Tier: B
Status: Accepted

## Related

References: CHE-0001, CHE-0017, CHE-0024, CHE-0037, CHE-0039

## Context

`Policy` (CHE-0017) reacts to a single event — the choreography pattern. Orchestration-style sagas use a coordinator tracking step completion and issuing compensation on failure. Cherry-pit's `Policy` has no saga coordinator, step tracking, or automatic compensation. Three considerations argue for deferral: `CommandBus` is unbuilt, `cherry-pit-agent` is unbuilt (saga design without composition layer is speculative), and `CorrelationContext` (CHE-0039) must be validated first.

## Decision

Deliberate deferral. Saga orchestration is out of scope for
cherry-pit pre-1.0.

R1 [5]: Use Policy::react for choreography-style coordination only;
  no saga orchestrator exists pre-1.0
R2 [5]: Model compensation as domain events reacted to by policies,
  not as automatic framework-level rollback

**What cherry-pit provides today:**

- `Policy::react` for choreography-style coordination.
- `CorrelationContext` (CHE-0039) for grouping related events across
  aggregates.
- Domain-level compensation: aggregates emit failure events (e.g.,
  `PaymentFailed`), and policies react to those events with
  compensating commands (e.g., `CancelOrder`).

**What cherry-pit does not provide:**

- No saga coordinator / process manager type.
- No step tracking or completion state machine.
- No automatic compensation on downstream command failure.
- No dead-letter handling for failed policy output commands.
- No timeout mechanism for steps that never complete.

## Consequences

- Framework stays minimal. Users own compensation logic entirely.
- Choreography-first is consistent with event-sourcing philosophy: events are facts, policies react to facts, compensation is itself a domain event.
- Dead-letter handling for failed policy outputs is the most likely near-term need — when `CommandBus` is built, it must decide what happens when a policy-triggered command is rejected.
- Revisit when: `cherry-pit-agent` is built, multi-step processes cannot decompose into independent policy reactions, or `CorrelationContext` (CHE-0039) has been validated in practice.
