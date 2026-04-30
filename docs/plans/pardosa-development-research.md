# Pardosa Development Research — Implementing the Identified Improvements

Date: 2026-04-29

Companion to `docs/plans/top-10-distributed-improvements.md` and `docs/plans/top-10-pardosa-alignment.md`.
This document captures concrete techniques, libraries, and patterns from current (2025–2026) industry practice that pardosa should adopt, organized by phase of the existing `pardosa-next.md` plan.

Sources (representative — full URLs at end of each section):
- S2.dev — DST for async Rust (Apr 2025)
- Polar Signals — State-machine DST in Rust (Jul 2025)
- TigerBeetle Vörtex / VOPR (Feb 2025, Jepsen 0.16.11 Jun 2025)
- Synadia — JetStream Expected Sequence Headers (Jan 2026)
- NATS Docs — JetStream Headers, KV revision CAS (current)
- ripienaar/nats-kv-leader-elect (Go reference impl)
- ricofritzsche.me — Durable telemetry pipeline with NATS+Rust (Oct 2025)
- madsim, turmoil, loom (current)

---

## Part A — Foundational architectural shift (do this first)

### A.1 Adopt the state-machine + message-bus architecture

**Source:** Polar Signals "Theater of State Machines" (sled simulation guide, applied in Rust).

**Why it matters for pardosa:** Pardosa is *already* state-machine-shaped at the fiber level (`(FiberState, FiberAction) → FiberState`). The Polar Signals model generalizes this to the **whole system**: every component (Dragline, NATS publisher, KV registry watcher, genome serializer, migration coordinator) is a `StateMachine` that exposes only `receive(Message) -> Vec<(Message, Destination)>` and `tick(Instant) -> Vec<(Message, Destination)>`. The message bus is the only scheduler.

**Concrete trait** (adapted to pardosa):

```rust
pub trait StateMachine {
    type Msg;
    fn receive(&mut self, m: Self::Msg) -> Vec<(Self::Msg, Destination)>;
    fn tick(&mut self, now: Instant) -> Vec<(Self::Msg, Destination)>;
}
```

**Critical constraint:** the trait is **not async**. Async leaks non-determinism (runaway futures, scheduler choices). Anything that *must* be async (NATS publish) becomes a message → driver dispatch → driver synchronously does `block_on` in tests, real tokio in production.

**Pardosa mapping:**
| State machine | Owns | Messages it receives | Messages it emits |
|---|---|---|---|
| `Dragline` | line, lookup, fiber states, counters | `Reserve(Action)`, `Commit(reserved, jetstream_seq)` | `Reserved(envelope)`, `Applied(event_id)`, `Rejected(reason)` |
| `Publisher` | JetStream connection, in-flight reservations | `Publish(envelope)`, `JsAck(seq)`, `JsNack(reason)` | `Commit(reserved, seq)`, `PublishFailed(reason)` |
| `RegistryWatcher` | NATS KV watcher state, current generation | `KvUpdate(key, value, rev)` | `GenerationChanged(new_gen)` |
| `MigrationDriver` | migration phase, source/dest stream refs | `BeginMigration`, `KvCasOk`, `CopyProgress` | `Phase(state)`, `CutoverDone` |
| `Genome` | encode/decode workspace | `Serialize(value)`, `Deserialize(bytes)` | `Bytes(out)`, `Value(out)`, `DecodeFailed` |

**Concrete benefit:** failure injection becomes free. The bus chooses to drop, delay, duplicate, or reorder any message — no per-component fault hooks. This is exactly what S2 and Polar Signals report as the ROI of DST.

**Tradeoff:** as Polar Signals warns, code that "should be in the state machine" tends to leak into the production driver. Mitigation: lint or design-test that asserts every public pardosa entry point dispatches into the bus, never bypasses it. This deserves its own ADR.

**Recommendation:** Land this **before** Phase 2's concurrency work. The current `Dragline` synchronous API is already mostly state-machine-shaped; the refactor is moderate. Doing it after concurrency lands means rewriting the concurrency layer.

> Sources: https://www.polarsignals.com/blog/posts/2025/07/08/dst-rust · https://sled.rs/simulation.html

---

### A.2 Reserve / Commit split for `Dragline`

**Why:** PAR-0008 (publish-then-apply) is currently violated. Today `Dragline::create` returns `AppendResult` *after* mutating state. Once JetStream is in the path, the mutation must be reverted on broker NACK or we get state-broker divergence on every retry.

**Refactor target:**

```rust
// Phase 1: validate, allocate, freeze (no state mutation)
pub fn reserve(&self, action: FiberAction, payload: T)
    -> Result<ReservedEvent<T>, PardosaError>;

// Phase 2: only after broker ack (or in tests, after bus dispatch)
pub fn commit(&mut self, reserved: ReservedEvent<T>, broker_seq: StreamSeq)
    -> Result<AppendResult, PardosaError>;

// Cleanup on broker NACK
pub fn abandon(&mut self, reserved: ReservedEvent<T>);
```

`ReservedEvent` carries `event_id`, allocated `index`, fiber-state delta, and the encoded genome bytes. It's not `Clone` and has a `Drop` impl that logs/asserts if dropped without commit/abandon — this catches the common bug where an error path forgets to release a reservation.

**Determinism:** `reserve` is pure (given current state). `commit` only mutates. This is the precondition for every other improvement.

**Reservation table:** keep `HashMap<EventId, ReservedEvent<T>>` so that on JetStream ack the message bus can locate the reservation. Bounded — reject new reservations when N in-flight to enforce backpressure (#A.5).

> Linked decision: this change makes the `Dragline` an authoritative-mode state machine; combined with #B.1 below it cleanly separates from the replay/replica use case.

---

### A.3 Identity rename: `event_id` → `StreamMonotonic` newtype

**Why:** PAR-0016 forbids cross-stream comparison; the current `u64` does not enforce this. Compile-time prevention beats documentation. Generalization of `Index` and `DomainId` newtype pattern already in pardosa.

**Concrete:**

```rust
/// Monotonic sequence within a single (stream, generation). Cross-stream
/// comparison is meaningless and statically rejected.
pub struct StreamMonotonic { stream: StreamId, gen: Generation, seq: NonZeroU64 }

impl PartialOrd for StreamMonotonic {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        if self.stream != other.stream || self.gen != other.gen { None } else { self.seq.partial_cmp(&other.seq) }
    }
}
```

`PartialOrd` returning `None` for cross-stream comparison is the key — code that tries `a < b` on incomparable values gets `false` for both directions, surfacing as a test failure rather than wrong-but-plausible ordering.

---

## Part B — JetStream integration (Phase 5/6)

### B.1 Idempotent publish via `Nats-Msg-Id`

**Source:** NATS docs, ricofritzsche pipeline, JetStream message deduplication.

**Pattern:**
- Set `Nats-Msg-Id = event_id` on every publish.
- JetStream maintains a sliding-window dedup table per stream (`duplicate_window`, defaults to 2 minutes; configurable up to stream retention).
- Retried publishes with the same `event_id` are silently absorbed by the broker — no duplicate stream entry.

**Pardosa-specific:**
- `event_id` is already monotonic per stream (PAR-0007). Wire it to the header.
- Configure `duplicate_window` ≥ longest expected publisher retry budget (recommend 10× p99 publish latency, minimum 1 minute).
- For long-horizon idempotency (rare retries hours later), the dedup window is *not* sufficient — that's why we still need #B.4 (consumer-side dedup) and #C.3 (command idempotency keys).

> Source: https://docs.nats.io/nats-concepts/jetstream/streams (Message Deduplication section)
> Source: https://ricofritzsche.me/building-a-durable-telemetry-ingestion-pipeline-with-rust-and-nats-jetstream/

---

### B.2 Optimistic concurrency via `Nats-Expected-Last-Subject-Sequence`

**Source:** Synadia "JetStream Expected Sequence Headers" (Jan 2026).

**Critical clarification from the Synadia post:** there is **only one sequence counter per stream**. The three header types (`-Last-Sequence`, `-Last-Subject-Sequence`, pattern-based) all check against that single counter, just with different filters. **Pardosa implication:** the per-fiber concurrency control we want is `NATS-Expected-Last-Subject-Sequence` with the subject scoped to `pardosa.{stream}.{domain_id}`.

**Subject design for pardosa:**
```
pardosa.{stream_name}.{domain_id}
```
- Each fiber → its own subject.
- Per-subject sequence headers give us per-fiber optimistic concurrency for free.
- Pattern-based variant (`-Last-Sequence-Subject` + pattern) is the right tool for cross-fiber operations like create-with-uniqueness if we need it later.

**Publish flow:**
```rust
let msg = ReservedEvent { ... };
let headers = Headers::new()
    .insert("Nats-Msg-Id", msg.event_id.to_string())
    .insert("Nats-Expected-Last-Subject-Sequence", msg.fiber_prev_seq.to_string());
context.publish_with_headers(subject, headers, payload).await
```

**Conflict handling:** broker rejects with "wrong last sequence: N". Pardosa must:
1. Abandon the local reservation.
2. Replay from JetStream to catch up to seq N.
3. Re-attempt the user operation against the now-current state (or surface a domain-level conflict error).

This is exactly the optimistic concurrency loop that cherry-pit-gateway already runs against files; the same pattern, different backend.

> Source: https://www.synadia.com/blog/understanding-jetstream-expected-sequence-headers
> Source: https://docs.nats.io/nats-concepts/jetstream/headers

**Caveat:** GitHub issue nats-server#7361 (Sep 2025) flagged a bug where `Nats-Expected-Last-Subject-Sequence` was compared against stream-global last seq instead of subject-level. Verify against `nats-server` ≥ the version where this is fixed; encode the required minimum server version in pardosa's `Cargo.toml` comments and CI.

---

### B.3 Lease via NATS KV CAS (registry-as-lease-store)

**Source:** ripienaar/nats-kv-leader-elect (Go reference), NATS KV docs.

**Pattern from the Go reference, translated to pardosa:**

The KV bucket has a TTL (e.g. 60s). The lease key (`pardosa.registry.{name}.active`) is created with `Create()` (succeeds only if absent). The current leader periodically `Update(key, value, expected_revision)` — a CAS that succeeds only if the revision matches the leader's last-known. If the leader pauses (GC, network, OS hiccup) longer than TTL, the key expires; another writer's `Create()` succeeds; old leader's next `Update` fails with revision mismatch → cleanly steps down.

**Pardosa lease lifecycle:**

```
┌─────────────┐  Create(key)→ok ┌─────────────┐  Update(key, rev) every 0.75*TTL
│  Candidate  ├────────────────▶│   Leader    │◀─────────────────────────────────┐
│ (no writes) │                 │ (admits     │                                  │
└──────┬──────┘                 │  writes)    ├──────────────┐                   │
       ▲                        └──────┬──────┘  Update fail │                   │
       │                               │          (revision  │                   │
       │       TTL expires             │           mismatch  │                   │
       └───────────────────────────────┘           or KV     │                   │
                                                   gone)     ▼                   │
                                                       ┌─────────────┐           │
                                                       │  Stepping   ├───────────┘
                                                       │    Down     │
                                                       └─────────────┘
```

**Registry value schema** (extends pardosa-next PAR-0013):

```rust
struct RegistryValue {
    generation: NonZeroU64,        // monotonic, only increases
    stream_name: String,           // PARDOSA_<name>_g<gen>
    writer_id: Uuid,               // identity of current leader
    cutover_ts: jiff::Timestamp,   // when this generation became active
    prev_stream: Option<String>,   // for grace-window readers
    health: HealthStatus,          // Healthy | Quarantined { reason }
}
```

The KV revision (returned by Update/Get) **is** the lease's fencing token. Every JetStream publish path captures the current revision; on `Update` failure, the writer becomes a candidate and stops publishing **before** abandoning in-flight reservations.

**Critical operational parameters (from leader-elect lib):**
- TTL: minimum 30s, maximum 1h (their bounds; recommend 60s for pardosa).
- Campaign interval: `0.75 × TTL` (45s for 60s TTL).
- Initial campaign splay: random 0–5s to spread restarts.

**Why CAS not Raft:** NATS KV is itself backed by JetStream which uses Raft for replication. Building lease on top of KV CAS gets us linearizable lease state without re-implementing consensus.

> Source: https://github.com/ripienaar/nats-kv-leader-elect
> Source: https://docs.nats.io/using-nats/developer/develop_jetstream/kv

---

### B.4 Consumer dedup + checkpointed catch-up

**Source:** PAR-0015 (proposed), NATS exactly-once playbook, ricofritzsche pipeline.

**Three-layer dedup:**
1. **Publisher side** (#B.1): `Nats-Msg-Id` — protects against publisher retry storms.
2. **Broker side**: `duplicate_window` — already configured by #B.1.
3. **Consumer side**: idempotent processing keyed by `event_id`. Required because `AckExplicit` is at-least-once, not exactly-once.

**Consumer trait (pardosa):**

```rust
pub trait Consumer: StateMachine {
    /// Returns the dedup key for this event. Default: event_id.
    /// Custom impls allow application-level keys (e.g. saga step id).
    fn dedup_key(envelope: &EventEnvelope) -> DedupKey { ... }

    /// Idempotent — must produce the same effect for the same dedup_key.
    fn handle(&mut self, envelope: EventEnvelope) -> Effects;
}
```

**Checkpoint storage:** consumer's last-processed `(event_id, jetstream_seq)` pair, persisted via NATS KV (separate bucket per consumer). On restart: read checkpoint → bind durable consumer to JetStream from `seq + 1` → resume.

**Crucial pattern:** checkpoint write **after** effect commit, not before. With at-least-once delivery, replay of an already-applied event is normal and must be detected via dedup_key, not avoided via pre-write checkpoint.

> Source: https://medium.com/@hadiyolworld007/nats-jetstream-playbook-exactly-once-minus-the-bloat

---

### B.5 Startup high-watermark reconciliation (PAR-0007)

**Why:** A writer restart must prove no gaps before accepting writes. Three values must agree:
1. `next_event_id` from local state (in-memory or genome snapshot).
2. JetStream tail sequence for the stream.
3. KV registry generation matches the stream we're about to write to.

**Reconciliation routine:**

```
on_startup():
  registry = kv.get("pardosa.registry.{name}.active")
  if registry.health != Healthy: refuse to start
  if registry.generation != my_snapshot.generation: rebuild from JetStream
  js_tail = jetstream.last_seq(registry.stream_name)
  expected_event_id = derive_from(js_tail)  // event_id == js_tail.payload_event_id
  if expected_event_id != my_snapshot.next_event_id - 1:
    panic  // unrecoverable: snapshot inconsistent with broker truth
  acquire_lease()
  begin_admitting_writes()
```

**Subtle case:** writer crashed *after* JetStream ack but *before* applying locally → `expected_event_id > my_snapshot.next_event_id - 1`. Recovery: read events from JetStream and apply to bring local state forward. This is why JetStream is authoritative.

> Source: PAR-0007 (cherry-pit ADR), pardosa-next.md

---

## Part C — Determinism, simulation, and tests

### C.1 DST harness for pardosa: `pardosa::sim`

**Source:** S2.dev (Apr 2025), Polar Signals (Jul 2025), madsim, turmoil, FoundationDB primer.

**Architecture decision: which approach?**

| Approach | When to use | Pardosa fit |
|---|---|---|
| **madsim** (drop-in tokio replacement, libc symbol overrides) | Existing async codebase, want minimal refactor | Possible, but fights against pardosa's natural state-machine shape |
| **turmoil** (host simulation, tokio-based) | Network-heavy, need multiple "hosts" in one process | Useful for multi-writer/leader-election scenarios |
| **State-machine + message bus** (Polar Signals model) | Greenfield, willing to architect for it | **Best fit — pardosa is already shaped this way** |
| **Hybrid mad-turmoil** (S2's combination) | Need both message-passing and libc-level control | Recommended for mixing pardosa + external NATS in same sim |

**Recommendation: state-machine bus + selective madsim.** Use the bus (#A.1) as the primary deterministic substrate. Use madsim only at the edges where third-party code (async-nats, jiff, uuid generation) could leak non-determinism. S2 reports they had to override `getrandom`, `getentropy`, `clock_gettime`, and (Mac) `CCRandomGenerateBytes` to catch all leaks — pardosa will face the same.

**Concrete CI setup (from S2):**
1. Run DST every PR with a fresh random seed.
2. Run nightly with thousands of seeds.
3. **Meta-test:** rerun the same seed twice, compare `TRACE`-level logs byte-for-byte. Any divergence is a bug in the harness or a non-determinism leak. This catches issues that simple seed-replay misses.

**Failure injection knobs (from VOPR / Polar Signals):**
- Drop messages.
- Duplicate messages.
- Reorder messages within bounded window.
- Delay messages.
- Crash a state machine (drop its state) and replay its inputs from the bus event log.
- Partition the network (drop messages between specific pairs).
- Disk I/O fault injection (return EIO on genome writes).
- KV CAS race injection (spurious revision-mismatch returns).

**Coverage target:** every PAR ADR's invariant should have a named simulation scenario that would catch its violation. Tag tests with the ADR number for traceability (matches the rigormortis approach already used in pardosa).

> Sources: https://s2.dev/blog/dst · https://www.polarsignals.com/blog/posts/2025/07/08/dst-rust · https://github.com/madsim-rs/madsim · https://docs.rs/turmoil

---

### C.2 Loom for the in-process two-level concurrency

**Why:** Even with DST at the system level, pardosa-next's `tokio::sync::RwLock` + per-fiber locking has internal interleavings that DST won't explore. Loom does C11-memory-model permutation testing of every interleaving.

**Scope:** Apply loom only to the locking primitives and the reserve/commit handoff in `Dragline`. Loom is too slow for full-system tests; it's a microscope for known-tricky code.

**Pattern (from matklad's "Properly Testing Concurrent Data Structures"):**
```rust
#[cfg(loom)]
mod loom_tests {
    use loom::sync::Arc;
    use loom::thread;
    #[test]
    fn reserve_commit_race() {
        loom::model(|| {
            let dragline = Arc::new(DraglineForLoom::new());
            // ... spawn two threads doing reserve+commit on different fibers
            // assert no fiber's state diverges
        });
    }
}
```

> Source: https://github.com/tokio-rs/loom · https://matklad.github.io/2024/07/05/properly-testing-concurrent-data-structures.html

---

### C.3 Property-based fuzzing of state machine totality

**Source:** existing pardosa proptest, extended with VOPR-style oracles.

**Three property classes pardosa must verify:**

1. **State-machine totality** (PAR-0001): for every `(FiberState, FiberAction)` pair, `transition()` returns either `Ok(state)` or a *named* error. No panics, no unreachable. Implement as a const validation at compile time if possible (table-driven), else a startup test.

2. **Sequence/precursor invariants** (PAR-0012, PAR-0042): under any arbitrary sequence of operations, `verify_precursor_chains` succeeds. Already partially covered; extend to include migration phases.

3. **Linearizability oracle** (Knossos/Porcupine-style): record concurrent-operation history from the simulator, model-check against a sequential spec. Off the shelf: `porcupine` (Go); for Rust, the Polar Signals/sled approach is to hand-write the sequential spec as another state machine and compare bus outputs.

> Source: https://jepsen.io/analyses/tigerbeetle-0.16.11 (description of TigerBeetle's oracle approach)

---

## Part D — Cryptographic and compatibility

### D.1 BLAKE3 hash chain on `Event<T>`

**Why:** Today pardosa has a structural precursor chain (index-based). To make audit non-repudiable (#5 in the top-10), each event must carry the BLAKE3 hash of its predecessor's canonical genome bytes.

**Layout addition:**

```rust
#[non_exhaustive]
pub struct Event<T> {
    event_id: u64,
    timestamp: i64,
    domain_id: DomainId,
    detached: bool,
    precursor: Index,
    precursor_hash: [u8; 32],   // NEW: BLAKE3 of precursor event canonical bytes; zero for first-in-fiber
    domain_event: T,
}
```

**Verification cost:** the same O(n) walk `verify_precursor_chains` already does, plus a 32-byte BLAKE3 compare per event. BLAKE3 on a typical envelope (~200 bytes) is ~50ns on modern hardware, so 1M events ≈ 50ms — negligible at startup.

**Global frontier hash:** in addition to per-fiber `precursor_hash`, maintain `Dragline::frontier: [u8; 32]` rolled forward across **all** events in append order. Anchor periodically:
- To a NATS subject `pardosa.{stream}.frontier` (broadcasts current frontier).
- To an external transparency log (Sigsum, Trillian) for cross-organizational non-repudiation.

**Why BLAKE3 not SHA-256:**
- BLAKE3 is ~6× faster on modern x86 with AVX2/AVX-512.
- Tree-hash mode allows incremental verification of large events.
- Still cryptographic — no known attacks on collision/preimage at 256-bit output.

> Source: https://www.reddit.com/r/Observability/comments/1rwm9vh/ (industry pattern for hash-chained audit logs)
> Source: https://github.com/EulBite/spine-oss (open-source Rust hash-chain audit reference)

---

### D.2 Schema hash drift detection (GEN-0003 in the pardosa context)

**Pattern:** pardosa-next mandates that schema_hash mismatch between generations is detected by the registry. Extend with a **CI gate**:

```yaml
# .github/workflows/genome-fixtures.yml
- name: Schema hash drift check
  run: cargo test -p pardosa --test schema_fixtures -- --exact
```

The `schema_fixtures` test computes `schema_hash` for every public type in pardosa and compares against a checked-in fixture file (`tests/fixtures/schema_hashes.toml`). If a hash changes:
- PR author must intentionally update the fixture **and** add a migration to the new-stream model.
- Bare hash bumps without migration code fail CI.

**Failure mode this prevents:** Rust syntax change (e.g. macro_rules expansion difference between rustc versions) silently shifts schema_hash. Without the fixture lock, two pardosa binaries built from "the same code" disagree on what stream they can read.

> Source: GEN-0003 ADR, plus pardosa-next.md genome spec changes section.

---

## Part E — Observability

### E.1 Span/metric design (cardinality budget)

**Source:** OpenTelemetry naming guides, Honeycomb best practices, OneUptime Rust + OTel guide.

**Explicit cardinality rules for pardosa:**

| Identifier | Span attribute? | Metric label? | Rationale |
|---|---|---|---|
| `domain_id` | ✅ Yes | ❌ Never | Unbounded (one per fiber). Use as span attr for trace lookup, never as metric label. |
| `event_id` | ✅ Yes | ❌ Never | Strictly monotonic; unbounded over time. |
| `stream_name` | ✅ Yes | ✅ Yes | Bounded (handful per deployment). |
| `generation` | ✅ Yes | ✅ Yes | Bounded (one new per migration; ~daily at most). |
| `fiber_state` | ✅ Yes | ✅ Yes | Enum, fixed cardinality. |
| `action` | ✅ Yes | ✅ Yes | Enum (Create/Update/Detach/Rescue/Migrate). |
| `result` | ✅ Yes | ✅ Yes | Enum (Ok/Rejected/Conflict/...). |
| `correlation_id` | ✅ Yes | ❌ Never | Per-request, unbounded. |
| `writer_id` | ✅ Yes | ✅ Yes | Bounded (number of pardosa nodes). |

**Required metrics (low-cardinality only):**
```
pardosa_events_total{stream, action, result}
pardosa_fibers_active{stream, state}
pardosa_jetstream_publish_seconds{stream, result}    // histogram
pardosa_jetstream_publish_inflight{stream}            // gauge
pardosa_lease_state{writer_id}                        // gauge: 0=candidate, 1=leader, 2=stepping_down
pardosa_registry_generation{stream}                   // gauge
pardosa_dedup_window_hits{stream}                     // counter
pardosa_migration_phase{stream, phase}                // gauge: one-hot
```

**Instrumentation pattern (`tracing`):**
```rust
#[tracing::instrument(skip(self, payload), fields(stream = %self.stream, domain_id = %domain_id))]
pub fn reserve(&self, action: FiberAction, payload: T) -> Result<ReservedEvent<T>, PardosaError> {
    // ...
}
```

**OpenTelemetry .NET 4-to-8-tag rule (from OTel best practices) translated:** if a span needs more than ~8 attributes routinely, factor into structured events emitted from inside the span instead.

> Source: https://opentelemetry.io/docs/languages/dotnet/metrics/best-practices/
> Source: https://oneuptime.com/blog/post/2026-01-07-rust-opentelemetry-instrumentation/view
> Source: https://www.honeycomb.io/blog/opentelemetry-best-practices-naming

---

### E.2 Redaction discipline

**Pattern:** wrap user-controlled `T` in pardosa's serialization layer with a `Redacted<T>` wrapper that:
- `Debug` outputs `***` (no plaintext).
- `Display` outputs `***`.
- `Serialize` only inside `pardosa::serialization::trusted_scope()` — a thread-local guard set by the genome encoder. Outside that scope, `Serialize` panics or returns a placeholder.

This means even if a developer accidentally `tracing::info!(?event)`, secret payloads in `T` print as `***`.

**Test discipline:** add `tests/no_secret_in_logs.rs` that runs each public API path under a capturing `tracing::Subscriber` and asserts no `Redacted` plaintext appears in any emitted event.

---

## Part F — Operational tooling

### F.1 `pardosa-admin` CLI (bounded-authority repair tool)

**Inspired by:** TigerBeetle's `tigerbeetle inspect`, etcdctl, NATS CLI patterns.

**Required capabilities:**
- `pardosa-admin status --stream X` — read registry, JetStream tail, frontier hash.
- `pardosa-admin verify --stream X` — re-walk precursor chain + frontier hash, report divergence.
- `pardosa-admin quarantine --stream X --reason "..."` — set registry health to Quarantined; rejects writes, allows reads.
- `pardosa-admin migrate --stream X --new-schema-hash H --confirm $frontier` — initiate new-stream migration; the `--confirm` argument requires the operator to paste the current frontier hash, preventing fat-finger mistakes.
- `pardosa-admin drop-tail --stream X --back-to $event_id --confirm $frontier` — repair tool for "writer crashed mid-publish-batch, last 3 events are orphaned." Emits a `Repair` event into the stream documenting the action.

**Authority model:** every operation is logged into a `pardosa.audit.{stream}` JetStream subject with operator identity (from NATS user JWT). The repair tool can never delete history — only mark it as superseded by a new generation.

---

## Part G — Sequencing & ADR follow-ups

### G.1 New ADRs to write

1. **`PAR-0017 State Machine Bus Architecture`** — codify the message-bus model from #A.1 as a pardosa invariant.
2. **`PAR-0018 Reserve/Commit API Discipline`** — make #A.2 normative; mandates `ReservedEvent` Drop logging.
3. **`PAR-0019 JetStream Cluster Topology Assumptions`** — replication factor, leader-failover behavior, KV CAS semantics under partition; minimum nats-server version.
4. **`PAR-0020 Lease via NATS KV CAS`** — TTL bounds, campaign interval, fencing token = KV revision.
5. **`PAR-0021 Frontier Hash and Per-Fiber Hash Chain`** — BLAKE3, anchoring, verification cost.
6. **`PAR-0022 Deterministic Simulation Harness`** — message bus model, failure injection knobs, meta-test for determinism.
7. **`PAR-0023 Observability Cardinality Budget`** — the table in #E.1 as binding policy.

### G.2 Suggested implementation order (revised against pardosa-next phases)

| Order | Item | Maps to pardosa-next phase |
|---|---|---|
| 1 | State-machine bus refactor (#A.1) | **Pre-Phase 2** — must precede concurrency |
| 2 | Reserve/commit split (#A.2) | Pre-Phase 2 |
| 3 | StreamMonotonic newtype (#A.3) | Pre-Phase 2, free with #A.2 |
| 4 | DST harness + meta-test (#C.1) | Pre-Phase 2, alongside refactor |
| 5 | Genome encoder/decoder | Phase 5 prerequisite (not pardosa work, but blocking) |
| 6 | Hash chain + frontier (#D.1) | Genome layout finalization |
| 7 | Observability scaffold (#E.1, #E.2) | Phase 5 — instrument as NATS code lands |
| 8 | JetStream publish path (#B.1, #B.2) | Phase 5 |
| 9 | Reconciliation routine (#B.5) | Phase 5 |
| 10 | KV registry + lease (#B.3) | Phase 6 |
| 11 | Migration state machine (PAR-0005) | Phase 4, but unblocked by #1-#3 |
| 12 | Consumer lifecycle + dedup (#B.4) | Phase 6 |
| 13 | Loom microtests on locks (#C.2) | Throughout — apply per primitive |
| 14 | Schema fixture CI gate (#D.2) | Lands with genome |
| 15 | `pardosa-admin` (#F.1) | Phase 6 hardening, before any production claim |

### G.3 Three forcing-function tests for "is pardosa ready?"

Borrowed from TigerBeetle/Jepsen:

1. **The 1B-event soak.** Run DST with 1B simulated events over a simulated month. Measure: zero panics, all invariants hold, p99 ops/sec stable, no leaks (in-memory tables bounded).
2. **The chaos cluster test.** Three pardosa writers, one stream. DST kills leader every 30s of simulated time, partitions network 10% of the time, drops 5% of messages. After 1h simulated: every committed event is in JetStream exactly once; no event is lost; lease never has two simultaneous holders.
3. **Mixed-version migration test.** Generation N and generation N+1 (different schema_hash) running concurrently against the same registry. Old readers continue reading old stream during grace window; new writers write new stream; cutover is atomic; rollback is atomic.

If any of these reproduces a bug, a fix lands plus a new named scenario in the suite. Coverage = number of distinct named scenarios passing across 10k seeds.

---

## Bibliography (primary sources only)

**Distributed Systems Testing**
- S2.dev, "Deterministic simulation testing for async Rust" — https://s2.dev/blog/dst (Apr 2025)
- Polar Signals, "Deterministic Simulation Testing in Rust: A Theater Of State Machines" — https://www.polarsignals.com/blog/posts/2025/07/08/dst-rust (Jul 2025)
- TigerBeetle, "A Descent Into the Vörtex" — https://tigerbeetle.com/blog/2025-02-13-a-descent-into-the-vortex (Feb 2025)
- Jepsen, "TigerBeetle 0.16.11" — https://jepsen.io/analyses/tigerbeetle-0.16.11 (Jun 2025)
- FoundationDB Simulation primer — https://pierrezemb.fr/posts/diving-into-foundationdb-simulation/ (Oct 2025)
- Antithesis DST primer — https://amplifypartners.com/blog-posts/a-dst-primer-for-unit-test-maxxers (Nov 2025)

**Rust simulation libraries**
- madsim — https://github.com/madsim-rs/madsim
- turmoil — https://docs.rs/turmoil
- loom — https://github.com/tokio-rs/loom
- matklad on concurrent data structure testing — https://matklad.github.io/2024/07/05/properly-testing-concurrent-data-structures.html

**NATS / JetStream**
- Synadia, "JetStream Expected Sequence Headers" — https://www.synadia.com/blog/understanding-jetstream-expected-sequence-headers (Jan 2026)
- NATS Docs, JetStream Headers — https://docs.nats.io/nats-concepts/jetstream/headers
- NATS Docs, KV Store — https://docs.nats.io/using-nats/developer/develop_jetstream/kv
- ricofritzsche, "Building a Durable Telemetry Ingestion Pipeline with Rust and NATS JetStream" — https://ricofritzsche.me/building-a-durable-telemetry-ingestion-pipeline-with-rust-and-nats-jetstream/ (Oct 2025)
- nats-kv-leader-elect — https://github.com/ripienaar/nats-kv-leader-elect

**Observability**
- OpenTelemetry .NET metrics best practices — https://opentelemetry.io/docs/languages/dotnet/metrics/best-practices/
- OneUptime, "How to Instrument Rust Applications with OpenTelemetry" — https://oneuptime.com/blog/post/2026-01-07-rust-opentelemetry-instrumentation/view (Jan 2026)
- Honeycomb, "OpenTelemetry Best Practices: Naming" — https://www.honeycomb.io/blog/opentelemetry-best-practices-naming

**Tamper-evident audit**
- ArXiv, "Rethinking Tamper-Evident Logging: Nitro" — https://arxiv.org/html/2509.03821v2 (Sep 2025)
- spine-oss (Rust hash-chain reference) — https://github.com/EulBite/spine-oss
