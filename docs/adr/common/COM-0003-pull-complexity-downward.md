# COM-0003. Pull Complexity Downward

Date: 2026-04-25
Last-reviewed: 2026-04-25
Tier: A

## Status

Accepted

## Related

- References: COM-0002

## Context

Ousterhout (Ch. 8, "Pull Complexity Downwards") addresses where
complexity should live when it must exist somewhere. The asymmetry:
a module is implemented once but called many times. Complexity in the
implementation is paid once; complexity in the interface is paid by
every caller.

**The principle:** when a design choice exists between handling
complexity inside a module or exposing it to callers, the module
should absorb it. "It is more important for a module to have a simple
interface than a simple implementation."

**Configuration parameters** are a specific case. Each configuration
option is complexity pushed to the caller: the caller must understand
the option, choose a value, and accept responsibility for the
consequences. Ousterhout argues that configuration parameters should
require justification — sensible defaults are mandatory, and
parameters should only be exposed when "it is impossible for the
system to determine the right value automatically."

Cherry-pit applies this principle extensively:

- **Store creates envelopes** (CHE-0016) — callers pass `Vec<Event>`,
  the store handles ID assignment, sequencing, timestamping, and
  envelope construction. Complexity pulled down.
- **Infrastructure owns identity** (CHE-0020) — callers never create
  `AggregateId` values. The store assigns IDs. Complexity pulled
  down.
- **Two-level concurrency** (CHE-0035) — callers call `create`,
  `load`, `append` without awareness of global mutexes, per-aggregate
  locks, or lock-free reads. Complexity pulled down.
- **Process-level file fencing** (CHE-0043) — callers do not manage
  lock files. The store acquires the fence lazily on first write.
  Complexity pulled down.

## Decision

When complexity cannot be eliminated, pull it into the
implementation rather than exposing it through the interface.

### Rules

1. **Callers pass minimal information.** If the module can compute,
   derive, or default a value, it should. The caller should not be
   asked to provide information that the module could determine
   itself.

2. **Configuration parameters require justification.** Every
   configuration option is complexity pushed to the caller. Before
   adding a parameter:
   - Can the module determine the right value automatically?
   - Can a sensible default eliminate the need for configuration?
   - Is the parameter necessary for correctness, or is it a
     performance knob that could be auto-tuned?

3. **Sensible defaults are mandatory.** If a configuration parameter
   is justified, it must have a default value that produces correct
   behavior. The caller should be able to use the module without
   configuring anything.

4. **Error handling follows this principle.** When a module can handle
   an error internally (retry, fallback, default), it should not
   propagate the error to the caller. See COM-0005 for the
   complementary principle of eliminating errors entirely.

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
