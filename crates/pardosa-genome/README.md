# pardosa-genome

Binary serialization format with zero-copy reads and serde integration.

Combines FlatBuffers' zero-copy read performance with RON's full algebraic
data model. Standard serde with a lightweight `GenomeSafe` marker derive.

## Status

Scaffold. Traits (`GenomeSafe`, `GenomeOrd`), format constants, config types,
and error catalog are implemented. Serializer and deserializer are not yet built.

Part of the [cherry-pit](../../README.md) workspace.
