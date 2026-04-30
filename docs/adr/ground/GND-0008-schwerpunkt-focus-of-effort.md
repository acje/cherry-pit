# GND-0008. Focus of Effort Is Named and Time-Boxed

Date: 2026-04-30
Last-reviewed: 2026-04-30
Tier: S
Status: Accepted

## Related

References: GND-0003, GND-0001

## Context

The corpus is uniform — every directive equally weighted on the page.
Reality is not — at any given period a small subset is load-bearing
for what the system is actively trying to achieve. The German term
*Schwerpunkt* names this focus of effort. Without it executors face
the corpus as undifferentiated checklist; with it they know which
decisions matter most *now*.

Three options:

1. **No focus signal** — every directive equally weighted always.
   Executors hedge across all of them or pick arbitrarily.
2. **Re-weight directives directly** — promote some, demote others,
   in the corpus itself. Confuses durable intent with transient
   priority.
3. **Maintain a separate, time-boxed focus pointer** — names the
   outcome, which directives are load-bearing for it, and what is
   explicitly out of focus; replaced when the period ends. Corpus
   stays intact; pointer changes.

Option 3 chosen: separates *what we believe* from *what we are
working on now*.

## Decision

A focus-of-effort artefact names the outcome the system is currently
concentrating effort on, the small set of directives load-bearing for
that outcome, and the domains explicitly out of focus for the period.
The artefact is time-boxed, replaceable without amending the corpus,
and consulted during backbriefing so executors confirm proposed action
against current focus.

R1 [3]: Maintain a focus-of-effort artefact that names the outcome,
  the load-bearing directives by ID with rationale, the domains
  explicitly out of focus, and the period covered
R2 [3]: Set the period short enough to remain assessable at its end
  and long enough to plausibly reach the outcome; replace the
  artefact explicitly when the period ends rather than silently
  extending
R3 [3]: Reference the current focus artefact during GND-0006
  backbriefing; the executor confirms the proposed action serves
  an in-focus directive or names why it does not
R4 [3]: Keep focus changes out of the directive corpus itself;
  amending an ADR to change focus weight conflates durable intent
  with transient priority

## Consequences

- **Resolves a long-standing anti-pattern.** Re-weighting ADRs to
  reflect current priorities pollutes durable intent with transient
  signal. The focus artefact carries the transient signal alone.
- **Closes the loop with GND-0006.** Backbriefing references current
  focus; work that touches out-of-focus domains is not forbidden but
  must be named as such, surfacing scope drift early.
- **Cost.** A new artefact to maintain. Mitigation: the artefact is
  short, time-boxed, and disposable by design.
- **Implementation latitude.** GND mandates the mechanism, not its
  form. A team may choose `FOCUS.md`, milestone labels, OKR
  pointers, or equivalent; the obligation is that *some* artefact
  exists and meets R1–R4.
