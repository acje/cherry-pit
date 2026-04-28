# AFM-0003. Advisory-Only Validation With Exit-Code Semantics

Date: 2026-04-27
Last-reviewed: 2026-04-28
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
overlooked warnings without process discipline).

## Decision

`adr-fmt` exits 0 for all lint findings and exits 1 only for
infrastructure errors. All validation rules emit warnings, never
errors.

R1 [5]: Exit 0 means lint completed successfully; exit 1 means
  the tool could not function (missing config, unreadable files,
  invalid configuration)
R2 [5]: All diagnostics use warning severity; no error severity
  exists for rule violations
R3 [5]: Zero-warning enforcement is a process concern delegated
  to CI wrapper scripts that parse stderr for warning counts

## Consequences

Authors can write Draft ADRs with incomplete sections without being
blocked. CI integration requires a wrapper if zero-warning
enforcement is desired. The "exit 0 does not mean clean" semantics
must be documented. Future `--error-on-warning` flag is compatible
as a mode change. The model aligns with Rust conventions: `cargo
fmt` and `cargo clippy` default to non-blocking output.
