# AFM-0012. Per-Rule Meadows Layer Annotation

Date: 2026-04-28
Last-reviewed: 2026-04-28
Tier: S
Status: Accepted

## Related

References: AFM-0001, AFM-0011

## Context

AFM-0011 established tier classification at the ADR level. Individual
rules within an ADR often target a different leverage layer than the
ADR's overall tier — an S-tier governance ADR might enforce via
layer 5 (structural, B-tier). The old format (`- **R1**: text`)
carried no per-rule metadata, making this invisible.

Options: (1) per-rule layer annotation `R1 [5]: text` — clean,
enables tension analysis; (2) separate metadata table — drift-prone;
(3) inherit ADR tier — lossy. Option 1 chosen: layer is a property
of the intervention, not the decision.

## Decision

Per-rule Meadows layer annotations classify each tagged rule by
the type of systemic intervention it represents, independent of
the ADR's overall tier.

R1 [5]: Tagged rules use `RN [L]: text` format where N is the
  sequential rule ID and L is the Meadows leverage layer (1-12)
R2 [5]: The parser regex `^R(\d+)\s*\[(\d+)\]:\s*(.+)` extracts
  rule ID, layer, and text in a single pass
R3 [5]: T016 validates layer range 1-12 for all statuses with no
  Draft or Proposed exemption from tagged rule requirements
R4 [7]: `--critique` computes tension as abs_diff between ADR tier
  rank and rule layer-derived tier rank, displaying non-zero
  distances as `Tension: RN (+D from X→Y)`
R5 [6]: `--context` renders rules with layer suffix in the global
  identifier format `[PREFIX-NNNN:RN:LN]`

## Consequences

- **Tension visibility.** S-tier ADRs with layer-5 rules surface
  tension=2 — governance enforced structurally. Expected, not defect.
- **No dual-format.** Old `- **R1**: text` no longer parses. All 12
  files migrated atomically.
- **Layer ≠ tier.** Layer = intervention type; tier = significance.
  Orthogonal classifications providing richer architectural insight.
- **R0 removed.** ADRs without tagged rules produce empty vec + T016
  warning.
