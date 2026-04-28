# PAR-0015. Consumer Delivery Semantics and Acknowledgment Policy

Date: 2026-04-28
Last-reviewed: 2026-04-28
Tier: B
Status: Proposed

## Related

References: PAR-0004, PAR-0007, PAR-0008, PAR-0013

## Context

Pardosa's publish path is rigorously specified (PAR-0007, PAR-0008) but
the consume path has zero ADRs. JetStream supports AckExplicit, AckAll,
and AckNone — each with different failure modes. Without explicit policy,
consumers choose ad-hoc: AckNone loses events on crash, AckAll creates
head-of-line blocking. An event published but never consumed is data loss.

1. **AckExplicit + bounded redelivery** — per-event ACK, dead-letter on
   exhaustion. Highest safety.
2. **AckAll + checkpoint** — batch ACK, reprocess from checkpoint on crash.
3. **AckNone** — fire-and-forget.

Option 1 chosen per CHE-0001 P1 (correctness-first).

## Decision

All Pardosa consumers use JetStream AckExplicit pull subscriptions
with bounded redelivery and a dead-letter subject for poison messages.

R1 [5]: All JetStream consumers use AckExplicit acknowledgment policy
  with pull-based subscriptions for explicit flow control
R2 [5]: Configure MaxDeliver on each consumer to bound redelivery
  attempts and route exhausted messages to a dead-letter subject
R3 [5]: Track consumer position via JetStream durable consumer name
  so subscriptions resume from the last acknowledged sequence on restart
R4 [8]: Consumers acknowledge each event only after successful
  processing to prevent data loss on crash

## Consequences

- **Publish-consume symmetry.** Both sides now have explicit correctness
  contracts: PAR-0007/PAR-0008 for publish, PAR-0015 for consume.
- **Dead-letter visibility.** Poison messages are captured rather than
  silently retried forever or dropped.
- **Per-message ACK overhead.** Higher than AckAll but bounded by
  JetStream batching at the transport level.
- **Replay safety.** Durable consumers resume from last ACK, enabling
  safe restart without full replay.
- **Projection rebuild** requires resetting consumer position to
  stream start — operational tooling needed.
