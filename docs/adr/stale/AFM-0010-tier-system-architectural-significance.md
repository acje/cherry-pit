# AFM-0010. Tier System for Architectural Significance Classification

Date: 2026-04-27
Last-reviewed: 2026-04-27
Tier: A
Status: Superseded by AFM-0011

## Related

References: AFM-0001

## Context

Not all architectural decisions carry equal weight. Treating all
ADRs as equally significant creates review effort misallocation
(equal time on foundational vs. implementation decisions) and
change impact opacity (no signal about propagation scope). The
S/A/B/C/D grading convention provides visually distinct,
naturally-ordered tiers assigned via descending blast-radius
questions from "rewrite the framework?" (S) down to "single crate
internals?" (D).

## Decision

Every ADR must be assigned a tier from S/A/B/C/D declared as a
metadata field and validated by T004.

- **R1**: Assign tiers using the descending question method: start
  at S, work down, first "yes" determines the tier
- **R2**: Higher tiers (S/A) require a superseding ADR for changes;
  lower tiers (B/C/D) permit amendment with a dated note
- **R3**: Tier appears in generated domain README tables and
  `--tree` output for significance-based filtering

## Consequences

Reviewers triage by tier: S-tier demands thorough review, D-tier
is lightweight. The descending question method provides a
reproducible protocol — two developers should arrive at the same
or adjacent tier. Tier assignment remains a judgment call;
disagreements reveal differing assumptions about blast radius and
are productive design discussions.

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
