# pardosa-genome-derive

Derive macro for the `GenomeSafe` trait.

Generates `SCHEMA_HASH` (xxHash64 fingerprint) and `SCHEMA_SOURCE` from
struct/enum declarations. Rejects serde attributes that would produce
non-deterministic or layout-incompatible serialization at compile time.

## Status

Implemented.

Part of the [cherry-pit](../../README.md) workspace.
