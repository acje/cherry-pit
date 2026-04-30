# GND-0006. Backbriefing Precedes Non-Trivial Action

Date: 2026-04-30
Last-reviewed: 2026-04-30
Tier: S
Status: Accepted

## Related

References: GND-0002, GND-0001

## Context

Bungay treats backbriefing as the load-bearing mechanism of mission
command: the executor restates the received intent, names the action
planned, and identifies the expected effects; the issuer confirms or
corrects *before* execution. This is the primary alignment-gap closer
under GND-0001, because it surfaces misunderstanding before action
ossifies into outcome. Without it, briefing is a one-way broadcast
and misalignment is discovered only through failed effects.

Three options:

1. **Brief only** — issuer states intent, executor proceeds. Misalignment
   surfaces post-hoc through failed effects.
2. **Prescribe action in the brief** — eliminates ambiguity by
   eliminating discretion. Forbidden by GND-0002.
3. **Brief, then backbrief** — executor's restatement of intent and
   plan returns to the issuer for confirmation before action begins.

Option 3 chosen: it preserves discretion (GND-0002) while closing
the alignment gap before action commits.

## Decision

Non-trivial action under any directive is preceded by an explicit
backbriefing exchange in which the executor states received intent,
proposed action, and expected effects, and the issuer confirms or
corrects. The exchange is recorded in a channel the directive's
owner can review.

R1 [3]: Conduct a backbriefing exchange before non-trivial work
  begins under any directive; the executor names received intent,
  proposed action, and expected observable effects, and the issuer
  confirms or corrects in writing, because alignment is cheaper to
  repair before action than after
R2 [3]: Define *non-trivial* per directive owner — typically work
  whose cost to undo exceeds the cost of the backbriefing exchange
  itself — and record the threshold in the directive
R3 [3]: Persist backbriefing exchanges in a channel linkable to the
  directive ID — pull-request description, ADR amendment proposal,
  or task ticket — so the trail survives session and personnel
  turnover
R4 [3]: Treat repeated backbriefing corrections against the same
  directive as evidence the directive's intent is unclear and
  trigger refinement under GND-0007

## Consequences

- **Highest-leverage feedback mechanism.** Bungay ranks backbriefing
  as the load-bearing piece of mission command; this ADR makes it
  load-bearing for any directive in any domain.
- **Cost is real.** Backbriefing adds a synchronisation point.
  Mitigation: the threshold for *non-trivial* is set per directive,
  not globally.
- **Adapts to coding agents.** A coding agent's plan-mode proposal
  is a backbriefing exchange; this ADR is the principle that
  formalises plan-mode as mandatory for non-trivial agent work.
- **Connects to GND-0004.** Backbriefing reveals intended deviations
  before they happen; GND-0004 governs deviations that emerge during
  execution.
- **Connects to GND-0008.** When focus of effort is named, the
  backbriefing exchange explicitly references the current Schwerpunkt
  and confirms the proposed action serves it.
