# GND-0001. Information Systems Operate Under Irreducible Uncertainty

Date: 2026-04-30
Last-reviewed: 2026-04-30
Tier: S
Status: Accepted

## Related

Root: GND-0001

## Context

Every information system is built and operated under conditions its
designers cannot fully observe and cannot reliably predict. Clausewitz
named this *friction*; Bungay (*The Art of Action*, 2011) generalised
it to organisational endeavour, identifying three persistent gaps:
between plans and outcomes (knowledge), between plans and actions
(alignment), and between actions and outcomes (effects).

Three options for foundational stance:

1. **Pursue certainty** — gather more information, write more detailed
   plans, impose tighter controls. Bungay shows this widens the gaps.
2. **Ignore the gaps** — treat each project as if uncertainty were
   absent. Surfaces as recurring surprise and rework.
3. **Name uncertainty as the design substrate** — treat the three
   gaps as constants every directive must respond to.

Option 3 chosen: the gaps are not a problem to solve but the medium
in which all subsequent decisions occur.

## Decision

Treat irreducible uncertainty as the substrate for all subsequent
GND principles. Every directive in any domain answers to one or more
of the knowledge, alignment, or effects gap.

R1 [2]: Frame every architectural decision as a response to one or
  more of three gaps — knowledge (plans vs outcomes), alignment
  (plans vs actions), effects (actions vs outcomes) — and name the
  gap in the ADR's Context section
R2 [2]: Treat additional information, additional instruction, and
  additional control as suspect responses to gap-induced surprise;
  prefer adjusting the directive's scope, intent, or feedback loop
R3 [3]: Adopt directed opportunism — high alignment around intent,
  high autonomy around action — as the operating model that any
  GND-derived mechanism instantiates

## Consequences

- **Establishes the GND vocabulary.** Subsequent GND ADRs reference
  this one as the source of "the three gaps." Domain ADRs (COM, RST,
  SEC, CHE) inherit the framing through their GND parents.
- **Reframes existing failures.** Recurring surprise, rework, and
  drift become evidence of gap-handling defects, not of weak
  individual decisions.
- **Constrains tooling.** Tools that pursue certainty (exhaustive
  specs, detailed step-by-step plans) are suspect; tools that close
  feedback loops (tests, telemetry, backbriefing) are favoured.
- **Cost.** Every ADR Context section gains a small framing burden:
  identify the gap. The cost is paid in clarity downstream.
