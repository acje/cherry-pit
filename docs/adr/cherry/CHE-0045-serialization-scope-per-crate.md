# CHE-0045. Serialization Scope Per Crate

Date: 2026-04-25
Last-reviewed: 2026-04-25
Tier: B
Status: Accepted

## Related

References: CHE-0029

## Context

Cherry-pit is a multi-crate workspace (CHE-0029) spanning domain traits
(`cherry-pit-core`) and infrastructure adapters (`cherry-pit-gateway`).
The serialization decision in scope is:

1. **CHE-0031** — MessagePack named encoding for `cherry-pit-gateway`'s
   `MsgpackFileStore`. Optimised for forward-compatible event
   persistence with `#[serde(default)]` field evolution.

Without explicit scoping, a workspace with several adapters lets
format decisions leak across crate boundaries. In practice each
adapter serves different performance and compatibility
requirements, and external adapters (CHE-0029 R8) choose their own.

## Decision

Each crate owns its serialization strategy. No crate's choice
constrains another's.

R1 [5]: Each crate owns its serialization strategy independently; no
  crate may mandate a format for another crate
R2 [5]: Domain events are format-agnostic; the choice of serialization
  format is made at the infrastructure layer
R3 [5]: Feature flags gate serialization dependencies so users opt in
  explicitly

| Crate | Serialization | Governing ADR |
|-------|--------------|---------------|
| `cherry-pit-core` | None — domain traits are format-agnostic. `DomainEvent: Serialize + DeserializeOwned` enables any serde backend. | CHE-0010 |
| `cherry-pit-gateway` | MessagePack with named/map encoding (`rmp-serde`), unconditional today. Forward-compatible field evolution via `#[serde(default)]`. | CHE-0031 |
| `cherry-pit-web` (planned) | JSON via `serde_json` for HTTP API responses. Format determined by web conventions, not event storage. | — |

### Boundary Rules

1. **Domain events are format-agnostic.** A domain event type defined
   in user code works with any serde-compatible backend. The choice of
   encoding is made at the infrastructure layer,
   not the domain layer.
2. **No crate may mandate a serialization format for another crate.**
   `cherry-pit-gateway` mandates MessagePack only for its own
   `MsgpackFileStore`. External adapters (CHE-0029 R8) choose
   independently; alternative `EventStore` implementations coexist.
3. **Feature flags gate serialization dependencies.** R3 applies to
   every serialization dependency, unqualified: each sits behind its
   own feature so users opt in explicitly. Current implementation gap
   — `cherry-pit-gateway` depends on `rmp-serde` unconditionally and
   gates nothing. That is a known nonconformance with R3, recorded
   here rather than resolved by narrowing the rule.
4. **Event envelope wire format is store-specific.** The `EventEnvelope`
   schema (CHE-0016, CHE-0042) is serialised by the store implementation,
   not by the domain. Different stores may use different encodings for
   the same logical envelope.

## Consequences

- Adapter-scoped formats do not conflict — users choosing `cherry-pit-gateway` get MessagePack; external adapters bring their own encoding.
- Domain event portability — `cherry-pit-core` is format-agnostic, so moving to a different store encoding requires no domain code changes.
- Schema evolution for the gateway is additive field evolution with `#[serde(default)]` (CHE-0022); other stores may evolve differently.
- Multiple serialization strategies mean multiple sets of golden-file tests and encoding-specific bug surfaces.
