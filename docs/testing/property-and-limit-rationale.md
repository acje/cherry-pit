# Property and limit rationale (cherry-pit)

Purpose: preserve the design rationale that lives apart from Rust doc
comments, so each test or public type keeps its "why" traceable without
reproducing non-contract prose in the source. This is the durable home for
rationale that is not part of a runtime/API contract; contract text
(Errors/Panics/ownership/resource/cancel/retry/atomicity/spec references,
doctests) remains in the source doc comments.

## Per-aggregate ordering (crates/cherry-pit-app/tests/per_aggregate_ordering.rs)

Anchored at `per_aggregate_dispatch_order_matches_publish_order`.

Under the F2 design, synchronous bus fan-out (CHE-0024:§7) feeds a bounded
`tokio::sync::mpsc` channel drained by one sequential consumer. Per-aggregate
order is preserved because envelopes arrive in publish order and the channel
is FIFO: if `(agg=a, seq=n)` publishes before `(agg=a, seq=n+1)`, `n` enters
the channel first and the consumer pulls it first. Trivially true by
construction; the proptest observes it under a random schedule across
multiple aggregates, confirming that the orphan-`handle.spawn` design
(pre-F2) that violated it is gone.

Scope: the test exercises the bus → `enqueue_or_log` → channel → consumer
pipeline directly. `run_dispatch_consumer` is unit-tested in
`app.rs::tests`. Channel capacity is generous (`8 * N`) so back-pressure
does not drop envelopes here; drops are tested separately in
`app.rs::tests::full_dispatch_channel_drops_overflow_…`.

## Comparative rate-limit evidence (crates/cherry-pit-wq/tests/comparative.rs)

Anchored at the `p1_*` properties, `phantom_304_headline_contrast`, the
`l4_*` layer-4 tests, and `f3_seam_accepts_test_double_retry_after_regulator_current_has_no_injection_path`.

### Why the harness drives `BudgetGate` directly (no-refund pattern)

`run_worker_pool`'s `worker_loop` calls `budget_gate.acquire()` and never
calls `BudgetGate::refund`. The `JobExecutor` trait can express a
free/charged classification (`charge_of`), yet the legacy `BudgetGate`
path this harness mirrors still exhibits no refund (as `p1_current`
observes with `calls_made() == schedule.len()`); the harness therefore
exercises `BudgetGate` directly with the same no-refund-ever pattern
`worker_loop` uses, rather than routing through `run_worker_pool` itself.
That is the CURRENT design's phantom-304 behaviour, not a simplification
of it.

### F1 property (ii) — CURRENT headline evidence

Anchored at `p1_current_budget_gate_conflates_free_and_charged`. Because
`worker_loop` never calls `BudgetGate::refund`, `calls_made()` after any
schedule equals `schedule.len()` regardless of how many draws were actually
free. When `schedule.len() == limit` and every draw is free, CURRENT reaches
full epoch exhaustion purely from free (304) draws — phantom exhaustion, the
historical bug this evidence captures rather than fixes
(`out_of_scope`: the frozen path is not modified).

### F1 property (i) + B's structural contrast

Anchored at `p1_b_token_bucket_debits_uniformly_regardless_of_flag`.
`TokenBucketRegulator` is documented RATE-only with a no-op `settle`:
it debits every admitted draw regardless of the free/charged flag, by
construction, because there is no charge-tracking state to conflate.
`consumed <= capacity_tokens` (property i) holds under burst load. B does
not attempt, and structurally cannot corrupt, A's free/charged conservation
invariant because it never models that axis at all. See
`phantom_304_headline_contrast` for the decisive cross-design pause-behaviour
contrast.

### F1 property (iii) — A's draw-on-confirmed-charge conservation

Anchored at `p1_a_budget_regulator_conserves_on_free_settle` (128-case
proptest over `limit_and_schedule`: mixed free/charged schedules of all
lengths up to `limit`). This is the general conservation property; the
free-only `phantom_304_headline_contrast` case is a special case of it,
not the full property rationale.

For the A path's `BudgetRegulator`, `settle(Free)` releases the permit the
draw admitted under and `settle(Charged)` leaves it consumed.
Consequently `gate.calls_made()` after any schedule must equal exactly the
count of `charged` (non-free) draws — free draws leave zero residue. The
property checks final charged-count equality after each generated schedule
across 128 proptest cases. This conservation invariant is structurally
absent from
CURRENT (whose per-draw conservation is the `p1_current` conflation
evidence above) and is the target the free-only headline showcases in its
sharpest all-free form.

### Phantom-304 headline contrast

Anchored at `phantom_304_headline_contrast` (fixed illustrative case, not
proptest-shrunk): 5 free-only (304) draws against a limit/capacity of 5.

- CURRENT: `calls_made() == 5` — the epoch is fully exhausted purely by free
  draws; the next real call requires the full `wait_duration`
  cooldown-then-reset sleep (`budget.rs::acquire`) even though zero real
  charges occurred. This is the phantom-304 bug.
- A: `calls_made() == 0` — every free draw is refunded via `settle(Free)`;
  zero phantom cost, full headroom retained.
- B: consumes all 5 tokens (no free/charged distinction exists), but refill
  is a continuous pure function of elapsed time — advancing the clock by
  exactly one token's worth of time admits one more draw immediately, with
  no discrete "wait the full cooldown" stall the way CURRENT's
  elected-resetter path requires. B eliminates the phantom *pause* class
  structurally (no `resetting` election field exists in
  `TokenBucketRegulator`, contrasted with `BudgetGate`'s `resetting:
  AtomicBool`), even though it does not model per-draw free/charged
  accounting the way A does.

### Layer 4 generalisation

Anchored at the `l4_*` tests. `tokio::time::pause`/`advance` drives
simulated multi-hour epoch cycles deterministically (no wall-clock cost),
generalising `phantom_304_headline_contrast` from a single epoch to a
multi-hour storm — the shape Layer 4 asks for.

### F3 — why CURRENT has no Retry-After injection path

Anchored at the `f3_seam_*` test. CURRENT `cherry_pit_wq::run_worker_pool`
takes `budget_gate: Arc<BudgetGate>, rate_limit_state:
Arc<RateLimitState>` — two fixed concrete types, not a slice of trait
objects — so there is no way to substitute or add a Retry-After-aware gate
without changing its signature. This is a compile-time API fact, not a
runtime behaviour to assert. The `Regulator` seam accepts the
`RetryAfterTestDouble` (dyn-compatible, composes into an ordered
`&[Arc<dyn Regulator>]` chain beside `BudgetRegulator`); no concrete
Retry-After regulator ships from this work (`out_of_scope`).

## Error mapping rationale (crates/cherry-pit-web/src/middleware/error.rs)

Anchored at `dispatch_infrastructure_maps_to_503_retryable` and
`dispatch_rejected_maps_to_422_with_lossless_body`.

- `DispatchError::Infrastructure` sits at the dispatch layer, not the store
  layer R10 enumerates, but maps to the same 503 + `Retry-After` signal by
  the same retryable reasoning; 500 stays reserved for terminal
  `CorruptData` (baseline 503-contrast — `Indeterminate` outcomes also map
  to 500, so this line is not an exhaustive 500 map).
- `Rejected(E)`'s `Display` is preserved in full via `ErrorBody::message`
  (CHE-0015). A `Serialize` bound on the gateway error generic would tighten
  CHE-0049 R1, so none is required.

## Web limits rationale (crates/cherry-pit-web/src/middleware/limits.rs)

- The WS connection cap is deliberately not on `LayerLimits`. It lives on
  SEC-0012's `WsPolicy` beside the Origin policy, because a surface that
  mounts a `/ws` upgrade needs both to be safe; carrying them separately let
  `serve` take the cap and omit the policy — the reversal is recorded in
  SEC-0012's Consequences (CHE-0062:R1/R2).
- `LayerLimits` omits `Default`: a defaulted permissive value is a SEC-0003
  footgun. Tests use `permissive_for_tests`; production names both values.
  `NonZeroUsize` hard upper bounds (`NonZeroUsize::MAX` unbounded in
  practice), each unconditionally honoured per CHE-0062:R4.
- Adding a future `LayerLimits` field is a semver-major event for
  cherry-pit-web (CHE-0062:R6). The crate is internal and `Cargo.lock` is
  committed per the crate README, so the workspace tolerates this (this
  reason also appears in the type's rustdoc).

## Store doctest shape (crates/cherry-pit-core/src/store.rs)

Anchored at `EventStore` (trait-level doctest). The doctest type-checks the
trait surface without `.await`: RPITIT (CHE-0018:R2) needs a runtime, and
`cherry-pit-core` has zero async-runtime deps (CHE-0029:R4).

## Dispatch erasure and telemetry (crates/cherry-pit-app/src/dispatch.rs)

- Policies are stored as `Vec<Box<dyn ErasedPolicyDispatcher<G>>>` — a
  per-policy adapter. Erasure sits at the dispatcher boundary, never on
  the infra `Policy` port (CHE-0005:R1 forbids erasing infra ports);
  the dispatcher itself is the erasure point.
- `correlation_for`'s `tracing::debug!` is the paired runtime-telemetry
  mechanism for the correlation branch the type system cannot constrain
  (SEC-0005:R4), satisfying GND-0005:R1+R2 for CHE-0051:R6 / CHE-0039:R1–R3.
- `DispatcherList` is unit-tested but not yet driven by `App::run`.

## WebSocket policy carrier history (crates/cherry-pit-web/src/middleware/ws_auth.rs)

The two knobs (`max_connections`, `origin_policy`) are fused in
`WsPolicy` deliberately. They were once split across `LayerLimits`
(availability) and `WsAuthLimits` (authenticity) to keep CISQ primaries
MECE; that split let `serve::build_router` take the WS cap and never take
a policy, so no origin-strict WebSocket ran in production for the eight
weeks SEC-0012:R2 declared `Strict` the default. Grouping the knobs one
capability needs to be safe makes the omission a compile error rather
than an absence nobody can see (reversal recorded in SEC-0012's
Consequences).

## Scheduler store composition (crates/pardosa-cherry-pit-test-support/src/scheduler_store.rs)

`PgnoSchedulerStore` delegates to `PgnoEventStore<SchedulerEventDto>`
rather than adding a generic `PgnoEventStore<Ev, Dto, Converter>` variant:
the `SchedulerEvent <-> SchedulerEventDto` mapping is a fixed 1:1
relationship with exactly one consumer, so a converter parameter would
add an indirection layer (a converter trait plus its own bound set) only
this single call site would instantiate. Delegation reuses the substrate
(per-aggregate locking, optimistic-concurrency sequence checks,
single-event-only atomicity, restart recovery) with zero duplicated
logic; only the boundary conversion is new.

## F1 phantom-304 historical cause (crates/cherry-pit-wq/tests/f1_phantom_304.rs)

Historically `worker_loop_regulated` hardcoded
`regulator.settle(SettleOutcome::Charged)` for every admitted job
regardless of whether the result actually consumed the guarded resource.
This drove the F1 34x overcount / spurious 1h-freeze bug: a job whose
real-world effect was free (e.g. GitHub 304 not-modified) still charged
the budget permit. The acceptance test drives the live regulated path
with a `charge_of` reporting every outcome as `SettleOutcome::Free` and
asserts conservation (epoch limit 1, long cooldown).

## Origin-proptest donor provenance (crates/cherry-pit-web/tests/projection_proptest.rs)

The preflight and WS-origin proptest families derive from donor
`server.rs:3745`–3835 (pure-function path block) and `server.rs:3800`–
3835 (Origin/Host strategy). The pure-function block was 256 cases in
the donor; the HTTP-integration origin block trades case breadth for
end-to-end coverage at `ORIGIN_CASES = 64` (~192 server lifecycles,
~10–20 s wall-clock). Property intent is carried by the stable test
names; the donor line pointers were removed from the macro bodies.

## WebSocket origin policy — ambient-credentials rationale (crates/cherry-pit-web/src/middleware/ws_auth.rs)

Source-exact (8ae55f5): ambient credentials are the textbook CSWSH
vector — the browser attaches the victim's cookies to a cross-origin
upgrade — but they are not the only one, and a service that sets no
cookies is not therefore safe. The socket pushes data *after* the
handshake, so a successful cross-origin upgrade is a read primitive
over whatever the surface publishes regardless of how the request was
authenticated.

This paragraph was compressed out of the `WebSocketOriginPolicy` enum
doc during the comment-free budget reduction while the block sat at the
120-word enforced limit; it is preserved here so the SEC-0012 decision
reasoning stays discoverable.

## r3_apply idempotence anchor (crates/cherry-pit-projection/src/lib.rs)

Anchor: `r3_apply_is_idempotent_over_a_fixed_event_stream`.
Source-exact (8ae55f5): CHE-0048:R3 — `apply` is deterministic and
idempotent over a fixed event stream: replaying the same envelope
sequence twice against fresh projections yields equal final states.
Two independent replays exercise the property without depending on
driver-internal retry semantics.

This paragraph was compressed out of the `r3_apply` proptest doc
comment during the budget reduction; the property intent is carried by
the stable test name, and the full rationale is preserved here.
