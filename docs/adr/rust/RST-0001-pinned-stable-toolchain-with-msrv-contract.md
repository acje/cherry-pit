# RST-0001. Pinned Stable Toolchain with MSRV Contract

Date: 2026-04-27
Last-reviewed: 2026-04-28
Tier: B
Status: Accepted

## Related

- Root: RST-0001

## Context

Rust releases a new stable version every six weeks. Without an
explicit toolchain pin, each developer's local environment drifts
to whichever version `rustup update` last installed. This causes
friction in three ways:

1. **Surprise lint failures.** Each Rust release ships new clippy
   lints. A contributor opens a PR that compiled cleanly
   yesterday, but CI (or a teammate on a newer toolchain)
   reports unrelated warnings. The contributor must fix lint
   violations they did not introduce, polluting the PR with
   unrelated changes.

2. **Feature availability mismatch.** A developer on the latest
   nightly or stable may use a recently stabilized API. Another
   developer on an older toolchain gets a compile error on pull.
   Neither did anything wrong — the environment diverged silently.

3. **Irreproducible builds.** Without a pinned toolchain, the
   same `Cargo.lock` can produce different compilation results
   (different warnings, different optimizations, different
   diagnostic output) depending on which `rustc` is invoked.

The `rust-toolchain.toml` file solves this: `rustup` reads it and
automatically installs the specified toolchain when entering the
project directory. Every developer and CI runner uses the same
compiler version without manual coordination.

Separately, the `rust-version` field in `Cargo.toml` declares the
Minimum Supported Rust Version (MSRV). For application crates
(not libraries published to crates.io), MSRV communicates the
floor: "this project requires at least this Rust version." Cargo's
resolver respects MSRV when selecting dependency versions.

Cherry-pit is an application workspace, not a published library.
The MSRV policy is simple: latest stable. The workspace
`Cargo.toml` already declares `rust-version = "1.95"` and
`edition = "2024"`. What is missing is the `rust-toolchain.toml`
that pins the exact toolchain for contributor consistency.

Borsos (Swatinem, "Should I pin my Rust toolchain version?")
argues that pinning makes updates intentional: "Forcing everyone
onto the same toolchain version, and making updates intentional,
you will avoid friction both for new and old contributors."
Endler (Corrode, "Long-term Rust Project Maintenance") recommends
stable over nightly for any project with long-term maintenance
obligations.

## Decision

The Rust toolchain is pinned via `rust-toolchain.toml` at the
workspace root. The channel is stable. The MSRV is declared in
`[workspace.package].rust-version` and matches the pinned
toolchain version.

### Configuration

```toml
[toolchain]
channel = "1.95"
profile = "minimal"
components = ["clippy", "rustfmt"]
```

### Rules

1. **Stable only.** No nightly features. Nightly introduces
   instability risk that compounds over the project's lifetime.
   If a nightly feature is needed, it must be justified by an
   ADR and revisited when the feature stabilizes.

2. **Pin the major.minor version.** The `rust-toolchain.toml`
   specifies a `channel` like `"1.95"`, not `"stable"`. This
   ensures all contributors compile with the same version.
   Patch versions within the pinned minor are accepted
   automatically by `rustup`.

3. **MSRV equals the pinned version.** For application crates,
   the MSRV in `Cargo.toml` matches the `rust-toolchain.toml`
   channel. There is no reason to support older compilers when
   the project controls its deployment.

4. **Edition at workspace level.** The Rust edition is declared
   once in `[workspace.package]` and inherited by all crates.
   Edition migrations are intentional changes per RST-0002.

5. **Lockfile committed.** `Cargo.lock` is committed to version
   control for reproducible dependency resolution across
   environments.

## Consequences

- Contributors cloning the repository get the correct toolchain
  automatically. No "which Rust version do I need?" questions.
  `rustup` handles installation on first `cargo build`.
- CI runners use the same compiler as local development. Build
  results are reproducible across environments.
- Toolchain updates become deliberate, reviewable events
  (RST-0002), not ambient drift.
- The `profile = "minimal"` setting avoids installing unnecessary
  components (e.g., `rust-docs`). The `components` list explicitly
  includes `clippy` and `rustfmt` — these are required for
  RST-0003 lint and format governance.
- Tension with "always use latest Rust": pinning means the
  project does not automatically benefit from new compiler
  optimizations or diagnostics. RST-0002 (intentional evolution)
  addresses this by establishing an update cadence.
- The MSRV value appears in three files: `rust-toolchain.toml`,
  `clippy.toml`, and `Cargo.toml`. These must stay in sync
  manually during toolchain updates (RST-0002). Mechanized
  enforcement of this invariant is a candidate for future CI
  validation per COM-0017.
