# Top-10 Distributed Improvements vs Pardosa — Alignment Analysis

Date: 2026-04-29

Companion to `docs/plans/top-10-distributed-improvements.md`.
Reads against `crates/pardosa/`, `crates/pardosa-genome/`, ADRs PAR-0001..PAR-0016, and `docs/plans/pardosa-next.md`.

## Pardosa's distributed-systems posture in one paragraph

Pardosa is an in-memory append-only "line" of `Event<T>` records ("Dragline") with a per-fiber `(state, action) → state` transition table, an `Index` newtype with explicit `NONE` sentinel, and a `next_event_id` monotonic counter. The roadmap (`pardosa-next.md`) is to host this line on **JetStream** (one stream per generation), advertise the active generation via a **NATS KV registry**, snapshot to a **genome file** per generation, and migrate by writing a *new* stream + *new* file (never mutate). Single-writer per stream is enforced by JetStream `expected_last_subject_seq` + `event_id` monotonicity. The fiber state machine is data-driven and replayable. **Pardosa is unusually well-prepared for items #2, #4, #6, and #8**, and is the *driver* for #10 (genome). Items #1, #3, #5, #7 are mostly absent and need first-class introduction. Item #9 (object store) does not apply — JetStream is the chosen backend.

---

## Item-by-item alignment

### #1. Durable outbox + publish-then-apply — **PARTIAL ALIGNMENT, ALREADY DESIGNED**

**Status:** Pardosa has the most explicit answer in the corpus. PAR-0008 mandates publish-then-apply: JetStream is the durability boundary; in-memory `Dragline` only advances after broker ack. There is no separate "outbox" because the broker *is* the outbox.

**Alignment with the recommendation:**
- ✅ Durability-first is normative.
- ✅ "Reserved-event construction" (#1a) maps directly to PAR-0008's "build envelope → publish → on ack apply locally" sequence.
- ✅ DLQ (#1b) is partially covered by PAR-0015 (`AckExplicit` + dead-letter replay).
- ⚠️ **The `Dragline::create/update/detach/rescue` API in code today applies *before* publishing.** PAR-0008 is design-only; current code violates it. This is the single largest gap between ADRs and code in pardosa.
- ❌ Catch-up cursor (#1d) is partial: PAR-0015 covers consumer-side, but PAR-0007's "durable high-watermark reconciliation" (cross-checking `event_id` ↔ JetStream seq ↔ local position on startup) is not implemented.

**Pardosa-specific recommendation:** Refactor `Dragline` write methods into two phases — `reserve(action) -> ReservedEvent { event_id, index, fiber_delta }` (validates state machine, computes deltas, holds nothing) and `commit(reserved, jetstream_seq)` (applies state). Phase 5/6 of pardosa-next is the right time. Add a startup reconciliation routine that reads the JetStream tail and verifies `(event_id, jetstream_seq)` match before accepting writes.

---

### #2. Lease/epoch fencing — **STRONG ALIGNMENT, ALREADY DESIGNED**

**Status:** PAR-0004 and PAR-0013 together specify exactly the lease+epoch model recommended. The "stream generation" in the NATS KV registry **is the epoch**.

**Alignment:**
- ✅ Single-writer per stream via JetStream `expected_last_subject_seq` is fencing-by-construction at the broker.
- ✅ Generation number in `key: {name}.active → val: {gen}:{stream}` is a monotonic epoch.
- ✅ Migration creates new generation; old writers fail their next publish (broker rejects out-of-order seq, registry shows new gen).
- ⚠️ **Stale-writer rejection on read (#2a):** consumers in PAR-0015 watch the registry, but the design doesn't yet specify what a consumer does mid-stream when the generation flips. Recommendation: consumers must re-bind on registry change; reject events from a stream whose generation no longer matches active.
- ⚠️ **Writer identity (#2c):** `pardosa-next.md` PAR-0013 mentions registry value can carry writer identity but it's not in the schema yet. Recommendation: registry value becomes `{ generation, stream_name, writer_id, cutover_ts, prev_stream }` so split-brain is observable.

**Pardosa-specific recommendation:** Add a `Generation(NonZeroU64)` newtype to `pardosa::event` and embed it in the genome layout. Lease acquisition = NATS KV CAS on the registry key. Lease renewal is implicit in continued JetStream publish ack. Lease loss = registry watch fires with new generation → writer `panic!`s or transitions to `Quarantined` mode (see #8b below).

---

### #3. Idempotency keys as first-class boundary type — **WEAK ALIGNMENT, GAP**

**Status:** PAR-0007 (monotonic event_id for idempotent *publish*) covers the **broker-side** dedup story: JetStream's `Nats-Msg-Id = event_id` makes retried publishes idempotent at the network layer. **But pardosa has no answer for retried *application-level commands* that produce different `event_id`s.**

**Alignment:**
- ✅ Publish idempotency is solved (PAR-0007).
- ❌ Command idempotency is absent. `Dragline::create()` always allocates a fresh `domain_id`. A retried "create user X" produces two fibers.
- ❌ `create_reuse(domain_id)` requires the caller to remember the previous purged ID — places the burden on the application.

**Pardosa-specific recommendation:** Add an optional `correlation_id` (or `request_id`) parameter to `Dragline::create`/`update`/etc. and persist it on the `Event` envelope. Maintain an in-memory `HashMap<RequestId, AppendResult>` with TTL eviction; on duplicate, return the original `AppendResult`. This is naturally compatible with JetStream's `Nats-Msg-Id` — the same key can be reused at both layers. *Caveat:* this introduces non-determinism in replay if the dedup table isn't itself in the log; resolve by encoding the `request_id` in the event so replay re-establishes the dedup table.

---

### #4. Schema evolution with upcasters — **STRONG ALIGNMENT (BY EXCLUSION), ELEGANT**

**Status:** Pardosa-next explicitly **rejects in-place schema evolution** (per GEN-0002) and replaces it with the new-stream migration model: schema change = new generation = new JetStream stream + new genome file. Old stream stays read-only during a grace window.

**Alignment:**
- ✅ The new-stream model **is** the upcaster: migration code reads old stream events and writes new-shape events to the new stream.
- ✅ Mixed-version readers handled by registry generation pointer + grace window (PAR-0005).
- ✅ Rollback = bump registry back to previous generation (atomic CAS).
- ⚠️ **Compatibility matrix (#4a) does not exist.** Pardosa needs a generation manifest declaring `{ schema_hash, pardosa_version, genome_format_version, created_at }` so tooling and ops can reason about cross-generation reads.
- ⚠️ **Mixed-version replay test (#4b)** maps to: spin up generation N consumer against generation N+1 stream; assert clean rejection or rebind.

**Pardosa-specific recommendation:** Add a `pardosa::generation::Manifest` type stored alongside the genome snapshot. CI gate: schema_hash drift between generations N and N+1 is allowed only if a *named* migration policy exists in code that maps old → new events. Without that, the build fails — preventing accidental wire breakage.

---

### #5. Tamper-evident hash chain — **STRUCTURAL FOUNDATION EXISTS, NO CRYPTO**

**Status:** Pardosa already has a *structural* precursor chain: `Event::precursor: Index` + `Dragline::verify_precursor_chains()`. This is the load-bearing analogue to the recommendation, but it uses **indices, not cryptographic hashes**.

**Alignment:**
- ✅ Per-fiber chain shape is in place; `verify_precursor_chains` is called on startup (PAR-0012).
- ❌ Chain is index-based, so any privileged actor with file access can substitute event payloads or rewrite the line entirely; structure remains valid, semantics change silently.
- ❌ No global chain (across fibers) — only intra-fiber.

**Pardosa-specific recommendation:** Two changes, both small:
1. Add `Event::precursor_hash: [u8; 32]` (BLAKE3 of previous event in same fiber's canonical genome bytes). Verification cost on startup is the same O(n) walk that already happens.
2. Add a global `Dragline::frontier_hash` that rolls forward across all events in append order, periodically anchored to the JetStream subject metadata or to an external transparency log. This is the analogue of "checkpoint hash chain" and gives non-repudiation across the whole line, not just within a fiber.

This change is **cheap** because pardosa's serialization story is genome — canonical bytes are already a target. The hash field becomes part of the immutable layout (PAR-0003 handles non_exhaustive) and the schema_hash (GEN-0003) ensures readers across versions agree on what bytes to hash.

---

### #6. Deterministic simulation + fault injection — **STRONG ALIGNMENT, INFRASTRUCTURE NEEDED**

**Status:** Pardosa's design is *unusually* friendly to deterministic simulation:
- `Dragline` is single-threaded, in-memory, no I/O.
- The state machine is a pure data table.
- Timestamps are caller-provided (not `now()`).
- `next_event_id` and `next_id` are deterministic counters.

**Alignment:**
- ✅ Existing proptest (`arbitrary_sequences_preserve_precursor_chains`) is the seed of a real simulator.
- ✅ State machine (PAR-0001) is verifiable as a graph: totality, reachability, terminal paths.
- ❌ JetStream + KV layer (Phase 5/6) is where determinism gets hard. Need madsim or in-process JetStream mock.
- ❌ Linearizability checking (Knossos/Porcupine-style) is absent.

**Pardosa-specific recommendation:** Promote the existing proptest into a `pardosa::sim` module exposing `SimDragline` with injected clock, RNG, and a recording layer. Extend it with:
- Crash-and-replay: snapshot Dragline state, drop, replay from JetStream tail, assert byte-identical state.
- Concurrent-writer race: two writers, registry gen flip mid-flight, assert exactly one succeeds.
- State-machine totality validator: at startup or in a CI test, walk all `(FiberState, FiberAction)` pairs and assert the transition function is total over the intended domain.

This deserves its own ADR (`PAR-0017 Deterministic Simulation Harness`) because it's the substrate for verifying every other invariant.

---

### #7. Observability protocol — **NOT STARTED, MEDIUM URGENCY**

**Status:** No tracing, no metrics in pardosa today. Once the JetStream/NATS layer lands, observability becomes urgent because debugging distributed broker interactions without traces is masochistic.

**Alignment:**
- ❌ No `tracing` integration.
- ❌ No `Redacted<T>` — though `Event<T>` is generic, so user payloads might contain PII.
- ❌ No cardinality budget — `domain_id` and `event_id` are tempting metric labels and unbounded.

**Pardosa-specific recommendation:**
- Mandatory `tracing::instrument` on every `Dragline` write method, with span attributes `{ domain_id, event_id, fiber_state_before, fiber_state_after }` (high-cardinality fields → span attrs only, never metric labels).
- Metrics: `pardosa_events_total{action, result}`, `pardosa_fibers_active{state}`, `pardosa_jetstream_publish_latency`, `pardosa_registry_generation` (gauge). All low-cardinality.
- Per-fiber detail at debug span level only; sampling enforced.
- Genome serialization should support a `Redacted<T>` wrapper that produces deterministic placeholder bytes — important because `Event<T>` payloads end up in audit logs.

---

### #8. Crash-consistency protocol beyond atomic rename — **STRONG ALIGNMENT, EXPLICIT IN ROADMAP**

**Status:** Pardosa-next Phase 4 (migration) and Phase 5 (persistence) explicitly specify durable phase markers in the NATS KV registry. PAR-0005 names the migration state machine. PAR-0007 specifies startup high-watermark reconciliation.

**Alignment:**
- ✅ Migration phases (`Started, Copying, ReadyToCutover, CutoverDone, CleanupNeeded`) match recommendation #8 exactly.
- ✅ Idempotent resume from any phase (PAR-0005).
- ✅ Multi-resource commits (JetStream + KV registry + genome file) are exactly the case the recommendation calls out.
- ⚠️ **Quarantine mode (#8b)** is implicit in `FiberState::Locked` for individual fibers but there is no *stream-level* quarantine. Recommendation: add a registry value field `health: { Healthy | Quarantined { reason } }`; quarantined streams reject writes, allow reads, surface via observability.
- ⚠️ **Repair tool with bounded authority (#8a)** is missing. Operations like "drop the last N events because writer crashed mid-publish-batch" need a constrained CLI, not direct JetStream surgery.
- ❌ **fsync-failure handling on the genome file** — Phase 5 specifies atomic write but not fsync EIO behavior. Inherit cherry-pit-gateway's discipline + the EIO-fences-writer rule.

**Pardosa-specific recommendation:** Migration state machine in code is high priority for Phase 4. Stream-level quarantine and a `pardosa-admin` repair CLI are net-new and should land before any production deployment.

---

### #9. Object-store backend — **DOES NOT APPLY**

**Status:** Pardosa has chosen JetStream as durability + genome files as snapshots. Object store is not on the roadmap.

**Adjacent applicability:**
- Genome snapshot files **could** live on object store for archival; this is a future optimization, not a blocker.
- The recommendation's *substance* (per-provider consistency audit, conformance suite) translates to **JetStream cluster mode auditing**: behavior under broker failover, KV CAS semantics under partition, JetStream replication mode (R3, R5) effects on ordering. These are real concerns that PAR ADRs don't currently address.

**Pardosa-specific recommendation:** Add a `PAR-0018 JetStream Cluster Topology Assumptions` ADR specifying:
- Required JetStream replication factor (R3 minimum for production claims).
- Behavior under leader election (acks may be lost; client must retry; `Nats-Msg-Id` makes this safe).
- KV bucket replication mode and CAS semantics under partition.
- Conformance test against `nats-server` in cluster mode with chaos (kill leader mid-publish, mid-CAS).

---

### #10. Genome encoder/decoder — **PARDOSA IS THE DRIVER**

**Status:** Pardosa's persistence layer (Phase 5) is **completely blocked** on genome being usable. Genome's design choices (canonical encoding, schema hash, fixed layout) are exactly what pardosa needs to make the new-stream migration model work — schema_hash collisions across generations are the integrity check.

**Alignment:**
- ✅ Genome layout comments already in `pardosa::event` (`// GENOME LAYOUT: single u64 field`) — design is pardosa-aware.
- ✅ Pardosa's append-only Vec maps cleanly to a contiguous genome page sequence.
- ✅ Schema hash (GEN-0003) is the natural cross-generation compatibility check.
- ❌ Encoder/decoder unimplemented; pardosa Phase 5 cannot start.
- ❌ The `Event<T>` generic + `T: GenomeSafe` bound needs to land; today nothing constrains `T`.

**Pardosa-specific recommendation:**
- Add the `T: GenomeSafe` bound to `Event<T>` and `Dragline<T>` once genome ships. This is a clean break per the pardosa-next "no backwards compatibility" stance.
- Generate a golden genome fixture for each `pardosa::event::Event<TestPayload>` shape and lock it in CI. Schema_hash drift on `Event` itself (because pardosa adds a field, e.g. `precursor_hash` from #5) is detected immediately.
- Cross-language read export (#10d) is explicitly deferred per GEN-0031 but pardosa would benefit early — it makes pardosa streams consumable by non-Rust observability tooling.

---

## Summary alignment table

| Item | ADR coverage | Code status | Pardosa fit | Priority for pardosa |
|---|---|---|---|---|
| #1 Outbox / publish-then-apply | PAR-0008, PAR-0015 design-only | Code violates PAR-0008 (apply-then-publish) | Native — JetStream is the outbox | **HIGH** — fix Phase 2 API split |
| #2 Lease / epoch fencing | PAR-0004, PAR-0013 design-only | Not implemented | Native — gen+JetStream seq | **HIGH** — Phase 6 |
| #3 Idempotency keys | PAR-0007 publish-only | Command-level absent | Gap | **MEDIUM** — add `request_id` to write API |
| #4 Schema evolution | PAR-0005 (new-stream model) | Not implemented | **Excellent** — by exclusion | **HIGH** — Phase 4 |
| #5 Hash chain | None — only structural precursor | Index-based chain only | Easy retrofit | **MEDIUM** — small change, big audit gain |
| #6 Sim / fault injection | None | Proptest seed only | **Excellent** — pure data | **HIGH** — propose `PAR-0017` |
| #7 Observability | None | Not started | Medium | **MEDIUM** — needed before NATS lands |
| #8 Crash consistency | PAR-0005, PAR-0007 | Not implemented | **Excellent** — explicit phases | **HIGH** — Phase 4/5 |
| #9 Object store | N/A | N/A | N/A | Re-scope to JetStream cluster audit |
| #10 Genome | GEN-0001..0034 (design); pardosa-next Phase 5 | Scaffold only | **Pardosa is the driver** | **CRITICAL** — Phase 5 blocked |

## Three pardosa-specific recommendations the top-10 doesn't surface

1. **Cross-stream-ordering disclaimer is a load-bearing API contract.** PAR-0016 (proposed) says "no global cross-stream order, only per-stream + correlation keys." This is correct but currently *invisible* to consumers: `event_id` is monotonic *within* a stream/generation, and developers will reach for it cross-stream and silently get nonsense. Recommendation: name it `StreamMonotonic(u64)` not `event_id`, so the type itself prevents misuse. Compile-time prevention beats documentation.

2. **`FiberAction::Migrate` blurs domain and infra concerns.** The state machine treats migration as just another action, which is elegant for replay but means migration policy (Purge/LockAndPrune/Keep) is encoded in event payloads. This couples ops decisions to durable history. Recommendation: split into a separate `MigrationLog` stream (one per generation transition) so domain events stay pure and ops events stay separate. Also enables operator audit independent of domain replay.

3. **`Dragline` is an in-memory replica of JetStream — make this explicit in types.** Today `Dragline` has both authoritative semantics (assigns event_ids in `create()`) and replica semantics (tests inject events directly into `line`). Once Phase 5/6 lands, only JetStream is authoritative. Recommendation: rename to `DraglineReplica` with a constructor `from_jetstream_tail()` and the only mutating methods being `apply_committed(event)` (after broker ack). The `reserve/commit` split from #1 makes this natural. This is the "lease as identity" theme from the parent doc — only the lease-holder can reserve; everyone else can only apply committed events.

## Suggested pardosa sequencing

1. **Genome encoder/decoder (#10)** — unblocks everything else. Pardosa is downstream of this work.
2. **Simulation harness (#6)** — needs to land alongside Phase 2 refactor; cheap when Dragline is still pure.
3. **Reserve/commit API split (#1)** — refactor before adding any I/O, while the API has no external users.
4. **Hash chain (#5)** — bolt on while genome layout is being finalized; free integration with schema_hash.
5. **Observability (#7)** — instrument as Phase 5/6 NATS code lands, not after.
6. **Lease/epoch + KV registry (#2, #8)** — Phase 6 is exactly this.
7. **Idempotency keys (#3)** — last; needs the lease layer to be meaningful.
8. **JetStream cluster audit (replaces #9)** — production readiness gate; final.
