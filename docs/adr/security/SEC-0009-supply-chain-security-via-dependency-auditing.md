# SEC-0009. Supply Chain Security via Dependency Auditing

Date: 2026-04-28
Last-reviewed: 2026-04-28
Tier: B
Status: Proposed

## Related

References: SEC-0001, SEC-0002, SEC-0004, RST-0004

## Context

All SEC ADRs address application-level security; none cover the supply
chain. The xz-utils incident (CVE-2024-3094) demonstrates dependency
supply chain is the highest-impact attack vector. A compromised dependency
with `unsafe` undermines SEC-0004, SEC-0002, and SEC-0001. RST-0004
covers cargo-audit but no security ADR governs the threat model.

1. **Automated auditing** — `cargo-deny` + `cargo-audit` + pinned lock.
2. **First-party attestation** — `cargo-vet`. Higher assurance, higher
   burden.
3. **Status quo** — RST-0004 alone. Known advisories only.

Option 1 chosen; cargo-vet deferred until dependency count justifies it.

## Decision

Enforce automated dependency auditing in CI covering advisories,
license compliance, banned crates, and unsafe surface area.

R1 [5]: Run cargo-deny in CI checking advisories, licenses, bans,
  and duplicate dependency versions on every pull request
R2 [5]: Pin all dependencies via committed Cargo.lock to ensure
  reproducible builds and detect supply chain substitution
R3 [6]: Run cargo-geiger periodically to report transitive unsafe
  code surface area, flagged for review when new unsafe appears
R4 [5]: Dependency updates follow RST-0002 dedicated-PR discipline
  with cargo tree diff included in the PR description

## Consequences

- **Closes the infrastructure gap.** Application-level SEC rules now
  rest on a verified dependency foundation.
- **CI friction.** cargo-deny adds ~10s to CI. Acceptable given the
  threat surface.
- **False positives.** License detection and duplicate checks may
  flag legitimate dependencies, requiring deny.toml exceptions.
- **cargo-vet deferred.** First-party attestation is recommended
  when dependency count exceeds 50 direct dependencies.
