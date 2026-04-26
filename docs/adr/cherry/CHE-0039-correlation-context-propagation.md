# CHE-0039. Correlation Context Propagation

Date: 2026-04-25
Last-reviewed: 2026-04-25
Tier: A

## Status

Accepted

## Related

- Depends on: CHE-0016, CHE-0004, CHE-0017

## Context

`EventEnvelope` carries `correlation_id` and `causation_id` fields
(CHE-0016). These fields enable distributed tracing across aggregates
and bounded contexts. However, the current API makes it structurally
impossible to populate them:

- `EventStore::create(events: Vec<Self::Event>)` — no correlation
  parameter.
- `EventStore::append(id, expected_sequence, events)` — no
  correlation parameter.
- `MsgpackFileStore::build_envelopes` hardcodes both fields to
  `None`.
- `CommandBus` and `CommandGateway` trait signatures carry no
  correlation context.

The envelope schema is "tracing-ready" (CHE-0016) but the port
traits are not. The fields exist but are structurally dead.

The `tracing` crate is a workspace dependency but has zero usage in
source code. `tracing` Spans and `correlation_id` serve overlapping
but distinct purposes: Spans are process-local diagnostic context;
`correlation_id` is cross-process, persisted causal context.

Three propagation styles were considered:

1. **Explicit parameter** — add a `CorrelationContext` struct as a
   parameter to every port method that produces envelopes. Transparent,
   verbose, no magic. Consistent with CHE-0003 (compile-time
   preference) and CHE-0001 (P1 correctness).
2. **Task-local context** — use `tokio::task_local!` to propagate
   correlation IDs implicitly through the async runtime. Ergonomic
   but invisible — violates the "no magic" philosophy. Callers cannot
   see from the type signature that correlation is expected.
3. **Middleware/interceptor** — the `CommandGateway` injects
   correlation context as a cross-cutting concern. Deferred to
   `cherry-pit-agent` composition layer. Does not solve the `EventStore`
   API gap.

## Decision

Explicit parameter propagation. A `CorrelationContext` struct is
added to `cherry-pit-core` and threaded through all port methods that
produce `EventEnvelope`s.

### Type definition

`CorrelationContext` is a struct in `cherry-pit-core` with two fields:
`correlation_id: Option<uuid::Uuid>` (groups related events) and
`causation_id: Option<uuid::Uuid>` (the event_id that caused this
command). Three named constructors: `none()` (user-initiated, no
tracing), `correlated(id)` (first command in a chain), and
`new(corr, cause)` (policy-propagated from a prior event). Does not
implement `Default`. See `cherry-pit-core/src/correlation.rs`.

`CorrelationContext` does not implement `Default`. Every call site
must explicitly choose `none()`, `correlated(id)`, or
`new(corr, cause)`. This forces callers to think about correlation
at every dispatch point — forgetting correlation is a conscious
omission, not an accidental default.

### Port trait signature changes

All port trait methods that produce `EventEnvelope`s gain a
`context: CorrelationContext` parameter: `EventStore::create`,
`EventStore::append`, `CommandBus::create`, `CommandBus::dispatch`,
`CommandGateway::create`, `CommandGateway::send`. See
`cherry-pit-core/src/lib.rs` for full trait definitions.

### Propagation flow

Propagation is linear: adapter calls `CommandGateway::send` with a
`CorrelationContext`, which passes it to `CommandBus::dispatch`, which
passes it to `EventStore::append`. The store stamps `correlation_id`
and `causation_id` from the context onto each `EventEnvelope`. After
publication via `EventBus`, a `Policy::react` handler reads the
envelope's `event_id` and `correlation_id` to construct a new
`CorrelationContext::new(envelope.correlation_id, envelope.event_id)`
for downstream commands, preserving the causal chain.

### Relationship to tracing

`correlation_id` and `tracing::Span` serve different purposes:

| Concern | `correlation_id` | `tracing::Span` |
|---------|------------------|-----------------|
| Scope | Cross-process, persisted in events | Process-local, ephemeral |
| Lifetime | Permanent (stored in event log) | Request duration |
| Purpose | Causal chain reconstruction | Diagnostic logging |

A future integration could set `correlation_id` as a `tracing` span
field for bridging, but this is not required by this ADR.

## Consequences

- **Breaking change** to `EventStore`, `CommandBus`, and
  `CommandGateway` trait signatures. Acceptable pre-1.0. All
  existing implementations and call sites must add the
  `CorrelationContext` parameter.
- `MsgpackFileStore::build_envelopes` gains a `&CorrelationContext`
  parameter and stamps `correlation_id`/`causation_id` from it
  instead of hardcoding `None`.
- Existing tests that call `create`/`append` must add
  `CorrelationContext::none()` at each call site. Mechanical
  migration.
- The `CorrelationContext::none()` call is visible in code — a
  reviewer can immediately see that correlation was intentionally
  omitted, not accidentally forgotten.
- Policy implementations are responsible for constructing the
  `CorrelationContext` for downstream commands. The framework
  provides the types; the policy provides the values.
- `EventBus::publish` does NOT gain a context parameter — it
  receives already-stamped envelopes. Correlation is stamped at
  persistence time, not at publication time.
