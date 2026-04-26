# CHE-0040. Saga and Compensation Patterns (Deliberate Deferral)

Date: 2026-04-25
Last-reviewed: 2026-04-25
Tier: B

## Status

Accepted

## Related

- References: CHE-0017, CHE-0024, CHE-0037, CHE-0039

## Context

`Policy` (CHE-0017) reacts to a single event by producing zero or
more commands. This is the choreography pattern: each policy reacts
independently, with no coordinator tracking multi-step progress.

Event-sourced systems often need multi-step processes where step N
depends on step N-1 succeeding. Two coordination patterns exist:

1. **Choreography** — policies react independently to events.
   Compensation is modeled as domain events (e.g., `PaymentFailed`
   → policy → `CancelOrder`). No central coordinator. Each
   participant knows only its own role.
2. **Orchestration** — a saga coordinator (process manager) tracks
   step completion and issues compensation commands on failure.
   Central state machine. Each participant is directed by the
   coordinator.

Cherry-pit's `Policy` trait is a choreography primitive. No saga
coordinator, no step tracking, no automatic compensation exists in
the framework. The question is whether to add orchestration support
now.

Three considerations argue for deferral:

1. **`CommandBus` is unbuilt.** Sagas require dispatching commands
   and observing their outcomes. Without a `CommandBus`
   implementation, saga coordination cannot be tested end-to-end.
2. **`cherry-pit-agent` is unbuilt.** The composition layer that wires
   policies to buses determines how multi-step processes are
   configured. Saga design without `cherry-pit-agent` is speculative.
3. **`CorrelationContext` (CHE-0039) is new.** Saga step tracking
   requires correlation IDs to group related events. The propagation
   mechanism must be validated in practice before building on it.

## Decision

Deliberate deferral. Saga orchestration is out of scope for
cherry-pit pre-1.0.

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
- Complex business processes require careful policy graph design.
  Users must reason about failure paths manually.
- The choreography-first approach is consistent with event-sourcing
  philosophy: events are facts, policies react to facts, and
  compensation is itself a fact (a domain event).
- Dead-letter handling for failed policy outputs is the most likely
  near-term need. When `CommandBus` is built, it must decide what
  happens when a policy-triggered command is rejected by the target
  aggregate.

### Revisit criteria

Add saga orchestration when any of these conditions are met:

1. `cherry-pit-agent` is built and policy wiring complexity is concrete —
   the composition layer determines how multi-step processes are
   configured and monitored.
2. A user reports a multi-step process that cannot be decomposed
   into independent policy reactions without unreasonable complexity.
3. Dead-letter handling is needed for failed policy output commands
   — the `CommandBus` implementation reveals the failure modes.
4. `CorrelationContext` (CHE-0039) has been validated in practice
   and step tracking can build on it.

When revisiting, the saga design should:

- Define a `ProcessManager` trait with associated `State` type
  (the step tracking state machine).
- Use `CorrelationContext` to group all events in a saga instance.
- Support configurable compensation strategies (retry, compensate,
  ignore) per step.
- Handle timeout via `jiff::Timestamp` comparisons, not wall-clock
  timers.
