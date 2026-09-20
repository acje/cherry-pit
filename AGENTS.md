# Cherry-pit contributor instructions

## Authority and scope

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

## Rust policy

- Use the pinned **1.98.0** toolchain, MSRV **1.98**, edition **2024**, resolver
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
  and required error/panic/safety contracts. Do not mass-rewrite byte-identical
  imported source merely to tidy pre-existing comments during extraction.
- Source `AGENTS.md` and RST-0006:R1 mandate closed public error enums. Read the
  source conflict notes in governance before interpreting older open-enum prose.
  Do not silently change imported APIs to reconcile documentary inconsistencies.
- Domain handling/apply stays synchronous; I/O belongs at typed ports/adapters.
  Serialization is a consuming-boundary requirement, not a `DomainEvent` bound.
  Preserve explicit correlation and single-writer ownership. Failure of a probe
  is unknown/error, never evidence of absence. Cancellation is not rollback.

## Verification tiers

Derived from source `AGENTS.md` at the revision above; command scope is retained,
not the source's machine-specific timings or application-only checks. Complete
dependency build-script/proc-macro intake before Cargo execution. Never delete,
ignore or feature-gate tests to get green. `cargo test` includes doctests;
retain compile-fail, property, durability, fixtures and adapter conformance tests.

**INNER:** changed crate, each increment/review round. Both commands must exit 0:

```sh
CARGO_TERM_PROGRESS_WHEN=never cargo test --quiet --no-fail-fast -p <crate> --locked --message-format=short
CARGO_TERM_PROGRESS_WHEN=never cargo clippy --quiet -p <crate> --all-targets --locked --message-format=short -- -D warnings
```

**MID:** once at sub-mission completion, changed crates plus the mechanically
computed reverse-dependent workspace closure (including test consumers). Pass
each package as an explicit `-p <crate>` to both INNER commands. All selected
tests and all-target Clippy must exit 0. Neither INNER nor MID uses
`--workspace` or `--all-features`.

**BOUNDARY:** once at producer/epic acceptance, all four commands must exit 0:

```sh
cargo build --workspace --all-features --locked
timeout 900 cargo test --quiet --no-fail-fast --workspace --all-features --locked
cargo clippy --quiet --workspace --all-targets --all-features --locked --message-format=short -- -D warnings
cargo fmt --all -- --check
```

The 900-second timeout is the source contract's bound, not a measured destination
runtime. Exit 124 is an incomplete verdict requiring hang investigation, neither
a pass nor a test failure. Re-evaluate the bound with recorded conditions when
the workload or machine changes. Claims are tier-scoped; a documentation-only
window with `git diff --check` is not a MID or BOUNDARY pass.

## Required acceptance work beyond Cargo's four commands

Standalone gates use `python3.12 -B tools/verify.py static|graph|supply-chain`.
`rust` and `non-exhaustive` use the scoped local intake admission recorded in
`ghr-7wc6p.11.2`, with a direct installed compiler and sanitized environment.
Changed inputs/context require reassessment. The explicit GitHub-hosted Linux
x86_64 context checks official Rust artifact hashes from `ghr-02ld2`; its actual
execution remains subject to final review and the producer PR run.
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

GitHub inspection on 2026-09-20 reports both repositories public, zero destination
self-hosted runners and zero repository secrets. Source checkout uses the pinned
public revision without a custom credential. Hosted audit/deny provisioning uses
official pinned musl archives and member checksums from `ghr-02ld2`, checked
before installation/execution. Version checks are not provenance evidence.
`tools/verify.py provision` is limited to the explicit hosted Linux context;
it does not compile or overwrite existing tools. Final review and actual Linux
CI remain required. No permission changes follow.

Mandatory commander-dispatched adversarial Linus review precedes producer
commit/merge. Preserve actual required reviews and checks; no fabricated
approvals or bypasses. Record unresolved source-rule/code tensions for review.

## Local data and coordination

Preserve `.git`, `.beads` audit/scaffold, unrelated untracked files and secrets.
Do not stage the Beads database or runtime audit. Pin Beads discovery explicitly
and check the returned prefix/path. The active extraction contract and origin
ledger are in **gh-report's** pinned store (`ghr-7wc6p.1`), not this repository's
ambient store. No new tracking framework is needed.
