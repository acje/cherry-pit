# cherry-pit-storage

Synchronous filesystem primitives — atomic writes, run-locks, and
content-addressable signatures — for cherry-pit consumers. Per CHE-0051:
imported into canonical cherry-pit from the evolved donor
`Mattilsynet/gh-report@c850737`, which in turn absorbed `error`, `fs`,
`lock`, and `signature` from the original `quics-memoization` crate.

**Status**: Implemented (v0.1). Public API surface is flat over private
modules (`error`, `fs`, `lock`, `signature`) per CHE-0051:R3 — see
`docs/adr/cherry/CHE-0051-cherry-pit-storage-canonical-import.md`
for the design and CHE-0051:R3 for the enumerated re-export set.

## Provenance and licensing

Originally authored by Anders Jensen (acje) and contributed via
`Mattilsynet/gh-report`, where it is offered under `Apache-2.0 OR MIT`.
Canonical cherry-pit takes the **MIT** arm of that existing dual grant;
no relicensing occurred. See the repository `LICENSE`.
