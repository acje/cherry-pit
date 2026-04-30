# AFM-0011. Meadows-Aligned Tier Classification

Date: 2026-04-28
Last-reviewed: 2026-04-28
Tier: S
Status: Accepted

## Related

Supersedes: AFM-0010
References: AFM-0001, GND-0008

## Context

AFM-0010 introduced a five-tier system using blast-radius framing
("If this changed, would X change?"). This diverges from Meadows'
leverage-point hierarchy in two ways: over-classifying high-impact
parameters (timeout values triggered C-tier despite Meadows placing
parameters at the shallowest level) and under-classifying
information flows (CI gates landed in old C-tier despite Meadows
placing them at Design level 6). System-characteristic framing
("Does this decision define X?") corrects both misalignments while
splitting Self-organization from Design to prevent bucket overload.

## Decision

Classify ADRs by system characteristic using five tiers aligned
with Meadows' leverage-point hierarchy. Use system-characteristic
framing ("Does this decision define X?") instead of blast-radius
framing ("If this changed, would X change?").

R1 [5]: Classify ADRs using the first-yes-wins method: start at
  S-tier and assign the first tier whose classification question
  yields "yes"
R2 [5]: Frame tier classification questions as "Does this decision
  define X?" to classify by leverage type rather than blast radius
R3 [5]: Map tiers to Meadows' leverage hierarchy: S=Intent (levels
  1-3), A=Self-organization (level 4), B=Design (levels 5-6),
  C=Feedbacks (levels 7-8), D=Parameters (levels 9-12)

## Consequences

Information flows (CI gates, observability) now classify as B-tier
rather than old C-tier. Parameter values classify as D-tier
regardless of blast radius. All existing ADRs previously assigned
tier A, B, or C must be re-evaluated under system-characteristic
questions. The A-tier classification question references Rust
artifacts (traits, generics, plugin boundaries); extensibility
mechanisms not expressed through these may fall to B-tier — accepted
as rare edge cases.
