# cherry-pit-gateway

Infrastructure implementations for cherry-pit: event stores.

Provides `MsgpackFileStore` — a file-based, MessagePack-serialized event store
with atomic writes, process-level fencing, and optimistic concurrency.

## Status

Implemented. `MsgpackFileStore` is the only event store implementation.

Part of the [cherry-pit](../../README.md) workspace.
