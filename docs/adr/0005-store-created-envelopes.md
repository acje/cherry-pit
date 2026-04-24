# 5. Store-Created Envelopes with Correlation and Causation Tracking

Date: 2026-04-24

## Status

Accepted

## Context

When persisting domain events, metadata must be attached: event ID,
aggregate ID, sequence number, timestamp. Two approaches:

1. **Caller-constructed envelopes** — callers build `EventEnvelope`
   with all metadata before passing to the store. Risk: mismatched
   IDs, wrong sequences, inconsistent timestamps.
2. **Store-constructed envelopes** — callers pass raw `Vec<Event>`,
   the store stamps all metadata. Risk: none — correct by
   construction.

Additionally, distributed event-sourced systems need tracing metadata
to follow causal chains across aggregates and bounded contexts:

- **Correlation ID** groups all events produced by a single logical
  operation (a command and all downstream policy-triggered commands).
- **Causation ID** identifies the specific event that caused this
  event to be produced (via a policy or saga).

Adding these fields after data exists in production requires schema
migration of all persisted envelopes.

## Decision

Callers pass `Vec<Event>` to the store. The store creates
`EventEnvelope<E>` by stamping: `event_id` (UUID v7), `aggregate_id`,
`sequence`, `timestamp`, `correlation_id` (Option), and `causation_id`
(Option). Callers never construct envelopes directly.

Correlation and causation IDs are `Option<Uuid>` — `None` for events
from user-initiated commands without tracing context, `Some` when
propagated through policies or sagas.

## Consequences

- Malformed envelopes are impossible by construction.
- Single timestamp per batch ensures consistency within an atomic
  operation.
- Correlation/causation IDs enable distributed tracing across
  aggregates and bounded contexts without requiring external
  tracing infrastructure.
- The `Option<Uuid>` type means existing event creation (without
  tracing context) requires no changes — fields default to `None`.
- Adding tracing propagation to `CommandBus` and `CommandGateway`
  implementations is a future concern — the envelope schema is ready.
- `EventEnvelope` fields are `pub`, so external construction via
  struct literal syntax is technically possible. The safety guarantee
  relies on convention (only the store constructs envelopes in
  production code).
