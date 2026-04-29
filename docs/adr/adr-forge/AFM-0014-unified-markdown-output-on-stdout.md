# AFM-0014. Unified Markdown Output on Stdout

Date: 2026-04-28
Last-reviewed: 2026-04-28
Tier: B
Status: Accepted

## Related

Supersedes: AFM-0007
References: AFM-0001, AFM-0003

## Context

AFM-0007 specified compiler-style diagnostics on stderr with stdout
reserved for guidelines and tree output. As adr-forge grew to six
output modes (lint, guidelines, tree, critique, context, default),
maintaining two output formats created inconsistency. All modes now
produce markdown-formatted output on stdout, enabling uniform
piping, redirection, and composition. The compiler-style
`print_diagnostic` function in `report.rs` is dead code.

## Decision

All output modes produce markdown on stdout. Diagnostics use inline
markdown formatting rather than compiler-style stderr.

R1 [5]: Every output mode (lint, guidelines, tree, critique,
  context) writes markdown-formatted text to stdout exclusively
R2 [5]: Diagnostic entries use the format
  `- **severity[RULE]** path:line: description` as markdown list
  items, not compiler-style plain text
R3 [5]: Diagnostics are sorted by file path then by line number
  for stable deterministic output regardless of traversal order
R4 [5]: Stderr is reserved for fatal errors and panics only; no
  structured diagnostic output appears on stderr

## Consequences

All adr-forge output is uniformly piped and redirected with standard
shell tools. Markdown formatting enables richer presentation (bold
severity tags, structured headers) compared to compiler-style plain
text. Rule IDs remain greppable within the markdown format. The
trade-off is that machine parsing requires markdown-aware tooling
rather than simple line splitting, acceptable for a developer-facing
tool.
