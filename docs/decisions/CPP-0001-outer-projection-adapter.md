# CPP-0001. Persistent projection in an outer Pardosa adapter

Date: 2026-09-20
Last-reviewed: 2026-09-20
Tier: B
Status: Accepted

## Related

Root: CPP-0001

## Context

Source: Mattilsynet/gh-report at `c8507377b2748a015148751ce288be2bad9ec708`:
[CHE-0048](https://github.com/Mattilsynet/gh-report/blob/c8507377b2748a015148751ce288be2bad9ec708/docs/adr/cherry/CHE-0048-cherry-pit-projection-design.md),
[CHE-0084](https://github.com/Mattilsynet/gh-report/blob/c8507377b2748a015148751ce288be2bad9ec708/docs/adr/cherry/CHE-0084-extraction-eligibility-and-pardosa-adapter-placement.md),
[CHE-0097](https://github.com/Mattilsynet/gh-report/blob/c8507377b2748a015148751ce288be2bad9ec708/docs/adr/cherry/CHE-0097-projection-checkpoint-sequence-monotonicity.md).

The user authorized neutral producer extraction in gh-report bead `ghr-7wc6p.1`.
The source projection crate directly depended on Pardosa and re-exported its
persistent store. CHE-0048:R1/R2/R9/R10 explicitly locate that capability within
`cherry-pit-projection`; CHE-0084:R4–R8 require newly extracted Pardosa-dependent
mechanisms outside the Cherry DAG. Copying unchanged cannot satisfy both the
approved neutral boundary and the literal old location. Deleting persistence
would discard a first-class capability and its durability tests.

Placement was decided on 2026-09-20 by COMMANDER PLACEMENT DISPOSITION in
`ghr-7wc6p.1`, informed by oracle `ghr-et7r4` and parent `ghr-7wc6p`.
Decision acceptance does not certify implementation: producer verification
and mandatory Linus review remain pending.

## Decision

R1 [5]: Keep neutral drivers, in-memory projections, shared errors and checkpoint
  carriers in their source-derived neutral crates; the eight `cherry-pit-*` normal
  and build dependency closures MUST exclude Pardosa. No reverse adapter re-export.

R2 [5]: Place the source persistent implementation in
  `pardosa-cherry-pit-projection`, using Pardosa's public `store`/`prelude` facade.
  This package MUST NOT implement or expose substrate backend traits, generic
  backend parameters or ring internals. This changes CHE-0048's package location
  for this producer only; it does not amend the pinned gh-report source.
  Both outer packages are Pardosa-family adopters governed by source
  PGN-0008/PGN-0010 facade contracts. Cherry cohosts packaging and integration
  tests, not a third governance island. The substrate remains separately pinned.
  CHE-0048:R1/R2/R9/R10 contain the placement language; that source has no R11.

R3 [5]: Preserve both EPHEMERAL and PERSISTENT capabilities, snapshot-before-
  checkpoint ordering, aggregate/handler identity validation, checkpoint sequence
  non-regression, error categories, rebuild/replay contracts, single-writer scope
  and all transferred tests. Equal-sequence retry is allowed; a lower sequence
  must return terminal `CheckpointRegression` without writes. Serialization bounds
  remain at the persistent consumer, not on `DomainEvent` or neutral projection.

R4 [5]: Relocation MUST NOT invent a shared backend trait, add a third persistence
  backend, claim atomic snapshot/checkpoint commit, promise cross-process projection
  coordination, or silently repair source runtime behavior. Source-rule/code gaps
  are review findings, not grounds for documentation to claim stronger guarantees.
  The separately authorized checkpoint critical-section repair is approved in
  `ghr-co8tw`; relocation alone does not justify a runtime change.

R5 [5]: Dev/test edges may use the separate outer test-support package for pgno
  durability assertions. Independence is the normal/build closure contract; it is
  not an assertion that the workspace's complete test graph excludes Pardosa.
  This explicitly narrows source CHE-0084:R5's literal any-edge prohibition
  under the user-intent-derived commander disposition. The source rule does
  not already distinguish normal/build from dev edges.

R6 [5]: Before producer acceptance, re-establish CHE-0084:R9's build-time
  dependency tripwire for this placement. The destination mechanism is job
  `build-test-lint` in `.github/workflows/ci.yml`, step
  "deny async-trait and rearm pardosa-cherry-pit-projection DAG guard
  (gh-report CHE-0025:R1+R2 CHE-0084:R5+R9)", with local entry point
  `python3.12 -B tools/verify.py graph`. It requires the outer package and
  checks all eight neutral normal/build closures using locked offline Cargo
  metadata/tree. This is the destination mechanism amendment to
  source CHE-0084:R9; the immutable source decision is not rewritten.
  Verify resolved dependency closure and all moved unit, integration and doctest
  assertions; prove new/amended guards with plant/fail/revert/clean evidence.
  Local execution is admitted in `ghr-7wc6p.11.2`'s exact context only.
  Remote execution and final implementation acceptance remain unverified.

R7 [5]: Jobs `build-test-lint`, `non-exhaustive-check` and `supply-chain`
  MUST admit the recorded local or explicit GitHub-hosted Linux x86_64 context
  before Cargo fetch/execution: verify lock, direct compiler/Cargo and bundled
  loader SHA256 identities, reject redirected paths and unapproved Cargo configs,
  and use a fresh allowlisted environment with empty wrappers. Fetch MUST use
  `--locked` without build execution. Job `supply-chain` MUST verify pinned
  official audit/deny archive and regular-member lengths and SHA256 before
  exclusive installation, then recheck installed binary hashes before execution.
  `tools/verify.py intake|fetch|provision|supply-chain` enforce these requirements;
  step names and failure diagnostics MUST cite this rule. Ordinary OS loader
  trust remains; changed inputs require reassessment, not automatic approval.

This scoped admission statement was accepted under the user-directed
`ghr-miw1y` M2 repair on 2026-09-20, grounded in `ghr-7wc6p.11.2` and official
artifact evidence `ghr-02ld2`. It does not amend source RST-0004:R5, which
requires new-dependency justification rather than artifact admission. Final
implementation review and the producer PR's actual Linux run remain pending.

## Consequences

Neutral applications can compose projection drivers without a normal Pardosa
edge. Persistent adopters import `pardosa_cherry_pit_projection` explicitly and
retain the external Pardosa dependency. No compatibility re-export is added.
The source's common-internal-port wording and historical `FileProjectionStore`
name are not evidence of an implemented unified backend API; verify the actual
source-derived surface without constructing a new abstraction to match prose.

The full source rules remain linked, including CHE-0084:R9's 2026-09-05 amendment
that retired a vacuous old tripwire and required re-arming for the next adapter.
This record does not fabricate earlier destination acceptance or renumber source
ADRs. Final review must adjudicate source-code/contract discrepancies explicitly.
