# AFM-0018. Sibling-as-Parent Heuristic for Tree Inversion Detection

Date: 2026-04-30
Last-reviewed: 2026-04-30
Tier: B
Status: Draft

## Related

References: AFM-0011, AFM-0012, AFM-0013

## Context

L016 catches parent-tier weaker than child-tier. A second shape
slips through: child and first-References parent share the *same*
non-S tier under the same domain, parent non-root — peer treated
as structural parent. Three of nine recent COM drafts exhibited it.

Corpus on 155 parented ADRs: naive same-tier rule fires on 29
(18.7%), too noisy. Layer-inversion refinement: 0 hits since
AFM-0012 aligns tier and layer. Reference-load refinement (child
has more `References:` than parent): 9 hits (5.8%) — recent drafts
plus Accepted candidates worth re-review.

Options: ship naive (noise floods); ship refined (residual false
positives on cross-cutting cases); time-bounded experiment behind
a flag (calibrate, then promote or retire on data). Option 3
chosen: a rule firing on Accepted ADRs needs calibration first.

## Decision

Add an experimental L020 diagnostic flagging same-tier-non-root
parent edges where the child has strictly more `References:`
targets than the parent. Gate behind `--lint-experimental` and
calibrate against a baseline of currently-known hits before any
promotion decision.

R1 [5]: Define L020 diagnostic in `crates/adr-fmt/src/rules/links.rs`
  alongside L016, firing when child tier equals parent tier, neither
  is S, both are in the same domain, parent is non-root, and child's
  `References:` target count exceeds parent's
R2 [5]: Add a `--lint-experimental` boolean flag to the clap-derived
  CLI in `crates/adr-fmt/src/main.rs` per AFM-0013; the flag gates
  L020 emission and remains off by default
R3 [6]: Record L020 hit triage in `crates/adr-fmt/docs/l020-evaluation.md`
  using rows of `date | adr-id | parent-id | verdict | reviewer | note`
  where verdict is one of `true-positive`, `false-positive`, or
  `borderline`; the file is the single source of calibration data
R4 [12]: Configure L020 promotion thresholds (false-positive ceiling,
  minimum hit count, evaluation window) in the `[[rules]]` section of
  `docs/adr/adr-fmt/adr-fmt.toml` so tuning does not require an ADR edit
R5 [6]: Triage the nine baseline hits identified during corpus
  measurement and record their verdicts in
  `crates/adr-fmt/docs/l020-evaluation.md` before `--lint-experimental`
  is enabled in any contributor workflow
R6 [5]: Decide promotion or retirement of L020 via a successor ADR
  that cites the configured thresholds and the recorded verdicts in
  `crates/adr-fmt/docs/l020-evaluation.md` as evidence

## Consequences

- **Pairs with L016.** L016 catches inversion across tier boundaries;
  L020 catches inversion within a tier.
- **Cost.** New flag widens CLI surface; AFM-0014 (unified output)
  still holds since `--lint-experimental` reuses stdout contract.
- **Self-applying.** This ADR cites AFM-0011 first to avoid the
  same inversion it detects.
- **Operational ownership.** The triage file requires a curator;
  domain owner per COM-0033 fits if accepted, otherwise the
  AFM domain owner.
- **Fallback.** If L020 retires, the recorded verdicts inform
  whether a TEMPLATE.md documentation intervention fits instead.
- **Reusable pattern.** The dedicated triage file alongside
  TOML-configured thresholds generalizes to future experimental
  rules without ADR churn for tuning.
