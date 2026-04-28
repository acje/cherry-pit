# AFM-0011. Meadows-Aligned Tier Classification

Date: 2026-04-28
Last-reviewed: 2026-04-28
Tier: S

## Status

Accepted

## Related

- Supersedes: AFM-0010
- References: AFM-0001

## Context

AFM-0010 introduced a five-tier (S/A/B/C/D) classification system
using blast-radius framing: "If this changed, would X change?" This
approach classifies decisions by *consequence* — how widely a
reversal would propagate. While intuitive, blast-radius framing
diverges from Donella Meadows' leverage-point hierarchy in two
systematic ways:

1. **Over-classifies high-impact parameters.** A timeout change
   triggers "would runtime behaviour change?" (C-tier Feedbacks),
   but Meadows places parameter values at the shallowest leverage
   level (9–12). The timeout *mechanism* is a feedback loop; the
   timeout *value* is a parameter.

2. **Under-classifies information flows.** CI gates and observability
   requirements landed in the old C-tier ("only CI, lints, or test
   setup"). Meadows places information flows at level 6 (Design
   tier) — higher leverage than feedback loops (levels 7–8). This
   was a Meadows inversion.

Meadows' twelve leverage points cluster into five distinct types when
mapped to software architecture (a grouping inspired by Abson et
al.'s system-characteristics taxonomy, adapted and split at the
self-organization boundary). Level 4 — the capacity to evolve new
structure — is separated from levels 5–6 — rules and information
flows governing existing structure — because they represent different
kinds of architectural decisions.

Three alternatives evaluated:

1. **System-characteristic framing, 5 tiers** — classify by what
   the decision *is* (paradigm, extension point, rule, feedback
   loop, parameter). Aligns with Meadows. Requires reframing all
   questions from "If this changed..." to "Does this define...".
2. **Blast-radius framing, 4 tiers** — collapse to four groupings
   (levels 1–3, 4–6, 7–8, 9–12), keep "If this changed..."
   questions. Fixes the Meadows inversion but overloads the Design
   tier (levels 4–6 in one bucket).
3. **Keep AFM-0010 as-is** — accept the Meadows misalignment.

Option 1 chosen: system-characteristic framing provides correct
leverage ordering, and the five-tier split gives Self-organization
its own tier, preventing the Design bucket from becoming a junk
drawer.

## Decision

Classify ADRs by system characteristic using five tiers aligned
with Meadows' leverage-point hierarchy. Use system-characteristic
framing ("Does this decision define X?") instead of blast-radius
framing ("If this changed, would X change?").

- **R1**: Classify ADRs using the first-yes-wins method: start at
  S-tier and assign the first tier whose classification question
  yields "yes"
- **R2**: Frame tier classification questions as "Does this decision
  define X?" to classify by leverage type rather than blast radius
- **R3**: Map tiers to Meadows' leverage hierarchy: S=Intent (levels
  1-3), A=Self-organization (level 4), B=Design (levels 5-6),
  C=Feedbacks (levels 7-8), D=Parameters (levels 9-12)

## Consequences

- **Correct Meadows alignment.** Information flows (CI gates,
  observability) now classify as B-tier (Design) rather than the
  old C-tier. Parameter values classify as D-tier regardless of
  blast radius.
- **Reclassification required.** All existing ADRs previously
  assigned tier A, B, or C must be re-evaluated under the new
  system-characteristic questions. Old A splits into new A
  (Self-organization) and B (Design). Old B becomes new C
  (Feedbacks). Old C splits into new B (architectural CI gates)
  and D (tooling configuration).
- **Known tradeoff at A-tier.** The A-tier classification question
  references concrete Rust artifacts (trait definitions, generics,
  plugin boundaries). Extensibility mechanisms not expressed through
  these artifacts (proc macros, feature flags) may fall to B-tier.
  Accepted: these are rare edge cases, and B-tier is still a
  reasonable classification.
- **System-characteristic framing is more abstract.** Blast-radius
  questions referenced observable artifacts (trait signatures, call
  sites, CI files). System-characteristic questions reference what
  the decision *is*, which requires more judgment. The first-yes-
  wins protocol and tier-assignment guidance in GOVERNANCE.md § 2
  mitigate this.
