# COM-0004. Different Layer, Different Abstraction

Date: 2026-04-25
Last-reviewed: 2026-04-25
Tier: A

## Status

Accepted

## Related

- Depends on: COM-0002
- References: COM-0003

## Context

Ousterhout (Ch. 7, "Different Layer, Different Abstraction") observes
that well-designed systems have layers where each layer provides a
distinct abstraction. When adjacent layers have similar abstractions,
it signals a design problem: one of the layers is not adding value.

**Red flags:**

- **Pass-through methods** — a method that does nothing except
  forward its arguments to another method with a similar signature.
  The method exists but adds no abstraction. The caller could have
  called the lower layer directly.

- **Pass-through variables** — a variable passed through multiple
  layers of the call stack without being used in intermediate layers.
  Each intermediate layer pays interface complexity for a variable it
  does not use.

- **Signature mirroring** — when a method's parameter list mirrors
  the method it calls, the calling layer provides no additional
  abstraction. The question: "Why does this layer exist?"

Cherry-pit's store/bus layering demonstrates the principle:

- **Store layer** sees "event streams." An unknown aggregate is an
  empty stream. `load` returns `Vec::new()`. No `NotFound` concept.
- **Bus layer** sees "aggregate lifecycle." An empty stream before
  command dispatch means the aggregate was never created. The bus
  returns `DispatchError::AggregateNotFound`.

Each layer provides a different abstraction over the same underlying
reality. The store abstracts persistence; the bus abstracts dispatch
semantics.

## Decision

Adjacent layers in the architecture must provide distinct
abstractions. Each layer should add semantic value that justifies
its existence.

### Rules

1. **Pass-through methods are red flags.** If a method's
   implementation is `self.inner.same_method(same_args)`, question
   why the wrapping layer exists. Valid justifications: error
   translation, logging, authorization, caching. Invalid: "it might
   need logic later."

2. **Pass-through variables trigger refactoring.** If a parameter
   passes through 2+ layers unchanged, consider:
   - Context objects that bundle cross-cutting concerns
   - Moving the parameter to the layer that uses it
   - Eliminating the parameter entirely (COM-0003: can the lower
     layer compute it?)

3. **Each layer names its concepts differently.** If two adjacent
   layers use the same vocabulary for the same concept, they may be
   the same abstraction split artificially. The store layer says
   "event stream"; the bus layer says "aggregate." Different names
   reflect different abstractions.

4. **Layer elimination is valid.** If analysis shows a layer adds no
   abstraction, removing it is the correct response. Layers exist to
   serve the design, not the other way around.

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
