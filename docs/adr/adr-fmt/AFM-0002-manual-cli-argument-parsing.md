# AFM-0002. Manual CLI Argument Parsing Over Clap

Date: 2026-04-27
Last-reviewed: 2026-04-27
Tier: B
Status: Accepted

## Related

References: AFM-0001

## Context

`adr-fmt` has an exceptionally narrow argument surface: one optional
positional path and three mutually exclusive flags. Clap's dependency
tree (clap_builder, clap_lex, strsim, anstream, plus optional
proc-macro deps) provides no proportional benefit for three flags.
Manual parsing with `std::env::args()` requires approximately 40
lines of match logic against this fixed surface.

## Decision

Parse CLI arguments manually using `std::env::args()`. Do not
depend on `clap` or any argument parsing crate.

- **R1**: All CLI parsing lives in a single `resolve_args()`
  function returning a Mode enum and an optional PathBuf
- **R2**: Mutually exclusive flags produce an explicit error with
  no implicit precedence rules
- **R3**: Unknown arguments starting with `--` trigger an error
  message and non-zero exit; no silent ignoring
- **R4**: Reassess if the argument surface grows beyond five flags
  or introduces subcommands

## Consequences

The binary has exactly three runtime dependencies (regex, serde,
toml) with no proc-macro deps. Compile times remain minimal. Help
output must be manually kept in sync with behavior — acceptable
given the infrequent argument surface changes. Shell completions
are unavailable but unnecessary for an internal dev tool invoked
via `cargo run`.
