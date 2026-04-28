---
name: adr-fmt
description: Validate Architecture Decision Records against governance rules, lint for template/link/naming compliance, and provide critique/context analysis. Use when creating, editing, reviewing, or validating ADRs, or when user mentions "adr-fmt".
---

# adr-fmt

Read-only ADR analysis tool for cherry-pit. SSOT for all invariant ADR
governance rules. Never modifies files. stdout = output, stderr = errors.

## Usage

```text
adr-fmt [ADR_DIR]                          # default: print governance guidelines
adr-fmt --lint [ADR_DIR]                   # lint all ADRs
adr-fmt --critique <ADR_ID> [ADR_DIR]      # focal ADR + direct neighbors
adr-fmt --critique <ADR_ID> --depth N      # bounded transitive closure (default: 1)
adr-fmt --context <CRATE> [ADR_DIR]        # decision rules for a crate
adr-fmt --tree [DOMAIN] [ADR_DIR]          # domain dependency tree
```

Run via `cargo run -p adr-fmt` or `cargo run -p adr-fmt -- <args>`.

Auto-discovers `docs/adr/` by walking up from CWD looking for
`docs/adr/GOVERNANCE.md`. Pass explicit `ADR_DIR` to override.

## Exit Codes

- `0` — Analysis complete. **Parse stdout for diagnostics** — exit 0 does not mean "no issues."
- `1` — Infrastructure error (missing config, unknown ADR ID, unknown crate, no domain directories).

## Modes

### Guidelines (default, no flag)

Two behaviors:
- **No config:** Prints prescriptive setup guide (quickstart template).
- **Config present:** Prints complete governance reference derived from hardcoded
  rule catalog + config domains/overrides. Human-readable formatting.

### Lint

`--lint` — Runs all rules (template, link, naming, structure) across every ADR.
Diagnostic output format:

```text
## Diagnostics: E error(s), W warning(s) across N ADR(s)

- **severity[RULE_ID]** file:line: message
```

### Critique

`--critique CHE-0042` — BFS transitive closure around focal ADR, bounded by
`--depth N` (default: 1). Follows fan-out (forward relationships) and fan-in
(reverse/children). Stale ADRs are included without filtering. Output uses
Alternative 4 markdown: `◆ FOCAL`, `◇ CONNECTED` headers. Connected blocks
sorted by tier (S→D) then ID.

### Context

`--context cherry-pit-core` — tagged decision rules applicable to a crate.
Resolution: if any ADR in a domain has `Crates:` populated, ADRs without
`Crates:` are still included; only ADRs with a non-matching `Crates:` list
are excluded. If no domain ADR has `Crates:`, all domain ADRs included.
Foundation domains (e.g., COM) always included. Output sorted: foundation
first, then by tier, then ID.
Exits 1 if crate not found in any domain.

### Tree

`--tree [DOMAIN]` — domain tree with ADR listings. Optional domain prefix
filter (e.g., `--tree CHE`). Shows stale counts per domain.

## SSOT Architecture

| Source | Scope |
|--------|-------|
| `adr-fmt` (code) | Invariant rules: template, naming, vocabulary, lifecycle, links |
| `adr-fmt.toml` | Configurable: domains, crate mappings, rule param overrides, stale directory |
| Default mode output | Generated complete governance reference |
| `GOVERNANCE.md` | Rationale, process, judgment only |

## Rule Catalog

All rules emit warnings. `adr-fmt` is advisory. Rules are hardcoded in the
binary; config only provides parameter overrides.

### Template (T001–T016)

| Rule | Description |
|------|-------------|
| T001 | H1 title `# PREFIX-NNNN. Title` present |
| T002 | `Date:` field present |
| T003 | `Last-reviewed:` field present (all tiers) |
| T004 | `Tier:` field present |
| T005 | `## Status` section with status line present |
| T006 | Status value recognized; rejects `Amended`; no parenthetical annotations |
| T007 | `## Related` has ≥1 relationship (no orphans, no placeholders) |
| T008 | `## Context` section present |
| T009 | `## Decision` section present |
| T010 | `## Consequences` section present |
| T011 | Code block exceeds 30 lines |
| T013 | *(reserved)* |
| T014 | Section order: Status → Related → Context → Decision → Consequences (+ Retirement for stale) |
| T015 | Prose section word count 7–50 (Context, Consequences, Retirement). Configurable: `min_words`, `max_words` |
| T016 | Decision section tagged rules: ≥1 rule, sequential IDs, max 5, 7–60 words each. Exempt: Draft, Proposed |

### Link (L001–L009)

| Rule | Description |
|------|-------------|
| L001 | Dangling link — target ADR not found in any domain |
| L003 | Supersedes verb without matching `Superseded by` status on target |
| L007 | Stale reference — target ADR is in stale archive |
| L008 | Root verb target does not match own ID |
| L009 | Root and References coexist — Root ADRs may not also use References |

### Naming (N001–N004)

| Rule | Description |
|------|-------------|
| N001 | Filename matches `PREFIX-NNNN-kebab-slug.md` (prefix 2–4 uppercase) |
| N002 | Number in filename matches H1 title ID |
| N003 | Slug is valid lowercase kebab-case |
| N004 | Domain prefix not found in configuration |

### Structure (S004–S006)

| Rule | Description |
|------|-------------|
| S004 | Stale ADR missing `## Retirement` section (≥ min_words) |
| S005 | Active ADR has `## Retirement` section (forbidden outside stale/) |
| S006 | Terminal-status ADR not in stale directory |

## ADR Metadata

### Tiers

| Tier | Name | Scope |
|------|------|-------|
| S | Foundational | Design philosophy — reverberates through every crate |
| A | Core | Core trait design — major refactoring across crates |
| B | Behavioural | API semantics — coordinated updates across call sites |
| C | Tooling | DX, build — localized to config or test infra |
| D | Detail | Implementation detail — one crate's internals |

### Lifecycle States

`Draft` → `Proposed` → `Accepted`

Terminal: `Rejected`, `Deprecated`, `Superseded by PREFIX-NNNN`

Terminal states require: move to stale directory + `## Retirement` section.

Note: `Amended` is no longer a valid status. ADRs with Amended status should
be changed to `Accepted`. T006 fires on Amended.

### Relationship Vocabulary

Three permitted verbs:

| Verb | Meaning |
|------|---------|
| Root | Self-reference (`- Root: OWN-ID`) marking a tree root |
| References | Soft citation — citing another ADR |
| Supersedes | Replaces target entirely; target becomes Superseded |

Constraints: Root + References cannot coexist (L009). Root + Supersedes can.
Legacy verbs (Depends on, Extends, Illustrates, etc.) are parsed but produce
no L006 warning (rule removed); they remain as documentation of migration path.

### Tagged Rules

Decision sections must contain tagged rules:

```markdown
- **R1**: First rule or decision statement (7–60 words)
- **R2**: Second rule, can span multiple lines with
  continuation indented ≥2 spaces (7–60 words total)
```

Global identifier: `PREFIX-NNNN:RN` (e.g., `CHE-0042:R1`).
Constraints: sequential IDs, max 5 per ADR, 7–60 words each.
Multi-line: indent continuation ≥2 spaces; blank line terminates.
When no tagged rules found, entire Decision text is captured as R0 (triggers T016).

### Crates Metadata

```markdown
Crates: crate-a, crate-b
```

Placed after Date/Tier, before `## Status`. Used by `--context` mode.

## Configuration

`docs/adr/adr-fmt.toml` — required for `--lint`, `--critique`, `--context`,
`--tree` modes. Default mode shows setup guide if missing.

Rules are hardcoded in the binary. Config provides only domain definitions
and optional parameter overrides.

```toml
[stale]
directory = "stale"

[[domains]]
prefix = "CHE"
name = "Cherry Domain"
directory = "cherry"
description = "Architecture decisions"
crates = ["cherry-pit-core"]
foundation = false            # true = included in every --context query

# Optional: override hardcoded rule parameters
[[rules]]
id = "T015"
params = { min_words = 7, max_words = 50 }
```

Legacy config format (with `category`/`description` fields in `[[rules]]`)
still parses but emits a deprecation warning to stderr.

## Workflow

After creating or editing any ADR:

1. `cargo run -p adr-fmt -- --lint` — parse stdout for warnings
2. Fix reported issues
3. Re-run to confirm clean output
4. Commit
