# Top 10 Distributed-Systems Improvements — Synthesis from ADR Review

Date: 2026-04-29

Source: `docs/adr/DISTRIBUTED_SYSTEMS_ADR_REVIEW.md` (149 ADRs reviewed).
Cross-referenced against current code state in `cherry-pit-core`, `cherry-pit-gateway`, `pardosa`, `pardosa-genome`.

Selection rubric: (1) blast radius if wrong, (2) leverage — fixes many ADR concerns at once, (3) implementation gap — designed but not built, (4) precondition for other features.

---

## 1. Durable Outbox + Publish-Then-Apply

**ADR roots:** PAR-0008, CHE-0024, COM-0022, CHE-0040
**Current state:** `EventBus` trait exists; no outbox table, no transactional publish. `MsgpackFileStore` writes events but nothing reads them for delivery. CHE-0040 (sagas) and CHE-0024 (catch-up) both depend on this.

**Why top:** Without a durable outbox, every cross-aggregate workflow, projection, and policy is fundamentally lossy under crash. This is the keystone that unlocks sagas, projections, and consumer recovery.

**Core implementation:**
- An `Outbox` port in `cherry-pit-core` with `enqueue`, `claim_batch`, `ack`, `nack` operations.
- File-based outbox adapter mirroring `MsgpackFileStore`'s atomic-write discipline; per-aggregate write appends (event, outbox-row) atomically.
- Background publisher task: claim → publish → ack, with bounded retry and dead-letter on terminal errors.
- Consumer-side checkpoint/cursor port so catch-up is precise.

**Connected improvements:**
- **1a. Reserved-event construction (PAR-0008):** split append into `reserve` (sequence + envelope frozen, holds lock) and `commit` (releases lock), so network publish never holds the state mutex.
- **1b. Dead-letter queue (CHE-0040, PAR-0015):** terminal failures land in a DLQ stream with original envelope + error category + retry count; replay tool to resubmit.
- **1c. Idempotent consumer contract:** consumer trait must declare its dedup key (envelope id by default) and provide checkpoint storage; framework rejects non-idempotent consumers at compile time via marker trait.
- **1d. Catch-up cursor protocol:** `EventBus::subscribe_from(stream, after_seq)` with explicit "missed events" recovery, replacing best-effort fan-out.

---

## 2. Lease/Epoch Fencing for Single-Writer Ownership

**ADR roots:** COM-0018, CHE-0006, CHE-0043, PAR-0004, SEC-0006
**Current state:** Advisory `flock` only; works in-process, breaks across NFS, SMB, and most container restart races. No epoch in envelopes.

**Why top:** Every "single-writer" claim in the corpus is structurally fragile without fencing. Stale writers + retries = silent divergence. This is correctness-critical.

**Core implementation:**
- `WriterLease { writer_id: Uuid, epoch: NonZeroU64, expires: Timestamp }` issued by a coordinator (file-based for local; KV-based for NATS deployments).
- Each `EventEnvelope` gains a `writer_epoch: NonZeroU64` field; appends are rejected if the store-side max-epoch > envelope's epoch.
- Lease renewal task with jittered heartbeats; expired leases must be re-acquired with strictly increasing epoch.
- Crash recovery: on startup, read max epoch in stream, demand new epoch ≥ max+1 before any append.

**Connected improvements:**
- **2a. Stale-writer rejection on read path (SEC-0006):** projections refuse to apply events from epochs lower than the latest seen for that stream — detects split-brain at consumer side.
- **2b. NATS KV registry (PAR-0013) as authoritative lease store:** with `compare-and-swap` on `writer_epoch` value; rich metadata (writer identity, cutover timestamp, previous stream).
- **2c. Lease-aware `CommandGateway`:** gateway holds the lease; route commands to the holder; clean rejection (`StoreError::NotLeader { current: WriterId }`) for misrouted writes.

---

## 3. Idempotency Keys as First-Class Boundary Type

**ADR roots:** CHE-0041, CHE-0013, CHE-0020, COM-0022
**Current state:** Deferred to applications. Documented as "carry your own key in payload." Means every adopter rebuilds this and most do it wrong.

**Why top:** Retried `create` commands silently allocate distinct aggregate IDs today. This is the single most common distributed correctness bug in event-sourced systems and the framework currently has no answer.

**Core implementation:**
- `IdempotencyKey` newtype (NonZero bytes, opaque) on `Command` trait via associated type (default `()` = no dedup).
- `CommandGateway` looks up `(idempotency_key, command_type)` in a TTL'd dedup table before dispatch; on hit, returns cached result envelope.
- Integration with the outbox so dedup-table writes are atomic with event writes.

**Connected improvements:**
- **3a. Domain-uniqueness keys (CHE-0020):** for natural-key creates (e.g., `User.email`), a compile-time-checked `UniqueBy` trait that requires a domain index port; framework rejects duplicate creates regardless of idempotency key.
- **3b. Returning the original result:** dedup must return *the same* `CommandResult` (event ids, aggregate id) to retried callers — requires storing command outcome, not just the key.
- **3c. Compensation-aware dedup (CHE-0021):** classify dedup hits as `Replayed`, `Compensated`, `Failed`, so saga drivers can act correctly.

---

## 4. Schema Evolution Protocol with Upcasters and Rollback

**ADR roots:** CHE-0022, CHE-0010, COM-0021, GEN-0002 (genome takes the *opposite* stance — fixed layout)
**Current state:** Envelope has `#[serde(default)]` on a couple of fields; no upcaster trait, no version field, no rollback story. CHE-0031 specifies MessagePack-named encoding but doesn't address evolution.

**Why top:** First "remove a field" or "split a variant" requirement will stop the world. Rolling deploys = mixed-version writers/readers reading each other's bytes. Must be designed before persisted bytes accumulate.

**Core implementation:**
- `EventEnvelope` gains `payload_schema_version: u32` (NonZero, defaults to 1).
- `Upcaster<E>` trait: `fn upcast(version: u32, bytes: &[u8]) -> Result<E, UpcastError>`; framework chains upcasters version-by-version on load.
- `Downcaster` (optional, gated): for rollback windows where N+1 events must be readable by N-1 binaries during canary.
- Golden fixture per (event_type, version) committed to repo; CI fails on accidental wire change.

**Connected improvements:**
- **4a. Compatibility matrix (COM-0021):** machine-readable matrix declaring `wire`, `persisted`, `public-API` compatibility per crate version; `cargo cherrypit check-compat` job in CI.
- **4b. Mixed-version replay test:** property test loads events written by version N, applies binary N-1 and N+1, asserts round-trip equivalence on logical state.
- **4c. Upcaster registry collision detection (CHE-0010):** event-type discriminator strings + version pairs registered globally; macro forbids collisions across crates.

---

## 5. Tamper-Evident Hash Chain on Envelopes

**ADR roots:** SEC-0008, SEC-0011, GEN-0016, PAR (precursor chain has structural form already)
**Current state:** `pardosa::Event` has a precursor *index*, but no cryptographic linkage. `cherry-pit-core::EventEnvelope` has no chain at all. SEC-0008 only claims tamper evidence via "append-only API" — which is not tamper evidence.

**Why top:** Non-repudiation and integrity claims in SEC ADRs are currently aspirational. Adding a chain is mechanically simple, costs ~32 bytes/event, and turns audit logs into actually auditable artifacts.

**Core implementation:**
- `EventEnvelope.prev_hash: [u8; 32]` (BLAKE3 of previous envelope bytes; zero hash for sequence 1).
- Hash computed over canonical serialized envelope minus `prev_hash` field itself.
- Verification port: `verify_chain(stream)` walks events, recomputes hashes, returns first divergence. Run on startup (cheap) or on demand.
- Anchoring port (optional): periodic publish of latest hash to external log (S3 with object-lock, transparency log, NATS subject).

**Connected improvements:**
- **5a. Authenticated checksums for genome (GEN-0016):** offer BLAKE3-keyed-MAC variant alongside xxHash64 for cross-trust-boundary transport.
- **5b. Pardosa: cryptographic precursor (PAR-0012):** upgrade `Event::precursor` from index-based to hash-based; replay verification proves no semantic substitution, not just no missing predecessor.
- **5c. Authenticity binding (SEC-0005):** envelope gains optional `signature: Option<Signature>` produced by writer's lease key; correlation IDs are no longer forgeable.

---

## 6. Deterministic Simulation + Fault Injection Harness

**ADR roots:** COM-0017, COM-0024, CHE-0038, GEN-0034
**Current state:** Property tests cover local invariants (envelope serde, AggregateId, dragline). No simulation, no fault injection, no concurrent-schedule exploration. Distributed correctness is currently asserted, not tested.

**Why top:** Every other improvement on this list (outbox, fencing, idempotency, schema evolution) requires adversarial-schedule testing to prove correctness. Without a simulator, these are vibes.

**Core implementation:**
- `cherry-pit-sim` test crate: virtualizes time (`jiff::Timestamp` injection per CHE-0034), randomness, filesystem, network.
- Operations recorded as a deterministic seed-driven schedule; on failure, minimize and emit reproducer.
- Fault catalog: process crash mid-write, partial fsync, packet drop/duplicate/reorder, clock jump, disk-full, lease expiry race.
- Linearizability checker (Knossos-style): records concurrent op history and validates against sequential spec.

**Connected improvements:**
- **6a. Per-invariant fuzz targets (GEN-0034):** structured fuzzer per GEN-0011 verification check; CI tracks coverage map → invariant.
- **6b. Loom for in-process concurrency (CHE-0035):** model-check the two-level concurrency architecture under all interleavings of cancellation + lock acquisition.
- **6c. Madsim or shuttle integration:** swap tokio runtime in tests; run full integration suite under randomized scheduling.
- **6d. Conformance vectors:** byte-level golden fixtures for genome, envelope, and outbox formats — ensures independent reimplementations remain compatible.

---

## 7. Observability Protocol with Cardinality Budgets and Redaction

**ADR roots:** COM-0019, SEC-0007, SEC-0003
**Current state:** Zero. No `tracing`, no metrics, no redaction types. Operating distributed software without observability is operating blind; once added carelessly, observability becomes the leak vector.

**Why top:** Every production incident in distributed systems is debugged through logs/traces/metrics. The "designed-once" decisions here (cardinality limits, redaction, sampling) are extremely costly to retrofit.

**Core implementation:**
- `cherry-pit-observability` crate with `tracing` integration; mandatory span around every command dispatch, event apply, store operation, publish.
- `Redacted<T>` newtype with `Debug`/`Display` returning `***`; `Serialize` only inside `serialization::trusted_scope`.
- Cardinality limiter: metric labels checked at registration time against a budget; high-cardinality fields (aggregate id, correlation id) are span attributes only, not metric labels.
- Required spans/metrics declared per port via trait documentation contract.

**Connected improvements:**
- **7a. Logging-test framework (SEC-0007):** `assert_no_secret_in_logs!(test_body)` macro that runs the body with a capturing subscriber and fails if any `Redacted` field's plaintext appears.
- **7b. Per-tenant quota integration (SEC-0003):** observability tags requests with `OriginId`; rate-limit/quota enforcement uses the same identity.
- **7c. Sampling policy as type:** `Sampler` enum (`Always`, `RateLimit(per_sec)`, `Reservoir(n)`); compile-time enforced at high-volume span sites.
- **7d. Observability self-test:** synthetic span injected at boot, asserted to reach configured backend; fail-fast on misconfigured exporters.

---

## 8. Crash-Consistency Protocol Beyond Atomic Rename

**ADR roots:** CHE-0032, COM-0023, PAR-0005, PAR-0007
**Current state:** `MsgpackFileStore` does atomic temp+rename+fsync(parent); `recover_temp_files` cleans orphans on first write per process. Good — but only covers one shape of crash.

**Why top:** Real crash bugs hide in: directory entries created out of order, multi-file commits (event + outbox + dedup), durable phase markers for migrations. The current protocol doesn't extend to multi-resource commits.

**Core implementation:**
- `WriteAheadLog` for multi-file commits: log intent → fsync → execute → fsync → mark complete; recovery replays incomplete intents.
- Migration state machine (PAR-0005) with durable phase markers: `{ Started, Copying, ReadyToCutover, CutoverDone, CleanupNeeded }`; idempotent resume from any phase.
- Startup high-watermark reconciliation (PAR-0007): max envelope sequence, max writer epoch, max outbox cursor — proven consistent before accepting any writes.
- Sequence monotonicity tests under wraparound and clock rollback (COM-0023).

**Connected improvements:**
- **8a. Repair tool with bounded authority (COM-0020):** `cherrypit repair --stream X --action drop-tail --confirm $hash` — explicit safe path for the maintenance tasks that *will* be needed.
- **8b. Quarantine mode for poison events (CHE-0009):** instead of process-wide panic, mark stream as quarantined, surface via observability, allow other streams to continue. Recovery drains via repair tool.
- **8c. fsync-failure handling:** on EIO, the file's pages are gone — current code retries; correct behavior is to fence the writer and surface a fatal store error (Postgres learned this the hard way).

---

## 9. Object-Store Backend with Provider Consistency Audit

**ADR roots:** CHE-0044, CHE-0036 (file-per-stream limits), CHE-0019
**Current state:** Proposed only. File-per-stream-rewrite has known scale limits; object-store is the natural next backend, but each provider (S3, R2, GCS, Azure Blob, MinIO) has subtly different semantics for ETag, conditional put, listing, and read-after-write.

**Why top:** Choosing wrong here costs an entire backend implementation. A pre-implementation audit + conformance suite is cheap; finding out S3 Express has different semantics than S3 Standard mid-rollout is not.

**Core implementation:**
- `ObjectStore` port (use `object_store` crate as foundation).
- Conformance suite: every backend must pass tests for `put_if_match` race, `list_after_put` consistency, `delete_then_get`, multipart commit atomicity, ETag stability across copy.
- Per-provider compatibility matrix in repo; backend declares `ConsistencyProfile` (Strong, ReadAfterWrite, Eventual) and framework rejects unsafe operations under weak profiles.

**Connected improvements:**
- **9a. Segmented append-only storage (CHE-0036):** instead of file-per-stream rewrite, write append-only segments + manifest pointer; combines well with object-store's strong-consistent CAS on a single key (the manifest).
- **9b. Snapshot infrastructure compatibility (CHE-0037):** segment model is the natural snapshot host; ADR for snapshot introduction can land alongside object-store work.
- **9c. Backup/restore as first-class:** segments + manifest = trivially copyable; defines the recovery RPO/RTO contract operators need.

---

## 10. Genome Encoder/Decoder Implementation with Conformance Suite

**ADR roots:** GEN-0001 through GEN-0034 (the entire genome corpus is design-only)
**Current state:** Scaffold only. Schema-hash helpers and derive macro are done; encoder, decoder, two-pass writer, offset layout, verification catalog — all unimplemented. Without genome, pardosa-on-NATS has no on-wire format.

**Why top:** Pardosa, NATS integration, cross-process pardosa replay all blocked. Also the highest-density invariant cluster in the corpus (34 ADRs) — building it without conformance vectors guarantees future incompatibility.

**Core implementation:**
- Two-pass encoder per GEN-0005 with idempotency assertion (size pass output deterministically equals write pass).
- Decoder with the GEN-0011 inline check catalog as a typed pipeline; explicit fail-fast precedence.
- Page-class resource limits enforced per `DecodeOptions`, with trust-boundary defaults (GEN-0013).
- Canonical byte fixtures per type-shape committed to repo; cross-platform CI (big-endian QEMU emulation per GEN-0012).

**Connected improvements:**
- **10a. Reserved-field state machine (GEN-0015):** explicit reader behavior table (ignore | reject | upgrade-required) per reserved bit; tested via fuzz with reserved-bit mutations.
- **10b. Schema-hash collision audit (GEN-0003):** CI computes hashes for representative type set; fixture-locked to detect macro/syntax-driven hash drift across rust upgrades.
- **10c. Authenticated trailer (GEN-0025):** opt-in BLAKE3-MAC trailer for bare messages; lights up "transport-protected" claim for shared-memory and queue deployments.
- **10d. Cross-language read-only export (GEN-0031):** start a JSON Schema or Cap'n Proto-style schema dump now as diagnostic artifact; defers full interop but locks ground truth.

---

## Cross-cutting themes

These threads run through multiple items above and deserve their own meta-tracks:

- **Compatibility matrices everywhere.** Wire, persisted, public-API, and CLI surfaces all need explicit compatibility statements (#4, #9, #10, AFM-0013).
- **Property-based + fault-injection testing as a first-class crate.** #6 is leverage for #1, #2, #3, #4, #8.
- **Make the safe path the only path (COM-0020).** Every recovery/repair operation needs a constrained-authority tool; ad-hoc shell commands in production are how data dies (#1b, #8a, #8b).
- **Lease as identity.** Once leases exist (#2), they become the natural carrier for authenticity (#5c), routing (#2c), and observability tagging (#7b).

## Suggested sequencing

1. **#6 simulator** first — it underpins correctness claims for everything else.
2. **#2 fencing** + **#3 idempotency** in parallel — both required before any multi-process or remote-command deployment.
3. **#1 outbox** — depends on #2; unlocks projections, sagas, NATS.
4. **#4 schema evolution** — must land before persisted bytes are out in the wild at scale.
5. **#7 observability** — should land alongside #1; debugging the outbox without traces is masochism.
6. **#5 hash chain** — small, can land any time after #6.
7. **#8 crash-consistency hardening** — informed by #6 findings.
8. **#10 genome** — required for **#9 object store** and remote pardosa; sequenced after foundational invariants are testable.
9. **#9 object store** — last; benefits from everything above.
