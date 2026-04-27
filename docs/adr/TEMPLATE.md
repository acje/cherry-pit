# ADR Template — Golden Reference

Last-updated: 2026-04-27

This template defines the canonical structure for Architecture Decision
Records in the cherry-pit workspace. It serves two audiences
simultaneously:

1. **Humans** — Context, Related, and Consequences sections provide
   narrative rationale, rejected alternatives, and trade-off analysis.
2. **Agents** — Tagged rules in the Decision section are extracted
   verbatim by `adr-fmt --context <CRATE>` and delivered as a skill
   to the coding agent. The rule text is all the agent sees.

The dual-audience design means: write Context and Consequences for
people, write tagged rules for machines.

---

## Template

````markdown
# PREFIX-NNNN. Title

Date: YYYY-MM-DD
Last-reviewed: YYYY-MM-DD
Tier: S|A|B|C|D
Crates: crate-a, crate-b

## Status

Draft | Proposed | Accepted | Rejected | Deprecated | Superseded by PREFIX-NNNN

## Related

- Root: OWN-ID
- References: PREFIX-NNNN, PREFIX-NNNN
- Supersedes: PREFIX-NNNN

## Context

[Problem statement and motivation — 7–50 words of prose, excluding
code blocks.]

## Decision

[1–3 sentence summary of the chosen approach.]

- **R1**: [Tagged rule — 7–50 words, positive imperative voice]
- **R2**: [Tagged rule — 7–50 words, positive imperative voice]

## Consequences

[Trade-offs, impact, and what becomes easier or harder — 7–50 words
of prose, excluding code blocks.]
````

---

## Field Reference

### Title line

```
# PREFIX-NNNN. Title
```

**Format:** H1 with domain prefix, zero-padded 4-digit number, dot,
space, title text. The prefix must match one configured in
`adr-fmt.toml`. The number must match the filename.

**Guidance:** Title should name the *decision*, not the problem.
"EventEnvelope Construction Invariants" names the solution space.
"Events are broken" names the problem — wrong level.

**Enforced by:** T001 (title present), N001–N004 (filename/prefix).

---

### Date

```
Date: YYYY-MM-DD
```

The date the ADR was first drafted. Never updated after creation.

**Enforced by:** T002.

---

### Last-reviewed

```
Last-reviewed: YYYY-MM-DD
```

The most recent date someone confirmed the ADR is still current.
Update this whenever reviewing the ADR's continued validity, even
if no changes are made. Research shows 20–25% of architectural
decisions go stale within two months — this field is the staleness
signal.

**Enforced by:** T003.

---

### Tier

```
Tier: A
```

Architectural significance level. Determines stability expectations
and sort order in `--context` output (S first, D last). Assignment
questions (answer "Yes" → assign that tier, start from S):

| Tier | Test question |
|------|---------------|
| S | "If this changed, would we rewrite the framework?" |
| A | "If this changed, would trait signatures or type bounds change?" |
| B | "If this changed, would call sites or runtime behaviour change?" |
| C | "If this changed, would only CI, lints, or test setup change?" |
| D | "If this changed, would only one crate's internals change?" |

Higher tiers appear first in `--context` output — the agent sees
foundational constraints before implementation details. This
exploits primacy bias: LLMs attend more strongly to rules at the
start of context.

**Enforced by:** T004.

---

### Crates (optional)

```
Crates: cherry-pit-core, cherry-pit-gateway
```

Comma-separated list of crates this ADR applies to. Used by
`--context` to filter rules per crate. When omitted, the ADR
applies to all crates in its domain. When any ADR in a domain has
`Crates:` populated, ADRs without it are still included — only
ADRs with a non-matching list are excluded.

Use this when a decision applies to a strict subset of crates in
its domain. Omit it when the decision is domain-wide.

---

### Status

```
## Status

Accepted
```

Lifecycle state on its own line below the `## Status` heading.

| State | Meaning |
|-------|---------|
| Draft | Under development, not yet proposed for review |
| Proposed | Submitted for review, awaiting acceptance |
| Accepted | Active — rules extracted by `--context` |
| Rejected | Evaluated and declined — requires Retirement section |
| Deprecated | Was accepted, no longer recommended — requires Retirement |
| Superseded by PREFIX-NNNN | Replaced by another ADR — requires Retirement |

Only `Accepted` ADRs have their rules extracted by `--context`.
Terminal states (Rejected, Deprecated, Superseded) require moving
the file to `stale/` and adding a `## Retirement` section.

`Amended` is not a valid status. If a decision evolves, update the
existing ADR (keeping its ID) or write a new ADR that supersedes it.

**Enforced by:** T005, T006, S004–S006.

---

### Related

```
## Related

- References: CHE-0002, CHE-0010
- Supersedes: CHE-0015
```

Relationships to other ADRs using exactly three permitted verbs:

| Verb | Meaning | Use when |
|------|---------|----------|
| Root | Self-reference marking a tree root | This ADR is the root of a decision subtree |
| References | Soft citation | This ADR cites another for context or builds on it |
| Supersedes | Replaces target entirely | This ADR obsoletes a previous decision |

**Constraints:** Root and References cannot coexist (L009). Every
ADR must have at least one relationship — no orphans (T007).

**Why rejected alternatives matter here:** When an ADR References
another, the agent can traverse the graph via `--critique` to
understand the decision neighbourhood. Include References to
decisions that constrain or motivate this one — this prevents the
agent from proposing approaches that conflict with related decisions.

**Enforced by:** T007, L001, L003, L007–L009.

---

### Context

```
## Context

[Problem statement, motivation, constraints, and alternatives
evaluated.]
```

**Audience:** Humans. The agent does not see this section — only
tagged rules from Decision are extracted.

**Purpose:** Explain *why* this decision was needed. State the
problem, the forces at play, and the alternatives that were
evaluated. The alternatives section is critical: if you don't
document what was rejected and why, the agent will periodically
re-propose those rejected approaches as "improvements."

**Structure guidance:**

1. **Problem statement** — What triggered this decision? What
   constraint or failure motivated it? (1–3 sentences)
2. **Alternatives evaluated** — List options considered, with brief
   pros/cons for each. Number them for reference in Decision.
   (2–5 options typical)
3. **Selection rationale** — Why the chosen option won. What
   trade-offs tipped the balance. (1–2 sentences)

Code blocks illustrating the problem or alternatives are encouraged
but don't count toward the word minimum. Keep prose between 7–50
words (configurable via T015).

**Enforced by:** T008, T015 (word count), T014 (section order).

---

### Decision

```
## Decision

[1–3 sentence summary of what was chosen.]

- **R1**: [Rule text — positive imperative, 7–50 words]
- **R2**: [Rule text — positive imperative, 7–50 words]
```

**Audience:** Dual. The prose summary is for humans. The tagged
rules are for agents. Only tagged rules are extracted by
`--context`.

This section is the most important in the ADR because it produces
the agent-facing rules corpus. Everything below governs how to
write effective tagged rules.

#### Tagged Rules Format

```
- **R1**: Rule text here
- **R2**: Another rule here
```

- Pattern: `- **RN**: text` where N is a sequential integer
- Global identifier: `PREFIX-NNNN:RN` (e.g., `CHE-0042:R1`)
- Constraints: sequential IDs starting at R1, max 10 per ADR,
  7–50 words per rule
- Exempt from T016: Draft and Proposed status (rules crystallize
  during review)

When no tagged rules are found, the entire Decision text is
captured as R0 — a fallback that produces suboptimal agent context
(the agent receives a prose blob instead of scannable rules).
Every Accepted ADR should have explicit tagged rules.

#### Writing Agent-Optimal Rules

The tagged rule text is **all the agent sees** when working on
code for the relevant crate. The surrounding prose, Context,
Consequences, and even the ADR title are not included in
`--context` output. This means each rule must be self-contained.

**Research-backed principles for rules that stick:**

**1. Positive imperative voice — avoid the Pink Elephant.**

Telling an LLM "do not use tRPC" activates `tRPC` in the
attention window, potentially *causing* the violation. State what
to do, not what to avoid.

```markdown
# BAD — activates the unwanted concept
- **R1**: Never construct EventEnvelope via struct literal

# GOOD — states the positive action
- **R1**: Construct EventEnvelope exclusively through EventEnvelope::new(),
  which validates invariants and rejects nil event_id
```

Only use negative framing for genuine hard stops (security,
data loss) where there is no positive reframe.

**2. Embed consequence or rationale inline.**

The agent receives only the rule text. If it doesn't contain the
*why*, the agent treats it as an arbitrary directive — compliance
drops on edge cases. A brief "because" clause or consequence
anchors the rule to architectural reasoning.

```markdown
# WEAK — arbitrary directive
- **R1**: Use raw SQL for reporting queries

# STRONG — embedded rationale
- **R1**: Use raw SQL via query builder for reporting endpoints
  because ORM-generated report queries cause N+1 patterns and
  production latency spikes
```

Keep the rationale to one clause. The full story belongs in
Context, not in the rule.

**3. Use named principles where applicable.**

LLMs activate pre-trained knowledge clusters when they encounter
established principle names. "Follow hexagonal architecture" is
more effective than describing ports and adapters from scratch.
Reference COM principles by ID when relevant.

```markdown
- **R1**: Apply hexagonal architecture (COM-0004): domain logic
  in core crate with no infrastructure imports, adapters in
  gateway crate behind port traits
```

**4. One rule, one enforceable statement.**

Each rule should be a single testable constraint. If a rule
contains "and" joining two independent requirements, split it
into two rules. The agent can verify "did I follow R1?" more
reliably when R1 makes exactly one claim.

**5. Make rules position-resistant.**

Rules in `--context` output are sorted by tier (S→D) and then by
ADR ID. Higher-tier rules appear first, exploiting primacy bias.
But don't rely on position alone — each rule must make sense
regardless of where it appears in the rendered list.

**6. Prefer concrete over abstract.**

A code snippet or type name in a rule is more position-resistant
than an abstract description. "Use `EventEnvelope::new()`" is
more robust than "use the validated constructor."

#### Prose in Decision

Prose paragraphs, headings (### subsections), and code blocks in
the Decision section are for humans — they provide implementation
detail, design sketches, and rationale that doesn't fit in 50
words. They are NOT extracted by `--context`.

Use prose to:
- Explain implementation details (constructors, error types, etc.)
- Show code examples of the decided approach
- Document future work or phasing

Keep the tagged rules as the authoritative, scannable summary.
If a developer (or agent) reads only the tagged rules, they should
understand what to do.

**Enforced by:** T009, T016 (tagged rules), T011 (code block
length), T014 (section order).

---

### Consequences

```
## Consequences

[Trade-offs and impact — what becomes easier, what becomes harder,
what breaks, what improves.]
```

**Audience:** Humans. Not extracted by `--context`.

**Purpose:** Document the trade-offs of the decision. Every
decision makes something easier and something harder. Both
sides must be stated.

**Structure guidance:**

- Use bullet points, each starting with a bold summary
- State positive consequences (what improves) and negative
  consequences (what becomes harder or breaks) explicitly
- Reference affected ADRs, crates, or patterns
- Note breaking changes and migration scope
- Identify future work or open questions this decision creates

The Consequences section is where you record the information
that *would* be in a tagged rule's consequence clause if there
were room. The rule says "do X because Y breaks otherwise" —
Consequences elaborates on what Y breaking actually looks like.

**Enforced by:** T010, T015 (word count), T014 (section order).

---

### Retirement (terminal states only)

```
## Retirement

[Explanation of why this ADR left active service.]
```

Required when status is Rejected, Deprecated, or Superseded.
The file must be moved to the `stale/` directory. Retirement
explains *why* the decision was retired — not just the replacement
ID, but the reason the original decision no longer holds.

**Enforced by:** S004 (missing Retirement), S005 (active ADR has
Retirement), S006 (terminal ADR not in stale/).

---

## Worked Example

```markdown
# CHE-0042. EventEnvelope Construction Invariants

Date: 2026-04-25
Last-reviewed: 2026-04-25
Tier: A
Crates: cherry-pit-core, cherry-pit-gateway

## Status

Accepted

## Related

- References: CHE-0002, CHE-0010, CHE-0016, CHE-0033, CHE-0034, CHE-0039

## Context

EventEnvelope<E> wraps every domain event with metadata stamped by
the EventStore during create and append. CHE-0016 establishes that
callers never construct envelopes directly. However, all seven
fields are pub — any code can construct an envelope via struct
literal with wrong sequence, stale timestamp, or nil event_id.
The safety guarantee is convention, not enforcement. CHE-0002 says
every type must encode its invariants at the type level.

Three options evaluated:

1. **Private fields + validated constructor + accessors** — external
   code cannot construct malformed envelopes. Breaking change.
2. **`#[non_exhaustive]` on struct** — prevents external struct
   literal but fields remain pub readable. Same cross-crate problem.
3. **Accept with documentation** — keep pub fields, document
   convention. Zero breaking changes. CHE-0002 violation persists.

Option 1 chosen: the breaking change is mechanical (48 locations)
and eliminates a class of bugs permanently.

## Decision

Private fields with a validated public constructor that enforces
invariants at the type level.

- **R1**: Construct EventEnvelope exclusively through
  EventEnvelope::new(), which validates non-nil event_id and
  returns Result<Self, EnvelopeError> — struct literal construction
  is impossible due to private fields
- **R2**: Use NonZeroU64 for the sequence field so zero sequences
  are rejected at the type level by both constructor and serde
  deserialization
- **R3**: Access envelope fields through accessor methods (event_id(),
  aggregate_id(), sequence(), timestamp(), payload()) because all
  fields are private
- **R4**: Call EventEnvelope::validate() after deserialization in
  EventStore::load implementations to catch corrupt stored data
  that bypassed constructor validation

## Consequences

- **CHE-0002 compliance restored.** External code cannot construct
  malformed envelopes. The validated constructor rejects nil
  event_id. Zero sequence eliminated at the type level.
- **Breaking change.** 48 locations change: field access → accessor,
  struct literal → EventEnvelope::new(). All changes mechanical.
- **Serde bypass is defense-in-depth.** Window where invalid envelope
  exists in memory (between deserialization and validation) is
  contained within the store implementation.
```

---

## Agent-Extracted Output

Running `adr-fmt --context cherry-pit-core` extracts the tagged
rules from the worked example above as:

```
### CHE-0042 | Cherry Domain | Tier: A | Status: Accepted
- **CHE-0042:R1**: Construct EventEnvelope exclusively through
  EventEnvelope::new(), which validates non-nil event_id and
  returns Result<Self, EnvelopeError> — struct literal construction
  is impossible due to private fields
- **CHE-0042:R2**: Use NonZeroU64 for the sequence field so zero
  sequences are rejected at the type level by both constructor and
  serde deserialization
- **CHE-0042:R3**: Access envelope fields through accessor methods
  (event_id(), aggregate_id(), sequence(), timestamp(), payload())
  because all fields are private
- **CHE-0042:R4**: Call EventEnvelope::validate() after
  deserialization in EventStore::load implementations to catch
  corrupt stored data that bypassed constructor validation
```

This is all the agent receives. Notice:

- Each rule makes sense standalone — no reference to "option 1" or
  surrounding prose
- Positive imperative voice — "construct through", "use", "access
  through", "call"
- Embedded rationale — "which validates...", "so zero sequences
  are rejected...", "because all fields are private", "to catch
  corrupt stored data..."
- Concrete types and method names — `EventEnvelope::new()`,
  `NonZeroU64`, `validate()`
- No Pink Elephant — "struct literal construction is impossible"
  is stated as a fact, not "do not construct via struct literal"

---

## Migration Checklist for Existing ADRs

Existing ADRs use numbered lists (`1. **Bold.** Text`) in Decision
sections. The parser falls back to R0 (entire section as one blob).
To migrate:

1. Identify the core enforceable statements in the existing
   Decision prose
2. Distill each into a tagged rule (`- **R1**: text`)
3. Keep surrounding prose for human context but ensure the tagged
   rules are self-contained
4. Run `cargo run -p adr-fmt -- --lint` — T016 should stop firing
5. Run `cargo run -p adr-fmt -- --context <CRATE>` to verify the
   extracted rules read well in isolation

**Prioritize migration by tier:** S-tier ADRs first (they appear
at the top of `--context` output and set the agent's foundational
constraints), then A, B, C, D.

---

## Rule-Writing Checklist

Before submitting a tagged rule, verify:

- [ ] **Positive voice?** Does it say what to do, not what to avoid?
- [ ] **Self-contained?** Would the rule make sense if you saw only
      this line with no surrounding context?
- [ ] **Consequence embedded?** Does the rule say *why* — even
      briefly — so the agent can generalize to edge cases?
- [ ] **One statement?** Does the rule make exactly one enforceable
      claim, not two joined by "and"?
- [ ] **Concrete?** Does the rule name specific types, methods, or
      patterns rather than abstract concepts?
- [ ] **7–50 words?** Within the T016 bounds?
- [ ] **Named principle?** If a COM/RST principle applies, is it
      referenced by ID?
