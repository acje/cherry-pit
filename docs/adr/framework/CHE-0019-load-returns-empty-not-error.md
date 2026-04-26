# CHE-0019. Load Returns Empty Vec, Not NotFound Error

Date: 2026-04-25
Last-reviewed: 2026-04-25
Tier: B

## Status

Accepted

Amended 2026-04-25 — added COM cross-reference

## Related

- Depends on: CHE-0013
- Illustrates: COM-0004, COM-0005

## Context

When `EventStore::load` is called for an aggregate that has never been
created, two semantics are possible:

1. **Return error** — `StoreError::NotFound` or
   `StoreError::AggregateNotFound`. The store treats a missing
   aggregate as an error condition. The bus and gateway must handle
   this error.
2. **Return empty vec** — the store sees an empty event stream. A
   never-created aggregate is indistinguishable from one with zero
   events. The bus decides whether "empty stream" is an error.

## Decision

`EventStore::load` returns `Ok(Vec::new())` for unknown aggregates.
`StoreError` has no `NotFound` variant. `AggregateNotFound` is a
`DispatchError` variant at the `CommandBus` level — the bus maps
"empty load before dispatch" to `AggregateNotFound`.

This creates a clean semantic boundary:

- **Store layer:** an aggregate is an event stream. An unknown
  aggregate is an empty stream. Not an error.
- **Bus layer:** dispatching a command to an empty stream means the
  aggregate was never created. This IS an error — the bus returns
  `DispatchError::AggregateNotFound`.

`EventStore::append` to a never-created aggregate (file does not
exist) is an error: `StoreError::Infrastructure`. The aggregate must
be created via `create()` first. This prevents bypassing the `create`
path, which assigns the aggregate ID and guarantees ≥1 event.

## Consequences

- The store trait is simpler — `load` has only two outcomes: events or
  infrastructure error. No ambiguity between "not found" and "exists
  but empty" (which cannot occur since `create` requires ≥1 event).
- The bus is responsible for the semantic decision. Different bus
  implementations could interpret empty streams differently (e.g.,
  a bus for eventually-consistent systems might allow commands to
  empty aggregates).
- HTTP adapters mapping errors to status codes handle `NotFound` at
  the `DispatchError` level (→ 404), not at the store level.
- `append` to a never-created aggregate returns
  `StoreError::Infrastructure` with a descriptive message. This
  enforces the `create`→`append` lifecycle at the store level.
