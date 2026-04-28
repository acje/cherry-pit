# COM-0004. Different Layer, Different Abstraction

Date: 2026-04-25
Last-reviewed: 2026-04-28
Tier: B
Status: Accepted

## Related

References: COM-0001, COM-0002, COM-0003

## Context

Ousterhout (Ch. 7, "Different Layer, Different Abstraction") observes that well-designed systems have layers where each provides a distinct abstraction. When adjacent layers have similar abstractions, one is not adding value. Red flags include pass-through methods that merely forward arguments, pass-through variables threaded through layers unused by intermediaries, and signature mirroring where a method's parameters mirror the method it calls.

Cherry-pit's store/bus layering demonstrates the principle. The store layer sees "event streams" — an unknown aggregate is an empty stream, so `load` returns `Vec::new()` with no `NotFound` concept. The bus layer sees "aggregate lifecycle" — an empty stream before command dispatch means the aggregate was never created, returning `DispatchError::AggregateNotFound`. Each layer provides a different abstraction over the same reality: the store abstracts persistence; the bus abstracts dispatch semantics.

## Decision

Adjacent layers in the architecture must provide distinct
abstractions. Each layer should add semantic value that justifies
its existence.

R1 [5]: Pass-through methods are red flags; a wrapping layer must add
  semantic value such as error translation, logging, or caching
R2 [5]: A parameter passing through two or more layers unchanged must
  be refactored into a context object, moved, or eliminated
R3 [5]: Adjacent layers must use different vocabulary for their
  concepts; identical names signal artificially split abstractions
R4 [6]: If analysis shows a layer adds no abstraction, removing the
  layer is the correct response

## Consequences

- The store/bus split is validated: `load` returning `Vec::new()` vs
  `DispatchError::AggregateNotFound` demonstrates distinct
  abstractions at each layer (CHE-0019).
- New layers proposed for the architecture must demonstrate what
  abstraction they add. "Separation of concerns" is not sufficient
  justification — the concern must be at a different abstraction
  level, not just a different location.
- Pass-through methods in PRs are flagged with a COM-0004 citation,
  requiring the author to justify the layer's existence or refactor.
- This principle creates tension with hexagonal architecture's port
  pattern (CHE-0004), where adapter methods may appear pass-through.
  The resolution: adapters translate between abstractions (domain
  types ↔ infrastructure types), which is a valid abstraction
  difference even when the method structure appears similar.
