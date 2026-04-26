# COM-0001. Complexity Budget — Strategic Investment

Date: 2026-04-25
Last-reviewed: 2026-04-25
Tier: S

## Status

Accepted

## Related

- —

## Context

Software complexity is the root cause of most development friction:
slow onboarding, fragile changes, cascading bugs, and mounting
technical debt. Ousterhout (Ch. 3, "Working Code Isn't Enough")
distinguishes two programming mindsets:

1. **Tactical programming** — get features working as quickly as
   possible. Complexity is a cost paid later. "I'll clean it up later"
   is the hallmark phrase. Each shortcut is small, but they accumulate
   into a system that resists change.

2. **Strategic programming** — working code is necessary but not
   sufficient. The primary goal is a great design that also happens
   to work. Every change is an opportunity to improve the system's
   structure. The investment is small (10–20% additional time per
   task) but compounds over the system's lifetime.

The cherry-pit project adopted strategic programming from inception:
92 ADRs were written before most code existed. Compile-fail tests
verify type contracts. The ADR system itself is a complexity
management tool — it forces decisions to be explicit, reasoned, and
reviewable.

The question is whether this investment principle should be formally
documented as an ADR, making it citable by all other decisions.

## Decision

Every design decision must justify its complexity cost against a
fixed budget. Zero tolerance for incremental complexity — no change
is too small to evaluate.

### Rules

1. **Strategic over tactical.** Invest 10–20% additional time per task
   in design quality. This is not optional overhead; it is the primary
   output.

2. **Complexity requires justification.** Before adding a new
   abstraction, type parameter, trait bound, error variant, or
   configuration option, demonstrate that the complexity is
   unavoidable. "It might be useful later" is not justification.

3. **Complexity budget is finite.** Each module, trait, and API surface
   has a complexity budget. Additions that exceed the budget require
   refactoring to make room — not expanding the budget.

4. **Red flags trigger review:**
   - "I'll clean it up later" — tactical shortcut
   - Interface mirrors implementation — shallow module (COM-0002)
   - Caller passes information the callee could compute — complexity
     pushed upward (COM-0003)
   - New error variant for a recoverable condition — error not
     defined out of existence (COM-0005)
   - Configuration parameter without a sensible default — complexity
     pushed to the user (COM-0003)

### Measurement

Complexity is assessed qualitatively through code review, not
quantitatively through metrics. The relevant question is: "Does a
developer reading this code for the first time need to understand
more concepts than the problem requires?"

## Consequences

- The ADR system exists as a direct consequence of this principle.
  Explicit architecture decisions reduce cognitive load for future
  contributors by documenting the "why" that source code cannot
  express.
- Compile-fail tests (CHE-0028, CHE-0038) are justified as
  complexity investments: they cost development time but eliminate
  entire classes of runtime errors.
- Every subsequent COM ADR is a specific application of the
  complexity budget: deep modules reduce interface complexity
  (COM-0002), pulling complexity down reduces caller complexity
  (COM-0003), error elimination reduces error-handling complexity
  (COM-0005).
- The 10–20% investment estimate is a guideline, not a hard metric.
  Some decisions (like the ADR system itself) require substantially
  more upfront investment but pay off across the entire project
  lifetime.
- Tactical programming is not forbidden in emergencies, but tactical
  debt must be tracked and repaid. The ADR system provides the
  tracking mechanism.
