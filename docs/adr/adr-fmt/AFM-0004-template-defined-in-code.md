# AFM-0004. MADR Template Defined in Code Not as a File

Date: 2026-04-27
Last-reviewed: 2026-04-27
Tier: A

## Status

Accepted

## Related

- References: AFM-0001

## Context

The MADR (Markdown Any Decision Records) format originated as a
lightweight template: a markdown file with placeholder sections
that authors copy and fill in. The original MADR template by
Oliver Kopp includes optional sections like "Considered Options"
and "Pros and Cons" that projects adopt selectively.

Template-file governance has a structural weakness: the template
is a suggestion, not a specification. Nothing connects the template
file to the validation logic. A template can show "## Context" as
a required section, but only code can enforce that the section
exists, appears in the correct order, and contains sufficient
prose.

Three approaches to template definition exist:

1. **Template file** — a `.md` file with placeholders. Authors
   copy it. Validation is a separate concern with no guaranteed
   consistency. This is the traditional MADR approach.

2. **Schema file** — a JSON Schema or similar formal grammar
   defining the ADR structure. Validation is derived from the
   schema. The schema becomes the SSOT but requires a schema
   language that can express markdown structure — an awkward fit.

3. **Code-as-template** — the parser and validator *are* the
   template definition. The required sections, metadata fields,
   and structural rules exist as Rust types and match arms. The
   `--guidelines` flag generates human-readable documentation
   from the same code that performs validation.

## Decision

Define the MADR template entirely in Rust code. No standalone
template file exists. The parser module (`parser.rs`) defines
what constitutes a valid ADR, and the guidelines module
(`guidelines.rs`) generates human-readable documentation from the
same structural knowledge.

### Template Elements in Code

1. **Required metadata fields** — `Date`, `Last-reviewed`, `Tier`
   are extracted by the parser using line-prefix matching. Their
   presence is checked by template rules T001–T004.

2. **Required sections** — `Status`, `Related`, `Context`,
   `Decision`, `Consequences` are identified by H2 heading text.
   Section ordering is enforced by T014.

3. **Section content constraints** — minimum word counts (T015),
   code block length limits (T011), and retirement section
   requirements (S004) are expressed as rule functions operating
   on parsed `AdrRecord` structs.

4. **Status vocabulary** — the seven recognized status values
   (`Draft`, `Proposed`, `Accepted`, `Amended`, `Rejected`,
   `Deprecated`, `Superseded`) are variants of the `Status` enum.
   Unrecognized values trigger T006.

5. **Relationship vocabulary** — the three permitted verbs
   (`Root`, `References`, `Supersedes`) are variants of `RelVerb`.
   Legacy verbs trigger L006.

### Generated Documentation

`cargo run -p adr-fmt -- --guidelines` produces a complete,
authoritative ADR writing guide by querying the same code
structures that perform validation. This output replaces any
static template file — it is always consistent with the tool's
actual behavior.

## Consequences

- There is no template file to copy. Authors write ADRs from
  memory or by referencing `--guidelines` output. This trades
  copy-paste convenience for guaranteed consistency between
  documentation and enforcement.
- Adding a new required section or metadata field requires changes
  in three places: parser (extraction), rules (validation), and
  guidelines (documentation). All three live in the same crate,
  making inconsistency a compile-time or test-time failure rather
  than a drift-over-time problem.
- The MADR format is customized for this project's needs. The
  original MADR "Considered Options" and "Pros and Cons" sections
  are omitted — context and consequences sections serve the same
  purpose with less structural overhead.
- LLM agents authoring ADRs can invoke `--guidelines` to obtain
  the current template specification programmatically, rather than
  relying on a potentially stale template file in their context
  window.
