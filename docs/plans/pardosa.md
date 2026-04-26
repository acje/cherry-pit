# Pardosa: event serialization and transport

Pardosa is a novel serialization and transport system. Its best fit is
Event Carried State Transfer (ECST) patterns where events carry full state
and data deletion (e.g. GDPR) must be supported without breaking the
append-only invariant at read time.

## Properties

- **Serde-native** — events are Rust structs, serialized via serde
- **Dual backend** — append-only logs to local files or NATS/Jetstream
  streams
- **Schema evolution** — when event schemas change, Pardosa migrates from
  one append-only log to the next version of the log
- **Migration-time operations** — schema transformation and permanent
  message pruning happen during the migration phase, not at read time
- **Append-only invariant** — within a log version, data is only ever
  appended, never mutated

## Crates

- **`pardosa`** — serializer and transport layer (serde integration,
  NATS/Jetstream publishing and consuming)
- **`pardosa-genome`** — the append-only file format, log versioning,
  and migration engine
