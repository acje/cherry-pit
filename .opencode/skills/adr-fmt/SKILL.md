---
name: adr-fmt
description: Run the canonical adr-fmt CLI to lint ADRs, extract crate-scoped architecture rules via --context, list inbound citations via --refs, view domain trees, and author new ADRs conforming to the cherry-pit MADR template. Use when working with architecture decision records, writing or editing ADRs, or needing architecture constraints for a specific crate.
---

# adr-fmt

Read-only ADR governance tool. The binary is owned canonically by
[Mattilsynet/adr-fmt](https://github.com/Mattilsynet/adr-fmt) and is not a
member of this workspace. Install it with the tool's own pinned toolchain:

```bash
cargo +1.98.0 install --git https://github.com/Mattilsynet/adr-fmt \
  --rev ec9f833b13affdd448862b48668dd74ec0234750 --locked adr-fmt
```

Never modifies files.

## Invocation

Run from the **workspace root** (where `Cargo.toml` lives):

```bash
adr-fmt <args>
```

Discovers the corpus through the workspace-root `adr-fmt.toml` marker, whose `[corpus] root` points at `docs/adr`. No path argument needed when running from workspace root.

**Exit codes:**

- `0` — Analysis complete. Does **not** mean "no issues." Parse stdout for diagnostics.
- `1` — Infrastructure error: missing config, invalid configuration, unreadable directory, unknown ADR ID, unknown crate, no domain directories, path containment violation.

## Modes

### Guidelines (default — no flags)

```bash
adr-fmt
```

Prints the complete generated ADR governance reference combining code invariants and configuration. Use this to understand all enforced rules and their parameters.

### Lint

```bash
adr-fmt --lint
```

Validates all ADRs across every domain. All findings are advisory warnings (per AFM-0003). Outputs diagnostics in `warning[RULE_ID] file:line: message` format with a `## Diagnostics: N warning(s) across M ADR(s)` header. Exit 0 always when analysis completes — parse stdout for warnings. Exit 1 only for infrastructure errors (missing config, unreadable directories, path containment violations).

### Context

```bash
adr-fmt --context <CRATE>
```

Extracts tagged decision rules applicable to a specific crate. Foundation domains (COM, RST) are always included. Output is tier-sorted (S first, D last) with rule IDs and layer at end of each line (e.g., `[CHE-0042:R1:L5]`). Exits 1 if the crate is not found in any domain.

Use `--context` to retrieve architecture constraints before writing code for a crate.

### Refs

```bash
adr-fmt --refs <ADR_ID>
```

Lists the ADRs that cite the target through `References:` or `Supersedes:`.
Use it when editing an ADR to find the inbound citations that a change would
affect. The canonical CLI has no `--critique` mode.

### Tree

```bash
adr-fmt --tree
adr-fmt --tree CHE
```

Domain dependency tree with ADR listings. Optional domain prefix filter. Shows stale counts per domain.

## Writing a New ADR

### Section Order

```markdown
# PREFIX-NNNN. Title

Date: YYYY-MM-DD
Last-reviewed: YYYY-MM-DD
Tier: S|A|B|C|D
Status: Accepted

## Related

References: PREFIX-NNNN, PREFIX-NNNN | Supersedes: PREFIX-NNNN

## Context

[Problem statement — 7-100 words prose, alternatives evaluated]

## Decision

[1-3 sentence summary of chosen approach]

R1 [N]: [Tagged rule text — 7-60 words]
R2 [N]: [Tagged rule text — 7-60 words]

## Consequences

[Trade-offs — what becomes easier, what becomes harder]
```

### Tagged Rules

Format: `RN [L]: text` where N is sequential starting at R1 and L
is the Meadows leverage layer (1–12).

Layer mapping:

| Layer | Leverage point (Meadows) | Tier |
|-------|--------------------------|------|
| 1 | The power to transcend paradigms | S |
| 2 | The mindset or paradigm out of which the system arises | S |
| 3 | The goals of the system | S |
| 4 | The power to add, change, evolve, or self-organize system structure | A |
| 5 | The rules of the system (incentives, punishments, constraints) | B |
| 6 | The structure of information flows (who does and does not have access) | B |
| 7 | The gain around driving positive feedback loops | C |
| 8 | The strength of negative feedback loops | C |
| 9 | The lengths of delays, relative to the rate of system change | D |
| 10 | The structure of material stocks and flows | D |
| 11 | The sizes of buffers and other stabilizing stocks, relative to their flows | D |
| 12 | Constants, parameters, numbers | D |

Constraints:
- Maximum 10 rules per ADR
- 7-60 words per rule
- Layer must be 1-12
- Multi-line: indent continuation lines 2+ spaces
- All statuses require tagged rules (no exemptions)
- **Layer-tier alignment (T019):** rule layer should imply a tier within ±1 of the ADR tier (±2 for S-tier ADRs in foundation domains). A D-tier ADR with an L5 rule (B-tier) trips T019 — split the rule into a separate B-tier ADR or adjust the layer.

Every rule must satisfy all five criteria:

1. **Positive commission** — State what to do. Never state what not to do (naming the forbidden thing primes violations).
2. **Unconditional** — No "when X" or "if Y" qualifiers. Scope via the `Crates:` metadata field instead.
3. **Concrete** — Name at least one type, method, file, or trait. Abstract rules lose compliance.
4. **Self-contained** — Must make sense without surrounding prose, title, or other rules.
5. **One statement** — Exactly one enforceable claim per rule. Split "X and Y" into two rules.

### Tier Assignment

First-yes-wins from S downward:

| Tier | Question |
|------|----------|
| S | Does this define the system's paradigm, system-wide pattern, or decision governance? |
| A | Does this introduce/remove trait definitions, generic type params, or plugin boundaries? |
| B | Does this prescribe a structural rule or information flow (type contract, API boundary, visibility, enforcement gate)? |
| C | Does this define how components observe, notify, retry, or react at runtime? |
| D | Is this only a crate-internal detail or tooling config value? |

### Relationship Vocabulary

Three permitted verbs, pipe-separated on one line:

| Verb | Use when |
|------|----------|
| Root | Self-reference marking a tree root (`Root: OWN-ID`) |
| References | Soft citation of another ADR |
| Supersedes | This ADR replaces the target entirely |

Root and References cannot coexist. Every ADR must have at least one relationship.

**Reference load (T020):** `References:` count is capped per tier — S=3, A=5, B=7, C=8, D=5. `Root:` and `Supersedes:` are structural and do not count toward the cap. Excessive references signal the ADR scope is too broad; consider splitting.

### Crates Metadata (optional)

```markdown
Crates: cherry-pit-core, cherry-pit-gateway
```

Place anywhere in the metadata preamble before the first `## ` heading; convention is between Tier and Status. Used by `--context` to filter rules per crate. Omit when the decision applies to all crates in its domain.

### Filename Convention

```
PREFIX-NNNN-kebab-slug.md
```

Prefix must match a domain in `adr-fmt.toml`. Number is zero-padded 4 digits. Slug is lowercase kebab-case.

## Workflow

After creating or editing any ADR:

1. Run `adr-fmt --lint` — parse stdout for diagnostics
2. Fix reported issues
3. Re-run to confirm clean output
4. Run `adr-fmt --context <CRATE>` to verify extracted rules read well in isolation
5. Commit

## Configuration

`adr-fmt.toml` defines:

- **Domains** — prefix, name, directory, description, crate mappings, foundation flag
- **Rule parameters** — overrides for configurable rule thresholds (e.g., T015 word counts)
- **Stale directory** — where terminal-status ADRs are archived

Consult this file to determine valid domain prefixes and which crates belong to which domain.
