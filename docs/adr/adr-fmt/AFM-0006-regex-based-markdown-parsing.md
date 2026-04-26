# AFM-0006. Regex-Based Markdown Parsing Over AST Parsing

Date: 2026-04-27
Last-reviewed: 2026-04-27
Tier: B

## Status

Accepted

## Related

- References: AFM-0004

## Context

`adr-fmt` must parse markdown files to extract structural
information: H1 titles, metadata fields, H2 section headings,
status lines, relationship lists, code block boundaries, and
prose content. The parsing question is whether to use a full
markdown AST parser or line-oriented regex matching.

The Rust ecosystem offers mature markdown parsers:

- **pulldown-cmark** — the standard CommonMark parser, used by
  `rustdoc`. Produces a stream of events (heading start, text,
  code block start, etc.). Full CommonMark compliance including
  edge cases like setext headings, indented code blocks, and
  nested block quotes.

- **comrak** — a CommonMark + GFM parser with AST output.
  Supports tables, strikethrough, autolinks, and other GitHub
  Flavored Markdown extensions.

However, ADR files follow a constrained markdown subset:

- H1 and H2 headings are always ATX-style (`#` and `##`)
- Metadata fields are `Key: Value` lines before the first H2
- Relationship lines are `- Verb: TARGET-ID` list items
- Code blocks are fenced with triple backticks
- No nested block quotes, no HTML blocks, no setext headings
- No tables, footnotes, or other GFM extensions

For this constrained subset, a full AST parser provides
capabilities that are never exercised while adding dependency
weight and API complexity. Line-oriented regex matching directly
extracts the needed fields with simple, auditable patterns.

## Decision

Parse ADR markdown files using line-by-line iteration with
compiled regex patterns. Do not depend on `pulldown-cmark`,
`comrak`, or any markdown AST parser.

### Parsing Strategy

1. **Single-pass line iteration.** The parser reads all lines
   into memory and iterates once, tracking the current section
   context via an enum state machine.

2. **Compiled regex patterns.** All patterns are compiled once
   using `regex::Regex` and reused across files. Key patterns:
   - H1 title: `^# (PREFIX-NNNN)\. (.+)$`
   - Metadata: `^(Date|Last-reviewed|Tier): (.+)$`
   - H2 section: `^## (.+)$`
   - Relationship: `^- (Root|References|Supersedes): (.+)$`
   - Code fence: `` ^```  ``
   - Status line: specific match against known status values

3. **Section-aware accumulation.** Lines between H2 headings are
   accumulated into section content. Word counting excludes code
   blocks by tracking fence open/close state.

4. **No CommonMark edge cases.** Setext headings (`===` / `---`
   underlines), indented code blocks (4-space prefix), and
   reference-style links are not supported. ADR authors must use
   ATX headings and fenced code blocks exclusively. This is
   enforced by convention and documented in `--guidelines`.

### Reassessment Trigger

Revisit this decision if ADRs need to support tables (e.g., for
comparison matrices in "Considered Options" sections) or if
markdown structure becomes complex enough that line-oriented
parsing produces ambiguous results.

## Consequences

- The `regex` crate is the only parsing dependency. No markdown
  parser appears in the dependency tree.
- Parser behavior is transparent: every extraction rule is a
  visible regex pattern, not an opaque AST traversal. This makes
  the parser easy to debug and extend.
- ADR authors must follow the constrained markdown subset.
  Setext headings or indented code blocks would not be recognized.
  This is documented but not enforced — an unrecognized heading
  simply means the section is not extracted, producing a missing
  section warning from the template rules.
- Adding support for new structural elements (e.g., a new
  metadata field) requires adding one regex pattern and one
  extraction branch — typically fewer than 10 lines of code.
- The parser is approximately 750 lines of Rust with extensive
  inline tests. A pulldown-cmark equivalent would likely be
  shorter code but with less transparent behavior and heavier
  dependencies.
