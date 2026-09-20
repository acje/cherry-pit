# cherry-pit-gateway

Infrastructure helpers for cherry-pit port traits.

`StaleLockEvidence` and `stale_lock_evidence` capture filesystem metadata
for operator-side lock recovery. Event persistence is supplied externally
through `pardosa` or `pardosa-nats`.

## Operational recovery

See [RUNBOOKS.md](RUNBOOKS.md) for the stale-lock evidence procedure.

## Tests

The source-derived integration tests exercise event-store and projection
conformance through `pardosa-cherry-pit-test-support`, including persistence
across reopening a `.pgno` store. This is a development dependency, not a
production dependency of the gateway.

Part of the [cherry-pit](../../README.md) workspace.
