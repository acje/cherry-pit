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

R6 [10]: The lock file is a stable coordination inode that MUST NOT be
  unlinked by this crate. Every ownership transaction — acquire, reclaim,
  renew, release and drop — MUST take an OS-backed advisory lock
  (`std::fs::File::try_lock`) on that inode for the duration of that
  transaction only, never for the lifetime of the guard, and MUST compare
  and mutate ownership under that same guard. Each successful acquisition
  MUST record a fresh unique generation, derived from neither run id, pid
  nor timestamp, so that ABA confusion across a takeover is impossible; the
  generation is private to the ownership envelope and is not a required
  field of the public `LockMetadata` DTO. A displaced holder's renew MUST
  report lost ownership, and its release and drop MUST be no-ops that leave
  the successor's record intact. Stale reclaim (TTL expiry, or a dead holder
  on the *same* host) and forced takeover apply to a *live* holder as well:
  availability of hung-holder recovery is retained. A lock file without a
  recorded hostname remains TTL-only. Verified-invalid content is replaced
  under the held guard. Every guard is non-blocking — contention yields
  `LockFailed` immediately, with no wait, spawn or retry loop, and drop never
  waits. The guarantee is scoped to this crate's lock metadata transactions,
  for cooperating clients in a trusted lock directory on a local filesystem
  supporting advisory locking; it fences neither arbitrary external
  unlinking nor application writes performed by a displaced holder.
  Resolves ghr-wsf5u H3 and ghr-1r175 H1.

R7 [5]: `LockMetadata` is a serde DTO. Schema evolution is handled by
  field-presence plus `#[serde(default)]`, NOT by `#[non_exhaustive]`.

R8 [5]: `PersistenceError` and `RetryClass` are closed (non-`#[non_exhaustive]`)
  public enums. This is an explicit **storage-only exception** to canonical
  `COM-0021:R1`, authorized here and scoped to this crate: the taxonomy is
  small, producer-owned and exhaustively matched by callers for retry
  decisions. It ratifies nothing for `cherry-pit-core` or any other crate, and
  it does not adopt `CHE-0049`, which is Proposed, is not current authority,
  and targets core.

R9 [10]: A lock read MUST distinguish verified outcomes (unowned,
  owned, verified-invalid content) from unknown ones. An unreadable or
  otherwise failing read is NOT corruption: it MUST propagate as an error
  with no mutation of the lock file. Resolves ghr-wsf5u H4.

R10 [10]: The coordination inode MUST be opened without following symlinks
  and without blocking, and MUST be rejected unless it is a regular file with
  exactly one link; rejection happens before any read or mutation, so an
  unrelated linked target is never truncated and a FIFO never stalls a
  diagnostic read. Lock content MUST then be read through that same opened
  handle, bounded to 1 MiB plus one oversize-detection byte per call, into a
  buffer allocated once at exactly that capacity so the raw content buffer
  can neither reallocate nor retain beyond the budget; oversize input is
  rejected without mutation. Transient JSON parser overhead is explicitly
  excluded, and no process-wide memory or no-allocation claim is made.
  Resolves ghr-wsf5u M2 and ghr-1r175 H2/M1.

R11 [5]: Atomic write MUST normalize a bare relative filename — whose
  `Path::parent()` is `Some("")` — to the current directory, so the
  durability fsync step succeeds. Resolves ghr-wsf5u M3.

## Provenance

| Donor | `Mattilsynet/gh-report` @ `c850737` |
|---|---|
| Donor path | `crates/cherry-pit-storage/` |
| Canonical path | `crates/cherry-pit-storage/` |
| Files imported | 9 (Cargo.toml, README.md, 5 × src (lib + four modules), 2 × tests) |
| Donor line counts | error 186, fs 104, lib 68, lock 1446, signature 278, properties 226, smoke 5 |

At the original import revision (`c1055ef`), source `src/error.rs`,
`src/fs.rs`, `src/lock.rs`, `src/signature.rs` and `tests/properties.rs` were
byte-identical to the donor except where a donor-local ADR citation was
rewritten to this ADR. Subsequent canonical repairs have since diverged
`src/fs.rs` (bare-relative parent normalization, R11) and `src/lock.rs`
(coordination-inode ownership, bounded typed reads and the file-kind/link
boundary, R6/R9/R10), together with their tests. `src/lib.rs`, `README.md`,
`tests/smoke.rs` and `Cargo.toml` differ only in ADR citation, repository URL,
and homepage metadata.

Donor doc-comments retain references to ADRs that must be read as
**donor-corpus** records, qualified by source: `CHE-0088` and `PGN-0016` exist
only in `Mattilsynet/gh-report` @ `c850737` under that repository's
`docs/adr/`, and are historical provenance, not canonical authority. Where such
a citation appears in an API contract or a test assertion it is normative in
the donor only; the adopted canonical contract for this crate is R1–R8 above.
In particular, donor `PGN-0016:R10` forbids in-append resync, and donor
`CHE-0088` carries application-specific convergence policy — neither is a
generic contract for this crate. Any donor "A8 replaces this" assertion is
donor history and is not a canonical statement; canonical R5 retains only the
test. `CHE-0021`, `SEC-0006` and `COM-0025` DO exist in canonical and their
cited rules were read against the canonical corpus; those citations stand as
canonical. Resolving or retargeting the donor-only references is follow-up
work, not part of this import.

## Licensing

The donor repository offers this source under `Apache-2.0 OR MIT`. Canonical
cherry-pit takes the **MIT** arm of that existing dual grant. This is grant
selection, not relicensing; no new licence was applied. The donor DOES carry a
`LICENSE-MIT` at `c850737` reading "Copyright (c) 2026 acje", and its terms
require that copyright and permission notice to be retained. It is therefore
preserved verbatim at `crates/cherry-pit-storage/LICENSE-MIT` alongside the
repository-level `LICENSE` ("cherry-pit contributors"); the crate README links
it. Prose attribution alone is NOT notice preservation. An earlier import
record asserting the donor had no licence file was false and is corrected here.
Original authorship: Anders Jensen (acje).

## Dependency intake

This section supersedes the earlier intake claim, which was false: it named
`rustix` as the only build-executing surface at version 1.1.4, while the
as-imported `Cargo.lock` had been wholly regenerated and resolved `rustix`
1.1.5 plus ~57 other upgraded pre-existing packages. That regeneration is
reverted; the lock is now the `3fcfc81` baseline with only the additive
storage closure resolved on top. No pre-existing resolution changed.

Exact lock delta versus baseline `3fcfc81`: 0 packages removed, 0 pre-existing
packages upgraded, 16 nodes added — `cherry-pit-storage` 0.1.0 (workspace),
`block-buffer` 0.12.1, `const-oid` 0.10.2, `cpufeatures` 0.3.1,
`crypto-common` 0.2.2, `digest` 0.11.3, `gethostname` 1.1.0,
`hybrid-array` 0.4.15, `sha2` 0.11.0, `syn` 3.0.6 (additive, alongside the
retained 2.0.117), `thiserror` 2.0.20, `thiserror-impl` 2.0.20,
`tracing` 0.1.44, `tracing-attributes` 0.1.31, `tracing-core` 0.1.36,
`typenum` 1.20.1. `rustix` is NOT a new node: it is pre-existing at the
baseline 1.1.4 and unchanged by this import.

Build-time execution surfaces among those added nodes, read at the actual
resolved versions:

- `thiserror` 2.0.20 — the only `build.rs` (195 lines). It writes a generated
  `private.rs` into `OUT_DIR`, then invokes `$RUSTC` on the vendored
  `build/probe.rs` to detect `error_generic_member_access`, and reads
  `rustc --version` for a minor-version gate. It emits only `cargo:` cfg,
  check-cfg and rerun-if directives and deletes its probe subdirectory. No
  network access, no downloads, no writes outside `OUT_DIR`.
- `thiserror-impl` 2.0.20 and `tracing-attributes` 0.1.31 — proc-macros, no
  build script. Neither references `std::fs`, `std::net`, `std::process`,
  `std::env`, `include_str!` or `include_bytes!` in its sources; the only
  `std::io` occurrences in `tracing-attributes` are inside doc-comment
  examples.
- The remaining 12 added nodes carry neither a build script nor a proc-macro
  target.

No maliciousness is inferred anywhere in this intake. It is recorded honestly
that the imported code was built and tested at `c1055ef` BEFORE this intake was
completed; that execution preceded clearance and is not retroactively
sanctioned by this section.

## Consequences

Canonical gains a reusable synchronous storage substrate independent of the
event-sourcing core. The consumer repository retains its copy until it adopts
canonical by exact git revision; until then the two must be reconciled
deliberately rather than drifting.
