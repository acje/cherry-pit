# CHE-0050. Delivery Commitment and Batch Atomicity Proposal

Date: 2026-09-19
Last-reviewed: 2026-09-19
Tier: C
Status: Proposed

## Related

References: CHE-0024, CHE-0004, COM-0025, GND-0001

## Context

CHE-0024 is the accepted delivery model and stays in force: persist before publish, non-fatal publication failure, checkpointed replay from EventStore::load. This document holds the amendments proposed during review of that model, which sharpen commit knowledge and batch guarantees. No adapter in this workspace distinguishes indeterminate commitment or declares batch atomicity as a capability, so these rules describe wanted future contracts rather than current behavior. They apply only if a later decision supersedes or amends CHE-0024.

## Decision

Proposed only. Nothing below is current authority for any crate.

R1 [7]: Committed-event notification would be published only after persistence reports known commitment, keeping commit knowledge separate from notification failure so failed delivery cannot reclassify committed events as rejected
R2 [7]: Replay recovery would qualify each adapter by its declared retention, durability, and stable-identity capabilities rather than assuming every source supports it
R3 [7]: Persistence ports would distinguish known committed, known not committed, and indeterminate outcomes, with domain rejection separate and indeterminate operations reconciled before retry or publication
R4 [7]: EventStore batch atomicity would be an explicit implemented capability; adapters without all-or-none batch persistence would reject unsupported operations before writing or expose a separate weaker contract

## Consequences

+ becomes easier: truthful commit and delivery reporting across adapters, and honest capability negotiation for replay.
− becomes harder: composition must supply recovery capabilities and a bounded retry or dead-letter policy, and every existing adapter needs an audited capability declaration.
risks/migration: qualify existing persistence promises before adding weaker adapters. No universal exactly-once claim follows from these rules, and adopting them requires amending CHE-0024 rather than silently reinterpreting it.
