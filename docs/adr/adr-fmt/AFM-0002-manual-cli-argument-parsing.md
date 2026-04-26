# AFM-0002. Manual CLI Argument Parsing Over Clap

Date: 2026-04-27
Last-reviewed: 2026-04-27
Tier: B

## Status

Accepted

## Related

- References: AFM-0001

## Context

Rust CLI tools conventionally reach for `clap` as the argument
parsing framework. Clap provides derive macros for declarative
argument definitions, automatic help generation, shell completions,
subcommand routing, and extensive validation. It is the de facto
standard for Rust command-line applications.

However, `adr-fmt` has an exceptionally narrow argument surface:

- Zero or one positional argument (ADR root directory path)
- Three mutually exclusive flags (`--report`, `--guidelines`, `-h`)
- No subcommands, no value arguments, no environment variable
  binding, no shell completion requirements

Clap's dependency tree is substantial. As of clap v4, the crate
pulls in `clap_builder`, `clap_lex`, `strsim`, `anstream`,
`anstyle`, and optionally `clap_derive` with its proc-macro
dependencies (`syn`, `quote`, `proc-macro2`). For a tool with
three flags, this dependency weight provides no proportional
benefit.

The workspace already uses Rust edition 2024 with `std::env::args()`
readily available. Manual parsing of three flags and one optional
positional argument requires approximately 40 lines of
straightforward match logic.

## Decision

Parse CLI arguments manually using `std::env::args()` and explicit
match arms. Do not depend on `clap` or any argument parsing crate.

### Implementation Rules

1. **Argument resolution in a single function.** All CLI parsing
   lives in `resolve_args()` which returns a `Mode` enum
   (`Lint`, `Report`, `Guidelines`) and an optional `PathBuf`.

2. **Mutual exclusivity enforced explicitly.** If both `--report`
   and `--guidelines` are provided, print an error and exit. No
   implicit precedence rules.

3. **Help output is hand-written.** The `-h` and `--help` flags
   print a concise usage message to stdout and exit with code 0.
   The help text is maintained inline — no generated formatting.

4. **Unknown arguments are errors.** Any argument starting with
   `--` that is not recognized triggers an error message and
   non-zero exit. No silent ignoring of unrecognized flags.

### Reassessment Trigger

Revisit this decision if the argument surface grows beyond five
flags or introduces subcommands. At that point, clap's derive
macro approach would provide net value over manual parsing.

## Consequences

- The `adr-fmt` binary has exactly three runtime dependencies:
  `regex`, `serde`, and `toml`. No proc-macro dependencies appear
  in the dependency tree at all.
- Compile times remain minimal — the entire crate compiles in
  seconds, not the tens-of-seconds that clap's derive macros can
  add to cold builds.
- Help output must be manually kept in sync with actual behavior.
  This is acceptable because the argument surface changes
  infrequently and the help text is fewer than 20 lines.
- Shell completions are not available. This is acceptable for an
  internal development tool invoked via `cargo run`.
- The decision exemplifies COM-0002 (deep modules over shallow
  abstractions): a 40-line function is simpler than a framework
  dependency that hides 40 lines of work behind a derive macro.
