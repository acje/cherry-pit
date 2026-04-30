# AFM-0019. Rule Enforcement Evidence Metadata

Date: 2026-04-30
Last-reviewed: 2026-04-30
Tier: B
Status: Draft

## Related

References: AFM-0012, AFM-0014, COM-0017, GND-0005

## Context

AFM-0012 annotates each ADR rule with a Meadows leverage layer.
COM-0017 requires invariants to state their enforcement mechanism, but
`adr-fmt` does not expose or validate that information per rule. As the
corpus grows, rules without enforcement evidence become reviewer memory
rather than executable governance.

Three options:

1. **Keep enforcement in prose** — flexible, but invisible to
   `--context` and lint.
2. **Infer enforcement from wording** — convenient, but ambiguous and
   brittle.
3. **Add explicit metadata** — one controlled field per rule, surfaced
   by lint and context output.

Option 3 chosen: enforcement should travel with the rule that depends
on it.

## Decision

`adr-fmt` will support explicit per-rule enforcement metadata. Lint
warns when tagged rules omit enforcement evidence, and context output
shows the mechanism beside each extracted rule.

R1 [5]: `adr-fmt --lint` reports `T021` for each tagged ADR rule
  lacking an `Enforcement:` line
R2 [5]: `Enforcement:` values use the controlled vocabulary
  type-system, compile-fail, property-test, fuzz-test, golden-fixture,
  lint, CI, observability, or review
R3 [6]: `adr-fmt --context <CRATE>` prints each rule with its
  `Enforcement:` value beside the layer tag
R4 [5]: `docs/adr/TEMPLATE.md` includes `Enforcement:` immediately
  after each `RN [L]` rule block
R5 [6]: `warning[T021]` diagnostics name the ADR rule id and the
  missing or unknown enforcement value

## Consequences

- **Makes rules actionable.** Agents and reviewers see how each rule
  is enforced without rereading surrounding prose.
- **Adds migration load.** Existing ADRs lack `Enforcement:` fields;
  T021 may need staged rollout or draft-only incubation.
- **Pairs with context output.** AFM-0014 keeps diagnostics and
  context markdown-compatible while adding richer metadata.
- **Tightens governance.** Rules that cannot name an enforcement path
  become visible as review-only exceptions.
