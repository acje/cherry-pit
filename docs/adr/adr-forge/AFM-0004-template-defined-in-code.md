# AFM-0004. MADR Template Defined in Code Not as a File

Date: 2026-04-27
Last-reviewed: 2026-04-27
Tier: A
Status: Accepted

## Related

References: AFM-0001

## Context

Template-file governance has a structural weakness: the template
is a suggestion, not a specification. Nothing connects a template
file to validation logic. Three approaches exist: a markdown
template file (copy-paste, no enforcement), a JSON schema (awkward
fit for markdown structure), or code-as-template where the parser
and validator are the template definition and `--guidelines`
generates documentation from the same code that validates.

## Decision

Define the MADR template entirely in Rust code. No standalone
template file exists.

R1 [5]: The parser module defines valid ADR structure; required
  metadata, sections, and vocabularies are Rust types validated by
  rule functions
R2 [5]: `--guidelines` generates human-readable documentation
  from the same code structures that perform validation
R3 [5]: Adding a structural rule requires changes in parser,
  rules, and guidelines within the same crate — inconsistency
  becomes a compile-time or test-time failure
R4 [5]: The MADR format is customized for this project; original
  optional sections are omitted where Context and Consequences
  serve the same purpose

## Consequences

Authors write ADRs from memory or `--guidelines` output — no
template file to copy. Adding a required section or metadata field
requires three co-located code changes (parser, rules, guidelines).
LLM agents can invoke `--guidelines` to obtain the current template
specification programmatically rather than relying on a potentially
stale file.
