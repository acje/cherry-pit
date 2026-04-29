# adr-fmt

Read-only ADR analysis tool for cherry-pit. Single source of truth for
all invariant ADR governance rules. Never modifies files. stdout =
output, stderr = errors.

Part of the [cherry-pit](../../README.md) workspace.

## Usage

```text
adr-fmt                                # show governance reference (default)
adr-fmt --lint [ADR_DIR]               # validate all ADRs (advisory)
adr-fmt --critique <ADR_ID> [ADR_DIR]  # focal ADR + transitive closure
adr-fmt --context <CRATE> [ADR_DIR]    # decision rules for a crate
adr-fmt --tree [DOMAIN] [ADR_DIR]      # domain dependency tree
```

Run via `cargo run -p adr-fmt` or `cargo run -p adr-fmt -- <args>`.

Auto-discovers `docs/adr/` by walking up from CWD looking for
`docs/adr/GOVERNANCE.md`. Pass explicit `ADR_DIR` to override.

## Exit Codes

- `0` — Analysis complete. **Parse stdout for diagnostics** — exit 0
  does not mean "no issues."
- `1` — Infrastructure error: missing config, unreadable directory,
  invalid configuration, unknown ADR ID, unknown crate, no domain
  directories, path containment violation.

The advisory-only exit policy is governed by AFM-0003. CI scripts
should parse the `## Diagnostics: N warning(s)` header on stdout and
fail the job when N exceeds the project threshold.

## Modes

### Default — Guidelines Reference

```bash
adr-fmt
```

With no flags, prints the complete generated ADR governance reference
combining code invariants and configuration. If no `adr-fmt.toml`
is found, prints a setup guide instead.

### Lint

```bash
adr-fmt --lint
```

Runs all rules (template, link, naming, structure, parser-stage)
across every ADR. All findings emit at warning severity per AFM-0003;
lint exits 0 on completion regardless of warning count. Exit 1 is
reserved for infrastructure errors per AFM-0003 R1.

Diagnostic output format:

```text
## Diagnostics: N warning(s) across M ADR(s)

- **warning[RULE_ID]** file:line: message
```

### Critique

```bash
adr-fmt --critique CHE-0042
adr-fmt --critique CHE-0042 --depth 3
```

BFS transitive closure around a focal ADR. Default depth 1. Follows
fan-out (forward relationships) and fan-in (reverse/children). Stale
ADRs filtered with count note. Output uses Alternative 4 markdown:
`◆ FOCAL`, `◇ CONNECTED`, `◈ EXCLUDED` headers. Connected blocks
sorted by tier (S→D) then ID. Focal block includes per-rule tension
analysis showing tier-distance from the ADR's tier.

The ADR ID must conform to the strict shape `^[A-Z]{2,4}-[0-9]{4}$`
(uppercase prefix, exactly four digits). Lowercase or malformed input
exits 1.

### Context

```bash
adr-fmt --context cherry-pit-core
```

Tagged decision rules applicable to a crate. Resolution: if any ADR
in a domain has `Crates:` populated, ADRs without `Crates:` are still
included; only ADRs with a non-matching `Crates:` list are excluded.
If no domain ADR has `Crates:`, all domain ADRs are included.
Foundation domains (e.g., COM, RST) are always included. Output
sorted: foundation first, then by tier, then ID. Exits 1 if the crate
is not found in any domain.

### Tree

```bash
adr-fmt --tree
adr-fmt --tree CHE
```

Domain dependency tree with ADR listings. Optional domain prefix
filter. Shows stale counts per domain.

## SSOT Architecture

| Source | Scope |
|--------|-------|
| `adr-fmt` (code) | Invariant rules: template, naming, vocabulary, lifecycle, links, containment |
| `adr-fmt.toml` | Configurable: domains, crate mappings, rule params, stale directory |
| Default-mode output | Generated complete reference |
| `GOVERNANCE.md` | Rationale, process, judgment only |

## Rule Catalog

All rules emit warnings. `adr-fmt` is advisory per AFM-0003.

### Template (T001–T020)

| Rule | Description |
|------|-------------|
| T001 | H1 title `# PREFIX-NNNN. Title` present |
| T002 | `Date:` field present |
| T003 | `Last-reviewed:` field present (all tiers) |
| T004 | `Tier:` field present |
| T005 | `## Status` section with status line present |
| T005b | Dual status — both preamble field and `## Status` section present |
| T005c | Legacy `## Status` section — migrate to `Status:` preamble field |
| T006 | Status value recognized; no parenthetical annotations |
| T007 | `## Related` has ≥1 relationship (no orphans, no placeholders) |
| T008 | `## Context` section present |
| T009 | `## Decision` section present |
| T010 | `## Consequences` section present |
| T011 | Code block exceeds 20 lines |
| T012 | Amendment date ≥ creation date; valid ISO 8601 |
| T013 | *(reserved)* |
| T014 | Section order: Status → Related → Context → Decision → Consequences (+ Retirement for stale) |
| T015 | Prose section below minimum word count (configurable `min_words`, default 10) |
| T016 | Decision section lacks tagged rules (`RN [L]: text`) or has non-sequential IDs. Exempt: Draft, Proposed |
| T017 | *(reserved)* |
| T018 | *(reserved)* |
| T019 | Rule-tier tension: rule's Meadows layer implies a tier >1 rank from the ADR tier |
| T020 | Reference load: `References:` count exceeds tier-scaled cap (S=3, A=5, B=7, C=8, D=5) |

### Link (L001–L009)

| Rule | Description |
|------|-------------|
| L001 | Dangling link — target ADR not found in any domain |
| L003 | Supersedes verb without matching `Superseded by` status on target |
| L006 | Legacy relationship verb — vocabulary restricted by AFM-0009 to Root, References, Supersedes |
| L007 | Stale reference — target ADR is in stale archive |
| L008 | Root verb target does not match own ID |
| L009 | Root and References coexist — Root ADRs may not also use References |

### Naming (N001–N004)

| Rule | Description |
|------|-------------|
| N001 | Filename matches `PREFIX-NNNN-kebab-slug.md` (prefix 2–4 uppercase ASCII) |
| N002 | Number in filename matches H1 title ID |
| N003 | Slug is valid lowercase kebab-case |
| N004 | Domain prefix not found in configuration |

### Structure (S004–S006)

| Rule | Description |
|------|-------------|
| S004 | Stale ADR missing `## Retirement` section (≥ min_words) |
| S005 | Active ADR has `## Retirement` section (forbidden outside stale/) |
| S006 | Terminal-status ADR not in stale directory |

### Parser-stage (P001–P002)

Per AFM-0017, files matching the prefix filename pattern that fail to
parse cleanly surface as advisory warnings rather than vanishing.

| Rule | Description |
|------|-------------|
| P001 | File matches filename pattern but fails to read (permission, file-system race, EISDIR) |
| P002 | File is readable but lacks a valid H1 title (empty, missing `#`, or prefix mismatch) |

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

Terminal: `Rejected`, `Deprecated`, `Superseded by PREFIX-NNNN`.
Terminal states require: move to stale directory + `## Retirement`
section. `Amended` is not a valid status.

### Relationship Vocabulary

Three permitted verbs (AFM-0009):

| Verb | Meaning |
|------|---------|
| Root | Self-reference (`Root: OWN-ID`) marking a tree root |
| References | Soft citation — citing another ADR |
| Supersedes | Replaces target entirely; target becomes Superseded |

Constraints: Root + References cannot coexist (L009). Root +
Supersedes can. Legacy verbs (Depends on, Extends, Illustrates, etc.)
trigger L006 with migration guidance.

### Tagged Rules

Decision sections must contain tagged rules:

```markdown
R1 [5]: First rule — concrete, positive imperative, unconditional
R2 [5]: Second rule
```

Format: `RN [L]: text` where N is sequential starting at R1 and L is
the Meadows leverage layer (1–12). Global identifier: `PREFIX-NNNN:RN:LN`
(e.g., `CHE-0042:R1:L5`). Maximum 10 rules per ADR; 7–60 words per
rule. When no tagged rules are found, the entire Decision text is
captured as R0 (triggers T016).

### Crates Metadata

```markdown
Crates: crate-a, crate-b
```

Placed in the metadata preamble (anywhere before the first `## `
heading; convention is between Tier and Status). Used by `--context`
to filter rules per crate.

## Configuration

`docs/adr/adr-fmt.toml` — required for all non-default modes
(`--lint`, `--critique`, `--context`, `--tree`). When absent, the
default mode prints a setup guide instead of erroring. Config errors
and path containment violations are hard failures (exit 1).

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

[[rules]]
id = "T015"
[rules.params]
min_words = 10
```

Domain directories are joined to the canonical ADR root with strict
path containment (AFM-0016): absolute paths, `..` traversal, and
symlinks that resolve outside the root are rejected at startup.

## Workflow

After creating or editing any ADR:

1. `cargo run -p adr-fmt -- --lint` — parse stdout for warnings
2. Fix reported issues
3. Re-run to confirm clean output
4. `cargo run -p adr-fmt -- --context <CRATE>` to verify extracted
   rules read well in isolation
5. Commit
