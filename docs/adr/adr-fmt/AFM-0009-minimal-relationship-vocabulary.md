# AFM-0009. Minimal Relationship Vocabulary With Three Verbs

Date: 2026-04-27
Last-reviewed: 2026-04-27
Tier: S

## Status

Accepted

## Related

- References: AFM-0001

## Context

ADR relationship systems face a vocabulary explosion problem. The
original MADR format suggests free-form relationships. Projects
that adopt rich relationship vocabularies (Depends on, Extends,
Illustrates, Contrasts with, Scoped by, Refines, Motivates,
Enables, Constrains) create a semantic graph that is difficult to
reason about, validate, and visualize.

Each additional relationship verb introduces:

- **Semantic ambiguity** — what is the precise difference between
  "Depends on" and "References"? Between "Extends" and "Refines"?
  Authors make inconsistent choices, degrading the graph's value.
- **Validation complexity** — each verb may have different
  consistency requirements. "Supersedes" implies bidirectional
  state changes; "Illustrates" does not. The validator must encode
  the semantics of every verb.
- **Visualization overhead** — graph renderings with 8+ edge types
  become unreadable. Color-coding and legend space dominate the
  visual field.

Analysis of the cherry-pit ADR corpus revealed that all meaningful
relationships can be expressed with three verbs that form a
complete, non-overlapping vocabulary:

1. **Root** — declares this ADR as the root of a decision tree.
   Self-referential (`Root: OWN-ID`). Exactly one ADR per tree
   is the root. All other ADRs in the tree reference it.

2. **References** — cites another ADR as related context. The
   referenced ADR informs or motivates this decision. This is the
   general-purpose "related to" verb that subsumes Depends on,
   Extends, Illustrates, and all other citation-style verbs.

3. **Supersedes** — this ADR replaces the target ADR. The target
   must have `Superseded by PREFIX-NNNN` status. This is the only
   verb with bidirectional consistency requirements.

### Tree Structure

The Root/References pair creates an implicit tree structure. Root
ADRs are tree roots. ADRs that reference a root (or reference
another ADR that references a root) form the tree's branches. The
`--report` flag computes and displays this tree by inverting the
forward links.

## Decision

Restrict the relationship vocabulary to exactly three verbs: Root,
References, and Supersedes. All other verbs are legacy and trigger
a warning (L006).

### Verb Semantics

| Verb | Direction | Target | Consistency Rule |
|------|-----------|--------|-----------------|
| Root | Self | Own ID | L008: target must match own ID |
| References | Forward | Any ID | L001: target must exist |
| Supersedes | Forward | Any ID | L003: target must have Superseded-by status |

### Constraints

- **Root and References are mutually exclusive.** An ADR is either
  a tree root or a tree branch. Root ADRs may not also have
  References relationships (L009). This prevents ambiguous tree
  membership.

- **Multiple References are permitted.** An ADR may reference
  several other ADRs. This supports cross-cutting concerns that
  draw on multiple prior decisions.

- **Supersedes implies lifecycle change.** Using the Supersedes
  verb requires that the target ADR has been moved to the stale
  directory with a Retirement section and Superseded-by status.

### Legacy Verbs

The following verbs were used in earlier iterations and are now
deprecated: Depends on, Extends, Illustrates, Contrasts with,
Scoped by, and their reverse forms. L006 warns when any legacy
verb is encountered. Migration path: replace with References.

## Consequences

- The relationship graph has exactly three edge types. Visualization
  is straightforward: Root edges define trees, References edges
  link between trees, Supersedes edges connect active and stale
  ADRs.
- Authors never debate which verb to use for a citation-style
  relationship — it is always References. The semantic distinction
  between "depends on" and "references" was never actionable.
- The tree structure computed by `--report` is well-defined: every
  non-root ADR belongs to exactly one tree (determined by following
  References edges to a Root). Orphan ADRs (no relationships)
  trigger T007.
- Adding a fourth verb in the future requires careful justification:
  it must express a semantic distinction that References cannot
  capture and that has concrete validation or visualization value.
