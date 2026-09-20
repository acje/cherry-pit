# CHE-0051. cherry-pit-storage Canonical Import

Date: 2026-09-20
Last-reviewed: 2026-09-20
Tier: D
Status: Accepted

## Related

References: CHE-0032, CHE-0036, CHE-0043, CHE-0045

## Context

Canonical cherry-pit (`acje/cherry-pit`) had no synchronous filesystem
primitive crate. An evolved implementation — crash-safe atomic writes, an
RAII run-lock with TTL and dead-holder stale detection, and canonical-JSON
SHA-256 content signatures — matured in the consumer repository
`Mattilsynet/gh-report` at revision `c850737`, where it was governed by that
repository's local `CHE-0053`.

That donor ADR text is not accurate for canonical cherry-pit: it describes a
donor-local crate layout and re-export set, and canonical has no such ADR
number allocated. Importing the code while citing a non-existent canonical
ADR would leave every doc-comment reference dangling. This ADR is the narrow,
accurate canonical replacement, scoped strictly to the imported crate. It
ratifies the behaviour actually present in the imported source — nothing more.

Canonical `cherry-pit-core` and `cherry-pit-gateway` are untouched by this
import.

## Decision

Adopt `cherry-pit-storage` into canonical cherry-pit as a producer-only crate,
reconciled to donor revision `c850737`.

R1 [10]: `cherry-pit-storage` MUST NOT depend on `cherry-pit-core`. Retry
  classification is expressed by the crate-local `RetryClass` enum, mirroring
  the intent of `cherry_pit_core::ErrorCategory` without the dependency.

R2 [10]: Atomic writes MUST use temp-file + fsync + rename + parent-directory
  fsync. The parent-directory fsync is load-bearing for crash-safety
  (consistent with CHE-0032); removing it is a SemVer-major break.

R3 [10]: The public API is flat over private modules (`error`, `fs`, `lock`,
  `signature`). The re-export set is exactly: `PersistenceError`,
  `atomic_write_bytes`, `atomic_write_text`, `DEFAULT_LOCK_FILENAME`,
  `DEFAULT_LOCK_TTL`, `LockMetadata`, `RunLock`, `acquire`, `lock_path`,
  `build_snapshot_signature`.

R4 [10]: The public surface MUST remain synchronous — no `async fn`, tokio, or
  futures-util in the public API. Async callers wrap invocations in
  `tokio::task::spawn_blocking`. `tokio` is a dev-dependency only.

R5 [5]: Test witnesses imported from the donor MUST be retained in full. The
  donor's `tests/smoke.rs` scaffold placeholder is retained verbatim pending
  the crate's own taxonomy work.

R6 [10]: Lock acquisition MUST be TOCTOU-free, published via
  `persist_noclobber` (`link(2)`). Stale locks are reclaimed only on
  TTL expiry or a dead holder on the *same* host; a lock file without a
  recorded hostname is TTL-only and MUST NOT be auto-stolen.

R7 [5]: `LockMetadata` is a serde DTO. Schema evolution is handled by
  field-presence plus `#[serde(default)]`, NOT by `#[non_exhaustive]`.
  Canonical closed-error-enum policy (CHE-0049) is scoped to public error
  types, not infrastructure DTOs.

## Provenance

| Donor | `Mattilsynet/gh-report` @ `c850737` |
|---|---|
| Donor path | `crates/cherry-pit-storage/` |
| Canonical path | `crates/cherry-pit-storage/` |
| Files imported | 9 (Cargo.toml, README.md, 4 × src, 2 × tests) |
| Donor line counts | error 186, fs 104, lib 68, lock 1446, signature 278, properties 226, smoke 5 |

Source `src/error.rs`, `src/fs.rs`, `src/lock.rs`, `src/signature.rs` and
`tests/properties.rs` are byte-identical to the donor except where a donor-local
ADR citation was rewritten to this ADR. `src/lib.rs`, `README.md`,
`tests/smoke.rs` and `Cargo.toml` differ only in ADR citation, repository URL,
and homepage metadata.

Donor doc-comments retain references to donor-corpus ADRs that do not exist in
canonical (`CHE-0021`, `CHE-0088`, `SEC-0006`, `COM-0025`, `PGN-0016`). These
are preserved as historical rationale rather than silently rewritten; resolving
or retargeting them is follow-up work, not part of this import.

## Licensing

The donor repository offers this source under `Apache-2.0 OR MIT`. Canonical
cherry-pit takes the **MIT** arm of that existing dual grant. This is grant
selection, not relicensing; no new licence was applied and no copyright notice
was removed. Original authorship: Anders Jensen (acje).

## Dependency intake

New workspace dependencies: `thiserror`, `tracing`, `sha2`, `gethostname`,
`rustix`. Build-time code execution was inspected before any build:
`thiserror-impl` and `tracing-attributes` are proc-macros with no build script;
`sha2`, `cpufeatures`, `tracing-core` and `gethostname` have no build script.
Only `rustix` carries a `build.rs` (286 lines), which probes `rustc` for cfg
detection and reads filesystem metadata — no network access, no downloads, and
it emits only `cargo` cfg directives.

## Consequences

Canonical gains a reusable synchronous storage substrate independent of the
event-sourcing core. The consumer repository retains its copy until it adopts
canonical by exact git revision; until then the two must be reconciled
deliberately rather than drifting.
