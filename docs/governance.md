# Source-qualified governance

Source authority is **Mattilsynet/gh-report** at
**`c8507377b2748a015148751ce288be2bad9ec708`**. Every `gh-report CHE-*`,
`RST-*` or `PGN-*` reference here means the document at that exact revision,
not a similarly numbered record from a prior Cherry repository.

[The source corpus](https://github.com/Mattilsynet/gh-report/tree/c8507377b2748a015148751ce288be2bad9ec708/docs/adr)
retains the full text, original dates, review notes, amendments, cross-references
and rejected alternatives. Local `docs/adr/cherry/` files are **link landings**
for imported source-relative README links, not copied/adopted ADRs with invented
acceptance dates. Follow the pinned source link to read the actual rules and
their scope. Bare references inside a source document resolve in that source
corpus. This keeps the complete meaning accessible without importing its whole
tree, stale archive, retired tooling, or application policies.

Local producer decisions use the separate `CPP` prefix in `docs/decisions/`.
`adr-fmt.toml` indexes only those decisions. It does not claim local closure of
the source ADR graph; source-context retrieval needs the pinned source corpus.
CPP-0001 records the 2026-09-20 commander-accepted placement decision from
`ghr-7wc6p.1` and oracle `ghr-et7r4`. Implementation acceptance still requires
verification and Linus review; decision status is not a test or CI verdict.

## Current rule owners

| Boundary | Exact source decision |
|---|---|
| Single-aggregate associated-type ports | [CHE-0005](adr/cherry/CHE-0005-single-aggregate-design.md) |
| Load-bearing event bounds; no serde supertrait | [CHE-0010](https://github.com/Mattilsynet/gh-report/blob/c8507377b2748a015148751ce288be2bad9ec708/docs/adr/cherry/CHE-0010-domain-event-supertrait-bounds.md) |
| Static policy output, explicit correlation, compensation | [CHE-0017](adr/cherry/CHE-0017-policy-output-static-type.md), [CHE-0039](adr/cherry/CHE-0039-correlation-context-propagation.md), [CHE-0040](adr/cherry/CHE-0040-saga-compensation-pattern.md) |
| Persist before publish, replay/checkpoint and dead-letter contract | [CHE-0024](adr/cherry/CHE-0024-event-delivery-model.md) |
| Dependency DAG and flat public API | [CHE-0029](adr/cherry/CHE-0029-cargo-workspace-crate-dag.md), [CHE-0030](adr/cherry/CHE-0030-flat-public-api.md) |
| Test kinds and type-contract coverage | [CHE-0038](adr/cherry/CHE-0038-testing-strategy.md) |
| Serialization owned at the consuming boundary | [CHE-0045](https://github.com/Mattilsynet/gh-report/blob/c8507377b2748a015148751ce288be2bad9ec708/docs/adr/cherry/CHE-0045-serialization-scope-per-crate.md) |
| Bounded retry, terminal errors, cancellation after commit | [CHE-0046](adr/cherry/CHE-0046-retry-timeout-cancellation-semantics.md) |
| Neutral and persistent projection capabilities | [CHE-0048](adr/cherry/CHE-0048-cherry-pit-projection-design.md), [CHE-0097](https://github.com/Mattilsynet/gh-report/blob/c8507377b2748a015148751ce288be2bad9ec708/docs/adr/cherry/CHE-0097-projection-checkpoint-sequence-monotonicity.md) |
| Explicit app composition and consumer-owned runtime | [CHE-0051](adr/cherry/CHE-0051-cherry-pit-agent-design.md) |
| Synchronous storage primitives and local run locks | [CHE-0053](adr/cherry/CHE-0053-cherry-pit-storage-design.md) |
| In-process queue, correlation, regulators and consumer policy | [CHE-0055](adr/cherry/CHE-0055-cherry-pit-wq.md) |
| Neutral extraction and outer Pardosa adapter | [CHE-0084](https://github.com/Mattilsynet/gh-report/blob/c8507377b2748a015148751ce288be2bad9ec708/docs/adr/cherry/CHE-0084-extraction-eligibility-and-pardosa-adapter-placement.md) |
| Pardosa adopter facade, operation-specific bounds and sealed backend boundary | [PGN-0008](https://github.com/Mattilsynet/gh-report/blob/c8507377b2748a015148751ce288be2bad9ec708/docs/adr/pardosa/PGN-0008-eventstore-facade-and-operation-specific-bounds.md), [PGN-0010](https://github.com/Mattilsynet/gh-report/blob/c8507377b2748a015148751ce288be2bad9ec708/docs/adr/pardosa/PGN-0010-backend-abstraction-and-nats-jetstream-constraints.md) |
| MessagePack store retirement and durability-test preservation | [CHE-0100](https://github.com/Mattilsynet/gh-report/blob/c8507377b2748a015148751ce288be2bad9ec708/docs/adr/cherry/CHE-0100-retire-gateway-msgpackfilestore.md) |
| Pinned toolchain/MSRV and committed lock | [RST-0001](https://github.com/Mattilsynet/gh-report/blob/c8507377b2748a015148751ce288be2bad9ec708/docs/adr/rust/RST-0001-pinned-stable-toolchain-with-msrv-contract.md) |
| Workspace lints, suppression reasons and dead-code rule | [RST-0003](https://github.com/Mattilsynet/gh-report/blob/c8507377b2748a015148751ce288be2bad9ec708/docs/adr/rust/RST-0003-workspace-lint-and-format-governance.md) |
| Dependency inventory, minimal features, audit/deny | [RST-0004](https://github.com/Mattilsynet/gh-report/blob/c8507377b2748a015148751ce288be2bad9ec708/docs/adr/rust/RST-0004-cargo-dependency-governance.md) |
| Workspace unsafe prohibition | [RST-0005](https://github.com/Mattilsynet/gh-report/blob/c8507377b2748a015148751ce288be2bad9ec708/docs/adr/rust/RST-0005-workspace-wide-forbid-unsafe-code.md) |
| Closed-error checker rule and its limited coverage | [RST-0006](https://github.com/Mattilsynet/gh-report/blob/c8507377b2748a015148751ce288be2bad9ec708/docs/adr/rust/RST-0006-non-exhaustive-error-enum-checker-crate.md) |
| Gate authorship, citations and weakening discipline | [RST-0007](https://github.com/Mattilsynet/gh-report/blob/c8507377b2748a015148751ce288be2bad9ec708/docs/adr/rust/RST-0007-merge-gate-governance.md) |

## Scope and amendment provenance

- **Projection placement:** [CPP-0001](decisions/CPP-0001-outer-projection-adapter.md)
  records the user-authorized packaging amendment to gh-report CHE-0048's
  crate-location language under CHE-0084. The persistent capability is retained;
  this does not assert that the original CHE-0048 already placed it outside.
  Cohosted outer packages remain Pardosa-family facade adopters with an external
  pinned substrate. CPP-0001 amends CHE-0084:R5's any-edge wording to normal/build
  independence while preserving dev-test durability bridges.
- **Serialization:** CHE-0010 was last reviewed **2026-09-20**. Its R1–R3 remove
  serde from `DomainEvent` when the last trait-level consumer was retired; each
  serializing consumer supplies its bounds. CHE-0045's older Pardosa/Genome
  table is not a mandate to recreate retired crates or formats.
- **Retirement:** CHE-0100's **2026-08-20** refinement preserves CHE-0032:R1–R3
  as the general atomic-write pattern, retires its MessagePack exemplar, and
  re-homes the surviving local TTL run-lock invariant to CHE-0053:R13. It does
  not turn the storage helper into a power-loss guarantee: CHE-0053:R6 explicitly
  says the helper does not fsync the parent directory. Durable tests stay pgno;
  trait-only tests may be in-memory. No restored MessagePack fixtures/backend.
- **Errors:** the source `AGENTS.md` closed-error rule and RST-0006:R1 reverse
  older open-error text. RST-0006 Context/Consequences and CHE-0097 Consequences
  still contain contrary `non_exhaustive` prose at this exact revision. Preserve
  that evidence; do not treat the older prose as permission to reopen errors or
  claim the source corpus is internally consistent. Checker coverage is limited
  to literal public `thiserror::Error` enums; macro/manual cases need review.
- **Dispatch:** CHE-0046:R7 explicitly scopes automatic gateway retry away from
  the asynchronous policy-output consumer: retryable policy errors are logged
  and dropped there. CHE-0051:R7's older retry-path sentence is not evidence that
  such a path is wired. App queue-full behavior and snapshot-only projection
  transport must not be described as lossless live delivery.
- **API examples:** source CHE-0029/0030/0038 and CHE-0053/0055 retain historical
  crate counts, MessagePack examples, dependency lists and future-tense prose.
  The current source code/manifests and later specific decisions control the
  extraction; those narratives do not reinstate deleted crates or missing APIs.
  Source CHE-0049/0050 are web-design/router decisions, not the deleted old
  destination proposals with colliding numbers.
- **Lint policy:** source Cargo/AGENTS specify current 1.98/pedantic policy.
  `docs/clippy/POLICY-1.98.md` is explicitly **DEFERRED WIP — NOT ACTIVE** at its
  head; its older enablement narrative is not an instruction to adopt it.

## Enforcement status

Local intake is complete only in `ghr-7wc6p.11.2`'s admitted context.
`ghr-co8tw` approves the focused checkpoint/timeout repair, not this producer.

Resource review I2: 262144 bytes limits admitted serialized snapshot length.
Serialization allocates a Vec before checking that limit; its allocation,
concurrent waiters and aggregate process memory are not bounded by it.
I1 remains a future construction-boundary gap: create/open accept arbitrary
projection-name strings; 128-byte validation occurs later during use.
No boundary-validated name type or before-create rejection is claimed.

The source's CI job names and pass reports describe **gh-report**, not this
producer. Applicable gate transfer, a locally runnable entry point, supply-chain
intake, resolved normal/build dependency closure, full test/fixture reconciliation
and mandatory Linus review remain acceptance work. CHE-0084:R9's re-arm obligation
is live. No existing workflow is certified by these documents.

All external links pin source files; network availability is not asserted.
The local source checkout can supply the identical text with
`git -C /path/to/gh-report show c8507377b2748a015148751ce288be2bad9ec708:<path>`.
