# Cherry-pit contributor instructions

Repo-specific operational notes. General agent/OODA doctrine, bash hygiene,
and the Rust no-`//`-comments house style live in the global
`~/.config/opencode/AGENTS.md` (auto-loaded) — not repeated here.

## Section 1: Canonical Fleet Doctrine

### OODA Loop Roles
- **Copernicus** (Observe): Raw evidence gathering from environment, code, and external specs. Pure sensor; produces no hypotheses.
- **Feynman** (Orient): Produces ranked hypotheses with falsifiers; stress-tests against concrete examples.
- **Moltke** (Decide): Standing mission commander. Emits executable mission contracts, sets intent, boundaries, and abort criteria.
- **Hopper** (Act): Executes missions using Kent Beck TDD (red-green-refactor) with verify-before-claim discipline.
- **Linus** (Review): Mandatory pre-merge Rust reviewer for idiom conformance, type safety, unsafe soundness, and supply chain.
- **Hamilton** (Assurance): Architectural alignment and assurance reviewer running during CI wait windows.
- **Gardener** (GC): Post-mission cleanup specialist; reclaims transient scaffolding and closes completed mission beads.

### Priority Hierarchy
Tradeoffs strictly resolve in this five-tier priority order:
1. **Maintainability**: Pure trunk development, small deployable increments, minimal cognitive overhead, low complexity.
2. **Correctness by design**: Make illegal states unrepresentable via types, explicit state machines, and private invariant constructors.
3. **Response times**: Latency-sensitive read paths and prompt fact propagation across boundaries.
4. **Energy efficiency in code**: Minimize redundant polling, hot loops, unnecessary serialization, and idle CPU/memory burn.
5. **Features**: New functionality ranks last and must never compromise the higher tiers.

### Non-Interactive Shell Commands & Bash Hygiene
Subagents execute non-interactively. Commands that prompt for user confirmation stall execution indefinitely.
- Always use non-interactive and force flags: `cp -f`, `rm -f`, `rm -rf`.
- Streaming and batch mode: use `--batch`, `-y`, or `--quiet` where available.
- Stream separation: machine-readable findings route to `stdout`; diagnostics and logs route to `stderr`.

### Zero Plain Comments
In Rust source (`*.rs`), plain comments (`//` or `/* */`) are forbidden.
- Rationale belongs in commit messages, ADRs, or bead descriptions.
- Use `///` or `//!` contract doc-comments only when defining public API documentation (with required `# Errors`, `# Panics`, `# Safety` sections).
- Suppress lints with `#[expect(lint, reason = "...")]` rather than plain comments.

### Doctrine: "Make tools fast to iterate fast"
Developer and verification tooling must be compiled, ultra-fast Rust binaries operating directly on ASTs and files rather than slow interpreted wrappers or token-heavy in-context simulation. Fast tools enable high-frequency local feedback loops (INNER cadence) without friction.

### Doctrine: "Zero compliance theatre"
High-assurance testing techniques—such as property-based testing (proptest), fuzzing (cargo-fuzz), formal model checking, or fault injection—must be applied purposefully at critical serialization, concurrency, and storage boundaries (high-risk seams), not sprayed ubiquitously as box-ticking ceremony. Where type invariants and deterministic unit tests suffice, do not add compliance overhead.

## Section 2: Target-Specific Profile

### Target Classification & Entrypoint
- Target class: `service-unattended` (as mapped in `sf-sdlc.toml`).
- Canonical verification entrypoint: `scripts/verify.sh`

### Authority and Scope
Cross-repository operational authority is
[gh-report trunk delivery](../gh-report/docs/trunk-delivery.md) in the canonical
sibling checkout (`Mattilsynet/gh-report`, `docs/trunk-delivery.md`). Follow that
single policy for adoption and release; local source governance remains here.

This producer derives from `Mattilsynet/gh-report` revision
`c8507377b2748a015148751ce288be2bad9ec708`. Read
[docs/governance.md](docs/governance.md) before changing a governed boundary.
Source ADR identifiers are repository-qualified; their dates and amendment
history are not destination acceptance dates. Local decisions use `CPP`, never
reuse a source `CHE` number. Old canonical Cherry material is not authority.

Eight neutral `cherry-pit-*` crates must have no Pardosa in their normal/build
dependency closure. `pardosa-cherry-pit-projection` is the outer persistent
adapter; `pardosa-cherry-pit-test-support` is unpublished integration support.
Dev/test dependencies may exercise the outer adapters. Do not reverse-reexport
the persistent adapter from neutral projection or vendor the Pardosa substrate.
GitHub, organization, credential, report and tenant policy stays in consumers.

### Resource Contracts & Bounds (FLEET-RES-01)
Changes to ingestion, buffering, concurrency, retries, recursion, or hot paths
must define and satisfy explicit resource bounds:
- **Items and bytes accounted separately**: A bounded channel alone does not bound
  memory; admit work before unbounded allocation or payload retention.
- **Permit lifetimes & RAII**: Permits and resource charges must stay alive for the
  actual resource lifetime, including error paths and cancellations. Release exactly
  once via RAII.
- **Admission control**: Work admission must be bounded. If admission waits, bound
  the number of waiters and what they retain.
- **Progress, cancellation & shutdown**:
  - Individual work units, retries, and recursion depth must be bounded.
  - Service lifetime loops require reachable, supervised shutdown and bounded work
    between shutdown checks.
  - Dropping task handles does not cancel asynchronous tasks; use explicit cancellation
    tokens and await task completion during shutdown.
- **Atomic file write protocol (CHE-0032)**:
  All durable state written to disk must use the atomic sequence:
  `write temporary file` $\rightarrow$ `fsync file` $\rightarrow$ `atomic rename` $\rightarrow$ `fsync parent directory`.

### Verification Cadences (Three-Tier Cadence)
Canonical approval of a verified result repeats only per repository stable
candidate; targeted falsifiers remain allowed against any candidate.
Derived from source `AGENTS.md` at the revision above; command scope is retained,
not the source's machine-specific timings or application-only checks. Complete
dependency build-script/proc-macro intake before Cargo execution. Never delete,
ignore or feature-gate tests to get green. `cargo test` includes doctests;
retain compile-fail, property, durability, fixtures and adapter conformance tests.

- **INNER** (every hopper TDD increment and targeted-reviewer falsifiers;
  changed crate ONLY; exit-code criterion: test + clippy exit 0):
  ```sh
  CARGO_TERM_PROGRESS_WHEN=never cargo test --quiet --no-fail-fast -p <crate> --locked --message-format=short
  CARGO_TERM_PROGRESS_WHEN=never cargo clippy --quiet -p <crate> --all-targets --locked --message-format=short -- -D warnings
  ```
  `--all-targets` is mandatory on clippy to catch test/bench/example lints.
  `--workspace` and `--all-features` are forbidden at this tier.

- **MID** (once at sub-mission completion before done-claim; changed crates
  plus their reverse-dependent closure; exit-code criterion: test + clippy exit 0):
  Compute reverse dependents mechanically. Pass each package as an explicit
  `-p <crate>` to both INNER commands. All selected tests and all-target Clippy
  must exit 0. Neither INNER nor MID uses `--workspace` or `--all-features`.

- **BOUNDARY** (once per repository stable candidate; full workspace; exit 0 across all):
  ```sh
  sh scripts/verify.sh
  ```
  `scripts/verify.sh` is the single local host BOUNDARY execution owner; its
  command spelling and order are authoritative there and are not restated here.
  Per the native `tools/verify.py all` roster it runs the static checks
  (toolchain, dead-code, deny-lifecycle, ADR collision, citations), the graph
  check, the supply-chain gates (`cargo audit`, `cargo deny check`), then the
  Rust stages — `cargo build` (workspace, all-features, locked), `cargo test`
  with `timeout 900` and `--no-fail-fast` mandatory, Clippy (workspace,
  all-targets, all-features, locked, `-D warnings`), and `cargo fmt --all -- --check`
  — followed by the non-exhaustive check and the comment-free doc budget.
  Rust stages collect failures; any required stage failure makes the native
  run fail.
  - `timeout 900` on the test line is mandatory. Exit 124 is `Outcome::Surprise`,
    NEVER a test failure. Investigate the stall; do not fold it into a failure count.
  - `--no-fail-fast` on the test line is mandatory to ensure full blast-radius
    visibility in a single pass.

### Rust Policy & Toolchain
- Use the pinned **1.99.0** toolchain, MSRV **1.99**, edition **2024**, resolver
  **3**. Keep toolchain, Cargo MSRV and Clippy MSRV aligned; retain `Cargo.lock`.
- Members inherit root dependencies and `[lints] workspace = true`. The current
  source manifest's `pedantic = warn` and explicit warning roster are the bar,
  with `-D warnings` during verification. The source five-group Clippy proposal
  is deferred, not active policy. Do not import it or its migration allowances.
- Use stable-default rustfmt. Per-site lint exceptions need
  `#[expect(lint, reason = "…")]`; use `#[allow(lint, reason = "…")]` only when
  an expectation would be unfulfilled. No blanket module allows. No inner
  `allow(dead_code)` or `expect(dead_code)`, including `cfg_attr` forms.
- Every crate root forbids unsafe code. No new unsafe exception without its
  own reviewed decision. Dependencies' unsafe is a separate intake concern.
- New Rust prose comments are contract documentation only: public API rustdoc
  and required error/panic/safety contracts.
- Source `AGENTS.md` and RST-0006:R1 mandate closed public error enums. Read the
  source conflict notes in governance before interpreting older open-enum prose.
  Do not silently change imported APIs to reconcile documentary inconsistencies.
- Domain handling/apply stays synchronous; I/O belongs at typed ports/adapters.
  Serialization is a consuming-boundary requirement, not a `DomainEvent` bound.
  Preserve explicit correlation and single-writer ownership. Failure of a probe
  is unknown/error, never evidence of absence. Cancellation is not rollback.

### Supply Chain Gates
`cargo deny check` and `cargo audit` are supply-chain gates; run before publishing or bumping dependencies.

### Rustdoc Budget Gate
Run the same native check from the repository root locally and in CI:
```sh
comment-free --check-doc-budget --doc-advisory-words 80 --doc-max-words 120 --max-warning-files 0 .
```

Requires comment-free 0.2.0 at the canonical revision below:
```sh
cargo +1.98.0 install --git https://github.com/acje/comment-free --rev b4666626bbeee4e74ca41fd6ff1048b2f167dd27 --locked comment-free
```

The read-only native gate recursively scans Rust sources under `.` with the
tool's build/hidden pruning: 80 prose words is advisory; 120 is enforced.
Fenced code is excluded by the tool. Summary-only output retains full totals
while suppressing finding details; diagnostics remain visible.
Native gate exits are 0 for pass, 1 for enforced breach, and 2 for
unknown/error, including undecided payloads or empty scope. Policy and its
implementation/tests/proofs belong upstream; repository checks establish
integration only. No rewrite mode runs.
Macro-generated docs without spelled `doc` tokens remain outside detection;
this is not proof of semantic documentation coverage or process-memory bounds.

### TigerStyle Construction-Path Inventory
Invariant-bearing domain types must enforce "illegal states unrepresentable"
by design. For each changed constrained type, review all construction routes:
1. Public fields / struct literals (reject if fields allow inconsistent mutation).
2. Constructors & builders (`new()`, `builder()`).
3. `Default::default()` (must yield a valid domain state or be omitted).
4. Conversions (`From`, `TryFrom`).
5. Serde deserialization (custom validation if raw wire data could bypass invariants).
6. Mutation routes (setters, `DerefMut`).

Independent booleans remain valid booleans; genuine optionality remains `Option`.
Do not invent artificial domain restrictions where none exist.

### Closed Error Enum Policy (C4.5/C4.6)
Public error enums MUST NOT carry `#[non_exhaustive]`. Variant sets are complete
within a major semver line, making unhandled error states unrepresentable at
compile time. Enforced mechanically via `non-exhaustive-check` from the canonical
`tripwires` repository.

### Required Acceptance Work Beyond Cargo's Four Commands
Standalone gates use `python3.12 -B tools/verify.py static|graph|supply-chain`.
`rust` and `non-exhaustive` use the scoped local intake admission recorded in
`ghr-7wc6p.11.2`, with a direct installed compiler and sanitized environment.
Changed inputs/context require reassessment. The explicit GitHub-hosted Linux
x86_64 context checks official Rust artifact hashes from `code-7cu` (1.99
official-member evidence); its actual execution remains subject to final review
and the producer PR run.
A green BOUNDARY cannot stand in for the standalone gates. Before
producer acceptance, establish applicable source-derived supply-chain checks
(`cargo audit`, `cargo deny check`), toolchain consistency, core dependency
purity, neutral normal/build closure, closed-error and dead-code rules, and
gate-citation checks. Do not import gh-report application-specific gates blindly.

CPP-0001:R6 names the commander-decided destination CHE-0084:R9 mechanism amendment.
New/amended guards need plant → fail → revert → clean evidence.
RST-0007 requires citations in step names and failure diagnostics, stable job
ids, and explicit branch-protection migration for rendered-name changes.

The unsafe-root guard deliberately accepts only whitespace and line comments
before a literal first `#![forbid(unsafe_code)]` attribute. Block-comment,
conditional and string-literal spellings do not establish this invariant.

Mandatory commander-dispatched adversarial Linus review precedes producer
commit/merge. Preserve actual required reviews and checks; no fabricated
approvals or bypasses. Record unresolved source-rule/code tensions for review.

### Local Data and Coordination
Preserve `.git`, `.beads` audit/scaffold, unrelated untracked files and secrets.
Do not stage the Beads database or runtime audit. Pin Beads discovery explicitly
and check the returned prefix/path via `bd -C <repo-root>`.
Database resides at `.beads/embeddeddolt`.
The active extraction contract and origin ledger are in **gh-report's** pinned
store (`ghr-7wc6p.1`), not this repository's ambient store.
