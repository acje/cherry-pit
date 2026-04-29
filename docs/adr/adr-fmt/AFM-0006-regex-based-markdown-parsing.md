# AFM-0006. Regex-Based Markdown Parsing Over AST Parsing

Date: 2026-04-27
Last-reviewed: 2026-04-28
Tier: D
Status: Accepted

## Related

References: AFM-0001, AFM-0004

## Context

`adr-fmt` must extract structural information from markdown: titles, metadata, section headings, relationships, and prose content. ADR files follow a constrained subset — ATX headings, fenced code blocks, `Key: Value` metadata. For this subset, a full AST parser provides unused capabilities while adding dependency weight.

## Decision

Parse ADR markdown using line-by-line iteration with compiled regex
patterns. Do not depend on any markdown AST parser.

R1 [9]: Single-pass line iteration with compiled regex patterns
  tracks current section context via an enum state machine
R2 [9]: Word counting accumulates per-section, excluding lines
  inside fenced code blocks by tracking fence open/close state
R3 [9]: Only ATX headings and fenced code blocks are supported;
  setext headings and indented code blocks are not recognized
R4 [12]: Reassess if ADRs require tables or structure complex
  enough that line-oriented parsing produces ambiguous results

## Consequences

The `regex` crate is the only parsing dependency. Parser behavior
is transparent: every extraction rule is a visible regex pattern.
ADR authors must follow the constrained markdown subset —
unrecognized headings produce missing-section warnings. Adding
support for new structural elements requires one regex pattern and
one extraction branch.
