# PAR-0016. Timestamp Policy and Cross-Stream Ordering

Date: 2026-04-28
Last-reviewed: 2026-04-28
Tier: B
Status: Proposed

## Related

References: PAR-0004, PAR-0007, PAR-0008

## Context

PAR-0004 states wall-clock timestamp suffices for single-writer ordering.
Within one stream, monotonic event_id (PAR-0007) provides true order.
Across streams, wall-clock is unreliable: NTP drift, VM migration, leap
seconds. Lamport (1978) establishes wall-clock is insufficient for causal
ordering across independent writers.

1. **Advisory timestamps** — informational only; cross-stream uses
   JetStream sequences.
2. **Hybrid Logical Clocks** — causal ordering, 8-byte overhead per event.
3. **Vector clocks** — full causal history, overkill for single-writer.

Option 1 chosen: cross-stream consumers use JetStream sequences. HLC
deferred until multi-writer is required.

## Decision

Timestamps are advisory metadata — not a causal ordering mechanism.
Cross-stream event correlation uses JetStream stream sequences.

R1 [6]: Persist timestamps via jiff::Timestamp::now() as advisory
  metadata in EventEnvelope, documented as non-causal across streams
R2 [5]: Cross-stream event ordering uses JetStream-provided stream
  sequence numbers accessed through the consumer API
R3 [6]: Consumers correlating events across multiple streams compare
  JetStream sequences within each stream independently

## Consequences

- **Eliminates false ordering assumptions.** Developers cannot rely on
  timestamp comparison for cross-stream causality.
- **No additional per-event overhead.** No HLC or vector clock fields.
- **Limitation documented.** Systems requiring causal cross-stream
  ordering need explicit coordination (saga pattern, CHE-0040).
- **Timestamp still useful** for human-readable audit, approximate
  time-windowed queries, and single-stream ordering where event_id
  is unavailable.
- **Cross-stream sequences are incomparable.** JetStream sequence
  numbers are per-stream counters — sequence 42 in stream A has no
  causal relationship to sequence 42 in stream B. Consumers
  correlating across streams must join on domain-level keys (e.g.,
  order_id), not on sequence equality.
