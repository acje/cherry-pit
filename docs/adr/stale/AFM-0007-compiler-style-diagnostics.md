# AFM-0007. Compiler-Style Diagnostics on Stderr

Date: 2026-04-27
Last-reviewed: 2026-04-28
Tier: B
Status: Superseded by AFM-0014

## Related

Superseded-by: AFM-0014
References: AFM-0003

## Context

Diagnostic output format directly affects whether developers read
and act on warnings. Three styles are common: log-style (simple,
lacks context), JSON-structured (machine-readable, unreadable by
humans), and compiler-style (file path, rule ID, description —
matching `rustc` and `clippy` conventions). The compiler-style
format is deeply familiar to the target audience (Rust developers)
and reduces cognitive switching cost between compiler and linter
output.

## Decision

Format all diagnostics in compiler-style on stderr with file path,
rule ID, and actionable description.

R1 [5]: Each diagnostic follows the format
  `path: warning[RULE]: description` on stderr, matching Rust
  compiler conventions
R2 [5]: Diagnostics are sorted by file path then by rule ID for
  stable deterministic output regardless of traversal order
R3 [5]: Stdout is reserved exclusively for `--guidelines` and
  `--tree` output; diagnostics never appear on stdout

## Consequences

Developers accustomed to `cargo clippy` immediately understand
`adr-fmt` output. Rule IDs enable targeted discussions ("address
all T015 warnings"). The stderr/stdout split enables clean piping
(`--guidelines > file.md`). Future `--format json` is compatible
as an alternative mode. The `Diagnostic` struct is intentionally
simple (~30 lines, no diagnostic rendering dependencies).

## Retirement

Superseded-by: AFM-0014
Moved-to-stale: 2026-04-28
Reason: All six output modes now produce unified markdown on stdout.
The compiler-style stderr format was abandoned as adr-fmt expanded
beyond lint-only usage. The print_diagnostic function in report.rs
is dead code. Markdown formatting provides richer presentation
while maintaining greppable rule IDs.
