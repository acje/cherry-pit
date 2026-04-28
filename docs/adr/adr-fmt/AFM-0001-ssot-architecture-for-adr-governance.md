# AFM-0001. Single Source of Truth Architecture for ADR Governance

Date: 2026-04-27
Last-reviewed: 2026-04-27
Tier: S
Status: Accepted

## Related

Root: AFM-0001

## Context

ADR governance faces a consistency problem: rules in prose drift
from rules enforced by tooling. A governance document says "use
kebab-case slugs" but nothing prevents violations. A template file
shows structure but cannot enforce cross-file link integrity,
lifecycle consistency, or naming conventions. Three approaches
exist: prose-only governance (drift inevitable), template-only
governance (cannot enforce cross-file invariants), and code-as-SSOT
where the validation tool is the specification.

## Decision

Adopt a layered SSOT architecture where the `adr-fmt` binary is
the authoritative specification for all invariant ADR rules.

- **R1**: The `adr-fmt` binary owns all invariant rules: template
  structure, naming, relationships, lifecycle states, link integrity,
  and section ordering
- **R2**: `adr-fmt.toml` owns configurable aspects: domain
  definitions, crate mappings, stale directory path, and rule
  parameter overrides
- **R3**: `--guidelines` output is the generated reference document
  combining code invariants and configuration into a single
  authoritative output
- **R4**: `GOVERNANCE.md` contains rationale and judgment guidance
  only; no enforceable rules live in prose
- **R5**: A rule is invariant if violating it produces an
  inconsistent corpus regardless of project context; otherwise it
  is configurable

## Consequences

No rule exists in prose alone — if it cannot be a validation check,
it belongs in the judgment layer. The `--guidelines` flag eliminates
a separate writing guide that would drift. Adding invariant rules
requires code changes, a rule catalog entry, and an AFM-domain ADR.
The architecture is self-referential: `adr-fmt` validates its own
domain's ADRs.
