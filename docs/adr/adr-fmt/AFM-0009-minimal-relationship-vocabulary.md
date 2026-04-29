# AFM-0009. Minimal Relationship Vocabulary With Three Verbs

Date: 2026-04-27
Last-reviewed: 2026-04-27
Tier: S
Status: Accepted

## Related

References: AFM-0001

## Context

ADR relationship systems face a vocabulary explosion problem. Rich
vocabularies (Depends on, Extends, Illustrates, Contrasts with,
Scoped by) create semantic ambiguity (what distinguishes "Depends
on" from "References"?), validation complexity (each verb has
different consistency requirements), and visualization overhead
(8+ edge types produce unreadable graphs). Analysis of the
cherry-pit corpus revealed all meaningful relationships can be
expressed with three non-overlapping verbs: Root (tree root),
References (citation), and Supersedes (replacement).

## Decision

Restrict the relationship vocabulary to exactly three verbs. All
other verbs are legacy and produce a deprecation warning.

R1 [5]: Permit only Root, References, and Supersedes as
  relationship verbs; legacy verbs trigger a warning (L006)
R2 [5]: Root and References are mutually exclusive — an ADR is
  either a tree root or a branch, never both (L009)
R3 [5]: Supersedes requires the target ADR to carry
  `Superseded by PREFIX-NNNN` status (L003)
R4 [5]: Root target must match the ADR's own ID; validated by
  L008
R5 [5]: Multiple References are permitted to support
  cross-cutting concerns drawing on several prior decisions

## Consequences

The relationship graph has exactly three edge types, making
visualization straightforward. Authors never debate verb choice for
citations — it is always References. The tree structure is
well-defined: every non-root ADR belongs to exactly one tree.
Orphan ADRs with no relationships trigger T007. Adding a fourth
verb requires demonstrating a semantic distinction that References
cannot capture.
