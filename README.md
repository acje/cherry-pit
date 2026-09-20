# Cherry-pit

Typed building blocks for event-sourced Rust applications: synchronous domain
logic, asynchronous infrastructure ports, and explicit application composition.
This workspace is extracted from
[Mattilsynet/gh-report at c8507377b2748a015148751ce288be2bad9ec708](https://github.com/Mattilsynet/gh-report/tree/c8507377b2748a015148751ce288be2bad9ec708).

## Components

| Crate | Responsibility |
|---|---|
| `cherry-pit-core` | Aggregates, commands, events, typed ports, correlation, scheduling contracts |
| `cherry-pit-gateway` | Gateway recovery support; persistence adapters are supplied separately |
| `cherry-pit-storage` | Synchronous filesystem writes, run locks, snapshot signatures |
| `cherry-pit-wq` | In-process queues, worker pools and domain-neutral admission regulators |
| `cherry-pit-merger` | Generic read-side convergence |
| `cherry-pit-projection` | Neutral projection drivers, in-memory views and write-cell coordination |
| `cherry-pit-web` | HTTP/WebSocket serving and read-side transport adapters |
| `cherry-pit-app` | Explicit composition, policy dispatch, projections and scheduling |
| `pardosa-cherry-pit-projection` | Outer Pardosa-backed persistent projection adapter |
| `pardosa-cherry-pit-test-support` | Unpublished outer adapter for durable integration tests |

Applications own runtime construction, signal handling, domain policy and
infrastructure selection. Ports bind to one aggregate/event type through
associated types. `DomainEvent` requires `Clone + Send + Sync + 'static`;
serialization bounds belong to the consumers that serialize events.

The eight `cherry-pit-*` crates have a neutral normal/build dependency boundary.
Pardosa-dependent functionality lives in the separately named outer packages;
durable integration tests may depend on those packages. These are cohosted
Pardosa-family facade adopters under the explicit CPP-0001 placement decision.

Persistence precedes publication; publication is notification, not commit.
Cancellation after persistence does not imply rollback. The in-process queue
is not durable, the default dead-letter sink logs, and policy-output dispatch
is not an automatic retry engine. `MsgpackFileStore` is retired.

## Development

Rust **1.98.0**, edition **2024**, resolver **3**. Use the committed lockfile
and the root workspace lint configuration; pedantic is the standing bar.
[AGENTS.md](AGENTS.md) defines scoped local verification and the producer
acceptance gates. Scoped local dependency intake is recorded in `ghr-7wc6p.11.2`.
Producer acceptance still requires build, CI and mandatory pre-merge review;
existing workflow files are not
evidence that standalone CI is implemented or passing.

Read the crate READMEs for APIs and [governance](docs/governance.md) for
source-qualified decisions and the persistent-projection placement amendment.

## License

Licensed under either [Apache-2.0](LICENSE-APACHE) or [MIT](LICENSE-MIT), at
your option, as in the source workspace. The source notices are preserved.
