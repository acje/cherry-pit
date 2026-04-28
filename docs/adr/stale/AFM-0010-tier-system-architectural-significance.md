# AFM-0010. Tier System for Architectural Significance Classification

Date: 2026-04-27
Last-reviewed: 2026-04-27
Tier: A

## Status

Superseded by AFM-0011

## Related

- References: AFM-0001

## Context

Not all architectural decisions carry equal weight. A decision
about the aggregate identity type (S-tier, framework-wide impact)
is categorically different from a decision about test fixture
organization (D-tier, single-crate impact). Treating all ADRs as
equally significant creates two problems:

1. **Review effort misallocation** — reviewers spend equal time
   on foundational decisions and implementation details. High-impact
   decisions deserve deeper scrutiny.

2. **Change impact opacity** — when proposing to amend an ADR,
   there is no signal about how widely the change would propagate.
   Changing an S-tier decision may require rewriting the framework;
   changing a D-tier decision affects one crate's internals.

The tier system draws from the S/A/B/C/D grading convention (common
in Japanese media rating systems and tier-list culture) rather than
numeric scales (1-5) or priority labels (Critical/High/Medium/Low).
Letter tiers have two advantages: they are visually distinct in
markdown (no ambiguity between "Priority 1" meaning highest or
lowest), and they create a natural ordering that developers
intuitively understand.

### Tier Definitions

- **S** — "If this changed, would we need to rewrite the
  framework?" Foundational invariants. Examples: aggregate design,
  serialization format, single-writer assumption.

- **A** — "If this changed, would trait signatures or type bounds
  change?" API-level decisions. Examples: error type strategy,
  zero-copy deserialization, event envelope structure.

- **B** — "If this changed, would call sites or runtime behavior
  change?" Behavioral decisions. Examples: argument parsing
  strategy, diagnostic format, compression algorithm.

- **C** — "If this changed, would only CI, lints, or test setup
  change?" Process decisions. Examples: test organization, lint
  configuration, documentation generation.

- **D** — "If this changed, would only one crate's internal
  implementation change?" Local decisions. Examples: internal data
  structure choice, private function organization, buffer sizing.

## Decision

Every ADR must be assigned a tier from S/A/B/C/D. The tier is
declared as a metadata field (`Tier: X`) and validated by T004.

### Assignment Protocol

Assign tiers using the descending question method: start at S and
work down. The first question answered "Yes" determines the tier.

1. Would changing this decision require rewriting the framework? → S
2. Would trait signatures or type bounds change? → A
3. Would call sites or runtime behavior change? → B
4. Would only CI, lints, or test setup change? → C
5. Would only one crate's internals change? → D

### Tier Stability Expectations

Higher tiers imply greater stability expectations:

- **S/A** — changes require a new ADR (superseding the original)
  and broad review. Amendment is discouraged.
- **B/C** — changes may use the Amended status with a dated note.
  Review scope matches the impact scope.
- **D** — changes are lightweight. Amendment or supersession are
  both acceptable.

### Tier in Generated Indexes

Domain README tables include the tier column, allowing developers
to sort and filter ADRs by architectural significance. The tier
also appears in `--report` output alongside each ADR.

## Consequences

- Reviewers can triage ADR reviews by tier: S-tier ADRs demand
  thorough review; D-tier ADRs can be reviewed quickly.
- The descending question method provides a reproducible assignment
  protocol. Two developers independently assigning a tier to the
  same ADR should arrive at the same or adjacent tier.
- Tier assignment is a judgment call, not a mechanical computation.
  Disagreements about tier assignment are productive design
  discussions — they reveal differing assumptions about a
  decision's blast radius.
- The five-tier scale is granular enough to distinguish meaningful
  impact levels without creating false precision. Three tiers
  (High/Medium/Low) would force framework-level and API-level
  decisions into the same bucket.
- All 10 AFM domain ADRs demonstrate the tier system in practice:
  S-tier for naming and vocabulary decisions that affect every ADR,
  A-tier for template and lifecycle decisions, B-tier for
  implementation strategy decisions.

## Retirement

Superseded by AFM-0011. The blast-radius framing ("If this changed,
would X change?") diverged from Meadows' leverage-point hierarchy in
two systematic ways: high-impact parameters were over-classified
(a timeout change triggered C-tier Feedbacks, but Meadows places
parameter values at the shallowest level), and information flows
were under-classified (CI gates landed in old C-tier, but Meadows
places information flows in the Design tier above Feedbacks).
AFM-0011 replaces blast-radius framing with system-characteristic
framing ("Does this decision define X?"), aligned with Abson et al.
(2017) and Meadows' twelve leverage points.
