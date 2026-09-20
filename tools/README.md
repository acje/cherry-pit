# Standalone verification

Authority: Mattilsynet/gh-report **c8507377b2748a015148751ce288be2bad9ec708**.
Run from the repository root with Python 3.12 and the existing toolchain:

```sh
python3.12 -B tools/verify.py static
python3.12 -B tools/verify.py graph
python3.12 -B tools/test_verify.py
python3.12 -B tools/verify.py all
```

`static` reads files and immutable source ADR blobs; set `GH_REPORT_SOURCE` to
the existing gh-report checkout. CI checks out that exact revision under
`target/source-governance`. Missing source access is an error, not citation
validation success. Local link landings are not substituted for actual rules.
`CPP-*` citations resolve only to accepted local decisions; source identifiers
still resolve exclusively to immutable source blobs.
`graph` runs locked, offline metadata/tree probes, all features and all targets:
async-trait is forbidden throughout neutral dependency trees; Pardosa is forbidden
in neutral normal/build trees, while dev/test adapter edges remain permitted.
It also checks every workspace library/binary compilation root forbids unsafe.
`bash tools/tripwires.sh <check>` retains the applicable source dispatch names.

## Scoped execution admission and remaining integration

`ghr-7wc6p.11.2` supersedes the unconditional hold: serde's exact compiler-version
probe is Accepted and the separate source-relative scoped T3/T4 review is complete.
Checker edges are source-equivalent, not newly acquired. `intake` checks the
recorded local checkout or explicit GitHub-hosted Linux x86_64 context, lock,
compiler/loader/Cargo identities and Cargo config.
Executing groups use the recorded sanitized environment and direct installed
1.98.0 compiler, with no rustup dispatcher or inherited environment waiver.
This is scoped admission, not actual Linux CI approval or a complete source-integrity
monitor. Changed dependency sources, manifests, features, toolchain or build
context require renewed intake. Linux official member hashes come from `ghr-02ld2`;
the pinned source dtolnay action installs through rustup's verified distribution.
Every Cargo invocation then uses the absolute verified executable and fresh
allowlisted environment; extra Cargo configs (home and checkout ancestors),
redirected identity paths and unsupported contexts fail closed. System loader and
OS libraries remain the hosted-image trust boundary, not an audited whole OS.
Cargo's ordinary reviewed-build loader search extensions remain applicable.
No restored build cache is used. `fetch` downloads locked sources without executing
build scripts before offline graph probes. Source checkout is public and pinned.

The admitted local Rust command roster retains full workspace/all-feature
build and tests (including doctests, compile-fail, properties, durability and
adapter conformance), all-target Clippy with denied warnings, and formatting.
Tests have the source 900-second timeout; timeout is incomplete, never clean.
Ordinary command failures aggregate; timeout or incomplete process-group cleanup
aborts subsequent dispatch, per `ghr-co8tw`.

`crates/non-exhaustive-check` is necessary supporting tooling, not a product
crate: source RST-0006:R2 designates it as the sole enforcement point. Its
manifest is source-identical; all nine source unit tests are retained. Source
algorithm is retained, including accepted macro/parse under-coverage. The scope
list replaces gh-report with both outer libraries, and contradictory opening
prose is removed; diagnostic authority is source-qualified. **Root workspace
membership, inherited syn/proc-macro2 declarations and lock reconciliation are
restored.** The authoritative source-relative review in ghr-7wc6p.11.2 confirms
no new checker edges or features; source locked versions are retained.

Local supply-chain commands require existing cargo-audit and cargo-deny executables.
Hosted `provision` downloads audit 0.22.1 and deny 0.19.4 official musl assets,
checks exact archive length/SHA256, reads only the named regular member, checks
its length/SHA256, then installs without overwriting. Execution rechecks binary
identity; no `cargo install`, archive extraction or Linux execution on macOS.
The hashes pin observed bytes even when upstream release assets are mutable.
deny.toml retains source strictness, licenses and native targets; removes only
Leptos-only advisory ignores, the absent adr-fmt git source and browser target.

## Governance and proof status

Source gate origins (Git blob hashes):

| Source path | Blob |
|---|---|
| `.github/workflows/ci-reusable.yml` | `245570f174175a0eb0d874e24de027baa861d2b3` |
| `.github/workflows/ci.yml` | `39b9538f947b02bcf754a5e0249673b0e35f421e` |
| `tools/tripwires.sh` | `3d5ddabd901ee265e6c824c4510667e0f8479d7b` |
| `deny.toml` | `12e240cd7b434df5dfb8fb3690543c160e75163c` |
| `crates/non-exhaustive-check/Cargo.toml` | `fb4c5992084a3f828942c71b61525bb6822fca18` |
| `crates/non-exhaustive-check/src/main.rs` | `e3859e70466e8fd76a8590d75addb6cc0f8daac6` |

Python gate implementation is new standalone wiring scoped from the source
tripwires, using real TOML parsing and bounded checked subprocesses. It omits
gh-report mutex/fence/browser/Docker gates. Source scheduled ADR lint ratchet
was nonblocking and is not promoted into a merge gate here.

Live plant/fail/revert/clean covers unknown source rule R999, workflow toolchain
drift, bare advisory ignore and inner dead_code suppression (ghr-7wc6p.9).
ghr-b91rr records real Cargo manifest/lock Pardosa-edge, unsafe-root and duplicate
ADR plants and clean restoration, beyond the mocked graph tests.
ghr-7wc6p.1.2 records real non-exhaustive Error plants in BOTH outer library
roots, checker rejection naming both, and restored clean hashes; it also records
the local CPP rule citation plant and restoration. Production digest shape and
exact Cargo member regressions expose the prior transcription error using
ghr-02ld2's authoritative bytes. Final independent review remains pending.

CPP-0001:R6 names build-test-lint and step
“deny async-trait and rearm pardosa-cherry-pit-projection DAG guard” in the
source-qualified CHE-0084:R9 placement amendment. RST-0007:R5 applicability and
source-rule/code tensions require review. Citation checking proves id existence,
not invariant match or full governance acceptance. SEC-0013's 180-day review
history and reason-citation obligations still need review beyond expiry parsing.
CPP-0001:R7 states the scoped admission/fetch/provisioning invariant and stable
job identifiers; step names and diagnostics cite it. Source RST-0004:R5 governs
new-dependency justification, not compiler identity admission.

Old destination contexts `Workspace correctness gates` and `Supply-chain audit`
are replaced by source-scoped job names. Required-context migration must be
reviewed against actual branch protection before activation/merge; no remote
settings were changed. GitHub inspection on 2026-09-20 found no rulesets and
"Branch not protected"; successful old main contexts do not verify this tree.
Both repositories report public visibility, zero destination runners and zero
repository secrets. No custom source credential is needed. Hosted audit/deny
artifact provisioning and Rust-context checks are implemented; final review and
the actual producer PR Linux run remain acceptance prerequisites.

Resource contract: sequential commands, existing owned POSIX process groups;
120-second probes/fetch, 900-second tests, 1800-second other Rust/supply commands,
5-second TERM grace then KILL and 5-second reap. Curl has a 120-second deadline,
130-second supervisor bound, no retries, and the exact archive byte cap. At most
one archive and one binary (largest 14816000 bytes) are retained during provisioning;
temporary downloads are removed on success/error, installed tools live for the job.
Python/tar/runtime overhead and captured command output are not process-memory
bounds. No added services or caches. Workflow job deadlines further bound jobs.
Linux context tests use mocked installed bytes/platform, real config-file plants,
and real hashing; they are not Linux execution or OS-loader attestation.

## New correctness delta: ghr-7wc6p.12

The outer adapter's checkpoint comparison and snapshot/checkpoint writes now
share one owned session critical section. Encoding remains outside that lock;
the private checkpoint reader borrows the existing session without relocking.
This repairs inherited Linus H3, distinct from exact-copy/relocation provenance.
The new barrier-controlled regression forces sequence 10 to commit while sequence
7 is encoding, then requires 7 to be rejected and both stored values to remain 10.
It failed on the old implementation and passed after repair. Adapter tests moved
from 6 unit + 7 integration + 1 doctest to 7 + 7 + 1; none removed or ignored.
Existing 128-byte name and 262144-byte admitted snapshot limits are unchanged;
encoding allocations and concurrent waiters are not claimed to be process-bounded.
The byte constant limits admitted serialization length, not memory. I1's late
projection-name validation remains a future construction-boundary gap; rejection
before create/open file operations is not claimed.
The synchronous mutex spans comparison, writes and sync with no await or nested
lock; cancellation does not roll back IO. No new queue, retry or resource framework.
