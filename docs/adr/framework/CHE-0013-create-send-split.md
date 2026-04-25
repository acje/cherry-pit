# CHE-0013. Create/Send Split in CommandGateway and EventStore

Date: 2026-04-24
Last-reviewed: 2026-04-25
Tier: B

## Status

Accepted

## Related

- Depends on: CHE-0011
- Informs: CHE-0019, CHE-0020, CHE-0023
- Referenced by: CHE-0005, CHE-0012, CHE-0020

## Context

Aggregate lifecycle has two distinct phases:

1. **Creation** — the aggregate doesn't exist yet, no ID is known.
   The store must assign an ID.
2. **Mutation** — the aggregate exists, the caller knows its ID.
   The store appends to the existing stream.

A unified method with `Option<AggregateId>` would be awkward: callers
must remember to pass `None` for creation, the store must branch
internally, and the return type must accommodate both the new ID (for
creation) and just the envelopes (for mutation).

## Decision

The API separates aggregate lifecycle into two distinct operations:

- `CommandGateway::create(cmd)` → creates a new aggregate, store
  assigns the ID. Returns `(AggregateId, Vec<EventEnvelope>)`.
- `CommandGateway::send(id, cmd)` → targets an existing aggregate
  by known ID. Returns `Vec<EventEnvelope>`.

`EventStore` mirrors this with `create(events)` and
`append(id, expected_sequence, events)`.

Asymmetries:
- `create` rejects empty events (an aggregate must have ≥1 event).
- `append` treats empty events as a no-op.
- `append` requires `expected_sequence` for optimistic concurrency.

## Consequences

- Aggregate lifecycle states are explicit: "not yet created" vs
  "already exists."
- `CreateResult` and `DispatchResult` are distinct types — callers
  get exactly the information relevant to each operation.
- No `delete`/`archive`/`tombstone` at the infrastructure level.
  Termination is modeled as a domain event (e.g. `OrderClosed`).
  The aggregate's `apply` tracks its own terminated state.
- No `load`/`query` on the Gateway — correct CQRS separation. Reads
  go through projections, not the command gateway.
- The naming asymmetry (`send` on Gateway vs `dispatch` on Bus) is
  intentional: Gateway is the external API, Bus is the internal
  mechanism.
