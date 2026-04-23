# Infrastructure crates

Adapted infrastructure components for cherry-pit integration. Components
are pulled in as narrow, purpose-built crates under the `pit-*` namespace.

## Components

- **Gateway** (`pit-gateway`) — Concrete implementations of all four
  pit-core port traits: CommandGateway, CommandBus (driving side);
  EventStore, EventBus (driven side). Primary adapter scaffolding for
  webhook listeners and REST API pollers. Interceptor and retry
  middleware. In-memory implementations for testing.
- **Web serving** (`pit-web`) — HTTP endpoint scaffolding for inbound
  webhooks and query-side API serving.
- **Projection** (`pit-projection`) — read model storage and query serving
- **Aggregation** (`pit-aggregation`) — data aggregation pipelines
