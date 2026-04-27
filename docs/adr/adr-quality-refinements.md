# ADR Quality Refinement Corpus

Date: 2026-04-27
Updated: 2026-04-27 — evaluation verdicts and corrections added
Source: Systematic review of ADRs against industry best practices
Provenance: Quality evaluation artifact (2026-04-27), not a formal ADR
Benchmark: Nygard (2011), Zimmermann (2023), Fowler (2026), MADR,
  Henderson canonical repository, Michael R. David exemplar ADRs

---

## Methodology

Every ADR in the corpus was read in full and evaluated against seven
quality criteria derived from the benchmark sources:

1. **Context forces in tension** — does the Context section present
   competing forces, not just background? (Nygard: "value-neutral,
   states facts")
2. **Alternatives analysis** — are ≥2 substantive alternatives compared,
   not dummy options? (Zimmermann: ≥2 options required)
3. **Consequences depth** — are both positive AND negative consequences
   stated? (Nygard: "ALL consequences")
4. **Proportionality** — is the ADR's depth proportional to its tier?
   S-tier decisions deserve more thorough treatment than D-tier.
5. **Conciseness** — is the ADR ≤2 pages for its tier? (Fowler: "brevity
   is the most important thing")
6. **Self-contained comprehension** — can a qualified reader understand
   the decision without consulting other documents?
7. **Decision clarity** — is the decision stated assertively with
   concrete, actionable rules?

## Corpus Quality Summary

The cherry-pit ADR corpus is **significantly above industry average**.
Across all ADRs:

- 100% template conformance (validated by `adr-fmt`)
- 100% have both positive and negative consequences
- 95%+ cross-reference related decisions correctly
- Consistent vocabulary and naming throughout
- Tier system provides useful significance classification

**Internal benchmarks** — these ADRs represent the corpus's best work
and should serve as the standard for future ADRs:

- PAR-0006 (Genome as Primary Serialization) — gold-standard
  alternatives analysis
- CHE-0042 (EventEnvelope Construction Invariants) — exemplary
  problem framing with three evaluated options
- CHE-0036 (File-Per-Stream Storage) — clear topology × persistence
  strategy comparison
- RST-0001 through RST-0004 — consistently well-written with
  forces in tension and practical rules
- GEN-0011 (Inline Verification Check Catalog) — thorough technical
  catalog with clear error semantics
- GEN-0032 (Canonical Encoding Contract) — effective consolidation
  ADR linking scattered invariants

---

## Proposed Refinements

Only ADRs where a **significantly** more clear, concise, or accurate
formulation is achievable are listed. Minor editorial improvements
(word choice, punctuation) are excluded.

### 1. CHE-0004 — EDA with DDD and Hexagonal Architecture

**Tier:** S (most foundational decision in the project)
**Current length:** 58 lines
**Issue:** Disproportionately thin for its tier and importance. The
Context section is 15 lines for a decision that determines the entire
framework's architecture. The alternatives analysis is one sentence:
"CRUD with change-data-capture was considered but rejected because it
loses intent (commands) and makes replay non-deterministic."

**Comparison to benchmark:** Michael R. David's ADR-0042 (Adopt Event
Sourcing for Order Management) — the gold-standard public ADR for this
decision — evaluates four options (Event Sourcing, CRUD+CDC, Bi-temporal
DB, State-based with audit) with a decision matrix and explicit decision
drivers. The cherry-pit ADR covers the same decision space but at ~25%
of the depth.

**Proposed refinement — Context section:**

Replace the current 15-line Context with a formulation that articulates
the forces in tension:

> Cherry-pit is a composable systems-kernel for agent-first building.
> Three forces are in tension:
>
> 1. **Audit completeness.** Agents require full, replayable history of
>    every state change. Audit is not a bolt-on; it is the primary data
>    model. Any architecture that does not store the complete causal
>    chain of state transitions fails this requirement.
>
> 2. **Domain model fidelity.** The system must preserve command intent
>    (what the user asked for) separately from state changes (what
>    happened). CRUD systems collapse intent into state mutations —
>    the "why" is lost, making replay non-deterministic and debugging
>    forensically impossible.
>
> 3. **Infrastructure decoupling.** The kernel handles "undifferentiated
>    heavy lifting" — persistence, transport, fan-out — so users focus
>    on domain logic. Domain code must not know about serialization
>    formats, database schemas, or message brokers.
>
> Four architectural approaches were evaluated:
>
> | Approach | Audit | Intent | Decoupling | Complexity |
> |----------|-------|--------|------------|------------|
> | EDA + Event Sourcing + DDD + Hex | Full | Preserved | Full | High |
> | CRUD + Change Data Capture | Partial | Lost | Partial | Medium |
> | Bi-temporal Database | Full | Partial | Low | High |
> | State-based + Audit Log | Partial | Partial | Partial | Low |
>
> CRUD+CDC loses command intent: a CDC record says "field X changed to Y"
> but not "user issued command Z that caused the change." Bi-temporal
> databases preserve temporal state but do not naturally decompose into
> aggregates and bounded contexts. State-based+audit-log duplicates data
> (state + log) with no guarantee of consistency between them.

**Proposed refinement — Consequences section:**

Add explicit negative consequences that the current version omits:

> - **Steep learning curve.** EDA + event sourcing + DDD is one of the
>   most conceptually demanding architectural patterns in software
>   engineering. Developers must internalize commands, events, aggregates,
>   projections, policies, and eventual consistency before being
>   productive. This is the primary adoption barrier.
> - **Eventual consistency is inherent.** Read models are projections
>   rebuilt from events. They are always eventually consistent with the
>   write model. Developers accustomed to strong consistency (read-after-
>   write) must adapt their mental model. No amount of framework design
>   can eliminate this fundamental property.
> - **Event schema is append-only forever.** Once an event type is
>   persisted, it cannot be removed or renamed without migration
>   infrastructure. The event log is an append-only ledger of the
>   system's entire history — every schema decision is permanent.

---

### 2. PAR-0001 — Fiber State Machine as Inspectable Data Table

**Tier:** B
**Current length:** 53 lines
**Issue:** Context section assumes the reader knows what "fibers" are
in the pardosa domain. Jumps directly into |S| × |A| cardinality
notation without problem framing. A qualified reader encountering this
ADR for the first time cannot understand the decision without consulting
pardosa-design.md.

**Comparison to benchmark:** Zimmermann's ADR quality criterion #6:
"self-contained comprehension." Nygard: Context should describe "forces
at play" so a future developer understands the decision.

**Proposed refinement — Context section opener (prepend):**

> Pardosa models each aggregate instance's lifecycle as a **fiber** — a
> named entity with an event history anchored to a specific position in
> the line (pardosa's append-only event log). Fibers transition through
> five states: Undefined → Defined → Active → Locked → Detached. Seven
> actions drive these transitions (Create, Advance, Lock, Rescue, Detach,
> Reattach, Drop).
>
> The lifecycle must be enforced at runtime: invalid transitions (e.g.,
> advancing a locked fiber) must be rejected with specific error messages.
> The design question is how to encode the transition function.

This prepend adds ~80 words but makes the ADR self-contained. The
existing technical content (transition table, DOT generation, exhaustive
test) follows naturally.

---

### 3. GEN-0001 — Serde-Native Serialization with GenomeSafe Marker Trait

**Tier:** S (Root ADR for the entire Genome domain)
**Current length:** 44 lines
**Issue:** As the root ADR for 33 downstream decisions, this is the
thinnest S-tier Root ADR in the corpus. The alternatives analysis is
two sentences. Compare against PAR-0006 (138 lines), which covers the
same decision space (choosing genome over alternatives) with detailed
per-library analysis.

**Comparison to benchmark:** PAR-0006 is the internal benchmark for
this exact topic. GEN-0001 should be at least as thorough as the ADR
that chose it.

**Proposed refinement — Context section:**

Expand with explicit forces in tension:

> pardosa-genome must integrate with an existing Rust codebase where
> every data type already derives `Serialize` and `Deserialize`. Two
> forces are in tension:
>
> 1. **Zero-copy read performance.** The event storage hot path
>    deserializes millions of events during replay. Formats that
>    allocate per-field (JSON, bincode, postcard) impose a throughput
>    ceiling. Zero-copy formats (rkyv, FlatBuffers) avoid this but
>    require their own trait hierarchies.
>
> 2. **Adoption friction.** Introducing a second derive ecosystem
>    (rkyv's `Archive + Serialize + Deserialize`, FlatBuffers' codegen)
>    doubles the maintenance surface. Every type change requires updates
>    to both the serde representation and the zero-copy representation.
>    Mirror types spread through the entire codebase.
>
> Three approaches were evaluated:
>
> | Approach | Zero-copy | Serde compat | Mirror types | Schema hash |
> |----------|-----------|-------------|--------------|-------------|
> | serde + custom binary format | Partial (str, bytes) | Full | None | Custom |
> | rkyv | Full (struct-level) | None | Required | None |
> | FlatBuffers + codegen | Full | None | Required | External |
>
> rkyv achieves full struct-level zero-copy but at the cost of a parallel
> type hierarchy (`ArchivedFoo` for every `Foo`) that spreads through the
> entire codebase. FlatBuffers requires external `.fbs` schema files and
> code generation. Both sever the serde ecosystem connection — types
> cannot be used with JSON, TOML, or any other serde format without
> maintaining two serialization implementations.

**Proposed refinement — add cross-reference to PAR-0006:**

PAR-0006 contains the detailed alternatives analysis that GEN-0001
should reference. Add to Related section:

> - References: PAR-0006

This creates the explicit link between the design rationale (GEN-0001)
and its most detailed justification (PAR-0006).

---

### 4. CHE-0005 — Single-Aggregate Design with Compile-Time Type Safety

**Tier:** S
**Current length:** 55 lines
**Issue:** This ADR documents one of the most consequential design
decisions in the framework (every infrastructure port is bound to a
single aggregate type via associated types), but the alternatives
analysis is two options in three lines. The consequences don't explore
the practical impact of "object safety is sacrificed" — a reader who
hasn't worked with Rust trait objects won't understand the cost.

**Proposed refinement — expand alternatives analysis:**

> Two design approaches were evaluated:
>
> **Option 1: Generic per-call (type-erased).**
> ```rust
> trait EventStore {
>     fn load<E: DomainEvent>(&self, id: AggregateId) -> Vec<EventEnvelope<E>>;
> }
> ```
> The store accepts any event type per call. Callers choose the type at
> each call site. Runtime type checking prevents cross-aggregate
> confusion, but errors are discovered at runtime (deserialization
> failure), not compile time. The store is object-safe
> (`Box<dyn EventStore>`), enabling runtime polymorphism.
>
> **Option 2: Single-aggregate binding (associated types).**
> ```rust
> trait EventStore {
>     type Event: DomainEvent;
>     fn load(&self, id: AggregateId) -> Vec<EventEnvelope<Self::Event>>;
> }
> ```
> Each store instance is locked to one aggregate/event type. Cross-
> aggregate confusion is a compile error, not a runtime error. The
> store is NOT object-safe — `Box<dyn EventStore>` is impossible because
> the associated type prevents type erasure.

**Proposed refinement — expand "object safety sacrificed" consequence:**

> - **Object safety is sacrificed.** `EventStore`, `EventBus`,
>   `CommandBus`, and `CommandGateway` cannot be used as trait objects
>   (`Box<dyn EventStore>`). This means:
>   - No heterogeneous collections of stores for different aggregate
>     types
>   - No runtime selection of store implementations based on
>     configuration
>   - Every aggregate type requires its own concrete store instance,
>     wired at compile time
>   - The `cherry-pit-agent` builder API must solve the wiring ergonomics
>     so users don't manually construct typed infrastructure stacks for
>     each aggregate

---

### 5. CHE-0024 — Event Delivery Model

**Tier:** B
**Current length:** 49 lines
**Issue:** This ADR covers a critical distributed systems topic (event
delivery guarantees) in fewer lines than most D-tier decisions. The
statement "No at-least-once delivery guarantee at the `EventBus` level"
is stated without adequate explanation of what this means in practice.
The alternatives (at-most-once, at-least-once, exactly-once) are not
compared.

**Comparison to benchmark:** Industry ADRs on event delivery typically
compare three guarantee levels with their costs and trade-offs. The
CrackingWalnuts reference notes: "At-least-once is the pragmatic
default for event sourcing because the event log IS the delivery
guarantee."

**Proposed refinement — expand Context with delivery guarantee
comparison:**

> Event delivery systems offer three guarantee levels, each with
> different costs:
>
> | Guarantee | Mechanism | Cost | Risk |
> |-----------|-----------|------|------|
> | At-most-once | Fire and forget | None | Lost events |
> | At-least-once | ACK + retry | Retry logic, dedup | Duplicate events |
> | Exactly-once | Distributed transaction | 2PC or idempotent consumers | Complexity, latency |
>
> For event-sourced systems, exactly-once is typically achieved by
> combining at-least-once delivery with idempotent consumers — the
> event store provides the deduplication mechanism (events have unique
> IDs and are immutable once stored).
>
> Cherry-pit's current position: the `EventStore` provides the
> persistence guarantee (events are durably stored). The `EventBus`
> provides best-effort notification. If notification fails, consumers
> catch up by replaying from the store. This is effectively at-least-
> once delivery at the system level, even though the bus itself
> provides no delivery guarantee.

---

### 6. GEN-0005 — Two-Pass Serialization Architecture

**Tier:** A
**Current length:** 47 lines
**Issue:** The three alternatives (AST builder, back-patching,
two-pass) are mentioned but not compared with trade-offs. An A-tier
decision on a novel serialization architecture deserves clearer
comparison.

**Proposed refinement — expand alternatives comparison in Context:**

> Three serialization strategies were evaluated:
>
> 1. **Intermediate AST.** Build a tree of nodes representing the
>    serialized structure, then flatten to bytes. Memory cost: O(n)
>    proportional to the data. Used by JSON serializers. Simple
>    implementation but high peak memory.
>
> 2. **Back-patching (FlatBuffers approach).** Write data in-order,
>    patching offsets retroactively as later data resolves positions.
>    Single pass, but complex cursor management and mutable offset
>    fixups. Requires mutable offsets — incompatible with streaming
>    writes.
>
> 3. **Two-pass (sizing then writing).** First pass computes exact
>    buffer size. Second pass writes with zero reallocation. Memory
>    cost: O(1) beyond the output buffer. Requires the input to be
>    traversed twice — safe for immutable serde `Serialize` impls.
>
> | Strategy | Passes | Peak memory | Complexity | Streaming |
> |----------|--------|-------------|------------|-----------|
> | AST | 1 | O(n) | Low | No |
> | Back-patching | 1 | O(1) | High | No |
> | Two-pass | 2 | O(1) | Medium | No |

---

### 7. CHE-0002 — Make Illegal States Unrepresentable

**Tier:** S
**Current length:** 50 lines
**Issue:** As an S-tier principle referenced by many downstream ADRs,
the Context section explains the principle qualitatively ("the compiler
rejects illegal states rather than runtime guards catching them") but
does not quantify WHY type-level invariants categorically outperform
runtime checks. Adding the enforcement cost asymmetry — runtime guards
are O(call sites), type-level invariants are O(1) — strengthens the
rationale for downstream ADRs that cite this principle.

**Proposed refinement — add quantitative enforcement asymmetry to
Context:**

> Two enforcement strategies exist for invariants:
>
> 1. **Runtime guards** — `assert!(id != 0)`, `if seq == 0 { return
>    Err(...) }`. Guards run on every invocation. They can be bypassed
>    by bugs, disabled in release builds (`debug_assert!`), or forgotten
>    in new code paths. The invariant holds only as long as every code
>    path checks it. Maintenance cost scales with call sites.
>
> 2. **Type-level encoding** — `AggregateId(NonZeroU64)`. The
>    invariant is enforced once, at construction time. Every subsequent
>    use of the value benefits without any runtime cost. New code paths
>    inherit the guarantee automatically — the type system carries it.
>    Maintenance cost is O(1).
>
> The asymmetry is fundamental: runtime guards are O(call sites),
> type-level invariants are O(1). As the codebase grows, runtime
> guards become increasingly likely to miss a path. Type-level
> invariants cannot be circumvented by any amount of code growth.

---

## ADRs Reviewed — No Significant Refinement Needed

The following ADRs were evaluated and found to meet or exceed the
quality benchmark for their tier. No significant improvement in
clarity, conciseness, or accuracy was identified.

### RST Domain (4/4 — no refinements)

- RST-0001: Excellent — clear forces in tension (lint drift, feature
  mismatch, irreproducible builds), concrete configuration, practical
  rules
- RST-0002: Excellent — three dimensions of change clearly articulated,
  specific operational rules
- RST-0003: Excellent — well-structured with configuration examples and
  CI gate specifics
- RST-0004: Excellent — COM-0016 principle concretized with cargo-
  specific tooling and deny.toml configuration

### COM Domain (17/17 — no refinements)

All COM ADRs are well-written design principle records with appropriate
source attribution (Ousterhout, Martin, Ford, Evans, Skelton, Read).
Each follows a consistent pattern: cite source → explain principle →
show cherry-pit application → state rules → acknowledge tension. The
COM domain serves as the internal gold standard for principle ADRs.

Note: Zimmermann would classify some COM ADRs (COM-0008 "Design It
Twice", COM-0010 "Code Should Be Obvious") as "Blueprint in Disguise"
— they read more like design guidelines than architectural decisions.
However, the project's GOVERNANCE.md explicitly designates COM as a
foundation domain for "cross-cutting software design principles," making
this a deliberate design choice, not an anti-pattern.

### AFM Domain (10/10 — no refinements)

The AFM ADRs are consistently excellent. Highlights:
- AFM-0001 (SSOT Architecture): Clear three-option comparison,
  well-defined layer responsibilities
- AFM-0009 (Minimal Relationship Vocabulary): Excellent analysis of
  vocabulary explosion problem, clean three-verb resolution
- AFM-0008 (Domain-Scoped Prefix Naming): Clear three-option comparison
  with global uniqueness, domain affinity, and sortable ordering

### PAR Domain (13/14 — 1 refinement: PAR-0001)

PAR-0004, PAR-0006, PAR-0007, PAR-0008, and PAR-0013 are particularly
strong. PAR-0006 is the corpus's best alternatives analysis.

### CHE Domain (27/30 — 3 refinements: CHE-0004, CHE-0005, CHE-0024)

CHE-0042, CHE-0036, CHE-0041, and CHE-0022 are particularly strong.
CHE-0001 (Design Priority Ordering) is a clean, decisive S-tier ADR.

### GEN Domain (31/33 — 2 refinements: GEN-0001, GEN-0005)

GEN-0032, GEN-0011, GEN-0012, and GEN-0004 are particularly strong.
The genome domain has the most technically precise ADRs in the corpus.

---

## Anti-Pattern Audit

Zimmermann identifies 11 ADR anti-patterns. The cherry-pit corpus was
evaluated against each:

| Anti-Pattern | Present? | Notes |
|-------------|----------|-------|
| Fairy Tale (only pros) | No | Every ADR lists negatives |
| Sales Pitch | No | Technical, factual tone throughout |
| Free Lunch Coupon (no costs) | No | Consequences are honest |
| Dummy Alternative | No | Alternatives are substantive when present |
| Sprint/Rush (one option) | Partial | 7 ADRs lack explicit alternatives (listed above) |
| Tunnel Vision (local only) | No | Cross-domain consequences acknowledged |
| Maze (off-topic) | No | ADRs stay focused |
| Blueprint in Disguise | Partial | Some COM ADRs — intentional per GOVERNANCE |
| Mega-ADR | No | Longest ADR is 199 lines (CHE-0042) |
| Novel/Epic | No | Tone is consistently technical |
| Magic Tricks (fabricated) | No | Context grounded in code and references |

---

## Summary Table

| # | ADR | Tier | Issue | Refinement Type |
|---|-----|------|-------|-----------------|
| 1 | CHE-0004 | S | Thin for most foundational decision | Expand Context forces + alternatives analysis |
| 2 | PAR-0001 | B | Missing problem framing for fibers | Prepend domain context for self-containment |
| 3 | GEN-0001 | S | Root ADR too brief; alternatives sparse | Expand Context + add PAR-0006 cross-reference |
| 4 | CHE-0005 | S | Alternatives and consequences underexplored | Expand with code examples and object-safety impact |
| 5 | CHE-0024 | B | Delivery guarantees not compared | Add guarantee-level comparison table |
| 6 | GEN-0005 | A | Alternatives not compared | Add strategy comparison table |
| 7 | CHE-0002 | S | Qualitative principle; missing quantitative rationale | Add enforcement cost asymmetry explanation |

7 of 108 ADRs (as of 2026-04-27) have significant refinement potential
(6.5%). 101 meet or exceed the quality benchmark for their tier.

---

## Evaluation Verdicts (2026-04-27)

Each refinement was evaluated by reading the target ADR in full,
comparing against internal benchmarks (PAR-0006, CHE-0042, CHE-0001),
and verifying proposed text against source code. The external benchmark
citation (Michael R. David ADR-0042) is unverifiable from repo contents
— all verdicts are based on internal evidence.

| # | ADR | Verdict | Notes |
|---|-----|---------|-------|
| 1 | CHE-0004 | **Genuine** | Most foundational decision at 58 lines; B-tier downstream ADRs are deeper. Four-option comparison table is standard taxonomy. Proposed negative consequences (learning curve, eventual consistency, append-only schema) are real costs. |
| 2 | PAR-0001 | **Genuine** | Opens with formal notation without defining domain terms. Violates Zimmermann criterion #6 (self-contained comprehension). 80-word prepend is proportionate fix. |
| 3 | GEN-0001 | **Genuine** | Root ADR for 33 decisions at 44 lines. PAR-0006 contains the detailed justification GEN-0001 should reference. Cross-reference alone justifies refinement. |
| 4 | CHE-0005 | **Genuine** | "Object safety is sacrificed" stated in one bullet without explaining downstream impact. Code examples make the tradeoff tangible for readers unfamiliar with Rust's trait object system. |
| 5 | CHE-0024 | **Genuine** | Event delivery guarantees are a standard distributed systems topic requiring explicit guarantee-level comparison. The three-level table (at-most/at-least/exactly-once) is expected by qualified readers. |
| 6 | GEN-0005 | **Genuine** | Alternatives mentioned but not compared. **Correction applied:** back-patching description changed from "write data in reverse order" to "write data in-order, patching offsets retroactively" — FlatBuffers patches positions, it does not reverse write order. |
| 7 | CHE-0002 | **Borderline** | Existing ADR already explains the principle qualitatively (lines 17–20). Proposed text adds quantitative rationale (O(call sites) vs O(1)) which is valuable, but the original "missing why" diagnosis was overstated. **Reframed** from "missing why" to "adding quantitative enforcement asymmetry." |

---

## Process Observation

The primary pattern in all 7 refinements is the same: **S-tier and
A-tier decisions that are proportionally thinner than B-tier and C-tier
decisions in the same corpus.** The project's strongest ADRs (PAR-0006,
CHE-0042, CHE-0036) are B-tier and D-tier — they demonstrate the depth
that the S-tier decisions should aspire to.

The RST and COM domains, despite being "only" C-tier and A-tier, are
consistently thorough. The genome domain is technically precise. The
gap is specifically in the CHE and GEN domains' highest-tier decisions,
which were likely among the first written and predate the quality
standard established by later ADRs.

Recommendation: revisit the 7 identified ADRs with the quality standard
demonstrated by PAR-0006 and CHE-0042 as the target.
