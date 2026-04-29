# AFM-0003. Advisory-Only Validation With Exit-Code Semantics

Date: 2026-04-27
Last-reviewed: 2026-04-29
Tier: B
Status: Accepted

## Related

References: AFM-0001

## Context

Lint tools face a tension between strictness and adoption. Failing
builds on any warning pressures authors to suppress rather than fix.
ADR files are authored as prose — drafts are intentionally incomplete
and proposed ADRs may have placeholder relationships. Forcing zero
warnings before merge would discourage ADR creation. Two exit-code
strategies exist: non-zero on warnings (risks suppression) or zero
on warnings with non-zero only for infrastructure errors (risks
overlooked warnings without process discipline). The implementation
briefly drifted toward `error[T016]` and exit 1 on findings; the
drift was corrected on re-review.

## Decision

`adr-forge` exits 0 for all lint findings and exits 1 only for
infrastructure errors. All validation rules emit warnings, never
errors.

R1 [5]: Exit 0 when lint completes; exit 1 only for infrastructure
  failures (missing config, unreadable files, invalid configuration)
  signalled via stderr in main, never through the diagnostic channel
R2 [5]: Emit every rule finding as Severity::Warning via
  Diagnostic::warning in adr-forge/src/report.rs; the Severity enum
  exposes only the Warning variant for rule-driven diagnostics
R3 [5]: Delegate zero-warning enforcement to CI wrapper scripts that
  parse the `## Diagnostics: N warning(s)` header on stdout and fail
  the job when N exceeds the project threshold

## Consequences

Authors can write Draft ADRs with incomplete sections without being
blocked. CI integration requires a wrapper if zero-warning
enforcement is desired. The "exit 0 does not mean clean" semantics
must be documented. Future `--error-on-warning` flag is compatible
as a mode change. The model aligns with Rust conventions: `cargo
fmt` and `cargo clippy` default to non-blocking output.
