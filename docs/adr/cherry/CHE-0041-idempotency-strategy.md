# CHE-0041. Idempotency Strategy

Date: 2026-04-25
Last-reviewed: 2026-04-25
Tier: B

## Status

Accepted

Amended 2026-04-25 — added COM cross-reference

## Related

- Depends on: CHE-0008, CHE-0017, CHE-0039
- Illustrates: COM-0005

## Context

Cherry-pit has two idempotency requirements today, both enforced by
convention only:

1. **Command handling** (CHE-0008) — `HandleCommand::handle`
   returning `Ok(vec![])` means the command was accepted but no
   state change occurred. This is the idempotent acceptance pattern:
   the aggregate detects from its state that the command is a
   duplicate and produces zero events.
2. **Policy reactions** (CHE-0017) — `Policy::react` must be
   idempotent: "reacting to the same event twice must produce the
   same outputs." Event delivery may be at-least-once (especially
   over NATS), so policies must tolerate replays.

Neither requirement has framework-level enforcement. CHE-0002 says
"make illegal states unrepresentable," but idempotency is a
behavioral invariant — "same input produces same output" — which the
type system cannot structurally enforce for arbitrary functions.

Three layers where idempotency matters:

| Layer | Duplicate source | Current mitigation |
|-------|-----------------|-------------------|
| Command dispatch | User retries, webhook replays | `Ok(vec![])` convention in `handle` |
| Policy reactions | At-least-once event delivery | `react` purity convention |
| Event persistence | Concurrent appends | Optimistic concurrency (`expected_sequence`) |

Additionally, `event_id` (UUID v7, CHE-0033) provides a natural
deduplication key at the event level — every event has a globally
unique identifier.

## Decision

Convention-based idempotency with infrastructure-level support
planned for the `CommandBus`.

### Command-level idempotency

No framework idempotency key. Aggregates are the authority on
whether a command is a duplicate.

The aggregate's state (rebuilt from events) contains enough
information to detect duplicates. Examples:

- `CreateOrder` on an already-created aggregate → `handle` inspects
  state, returns `Err(AlreadyCreated)` or `Ok(vec![])`.
- `ConfirmPayment { payment_id }` when `payment_id` is already
  recorded → `handle` returns `Ok(vec![])`.

This approach is correct because the aggregate is the single source
of truth for its own state. External deduplication tables can go
stale; the aggregate cannot.

### Policy-level idempotency

`react` must be a pure function: same `EventEnvelope` input
produces the same `Vec<Output>`. At-least-once delivery is safe
when:

1. `react` is pure (same input → same output), AND
2. Downstream command handling is idempotent (duplicate commands
   produce zero events).

Together, replaying an event through a policy and re-dispatching
the resulting commands produces no additional state changes. The
system converges to the same state regardless of delivery count.

No deduplication table or processed-event tracking is required at
the policy level if both conditions hold.

### Store-level idempotency

Optimistic concurrency (`expected_sequence` on `append`) prevents
duplicate appends. If two concurrent attempts to append after the
same load race, one succeeds and the other receives
`StoreError::ConcurrencyConflict`. The caller retries with a fresh
load.

UUID v7 `event_id` uniqueness (CHE-0033) prevents exact-duplicate
events: two events produced by different `handle` calls will always
have different `event_id` values. This is not deduplication — it is
identity.

### Future: CommandBus idempotency key

When the `CommandBus` is implemented, it may add optional
idempotency key support for commands entering from external sources
(webhooks, message queues). Design sketch:

```rust
// Optional idempotency key on CommandGateway
fn send_idempotent<C>(
    &self,
    id: AggregateId,
    cmd: C,
    idempotency_key: IdempotencyKey,
    context: CorrelationContext,
) -> impl Future<Output = DispatchResult<Self::Aggregate, C>> + Send
where ...;
```

The bus would check a deduplication store before dispatching. If the
key has been seen, it returns the original result without
re-executing the command. This is infrastructure-level deduplication
layered on top of the domain-level idempotency.

This is deferred until `CommandBus` is built and the external
command ingestion pattern is concrete.

## Consequences

- **Type-system boundary acknowledged.** Idempotency is a
  behavioral invariant. The compiler cannot verify "same input →
  same output" for arbitrary `handle` or `react` implementations.
  Convention + documentation is the pragmatic answer.
- **Aggregate is the idempotency authority.** No external
  deduplication table needed for in-process command dispatch. The
  aggregate's state IS the deduplication state.
- **Policy purity is the key invariant.** If `react` is pure AND
  downstream handling is idempotent, at-least-once delivery is safe
  without policy-level deduplication. Testing should verify purity:
  call `react` twice with the same envelope, assert identical
  outputs (CHE-0038).
- **`event_id` (UUID v7) is a natural deduplication key** for
  event-level operations. External systems receiving events can use
  `event_id` for exactly-once processing.
- **No automatic retry in the framework.** `Command` has no `Clone`
  bound (CHE-0014), so the framework cannot clone and retry a
  command. Retry is the caller's responsibility — the caller
  reconstructs the command and re-dispatches.
