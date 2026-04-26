# AFM-0001. Single Source of Truth Architecture for ADR Governance

Date: 2026-04-27
Last-reviewed: 2026-04-27
Tier: S

## Status

Accepted

## Related

- Root: AFM-0001

## Context

ADR governance in a growing workspace faces a classic consistency
problem: rules documented in prose drift from rules enforced by
tooling. A governance markdown file says "use kebab-case slugs" but
nothing prevents violations from merging. A template file shows the
expected structure but cannot enforce section ordering or minimum
prose length.

The MADR (Markdown Any Decision Records) format provides a
lightweight starting point, but production governance requires
machine-enforceable invariants that go beyond what a static template
can express: cross-file link integrity, lifecycle state consistency,
domain-scoped naming, and generated index coherence.

The fundamental architectural question is where the source of truth
for ADR rules lives. Three options exist:

1. **Prose-only governance** — a GOVERNANCE.md document describes all
   rules in natural language. Humans enforce them during review.
   Drift is inevitable as the rule set grows.

2. **Template-only governance** — a MADR template file defines the
   expected structure. Copy-paste-and-fill workflows approximate
   compliance but cannot enforce cross-file invariants.

3. **Code-as-SSOT** — invariant rules are encoded in a validation
   tool. The tool is the specification. Prose documents rationale
   and judgment only; the tool documents structure and vocabulary.

## Decision

Adopt a layered single-source-of-truth architecture where the
`adr-fmt` binary is the authoritative specification for all
invariant ADR rules.

### Layer Responsibilities

1. **`adr-fmt` (Rust binary)** — invariant rules: template
   structure, naming patterns, relationship vocabulary, lifecycle
   states, link integrity, section ordering, prose minimums. These
   rules cannot be overridden by configuration.

2. **`adr-fmt.toml` (configuration)** — configurable aspects: domain
   definitions (prefix, directory, crate mappings), stale directory
   path, rule parameters (e.g., minimum word count). This file is
   the SSOT for what varies across workspaces.

3. **`--guidelines` output (generated)** — the complete reference
   document combining code invariants and configuration into a
   single readable output. This replaces prescriptive sections that
   would otherwise live in governance prose.

4. **`GOVERNANCE.md` (prose)** — rationale, process, and
   judgment-based guidance. Contains no enforceable rules — only
   explains *why* the system works the way it does and *how* humans
   should exercise discretion.

### Invariant Boundary

A rule is invariant if violating it would produce an inconsistent
ADR corpus regardless of project context: dangling links, missing
required sections, duplicate IDs, or unrecognized lifecycle states.
A rule is configurable if reasonable projects would choose different
values: minimum word counts, domain prefixes, crate mappings.

## Consequences

- No rule exists in prose alone. If a governance rule cannot be
  expressed as a validation check, it belongs in the judgment layer
  (GOVERNANCE.md), not the invariant layer.
- The `--guidelines` flag eliminates the need for a separate
  "ADR writing guide" document that would inevitably drift from
  the tool's actual behavior.
- Adding a new invariant rule requires a code change to `adr-fmt`,
  a new entry in the rule catalog (`adr-fmt.toml`), and an ADR in
  the AFM domain documenting the rationale.
- The SSOT architecture is self-referential: `adr-fmt` validates
  its own domain's ADRs, making the tool both the governor and a
  subject of governance.
