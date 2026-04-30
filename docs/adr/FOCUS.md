# Focus of Effort

Last-updated: 2026-04-30
Status: Active

## Period

Start: 2026-04-30
End: 2026-05-14
Label: 2026-W18 — Operationalize the GND Meta-Process

## Outcome

Convert the GND-0001..0008 mission-command primitives from accepted text
into mechanically-enforced practice. The period closes with FOCUS.md
linted by `adr-fmt`, at least one GND directive (GND-0005 observability
or GND-0006 backbriefing) integrated into the working loop with a
concrete code artefact, and the COM-0034..0037 Draft ADRs either
promoted to Accepted or explicitly rejected.

## In-Focus Directives

- **GND-0008** — Schwerpunkt is named, time-boxed, and consulted; this
  artefact's existence and lint coverage are the load-bearing test
- **GND-0005** — every directive names an observation mechanism; until
  this is mechanical the GND domain is aspirational
- **GND-0006** — backbriefing precedes action; the consultation point
  where focus is checked against proposed work
- **GND-0007** — lifecycle hygiene; the Draft-to-Accepted ratchet for
  COM-0034..0037 and AFM-0018..0019 is exactly this discipline
- **COM-0034** — ADR lifecycle feedback (Draft); operational signal
  triggering review is the COM-tier expression of GND-0007
- **COM-0035** — stabilization ratchet from signal to standard
  (Draft); names *how* directives graduate, which closes the GND-0007
  loop with a concrete promotion path
- **AFM-0019** — rule-enforcement evidence metadata (Draft); the
  AFM-tier expression of GND-0005 — every rule must name how it is
  observed

## Out-of-Focus Domains

- **`pardosa-genome` codec implementation** — scaffold-only state stays;
  no serializer / deserializer work this period despite GEN-0001..0034
  being fully specified
- **`pardosa` Phase 2+** — NATS persistence, KV lease, registry: not now
- **`cherry-pit-web`, `cherry-pit-projection`, `cherry-pit-agent`** —
  workspace comments remain commented; no crate creation
- **SEC and GEN domain ADR additions** — corpus is exhaustive enough;
  no new entries unless surfaced by GND directive work
- **New COM principles** — moratorium on COM-0038+; finish promoting or
  rejecting COM-0034..0037 first
- **CHE domain new entries** — no new framework ADRs while meta-process
  consolidates

## Notes

The dominant commit signal of the prior week pointed at "make adr-fmt
trustworthy" (Candidate A in the audit). This focus deliberately
overrides that signal because the GND domain birth on 2026-04-30
reframes the work: the question is no longer "is the tool good enough"
but "is the discipline the tool serves operational." Candidate A
collapses into Candidate C if the GND directives are real.

The hard sub-clause — **at least one GND directive integrated into the
working loop with a concrete code artefact** — exists to prevent this
period from becoming pure text production. The FOCUS.md lint checks
F001–F007 (in flight) qualify; a `--backbrief` mode in adr-fmt would
also qualify. Recursive ADR authoring without one such artefact does
not.

Pardosa distributed-systems ADRs (PAR-0017..0023) added 2026-04-30 are
out-of-focus *for implementation* this period but in-focus *as written
intent*: their existence does not signal pardosa-runtime work has
restarted.
