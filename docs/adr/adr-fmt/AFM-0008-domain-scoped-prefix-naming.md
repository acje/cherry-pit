# AFM-0008. Domain-Scoped Prefix Naming Convention

Date: 2026-04-27
Last-reviewed: 2026-04-27
Tier: S

## Status

Accepted

## Related

- References: AFM-0001

## Context

As an ADR corpus grows beyond a single directory, a naming scheme
must provide three properties simultaneously:

1. **Global uniqueness** — every ADR has an identifier that is
   unique across the entire workspace, not just within its domain.
   This enables unambiguous cross-domain references.

2. **Domain affinity** — the identifier reveals which domain an
   ADR belongs to without consulting an index. A developer reading
   a reference to `GEN-0007` immediately knows it is a Genome
   domain decision.

3. **Sortable ordering** — identifiers sort chronologically within
   a domain. Zero-padded four-digit numbers (`NNNN`) ensure that
   filesystem sorting matches creation order up to 9,999 ADRs per
   domain.

The `PREFIX-NNNN` scheme satisfies all three properties. The prefix
is a two-to-four letter uppercase code configured per domain. The
number is a zero-padded four-digit sequence. Combined, they form
an identifier like `CHE-0042` or `AFM-0001`.

### Filename Convention

The full filename extends the identifier with a kebab-case slug:

```
PREFIX-NNNN-kebab-case-slug.md
```

The slug provides human-readable context in directory listings and
git logs without requiring the file to be opened. It is validated
for lowercase kebab-case conformance (N003) and the number in the
filename must match the H1 title ID (N002).

### Alternatives Considered

- **Flat numbering** (`ADR-0001`, `ADR-0002`) — simple but loses
  domain affinity. Cross-references between 100+ ADRs become
  opaque without domain context.
- **Directory-only scoping** (`decisions/cherry/0001.md`) — the
  directory provides domain context but IDs are not globally unique.
  A reference to `0001` is ambiguous across domains.
- **UUID-based** — globally unique but unsortable, unreadable, and
  hostile to human memory.

## Decision

Every ADR filename follows the pattern `PREFIX-NNNN-kebab-slug.md`
where `PREFIX` is a configured domain code and `NNNN` is a
zero-padded sequence number. The H1 title must contain the same
`PREFIX-NNNN` identifier.

### Enforcement Rules

- **N001** — filename matches `PREFIX-NNNN-kebab-slug.md` pattern
- **N002** — number in filename matches H1 title ID
- **N003** — slug is valid lowercase kebab-case (letters, digits,
  hyphens; no leading/trailing hyphens, no consecutive hyphens)
- **N004** — domain prefix is found in `adr-fmt.toml` configuration

### Prefix Registration

Domain prefixes are registered in `adr-fmt.toml` under
`[[domains]]` entries. A file with an unregistered prefix triggers
N004. This prevents accidental typos from creating phantom domains.

### Number Allocation

Numbers are allocated sequentially within each domain. Gaps are
permitted (a rejected ADR may be moved to stale, leaving a gap).
Numbers are never reused — a superseded ADR's number is retired
permanently.

## Consequences

- Cross-domain references are unambiguous: `References: GEN-0007`
  can only refer to one ADR in the entire workspace.
- Directory listings sort ADRs chronologically within each domain
  without additional tooling.
- The three-to-four character prefix is short enough to type in
  references but long enough to be mnemonically distinct across
  domains.
- Adding a new domain requires only a `[[domains]]` entry in
  `adr-fmt.toml` — no code changes to `adr-fmt` itself.
- The 9,999 ADR limit per domain is sufficient for any realistic
  project. If exceeded, the zero-padding width would need to
  increase, requiring a migration.
