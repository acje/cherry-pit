# CHE-0039. Correlation Context Propagation

Date: 2026-04-25
Last-reviewed: 2026-04-25

## Status

Accepted

## Related

- Depends on: CHE-0016, CHE-0004, CHE-0017
- Informs: CHE-0040, CHE-0041, CHE-0042

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
   `pit-agent` composition layer. Does not solve the `EventStore`
   API gap.

## Decision

Explicit parameter propagation. A `CorrelationContext` struct is
added to `pit-core` and threaded through all port methods that
produce `EventEnvelope`s.

### Type definition

```rust
/// Context for correlating events across aggregates and bounded
/// contexts.
///
/// Passed explicitly through CommandGateway → CommandBus → EventStore.
/// The store stamps these values onto every EventEnvelope it creates.
///
/// Does not implement Default — callers must explicitly choose
/// `CorrelationContext::none()`, `::correlated(id)`, or
/// `::new(corr, cause)`. The name communicates intent.
#[derive(Debug, Clone)]
pub struct CorrelationContext {
    /// Groups related events into a single logical operation.
    pub correlation_id: Option<uuid::Uuid>,
    /// The event_id of the event that caused this command.
    pub causation_id: Option<uuid::Uuid>,
}

impl CorrelationContext {
    /// No correlation context — user-initiated command, no tracing.
    pub fn none() -> Self {
        Self { correlation_id: None, causation_id: None }
    }

    /// Full correlation context — typically propagated from a policy
    /// reacting to a prior event.
    pub fn new(correlation_id: uuid::Uuid, causation_id: uuid::Uuid) -> Self {
        Self {
            correlation_id: Some(correlation_id),
            causation_id: Some(causation_id),
        }
    }

    /// Correlation only — first command in a logical operation chain,
    /// no causation event yet.
    pub fn correlated(correlation_id: uuid::Uuid) -> Self {
        Self {
            correlation_id: Some(correlation_id),
            causation_id: None,
        }
    }
}
```

`CorrelationContext` does not implement `Default`. Every call site
must explicitly choose `none()`, `correlated(id)`, or
`new(corr, cause)`. This forces callers to think about correlation
at every dispatch point — forgetting correlation is a conscious
omission, not an accidental default.

### Port trait signature changes

```rust
// EventStore::create — adds context parameter
fn create(
    &self,
    events: Vec<Self::Event>,
    context: CorrelationContext,
) -> impl Future<Output = Result<(...), StoreError>> + Send;

// EventStore::append — adds context parameter
fn append(
    &self,
    id: AggregateId,
    expected_sequence: NonZeroU64,
    events: Vec<Self::Event>,
    context: CorrelationContext,
) -> impl Future<Output = Result<Vec<EventEnvelope<Self::Event>>,
                                 StoreError>> + Send;

// CommandBus::create — adds context parameter
fn create<C>(
    &self,
    cmd: C,
    context: CorrelationContext,
) -> impl Future<Output = CreateResult<Self::Aggregate, C>> + Send
where ...;

// CommandBus::dispatch — adds context parameter
fn dispatch<C>(
    &self,
    id: AggregateId,
    cmd: C,
    context: CorrelationContext,
) -> impl Future<Output = DispatchResult<Self::Aggregate, C>> + Send
where ...;

// CommandGateway::create — adds context parameter
fn create<C>(
    &self,
    cmd: C,
    context: CorrelationContext,
) -> impl Future<Output = CreateResult<Self::Aggregate, C>> + Send
where ...;

// CommandGateway::send — adds context parameter
fn send<C>(
    &self,
    id: AggregateId,
    cmd: C,
    context: CorrelationContext,
) -> impl Future<Output = DispatchResult<Self::Aggregate, C>> + Send
where ...;
```

### Propagation flow

```
User/Adapter
  │
  │ gateway.send(id, cmd, CorrelationContext::correlated(op_id))
  ▼
CommandGateway::send
  │
  │ passes context to bus
  ▼
CommandBus::dispatch
  │
  │ passes context to store
  ▼
EventStore::append(id, seq, events, context)
  │
  │ build_envelopes stamps correlation_id and causation_id
  ▼
EventEnvelope { correlation_id: context.correlation_id,
                causation_id: context.causation_id, ... }
  │
  │ published to EventBus
  ▼
Policy::react(&self, &EventEnvelope)
  │
  │ policy reads envelope.event_id and envelope.correlation_id
  │ constructs CorrelationContext::new(envelope.correlation_id, envelope.event_id)
  ▼
CommandGateway::send(target_id, cmd, context)
  │ ... chain continues with preserved correlation
```

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
