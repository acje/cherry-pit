#!/usr/bin/env python3
"""Suggest reference adds/removes based on fan-in mutuality and topical proximity.

Heuristic for ADD candidates:
  - X is in same domain as focal AND X is a domain root AND focal does not reference X.
  - X has higher tier (more foundational: lower TIER_RANK) AND focal references something
    that references X (transitive paradigm parent).

Heuristic for REMOVE candidates:
  - Reference is to a stale/missing ADR.
  - Reference is to a peer same-tier ADR that does NOT mutually reference focal AND
    focal is not in target's fan-in. (Weak link.)

This is advisory only — every suggestion needs human judgment.
"""

from __future__ import annotations

import sys
sys.path.insert(0, str(__import__("pathlib").Path(__file__).resolve().parent))

from rank_adr_refs import parse_adrs, is_domain_root, DOMAIN_ROOTS, FOUNDATION_PREFIXES, TIER_RANK


def main() -> None:
    adrs = parse_adrs()
    print("# Reference Add/Remove Candidates\n")
    print("Advisory only. Each suggestion requires human review.\n")
    print("---\n")

    # ADD candidates: missing own-domain root
    print("## Missing own-domain root reference\n")
    for adr_id, adr in sorted(adrs.items()):
        if adr.is_root or not adr.refs:
            continue
        roots = DOMAIN_ROOTS.get(adr.prefix, [])
        if isinstance(roots, str):
            roots = [roots]
        # CHE has dual roots; require at least one
        has_root = any(r in adr.refs for r in roots)
        if not has_root:
            print(f"- **{adr.id}** ({adr.tier}) — current refs: `{', '.join(adr.refs)}` — consider adding domain root `{roots[0]}`")
    print()

    # REMOVE candidates: refs to stale or missing ADRs
    print("## References to non-existent or stale ADRs\n")
    for adr_id, adr in sorted(adrs.items()):
        for r in adr.refs:
            if r not in adrs:
                print(f"- **{adr.id}** references `{r}` which is missing or stale")
    print()

    # Weak links: ref where target's fan-in does NOT include focal AND no mutual.
    # All References are by definition in fan-in, so this only catches dangling.
    # Instead: refs to a same-domain ADR that has no architectural connection signal
    # (peer same-tier, narrow target).
    print("## Potentially weak references (peer or narrower with no rule-level dependency)\n")
    print("_Heuristic flag — review the actual rule text before removing._\n")
    for adr_id, adr in sorted(adrs.items()):
        for r in adr.refs:
            ref = adrs.get(r)
            if ref is None:
                continue
            # Skip if ref is broader (lower tier rank) — those are definitionally relevant.
            if TIER_RANK[ref.tier] < TIER_RANK[adr.tier]:
                continue
            # Skip same tier: peer relations are common and meaningful.
            if TIER_RANK[ref.tier] == TIER_RANK[adr.tier]:
                continue
            # Ref is narrower (higher tier rank). Flag.
            # But only flag if focal does not appear in ref's fan-in (no reciprocal arch link).
            # Note: focal references ref by definition, so ref.fan_in already contains focal.
            # The question is: does the *narrower* target genuinely shape the broader focal?
            # Usually yes (the focal pre-dates and describes the abstraction), so just list as soft.
            print(f"- **{adr.id}** ({adr.tier}) → `{r}` ({ref.tier}) — narrower reference; verify {r} genuinely informs {adr.id} or move to fan-in only")


if __name__ == "__main__":
    main()
