# adr-fmt Redesign — Pure Analysis Tool with Critique and Context

## Overview

Transform adr-fmt from a lint-and-generate tool into a read-only, agent-first
analysis tool. Add `--critique` and `--context` modes, remove all file-writing
side effects, unify output into token-efficient Alternative 4 markdown format.

## Design Decisions

| Decision | Resolution |
|---|---|
| Entry point | `--critique CHE-0042` (single-ADR focused) |
| Critique scope | Transitive closure, stale ADRs filtered with count note |
| Consumer | Agent-first; human secondary |
| Output format | Alternative 4: concatenated markdown, `◆`/`◇` headers, `---` separators |
| Context feature | `--context <crate>` — Decision section rules only |
| Foundation domains | Always included in `--context` |
| `Crates:` field | Per-ADR metadata; authoritative when present, domain-level fallback |
| `Crates:` position | Tool advises on placement, not prescriptive |
| Rule extraction | `- **RN**: text` pattern, globally `CHE-0042:R1` |
| Non-conforming fallback | `R0: <full decision text>` |
| Rule numbering | Sequential enforced; gaps diagnosed under T016 |
| T016 scope | Exempt Draft and Proposed; covers both "no tagged rules" and "non-sequential IDs" |
| File writing | Removed entirely — read-only, idempotent |
| Index trees | `--index [DOMAIN]` stdout mode, optional domain filter |
| Backward compat | Not a concern |

## Output Format (Alternative 4)

All modes emit to stdout in concatenated markdown with structured header
blocks. No YAML frontmatter, no JSON. Pure markdown optimized for LLM token
efficiency.

```markdown
## ◆ FOCAL: CHE-0042 | Tier: A | Status: Accepted
## Domain: cherry | Crates: cherry-pit-core, cherry-pit-gateway
## Fan-out: References CHE-0001, References CHE-0003
## Fan-in: References ← CHE-0050, Supersedes ← CHE-0060

# CHE-0042 · Use Event Sourcing for Aggregate Persistence

[full ADR text]

---

## ◇ CONNECTED: CHE-0001 | Tier: S | Status: Accepted
## Path: CHE-0042 → References → CHE-0001

# CHE-0001 · Event-Driven Architecture

[full ADR text]

---

## ◈ EXCLUDED: 2 stale ADRs filtered from closure
```

## Steps

### Step 1 · `src/output.rs` (new) — unified output formatter

- `OutputBlock` enum: `Focal { meta, content }`, `Connected { meta, content,
  path }`, `Excluded { count, reason }`
- `HeaderMeta`: id, tier, status, domain, crates, fan-in summary, fan-out
  summary
- `render_blocks()` → Alternative 4 markdown with `## ◆ FOCAL` /
  `## ◇ CONNECTED` / `## ◈ EXCLUDED` / `---` separators
- `render_diagnostics()` → lint output as Alternative 4 markdown blocks to
  stdout
- `render_rules()` → context mode output: crate name header, then per-ADR rule
  blocks ordered by tier
- `render_index()` → domain tree with box-drawing to stdout
- All renderers return `String`; caller writes to stdout

### Step 2 · `src/model.rs` — type extensions

- Add `crates: Vec<String>` to `AdrRecord`
- Add `decision_rules: Vec<TaggedRule>` to `AdrRecord`
- Add `TaggedRule { id: String, text: String, line: usize }`
- Add `impl Default for AdrRecord` to reduce test helper duplication across
  `nav.rs`, `rules/template.rs`, `rules/links.rs`, `generate.rs` (4 copies of
  `make_record`)
- Add `decision_content: Option<String>` to `AdrRecord` (full Decision section
  text, needed for R0 fallback)

### Step 3 · `docs/adr/adr-fmt.toml` — rule catalog update

- Add `[[rules]] id = "T016"`, `category = "template"`,
  `description = "Decision section lacks tagged rules or has non-sequential
  rule IDs"`
- Remove I001, I002, I003 entries

### Step 4 · `src/parser.rs` — new extraction logic

- `find_crates_field()` — parse `Crates: crate-a, crate-b` from metadata
  preamble; return `Vec<String>`
- `extract_decision_content()` — capture full text of Decision section (for R0
  fallback)
- `extract_tagged_rules()` — regex match
  `^\s*-\s*\*\*R(\d+)\*\*:\s*(.+)` within Decision section; return
  `Vec<TaggedRule>`
- R0 fallback: when no `- **RN**:` matches found, produce single
  `TaggedRule { id: "R0", text: <decision_content>, line }`
- Populate new `AdrRecord` fields
- Unit tests: `Crates:` present/empty/absent; tagged rules
  normal/malformed/absent/mixed-with-prose; R0 fallback; sequential ID
  validation data

### Step 5 · `src/critique.rs` (new) — critique mode

- `critique(focal_id: &AdrId, records: &[AdrRecord], config: &Config) ->
  Vec<OutputBlock>`
- Resolve focal from records by ID → error to stderr + exit 1 if not found
- BFS transitive closure:
  - Fan-out: follow all `Relationship` edges from focal
  - Fan-in: use `nav::compute_children()` to find ADRs referencing focal
  - Expand transitively in both directions
  - `HashSet<AdrId>` visited guard prevents cycles
- Filter: exclude `is_stale` ADRs from output; count them
- Build `OutputBlock::Focal` for target (full file content)
- Build `OutputBlock::Connected` for each non-stale ADR in closure (full file
  content, relationship path, tier, status)
- Build `OutputBlock::Excluded` if stale count > 0: "N stale ADRs excluded
  from closure"
- Ordering: focal first → connected sorted by tier (S→D) then by ID →
  excluded note last

### Step 6 · `src/context.rs` (new) — context mode

- `context(crate_name: &str, records: &[AdrRecord], config: &Config) ->
  Vec<CrateRule>`
- Resolution chain:
  1. Find domains where `crate_name` ∈ `domain.crates` → candidate domains.
     Error to stderr + exit 1 if not found in any domain
  2. Within candidate domains: if any ADR has `crates` field populated, filter
     to ADRs where `crate_name` ∈ `adr.crates`; else include all ADRs in
     domain
  3. Always include all ADRs from `foundation = true` domains (COM, RST)
- Extract `decision_rules` from each resolved ADR
- Build `CrateRule { adr_id, tier, status, domain, rules: Vec<TaggedRule> }`
- Ordering: foundation domains first (sorted by prefix) → non-foundation by
  tier (S→D) → by ADR ID

### Step 7 · `src/main.rs` — control flow restructure

- Add clap args: `--critique <ADR_ID>`, `--context <CRATE>`,
  `--index [DOMAIN]`
- Mutual exclusion via clap `group` or `conflicts_with_all`: `--critique`,
  `--context`, `--index`, `--report`, `--guidelines`
- Remove `generate::generate_all()` call
- Mode dispatch:
  - `--guidelines` → early exit (existing)
  - `--critique` → parse all records → `critique::critique()` →
    `output::render_blocks()` → stdout
  - `--context` → parse all records → `context::context()` →
    `output::render_rules()` → stdout
  - `--index` → parse all records → build index tree →
    `output::render_index()` → stdout (optionally filtered by domain arg)
  - `--report` → parse all records → `nav::compute_children()` →
    `output::render_blocks()` → stdout (replaces `nav::print_report()`)
  - Default (no flags) → parse all records → `rules::run_all()` →
    `output::render_diagnostics()` → stdout
- stderr: infrastructure errors only
- Exit codes: 0 = analysis complete, 1 = infrastructure error

### Step 8 · `src/rules/template.rs` — add T016

- `check_tagged_rules(record: &AdrRecord, config: &Config) -> Vec<Diagnostic>`
- Two diagnostic variants under T016:
  - No tagged rules: `"Decision section lacks tagged rules (- **RN**:
    pattern)"` — only when `decision_rules` is empty or sole entry is R0
  - Non-sequential IDs: `"Tagged rule IDs not sequential (gap after R2)"` —
    parse R-IDs as integers, check for gaps
- Exempt `Status::Draft` and `Status::Proposed`

### Step 9 · Remove `src/generate.rs` and `src/rules/index.rs`

- `generate.rs`: extract dependency tree rendering logic (primary parent
  selection, box-drawing, cycle detection) into a pure function; move to
  `output.rs` or keep as `generate::render_tree()` (no I/O). Delete all
  `safe_write()`, README I/O, `<!-- Generated -->` guard logic
- `rules/index.rs`: delete entirely
- `rules/mod.rs`: remove `mod index` declaration and `index::check` call site

### Step 10 · `src/guidelines.rs` — sync with changes

- Add documentation for `--critique`, `--context`, `--index` modes
- Add T016 rule description
- Remove I001-I003 references
- Document `Crates:` metadata field convention
- Document tagged rule `- **RN**: text` convention
- Update mode/flag reference section

### Step 11 · `src/nav.rs` — update report output

- Keep `compute_children()` unchanged
- Replace `print_report()` direct `println!` with `output.rs` formatter call
  (or remove `print_report()` entirely — `main.rs` builds output blocks from
  children data)

### Step 12 · `src/config.rs` — cleanup

- Remove `#[allow(dead_code)]` from `DomainConfig.crates`

### Step 13 · `tests/integration.rs` — comprehensive test update

Remove:
- All README-generation tests (file existence, safe_write)
- Tests asserting stderr compiler-style output format

Add:
- `--critique CHE-0042`: focal + connected ADRs present, Alternative 4
  structure, tier ordering
- `--critique INVALID-9999`: exit 1, stderr error message
- `--critique` on isolated ADR (no relations): focal only, no connected blocks
- `--critique` with cycle in test corpus: terminates, all cycle members present
- `--critique` with stale ADR in closure: stale excluded, count note present
- `--context cherry-pit-core`: CHE rules + COM + RST foundation rules present
- `--context pardosa`: PAR rules + COM + RST foundation rules present
- `--context unknown-crate`: exit 1, stderr error
- `--context` with per-ADR `Crates:` annotations: only annotated ADRs included
- `--context` fallback (no annotations): all domain ADRs included
- `--index`: full domain tree output to stdout
- `--index CHE`: filtered to CHE domain only
- Read-only verification: no new/modified files in tempdir after any mode
- Mutual exclusion: `--critique` + `--context` → clap error
- T016: ADR with tagged rules → no warning; ADR without → warning; Draft ADR
  without → no warning; ADR with gap (R1, R3) → warning
- Default lint mode: output on stdout in Alternative 4 format

### Step 14 · `.opencode/skills/adr-fmt/SKILL.md` — update skill definition

- Update invocation examples for new flags
- Remove README side-effect documentation
- Add `--critique`, `--context`, `--index` mode descriptions
- Update exit code documentation

## Rigormortis Findings (Addressed)

| Finding | Severity | Mitigation |
|---|---|---|
| Transitive closure cycle safety | High | BFS with `HashSet<AdrId>` visited guard in `critique.rs` |
| `Crates:` dual-source precedence | High | Per-ADR field authoritative when present; domain-level is default |
| `generate.rs` removal breaks `main.rs` | High | Step 7 restructures control flow with mutual exclusion |
| `rules/index.rs` removal requires `mod.rs` update | High | Step 9 removes `mod index` and call site |
| `AdrRecord` struct expansion bloats helpers | Medium | `Default` impl in Step 2 |
| T016 missing from `adr-fmt.toml` | Medium | Added in Step 3 |
| `guidelines.rs` not in change list | Medium | Added as Step 10 |
| `DomainConfig.crates` has `dead_code` | Low | Removed in Step 12 |
| T016 on Draft ADRs | Low | Exempt Draft and Proposed |
