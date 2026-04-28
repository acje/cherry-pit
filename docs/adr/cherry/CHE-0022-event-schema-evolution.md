# CHE-0022. Event Schema Evolution Strategy

Date: 2026-04-25
Last-reviewed: 2026-04-25
Tier: B
Status: Accepted

## Related

- References: CHE-0009, CHE-0010, CHE-0021, CHE-0031, GEN-0002

## Context

Events are immutable facts persisted forever. Event enums grow as
domain models evolve. Envelope-level forward compatibility is handled
by named MessagePack encoding with `#[serde(default)]` (CHE-0031).
Domain event evolution — adding/removing enum variants and struct
fields — has no framework-level support. Pardosa migration is planned
but unbuilt.

## Decision

Additive-only event evolution:

1. **New enum variants**: allowed. Adding a variant is intentionally
   a compile-breaking change — all `apply` implementations must be
   updated. This is correct: CHE-0009 (infallible apply) requires
   total event handling.
2. **Removing variants**: forbidden. Persisted events are immutable.
3. **New fields on existing variants**: must be `Option<T>` with
   `#[serde(default)]` so historical events deserialize correctly
   (consistent with CHE-0031's named encoding).
4. **`event_type()` strings**: immutable once events exist in a log
   (per CHE-0010). Renaming breaks deserialization.
5. **`#[non_exhaustive]`**: NOT recommended on domain event enums.
   Unlike error types (CHE-0021), events require exhaustive matching
   in `apply` to maintain `state = f(events)`.
6. **Structural migration**: deferred to Pardosa (log-to-log rewrite
   with upcasters).

## Consequences

- Adding a variant forces compile-time updates to every `apply` — no
  silent ignoring of new events.
- Field evolution on existing variants is constrained to optional
  additions — required fields break deserialization.
- No runtime migration mechanism exists until Pardosa is built.
- Removing or renaming events requires a full Pardosa log migration.
- **Roll-forward only** — schema rollback is not supported. If a
  deployment introduces a new event variant and writes events with
  that variant, rolling back to the previous code version will cause
  `StoreError::Infrastructure` on `load` for any aggregate containing
  the unrecognized variant. The aggregate becomes unloadable until
  code rolls forward to a version that recognizes all persisted
  variants. This is intentional: silent data loss from ignoring
  unknown events is worse than a loud failure.
- **Golden-file serde regression** — a golden-file test in cherry-pit-core
  (CHE-0038) catches accidental serialization format changes from
  dependency updates (e.g., jiff, rmp-serde, uuid). The test
  serializes a deterministic `EventEnvelope` and compares against a
  committed fixture file byte-for-byte.
