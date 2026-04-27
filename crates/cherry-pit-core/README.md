# cherry-pit-core

Foundational traits for cherry-pit: aggregates, commands, events, ports.

Every infrastructure port is bound to a single aggregate type via associated
types. The compiler enforces end-to-end type safety from command dispatch
through event persistence and publication.

## Status

Implemented. All domain and port traits are exported and stable.

Part of the [cherry-pit](../../README.md) workspace.
