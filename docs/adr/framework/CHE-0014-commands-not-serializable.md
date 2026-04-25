# CHE-0014. Commands Not Serializable by Default

Date: 2026-04-24
Last-reviewed: 2026-04-25

## Status

Accepted

## Related

- Depends on: CHE-0004
- Referenced by: CHE-0008, CHE-0010

## Context

Commands are intent objects dispatched to aggregates. They may stay
within a single process (in-memory dispatch) or cross process
boundaries (via NATS/pardosa transport).

Options for the `Command` trait bounds:
1. **Full bounds** — `Serialize + DeserializeOwned + Debug + Clone +
   Send + Sync + 'static`. Every command pays for serialization
   overhead whether it needs it or not.
2. **Minimal marker** — `Send + Sync + 'static`. Users add serde,
   Debug, Clone derives only when needed.

Events must be serializable (they cross process boundaries via
persistence and transport). Commands do not share this requirement —
many commands are dispatched in-process and never serialized.

## Decision

The `Command` trait is a minimal marker: `Send + Sync + 'static`. No
`Serialize`, `Deserialize`, `Debug`, `Clone`, or `PartialEq` bounds.

Users add derives as needed:
- In-process commands: no serde overhead.
- Cross-process commands: add `#[derive(Serialize, Deserialize)]`.
- Debugging: add `#[derive(Debug)]` (most users will do this anyway).

## Consequences

- In-process command dispatch avoids serialization overhead entirely.
- No `Debug` requirement means commands can't be logged by the
  framework. Users must derive Debug per-command for diagnostic
  logging.
- No `Clone` requirement means the framework cannot implement
  automatic retry at the Gateway level (the command is moved into
  `handle` and consumed). Callers must reconstruct commands for retry.
- No command audit trail is built into the framework. Only events are
  persisted. Command logging requires user-level serialization.
- All behavior lives in `HandleCommand<C>`, not on the command trait
  itself — clean separation of data (command) from behavior (handler).
