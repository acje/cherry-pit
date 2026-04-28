# COM-0003. Pull Complexity Downward

Date: 2026-04-25
Last-reviewed: 2026-04-28
Tier: B
Status: Accepted

## Related

References: COM-0001, COM-0002

## Context

Ousterhout (Ch. 8, "Pull Complexity Downwards") addresses where complexity should live when it must exist somewhere. A module is implemented once but called many times, so complexity in the implementation is paid once while complexity in the interface is paid by every caller. The module should absorb complexity: "It is more important for a module to have a simple interface than a simple implementation."

Configuration parameters are a specific case — each option pushes complexity to the caller, who must understand, choose, and accept responsibility. Ousterhout argues parameters should only be exposed when the system cannot determine the right value automatically.

Cherry-pit applies this extensively. The store creates envelopes (CHE-0016) — callers pass `Vec<Event>` while the store handles ID assignment, sequencing, and timestamping. Infrastructure owns identity (CHE-0020) — callers never create `AggregateId` values. Two-level concurrency (CHE-0035) hides global mutexes and per-aggregate locks behind `create`, `load`, `append`. Process-level file fencing (CHE-0043) acquires the fence lazily on first write without caller involvement.

## Decision

When complexity cannot be eliminated, pull it into the
implementation rather than exposing it through the interface.

R1 [5]: Callers pass minimal information; if the module can compute,
  derive, or default a value, the caller must not provide it
R2 [5]: Every configuration parameter requires justification
  demonstrating the module cannot determine the value automatically
R3 [5]: Configuration parameters must have sensible defaults that
  produce correct behavior without caller configuration
R4 [6]: When a module can handle an error internally through retry,
  fallback, or default, it must not propagate the error to the caller

## Consequences

- Infrastructure implementations absorb operational complexity. The
  `EventStore` trait has 3 methods; the `MsgpackFileStore`
  implementation has 600+ lines handling concurrency, fencing, atomic
  writes, and serialization.
- New infrastructure ports are evaluated for interface simplicity.
  A port proposal that requires callers to manage locks, configure
  buffer sizes, or handle retry logic is challenged to pull that
  complexity down.
- The "many users, few developers" asymmetry justifies the investment:
  implementation complexity is paid once by the module author; interface
  simplicity benefits every caller, every time.
- There is tension with transparency: pulling complexity down can
  make it harder to debug when things go wrong. This is mitigated by
  structured logging and error messages that expose internal state
  when failures occur — the interface is simple during normal
  operation, detailed during failure investigation.
