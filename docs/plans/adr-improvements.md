# ADR Improvement Backlog

Date: 2026-04-27
Updated: 2026-04-27 — evaluation verdicts, composition model findings,
  priority adjustments added
Source: Distributed systems design evaluation of the ADR decision log
Provenance: Conversational evaluation artifact (2026-04-27), not a formal ADR
Verified: 2026-04-27 — code-level audit against source, ADRs, and POSIX
  specifications. Two false positives removed; see Appendix A.

See also: [ADR Quality Refinements](adr-quality-refinements.md)

This document consolidates all improvements identified during an
architectural review of the cherry-pit ADR corpus.
Items are grouped by category and ranked by priority.

Items marked **Blocked** depend on implementations that do not exist
yet; they are correctly identified gaps but cannot be actioned until
the prerequisite code exists.

---

## Medium Priority — New ADRs

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

### 11. Backpressure at CommandGateway level

**Status:** Blocked — `CommandGateway` and `CommandBus` are trait
definitions with no implementations.

**Finding:** No ADR covers upstream command ingestion rate limiting.
Unbounded command dispatch under load saturates the write lock queue.

**Action:** When `CommandBus` is built, add backpressure at the
command ingestion boundary. Consider:
- Bounded channel between CommandGateway and CommandBus
- Semaphore-based concurrency limit on dispatch
- Documented in a new ADR

---

## Summary Table

| # | Category | Priority | Domain | Status | Action |
|---|----------|----------|--------|--------|--------|
| 8 | New ADR | Medium | CHE | Blocked (no infra) | Observability and metrics |
| 11 | Refinement | Low | CHE | Blocked (no impl) | CommandGateway backpressure |

---

## Evaluation Verdicts (2026-04-27)

Each item was evaluated against source code, existing ADRs, and the
crate dependency graph.

### Verdict summary

- **Items #8 and #11:** All correctly categorized; each is blocked
  on unbuilt implementations or deployed infrastructure. No new
  decisions are pending.

### False positive audit

Appendix A's two removed items (flock semantics, BTreeMap ordering)
are correctly identified as false positives:

- `flock(2)` per-open-file-description semantics DO deny the second
  exclusive lock from separate `open()` calls. Confirmed by passing
  test `second_store_same_dir_fails_with_store_locked`.
- `BTreeMap` sorted-by-key iteration is a documented stable guarantee
  in the Rust standard library. Editions cannot change it without a
  semver break.

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

**Original claim:** Canonical encoding depends on `BTreeMap` iteration
order. The risk is theoretical but the dependency is load-bearing. A
future Rust edition could change iteration semantics.

**Why removed:** `BTreeMap`'s sorted-by-key iteration order is a
**documented, stable guarantee** in the Rust standard library — it is
part of the type's public API contract, not an implementation detail.
The official docs state: *"Iterators obtained from functions such as
`BTreeMap::iter` [...] produce their items in key order."* Changing
this would be a semver-breaking change, which Rust's stability policy
prohibits. Editions change syntax and language semantics, not standard
library API contracts. The risk is zero.
