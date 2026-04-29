# AFM-0008. Domain-Scoped Prefix Naming Convention

Date: 2026-04-27
Last-reviewed: 2026-04-27
Tier: S
Status: Accepted

## Related

References: AFM-0001

## Context

A growing ADR corpus requires a naming scheme providing global
uniqueness (unambiguous cross-domain references), domain affinity
(identifier reveals which domain without consulting an index), and
sortable ordering (filesystem sorting matches creation order). The
`PREFIX-NNNN` scheme satisfies all three: a 2–4 letter uppercase
domain code plus a zero-padded four-digit sequence number. Filename
extends this with a kebab-case slug for human-readable context in
directory listings and git logs.

## Decision

Every ADR filename follows `PREFIX-NNNN-kebab-slug.md` where PREFIX
is a configured domain code and NNNN is a zero-padded sequence
number.

R1 [5]: Filename matches `PREFIX-NNNN-kebab-slug.md` and H1 title
  contains the same `PREFIX-NNNN` identifier (N001, N002, N003)
R2 [5]: Domain prefixes are registered in `adr-forge.toml` under
  `[[domains]]`; unregistered prefixes trigger a warning (N004)
R3 [5]: Numbers are sequential within each domain, never reused;
  gaps from rejected or superseded ADRs are permitted
R4 [5]: Slugs must be lowercase kebab-case: letters, digits,
  hyphens only, no leading/trailing/consecutive hyphens

## Consequences

Cross-domain references are unambiguous (`References: GEN-0007`
identifies exactly one ADR). Directory listings sort chronologically
within each domain. Adding a new domain requires only a config
entry — no code changes. The 9,999 ADR-per-domain limit is
sufficient for any realistic project.
