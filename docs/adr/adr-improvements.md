# ADR Improvement Backlog

Date: 2026-04-27
Source: Distributed systems design evaluation of the ADR decision log
Provenance: Conversational evaluation artifact (2026-04-27), not a formal ADR
Verified: 2026-04-27 — code-level audit against source, ADRs, and POSIX
  specifications. Two false positives removed; see Appendix A.

This document consolidates all improvements identified during an
architectural review of the cherry-pit ADR corpus (~123 ADRs across 6
domains). Items are grouped by category and ranked by priority.

Items marked **Blocked** depend on unbuilt NATS integration code in the
pardosa crate; they are correctly identified gaps but cannot be actioned
until the prerequisite code exists.

---

## High Priority — Correctness and Safety

### 1. Clarify event_id identity mapping between cherry-pit-core and pardosa

**Finding:** CHE-0041:45–46 references `event_id (UUID v7, CHE-0033)`
while PAR-0007:31 defines `event_id: u64` (monotonic counter). These
are different identity schemes in different layers:

- cherry-pit-core (`event.rs:59`): `event_id: uuid::Uuid` — globally
  unique, sortable, 16 bytes
- pardosa (`event.rs:145`): `event_id: u64` — cross-generation
  monotonic, 8 bytes

The relationship between these two identifiers is unspecified. The two
crates are currently structurally independent — pardosa defines its own
`Event<T>` and `Dragline` with no dependency on cherry-pit-core's
`EventEnvelope<E>` or `EventStore` trait. They share zero types. The
identity conflict is a symptom of an **unresolved architectural
boundary**: if pardosa is intended to serve as cherry-pit-core's
runtime backend, the composition model and identity mapping must be
designed. If they are independent stacks, that independence should be
explicitly documented.

**Action:** Write a dedicated ADR mapping cherry-pit-core UUID v7
event identity to pardosa's u64 monotonic event_id. This ADR should
specify:
- Whether the two crates compose (pardosa wraps cherry-pit-core
  aggregates) or remain independent stacks
- If they compose: which identity is authoritative for deduplication
  at each layer, and how the mapping is performed
- If independent: document the boundary explicitly so future
  contributors do not assume composition

Amending CHE-0041 is insufficient — this is a cross-crate
architectural boundary decision, not a correction to one ADR.

### 2. Implement verify_roundtrip and add GenomeOrd adversarial test

**Finding:** GEN-0032 and GEN-0033 reference `verify_roundtrip` as
defense-in-depth against non-deterministic `Ord` implementations on
`GenomeOrd` types. Code audit reveals **`verify_roundtrip` does not
exist** — the only reference in the codebase is a doc comment at
`genome_safe.rs:72`:

```rust
/// [`verify_roundtrip`] provides defense-in-depth against incorrect implementations.
```

No function, method, or macro named `verify_roundtrip` is defined
anywhere. The safety net described in the ADRs is missing entirely,
not merely untested.

`GenomeOrd` is a safe marker trait (`pub trait GenomeOrd: GenomeSafe {}`)
with no enforcement beyond compile-time bounds. Any type can implement
it with an arbitrary `Ord` — including non-deterministic ones — and the
system has no runtime check to catch the violation.

**Action (two steps, ordered):**
1. **Implement `verify_roundtrip`** as described in GEN-0032: a
   function that serializes a value, deserializes it, re-serializes,
   and asserts byte-level equality. This is the missing safety net.
2. **Add an adversarial test:** implement a test type with `GenomeOrd`
   + deliberately non-deterministic `Ord` (e.g., `AtomicU64` counter
   in the `cmp` implementation). Prove `verify_roundtrip` detects the
   canonical encoding violation.
3. **Fix dangling doc reference:** `genome_safe.rs:72` references
   `[verify_roundtrip]` as if it exists — this generates a broken
   intra-doc link. Update the doc comment when the function is
   implemented.

### 3. Cross-stream ordering semantics ADR

**Finding:** PAR-0007 provides total ordering within a stream via
monotonic `event_id`. Cross-stream ordering (multiple pardosa
instances serving different aggregate types) is undocumented. This
is a critical property for downstream consumers building projections
or read models across multiple aggregate types — without an explicit
statement, consumers may incorrectly assume cross-stream ordering
from `event_id` comparison.

CHE-0039 provides `correlation_id`/`causation_id` for causal
ordering, but no ADR explicitly states: "there is no ordering
guarantee across streams."

**Action:** Write an ADR documenting:
- Within a stream: total order guaranteed by single-writer +
  monotonic event_id
- Across streams: no ordering guarantee (wall-clock timestamps
  are advisory, not authoritative)
- Causal ordering across streams: tracked by correlation_id /
  causation_id (CHE-0039), not by event_id comparison
- Implications for cross-aggregate queries and projections:
  consumers must tolerate out-of-order delivery and use
  causation chains for cross-aggregate consistency

---

## Medium Priority — New ADRs

### 4. Document orphan stream cleanup protocol

**Status:** Blocked — no NATS integration code exists in pardosa.

**Finding:** PAR-0013:73–74 states orphan stream cleanup during
migration CAS failure is "the migration caller's responsibility" with
no documented retry/cleanup protocol. Repeated partition-induced CAS
failures could accumulate orphan streams silently.

The gap is real but the NATS integration is unbuilt — pardosa's
`Dragline` is a purely in-memory data structure with no network
communication. Designing a cleanup protocol now would be speculative;
the actual failure modes during CAS may differ from what the ADR
anticipates.

**Action:** When NATS integration is implemented, extend PAR-0013 or
write a new ADR documenting a bounded retry + cleanup sequence for
migration CAS failures during partitions. Include: max retry count,
backoff strategy, orphan stream detection, and cleanup responsibility.
Design against observed failure modes, not hypothetical ones.

### 5. CAP theorem positioning ADR

**Finding:** The system is CP — writes are rejected during partitions
to preserve consistency. PAR-0004 already states this explicitly
("Correctness over availability: writes rejected during partitions"),
but the position is spread across multiple documents (CHE-0006,
PAR-0004, PAR-0008, PAR-0014).

**Action:** Write a single ADR (likely Pardosa domain) consolidating
the system's CAP position:
- Cherry-pit/pardosa chooses consistency over availability
- Writes are rejected during partitions
- Reads continue from in-memory state (PAR-0014 degraded mode)
- Reference CHE-0006, PAR-0004, PAR-0008, PAR-0014

This is a documentation consolidation — no new design decisions. The
information already exists; the ADR provides a single entry point.

### 6. Failure Modes and Effects Analysis (FMEA) ADR

**Finding:** Individual failure modes are well-documented per ADR
(PAR-0008 ACK-loss, PAR-0014 circuit breaker, PAR-0005 crash
recovery, CHE-0043 fencing), but no consolidated failure mode
analysis exists. This is a documentation consolidation exercise —
all failure modes and mitigations are already correctly documented
in their respective ADRs.

**Action:** Write an FMEA ADR with a single table:

| Failure | Detection | Impact | Mitigation | ADR |
|---------|-----------|--------|------------|-----|
| NATS ACK loss | Publish timeout | Phantom event | Idempotent replay (PAR-0007) | PAR-0008 |
| Dual writer | Sequence mismatch | Write rejected | NATS fencing (PAR-0004) | PAR-0004 |
| ... | ... | ... | ... | ... |

Best deferred until closer to production deployment when the full
failure surface is exercisable.

### 7. Startup and recovery protocol ADR

**Status:** Blocked — no NATS integration code exists in pardosa.

**Finding:** The startup sequence is documented across PAR-0008
(phantom event replay), PAR-0012 (precursor chain verification),
PAR-0013 (registry read), but no single ADR orders these steps.

**Action:** After NATS integration is implemented, write a Pardosa
domain ADR documenting the ordered startup protocol:
1. Read `{name}.active` from NATS KV registry (PAR-0013)
2. Connect to active JetStream stream
3. Replay all events into in-memory Dragline
4. Verify precursor chains (PAR-0012)
5. Deduplicate phantom events via event_id (PAR-0007)
6. Initialize circuit breaker state (PAR-0014)
7. Accept writes

Include crash-recovery semantics: what happens if the process
crashes at each step. These semantics depend on NATS client
reconnection behavior and JetStream consumer configuration, which
can only be validated with working code.

### 8. Observability and metrics ADR

**Status:** Blocked — no deployed infrastructure to instrument.

**Finding:** No observability instrumentation exists in the codebase.
The `tracing` crate is a transitive dependency via `tower-http`'s
`trace` feature, but `cherry-pit-web` (the only consumer of
`axum`/`tower-http`) is commented out of the workspace members.
`tracing` is not in the current dependency tree and zero `tracing`
usage exists in any `.rs` source file.

**Action:** When `cherry-pit-web` is activated and NATS integration
is built, write an ADR covering:
- Circuit breaker state exposure (open/closed/half-open)
- Write latency histograms (NATS publish duration)
- Replay duration on startup
- Health check contract (readiness vs liveness)
- Relationship between `correlation_id` (cross-process, persisted)
  and `tracing::Span` (process-local, ephemeral) per CHE-0039
- Integration plan for `tracing` (add as direct dependency, not
  rely on transitive availability)

---

## Low Priority — Refinements and Tests

### 9. Circuit breaker threshold tuning

**Status:** Blocked — no circuit breaker implementation exists
(PAR-0014 is design-only; zero matches for `CircuitBreaker` or
`circuit_breaker` in source code).

**Finding:** PAR-0014's circuit breaker threshold (3 consecutive
failures × 5s timeout = 15s worst-case trip time) may be too aggressive
for NATS leader elections (~1s). The ADR acknowledges this but defers a
time-window approach.

**Action:** When the circuit breaker is implemented:
- Add a test validating circuit breaker behavior during ~1s
  transient unavailability (simulated NATS leader election)
- Consider amending PAR-0014 to specify a time-window approach
  (e.g., 3 failures within 10s) as a concrete future refinement

### 10. Migration cutover test under simulated partition

**Status:** Blocked — no NATS integration code exists.

**Finding:** PAR-0005 step 4 + PAR-0013 CAS failure path during NATS
partition is documented but untested.

**Action:** When NATS integration is implemented, add an integration
test simulating NATS unavailability during migration cutover. Verify:
- CAS failure returns `RegistryConflict` or `RegistryUnavailable`
- Old stream remains intact and readable
- Orphan new stream is identifiable for cleanup

### 11. Backpressure at CommandGateway level

**Status:** Blocked — `CommandGateway` and `CommandBus` are trait
definitions with no implementations.

**Finding:** PAR-0014 addresses backpressure at the NATS publish
layer. No ADR covers upstream command ingestion rate limiting.
Unbounded command dispatch under load saturates the write lock queue.

**Action:** When `CommandBus` is built, add backpressure at the
command ingestion boundary. Consider:
- Bounded channel between CommandGateway and CommandBus
- Semaphore-based concurrency limit on dispatch
- Documented in a new ADR or as an amendment to PAR-0014

### 12. Amend CHE-0006 consequences for multi-node deployment

**Status:** Blocked — pardosa is not yet operational with NATS.

**Finding:** CHE-0006 states "Multi-node deployment requires an
external mechanism (NATS subject partitioning, process registry) to
route commands to the owning process — this is currently undesigned."
PAR-0004's transport-level fencing via `Nats-Expected-Last-Subject-Sequence`
is the concrete mechanism that enables multi-node deployment, but
the cross-reference is missing.

**Action:** Once pardosa is operational with NATS, add a
cross-reference from CHE-0006 to PAR-0004's transport-level fencing
as the concrete single-writer enforcement mechanism for multi-node
deployments.

---

## Summary Table

| # | Category | Priority | Domain | Status | Action |
|---|----------|----------|--------|--------|--------|
| 1 | Correctness | High | CHE + PAR | Open | Clarify event_id identity mapping and crate boundary |
| 2 | Safety | High | GEN | Open | Implement verify_roundtrip + adversarial test |
| 3 | New ADR | High | PAR | Open | Cross-stream ordering semantics |
| 4 | Safety | Medium | PAR | Blocked | Orphan stream cleanup protocol |
| 5 | New ADR | Medium | CHE/PAR | Open | CAP positioning (consolidation) |
| 6 | New ADR | Medium | PAR | Open | FMEA (consolidation) |
| 7 | New ADR | Medium | PAR | Blocked | Startup/recovery protocol |
| 8 | New ADR | Medium | CHE/PAR | Blocked | Observability and metrics |
| 9 | Refinement | Low | PAR | Blocked | Circuit breaker threshold tuning |
| 10 | Test | Low | PAR | Blocked | Migration cutover under partition |
| 11 | Refinement | Low | CHE/PAR | Blocked | CommandGateway backpressure |
| 12 | Refinement | Low | CHE | Blocked | Multi-node command routing |

---

## Appendix A — Removed Items

The following items from the original backlog (2026-04-27) were removed
after code-level verification revealed them to be false positives.

### Removed: Intra-process fencing gap in MsgpackFileStore

**Original claim:** `flock` is per-file-description. Two
`MsgpackFileStore` instances in the same process targeting the same
directory may both succeed in acquiring the lock (POSIX semantics:
same-process `flock` on a new file description succeeds).

**Why removed:** The claim conflates `flock(2)` semantics with POSIX
`fcntl()` lock semantics. `flock(2)` locks are per-open-file-description.
Two separate `open()` calls create independent file descriptions, and
the second exclusive `flock` attempt IS denied — the Linux man page
states: *"An attempt to lock the file using one of these file
descriptors may be denied by a lock that the calling process has
already placed via another file descriptor."* macOS `flock(2)` shares
the same per-open-file-description semantics. The same-process
non-conflict behavior only applies to `dup()`/`fork()`-derived FDs
sharing the same open file description, which `MsgpackFileStore` never
creates.

The codebase already contains a passing test that proves this:
`second_store_same_dir_fails_with_store_locked` (`msgpack_file.rs:1373`)
creates two independent `MsgpackFileStore` instances on the same
directory and asserts the second receives `StoreError::StoreLocked`.

### Removed: BTreeMap ordering stability documentation

**Original claim:** GEN-0032 depends on `BTreeMap` iteration order for
canonical encoding. The risk is theoretical but the dependency is
load-bearing. A future Rust edition could change iteration semantics.

**Why removed:** `BTreeMap`'s sorted-by-key iteration order is a
**documented, stable guarantee** in the Rust standard library — it is
part of the type's public API contract, not an implementation detail.
The official docs state: *"Iterators obtained from functions such as
`BTreeMap::iter` [...] produce their items in key order."* Changing
this would be a semver-breaking change, which Rust's stability policy
prohibits. Editions change syntax and language semantics, not standard
library API contracts. The risk is zero.
