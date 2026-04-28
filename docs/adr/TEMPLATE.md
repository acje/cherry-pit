# ADR Template — Golden Reference

Last-updated: 2026-04-28

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
Status: Draft | Proposed | Accepted | Rejected | Deprecated | Superseded by PREFIX-NNNN

## Related

Root: OWN-ID | References: PREFIX-NNNN, PREFIX-NNNN | Supersedes: PREFIX-NNNN

## Context

[Problem statement and motivation — 7–50 words of prose, excluding
code blocks.]

## Decision

[1–3 sentence summary of the chosen approach.]

R1 [N]: [Tagged rule — 7–60 words, positive imperative, unconditional]
R2 [N]: [Tagged rule — 7–60 words, positive imperative, unconditional]

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

Architectural significance level derived from Donella Meadows' twelve
leverage points. Determines sort order in `--context`
output (S first, D last). Uses **system-characteristic framing** —
classify by what the decision *is*, not by blast radius. First-yes-
wins: start at S, assign the first tier whose question yields "yes."

| Tier | System characteristic | Meadows levels | Classification question |
|------|----------------------|----------------|------------------------|
| S | **Intent** — paradigm, goals, governance | 1–3 | Does this decision define the system's paradigm, system-wide architectural pattern, or decision governance? |
| A | **Self-organization** — capacity to evolve structure | 4 | Does this decision introduce or remove trait definitions, generic type parameters, or plugin boundaries that enable new implementations? |
| B | **Design** — rules, information flows | 5–6 | Does this decision prescribe a structural rule or establish an information flow — a type contract, API boundary, visibility constraint, enforcement gate, or observability requirement? |
| C | **Feedbacks** — reinforcing and balancing loops | 7–8 | Does this decision define how components observe, notify, retry, or react to each other at runtime? |
| D | **Parameters** — constants, stocks, flows, delays | 9–12 | Is this only a crate-internal implementation detail or tooling configuration value? |

For theoretical foundation, assignment guidance, and disambiguation
rules, see [GOVERNANCE.md § 2](GOVERNANCE.md#2-tier-system).

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
Status: Accepted
```

Lifecycle state as a metadata field in the preamble (before any H2
heading). Legacy format (`## Status` section with value on next line)
is still recognized as a fallback. If both are present, the metadata
field takes precedence and a T005b warning is emitted.

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

**Enforced by:** T005, T005b, T006, S004–S006.

---

### Related

```
## Related

References: CHE-0002, CHE-0010 | Supersedes: CHE-0015
```

Pipe-separated relationships on a single line. Each segment is
`Verb: TARGET1, TARGET2`. Multiple segments separated by ` | `.
Three permitted verbs:

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

R1 [N]: [Rule text — positive imperative, 7–60 words]
  [Optional continuation indented ≥2 spaces]
R2 [N]: [Rule text — positive imperative, 7–60 words]
```

**Audience:** Dual. The prose summary is for humans. The tagged
rules are for agents. Only tagged rules are extracted by
`--context`.

This section is the most important in the ADR because it produces
the agent-facing rules corpus. Everything below governs how to
write effective tagged rules.

#### Tagged Rules Format

```
R1 [5]: Rule text here, naming specific types and methods
R2 [5]: Continuation-capable rule that wraps to
  the next line with two-space indent
```

- Pattern: `RN [L]: text` where N is a sequential integer and L
  is the Meadows leverage layer (1–12)
- Multi-line: indent continuation lines ≥2 spaces. A blank line
  or next `RN [L]:` terminates the rule. Continuation lines
  are joined with a space.
- Global identifier: `PREFIX-NNNN:RN:LN` (e.g., `CHE-0042:R1:L5`)
- Layer [N]: Meadows leverage point classifying intervention type:

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
- Constraints: sequential IDs starting at R1, max 10 per ADR,
  7–60 words per rule, layer 1–12
- All statuses require tagged rules (no Draft/Proposed exemption)

**Why max 10 rules per ADR:** Research shows P(all rules followed)
= P(individual)^N. At 90% per-rule compliance, 10 rules yield 35%
all-correct; 15 rules yield 21%. Fewer rules with higher
individual compliance beats more rules.

#### Writing Agent-Optimal Rules

The tagged rule text is **all the agent sees** when working on
code for the relevant crate. The surrounding prose, Context,
Consequences, and even the ADR title are not included in
`--context` output. This means each rule must be self-contained.

**Research-backed principles for rules that stick:**

**1. Positive commission only — no exceptions.**

Telling an LLM "do not use struct literals" activates `struct
literals` in the attention window, causing the violation. Research
(n=40,000) shows 87.5% of negative-constraint violations are
priming failures — naming the forbidden thing triggers it.

Every rule must state what to do. Every prohibition must be
reframed as a positive commission. No exceptions.

```markdown
# BAD — primes the unwanted concept (87.5% violation rate via priming)
- **R1**: Never construct EventEnvelope via struct literal

# GOOD — states the required action
R1 [5]: Construct EventEnvelope exclusively through
  EventEnvelope::new(), which validates non-nil event_id and
  returns Result<Self, EnvelopeError>

# BAD — primes the bypass path
- **R2**: Never bypass the domain layer

# GOOD — names the required path
R2 [5]: Route all domain operations through port traits
  defined in cherry-pit-core
```

**2. Unconditional — no "when X" patterns.**

Conditional rules ("when doing X, always Y") lose 15–20 percentage
points of compliance versus unconditional rules. Over 30% of
conditional-rule failures are condition-check errors — the model
fails to recognize the trigger condition.

If a rule applies only in specific contexts, scope it via the
`Crates:` metadata field, not via conditional rule text.

```markdown
# BAD — conditional framing (15–20pp compliance loss)
- **R1**: When implementing EventStore, validate envelopes
  after deserialization

# GOOD — unconditional, scoped via Crates: field
R1 [5]: Call EventEnvelope::validate() after deserialization
  in EventStore::load implementations to catch corrupt data
```

**3. Concrete — name types, methods, and files.**

Every rule MUST name at least one type, method, file, or trait.
Abstract descriptions ("use the validated constructor") are
position-fragile — they lose compliance when they appear in the
middle of the context window. Concrete names anchor attention.

```markdown
# BAD — abstract, position-fragile
- **R1**: Use the validated constructor for all envelope creation

# GOOD — concrete types and methods anchor attention
R1 [5]: Construct EventEnvelope exclusively through
  EventEnvelope::new() in cherry-pit-core/src/envelope.rs
```

**4. One rule, one enforceable statement.**

Each rule should be a single testable constraint. If a rule
contains "and" joining two independent requirements, split it
into two rules. The agent can verify "did I follow R1?" more
reliably when R1 makes exactly one claim.

**5. Self-contained.**

The rule must make sense if the agent reads only this line with
no surrounding prose, no ADR title, no other rules. References
to "option 1" or "the approach above" are invisible to the agent.

**6. Brief rationale (optional).**

A short "because X" clause (≤10 words) can help the agent
generalize to edge cases. This is optional — if the rule is
self-evidently correct from its concrete content, skip the
rationale and save tokens. Full rationale belongs in Context.

```markdown
# Without rationale — clear from the types
R1 [5]: Use NonZeroU64 for EventEnvelope sequence field

# With rationale — helps edge-case generalization
R1 [5]: Use NonZeroU64 for EventEnvelope sequence field so
  zero sequences are rejected at the type level
```

#### Prose in Decision

Prose paragraphs, headings (### subsections), and code blocks in
the Decision section are for humans — they provide implementation
detail, design sketches, and rationale that doesn't fit in 60
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
that *would* be in a tagged rule's rationale clause if there
were room. The rule says "do X because Y" — Consequences
elaborates on what Y actually looks like.

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
Tier: B
Crates: cherry-pit-core, cherry-pit-gateway
Status: Accepted

## Related

References: CHE-0002, CHE-0010, CHE-0016, CHE-0033, CHE-0034, CHE-0039

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

R1 [5]: Construct EventEnvelope exclusively through
  EventEnvelope::new(), which validates non-nil event_id and
  returns Result<Self, EnvelopeError>
R2 [5]: Use NonZeroU64 for the EventEnvelope sequence field so
  zero sequences are rejected at the type level
R3 [5]: Access EventEnvelope fields through accessor methods
  (event_id(), aggregate_id(), sequence(), timestamp(), payload())
R4 [5]: Call EventEnvelope::validate() after deserialization in
  EventStore::load implementations to catch corrupt stored data

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

Running `adr-fmt --context cherry-pit-core` produces a tier-grouped
flat list. The worked example above renders as:

```
# Architecture Rules

These rules are mandatory constraints for all code in crate `cherry-pit-core`.
Follow every rule without exception.

## B-tier
- Construct EventEnvelope exclusively through EventEnvelope::new(), which validates non-nil event_id and returns Result<Self, EnvelopeError> [CHE-0042:R1:L5]
- Use NonZeroU64 for the EventEnvelope sequence field so zero sequences are rejected at the type level [CHE-0042:R2:L5]
- Access EventEnvelope fields through accessor methods (event_id(), aggregate_id(), sequence(), timestamp(), payload()) [CHE-0042:R3:L5]
- Call EventEnvelope::validate() after deserialization in EventStore::load implementations to catch corrupt stored data [CHE-0042:R4:L5]
```

This is all the agent receives. Notice:

- **Preamble** — imperative framing tells the agent these are
  mandatory constraints, not suggestions
- **Tier-grouped** — rules sorted by architectural significance
  (S→D), not by ADR. Eliminates per-ADR metadata noise
- **Rule ID at end** — `[CHE-0042:R1:L5]` anchors traceability
  without leading the attention. The action comes first. Layer
  suffix enables tension analysis in `--critique`.
- **Positive commission** — "construct through", "use", "access
  through", "call" — no prohibitions
- **Unconditional** — no "when X" qualifiers
- **Concrete** — `EventEnvelope::new()`, `NonZeroU64`,
  `validate()`, named accessor methods
- **Self-contained** — each rule makes sense without surrounding
  prose or other rules

---

## Migration Checklist for Existing ADRs

Existing ADRs using old format (`- **R1**: text`) must migrate to
the layer-annotated format (`R1 [N]: text`). To migrate:

1. Identify the core enforceable statements in the existing
   Decision prose — maximum 10 per ADR
2. Assign a Meadows leverage layer [1–12] to each rule based on
   the type of intervention it describes
3. Reframe any prohibitions as positive commissions
4. Eliminate any conditional framing ("when X, do Y")
5. Ensure every rule names at least one concrete type, method, or file
6. Distill each into a tagged rule (`R1 [N]: text`)
7. Run `cargo run -p adr-fmt -- --lint` — T016 should stop firing
8. Run `cargo run -p adr-fmt -- --context <CRATE>` to verify the
   extracted rules read well in isolation

**Prioritize migration by tier:** S-tier ADRs first (they appear
at the top of `--context` output and set the agent's foundational
constraints), then A, B, C, D.

---

## Rule-Writing Checklist

Before submitting a tagged rule, verify:

- [ ] **Positive commission?** Does it state what to do? (No
      prohibitions — reframe "never X" as "always Y")
- [ ] **Unconditional?** No "when X" or "if Y" qualifiers?
- [ ] **Concrete?** Does it name at least one type, method, file,
      or trait?
- [ ] **Self-contained?** Would the rule make sense if you saw only
      this line with no surrounding context?
- [ ] **One statement?** Does the rule make exactly one enforceable
      claim, not two joined by "and"?
- [ ] **7–60 words?** Within the T016 bounds?
- [ ] **≤10 rules per ADR?** If you need more, split the ADR or
      merge related constraints.
