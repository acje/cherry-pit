# AFM-0003. Advisory-Only Validation With Exit-Code Semantics

Date: 2026-04-27
Last-reviewed: 2026-04-27
Tier: A

## Status

Accepted

## Related

- References: AFM-0001

## Context

Lint tools face a fundamental design tension between strictness and
adoption. A tool that fails the build on any warning creates
pressure to suppress warnings rather than fix them. A tool that
produces no signal on violations is ignored entirely. The useful
middle ground is advisory output that is visible, actionable, and
non-blocking.

Rust's own `clippy` navigates this tension with lint levels:
`warn` by default, `deny` opt-in. However, `clippy` operates
within the compiler's diagnostic infrastructure and benefits from
IDE integration. `adr-fmt` operates on markdown files in a
documentation directory — a fundamentally different feedback loop.

The ADR corpus is authored by humans writing prose. Draft ADRs are
intentionally incomplete. Proposed ADRs may have placeholder
relationships. Forcing a zero-warning state before a PR can be
opened would discourage ADR creation — the opposite of the desired
outcome.

Two exit-code strategies exist for advisory tools:

1. **Exit non-zero on warnings** — treats all diagnostics as
   failures. CI can gate on this. Risk: authors suppress or ignore
   warnings to unblock merges.

2. **Exit zero on warnings, non-zero only on infrastructure
   errors** — diagnostics are informational. The tool always
   succeeds unless it cannot function (missing config, unreadable
   files). Risk: warnings may be overlooked without process
   discipline.

## Decision

`adr-fmt` exits 0 for all lint findings and exits 1 only for
infrastructure errors. All validation rules emit warnings, never
errors.

### Exit-Code Contract

- **Exit 0** — lint complete. Zero or more warnings may be present
  on stderr. Callers must parse stderr to determine if issues
  exist. This code means "the tool functioned correctly" not "the
  ADRs are clean."

- **Exit 1** — infrastructure failure. The tool could not complete
  its work: missing `adr-fmt.toml`, no domain directories found,
  unreadable files, invalid configuration. This code means "the
  tool itself is broken or misconfigured."

### Severity Model

All diagnostics use a single severity level: `Warning`. There is
no `Error` severity for rule violations. The `Diagnostic` struct
carries a severity field for future extensibility, but the current
rule catalog never produces errors.

### Process Enforcement

Warning visibility is a process concern, not a tool concern.
Teams that want zero-warning enforcement can wrap `adr-fmt` in a
CI script that parses stderr for warning counts and fails the
pipeline accordingly. The tool does not embed this policy.

## Consequences

- Authors can write Draft ADRs with incomplete sections and still
  get useful feedback without being blocked from committing.
- CI integration requires a wrapper script if zero-warning
  enforcement is desired — the tool does not provide this out of
  the box.
- The "exit 0 does not mean clean" semantics must be documented
  clearly. Callers who check only exit codes will miss warnings.
- Future addition of an `--error-on-warning` flag is compatible
  with this architecture — it would be a mode change, not a
  semantic change.
- The advisory model aligns with Rust ecosystem conventions:
  `cargo fmt` exits 0 even when it reformats files, `cargo clippy`
  defaults to warnings not errors.
