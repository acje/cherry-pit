# GND-0007. Directives Have Lifecycle; Supersession Is Mandatory Hygiene

Date: 2026-04-30
Last-reviewed: 2026-04-30
Tier: S
Status: Accepted

## Related

References: GND-0001

## Context

Research cited in COM-0034 shows 20–25% of architectural decisions go
stale within two months. A directive that no longer matches reality
becomes friction: it primes wrong action, contradicts current
practice, and eats attention budget every time the corpus is read.
Bungay (Truppenführung): *"if the mission is made redundant by events,
the decision must take this into account."* The corpus is a living
standing-orders book, not an archive.

Three options:

1. **No lifecycle** — directives accumulate. The corpus calcifies;
   readers cannot tell which decisions still hold.
2. **Time-based review only** — calendar cadence catches some drift,
   misses fast drift between cycles.
3. **Triggered lifecycle with mandatory supersession on extraction
   or contradiction** — operational signal, accepted contradicting
   directive, and principle-extraction at a higher level all
   schedule review and require explicit resolution.

Option 3 chosen: it pairs time-based hygiene with event-driven
freshness and forces explicit handling of generalisation.

## Decision

Every directive carries an explicit lifecycle. Supersession is
mandatory when a contradicting directive is accepted, when operational
evidence shows the directive no longer holds, or when a higher-level
principle is extracted that subsumes it. Stale directives are moved
to a terminal state; they do not silently coexist with active ones.

R1 [3]: Mark a directive *Superseded by* and move it to terminal
  state when a higher-level principle is extracted that subsumes
  its intent, when an accepted contradicting directive enters the
  corpus, or when GND-0005 observability shows the directive's
  intent no longer holds
R2 [3]: Re-point structural-parent references from a superseded
  directive to its successor in the same change set so dependent
  directives remain anchored and the corpus tells one truth
R3 [3]: Treat the deviation log from GND-0004 as a supersession
  input — a directive with three or more recorded deviations is
  scheduled for review and either reaffirmed, refined, or superseded
R4 [3]: Record review outcome — reaffirmed, refined, superseded,
  retired — in the directive's Last-reviewed entry with a one-line
  rationale so future readers see the decision's history

## Consequences

- **Operationalises hard supersession.** The cherry-pit decision
  to hard-supersede when generalising into GND is enacted by R1's
  *higher-level principle extracted* trigger.
- **Aligns with COM-0034.** COM-0034 implements lifecycle feedback
  in software-design vocabulary; GND-0007 is its principle. COM-0034's
  structural parent re-points to GND-0007 on adoption.
- **Cost.** Supersession is paperwork. Mitigation: the trigger names
  the question; review is cheap when the question is precise.
- **Replaces silent drift with explicit transition.** A superseded
  directive carries forward its history; the corpus tells the truth
  about what it currently believes.
- **Observation mechanism (per GND-0005).** Lint plus review-gate:
  `Last-reviewed` field aging beyond a tier-specific threshold
  flags the ADR for review; supersession is recorded in the
  `Supersedes:` field and surfaced by `adr-fmt`'s lifecycle checks
  so terminal-status ADRs land in the stale directory.
