# AFM-0007. Compiler-Style Diagnostics on Stderr

Date: 2026-04-27
Last-reviewed: 2026-04-27
Tier: B

## Status

Accepted

## Related

- References: AFM-0003

## Context

Lint tools must communicate findings to developers. The diagnostic
output format directly affects whether developers read, understand,
and act on warnings. Poor formatting leads to warning fatigue;
good formatting leads to immediate comprehension and targeted fixes.

Three diagnostic styles are common in the Rust ecosystem:

1. **Log-style** — `WARNING: file.md: rule violated`. Simple but
   lacks context. Developers must open the file and search for the
   issue. Common in shell scripts and simple linters.

2. **JSON-structured** — machine-readable output for tool
   integration. Excellent for IDE plugins and CI dashboards but
   unreadable by humans in terminal output.

3. **Compiler-style** — `file.md:42: warning[T015]: prose section
   below minimum word count`. Includes file path, line number,
   severity, rule ID, and human-readable message. Matches the
   format developers see from `rustc`, `clippy`, `gcc`, and
   `typescript`. Terminal-native: clickable in many editors and
   terminals.

The Rust compiler's diagnostic format is deeply familiar to
`adr-fmt`'s target audience (Rust developers managing a Rust
workspace). Adopting the same visual conventions reduces cognitive
switching cost between compiler output and linter output.

## Decision

Format all diagnostics in compiler-style on stderr. Each
diagnostic includes the file path, rule ID, and human-readable
description.

### Diagnostic Format

```
path/to/ADR-NNNN-slug.md: warning[RULENUM]: description
```

Components:

- **File path** — relative to the ADR root directory. Clickable
  in terminals that support path detection.
- **Severity** — always `warning` (per AFM-0003, all rules are
  advisory).
- **Rule ID** — the catalog identifier (e.g., `T015`, `L001`,
  `N003`). Enables filtering, counting, and cross-referencing
  with the rule catalog.
- **Description** — concise, actionable message. States what is
  wrong, not how to fix it.

### Output Channels

- **Stderr** — all diagnostics and progress information. This
  ensures that `--report` and `--guidelines` stdout output is
  clean and pipeable.
- **Stdout** — only `--report` (children index) or `--guidelines`
  (reference document) output. Never diagnostics.

### Sorting

Diagnostics are sorted by file path, then by rule ID within each
file. This produces stable, deterministic output regardless of
directory traversal order. Developers reviewing output see all
warnings for a single file grouped together.

## Consequences

- Developers accustomed to `cargo clippy` output immediately
  understand `adr-fmt` output without learning a new format.
- Rule IDs in diagnostic output allow targeted suppression
  discussions: "should we address all T015 warnings?" rather than
  "should we fix the word count things?"
- The stderr/stdout split enables clean piping: `cargo run -p
  adr-fmt -- --guidelines > GUIDELINES.md` produces a clean file
  with no diagnostic noise.
- Future addition of `--format json` for structured output is
  compatible with this architecture — it would be an alternative
  output mode, not a replacement.
- The `Diagnostic` struct and its `Display` implementation are
  intentionally simple (~30 lines). No dependency on `codespan`,
  `ariadne`, or other diagnostic rendering crates.
