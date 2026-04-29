#!/usr/bin/env python3
"""Compute proposed `References:` orderings for every ADR.

Heuristic (broad → narrow):
  Bucket priority (lower = earlier in list):
    1. Foundation-domain root (COM-0001, RST-0001, SEC-0001)
    2. Foundation-domain non-root
    3. Same-domain root (e.g. CHE-0001 for CHE ADRs, GEN-0001 for GEN, etc.)
    4. Same-domain S-tier (broader paradigm than focal)
    5. Same-domain A-tier
    6. Same-domain B-tier
    7. Same-domain C-tier
    8. Same-domain D-tier
    9. Cross-domain non-foundation refs (placed after own-domain)

  Within a bucket: sort by tier-distance from focal descending (larger
  conceptual gap = broader relation), then ADR number ascending.

  Special: if focal IS a root, all refs sort by domain proximity then tier.
"""

from __future__ import annotations

import re
import subprocess
from collections import defaultdict
from dataclasses import dataclass
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
ADR_DIR = ROOT / "docs" / "adr"

FOUNDATION_PREFIXES = {"COM", "RST", "SEC"}
DOMAIN_ROOTS = {
    "COM": "COM-0001",
    "CHE": ["CHE-0001", "CHE-0004"],  # CHE has dual roots: priority + EDA
    "PAR": ["PAR-0001", "PAR-0004"],  # PAR has dual roots: fiber-state + single-writer
    "GEN": "GEN-0001",
    "RST": "RST-0001",
    "SEC": "SEC-0001",
    "AFM": "AFM-0001",
}

TIER_RANK = {"S": 0, "A": 1, "B": 2, "C": 3, "D": 4}


@dataclass
class Adr:
    id: str
    prefix: str
    tier: str
    path: Path
    refs: list[str]
    supersedes: list[str]
    is_root: bool
    fan_in: list[str]


def parse_adrs() -> dict[str, Adr]:
    adrs: dict[str, Adr] = {}
    for path in ADR_DIR.rglob("*.md"):
        if path.name in ("TEMPLATE.md", "README.md", "GOVERNANCE.md"):
            continue
        if "/stale/" in str(path):
            continue
        text = path.read_text()
        # Title line: # PREFIX-NNNN. ...
        m = re.search(r"^# ([A-Z]{3}-\d{4})\.", text, re.M)
        if not m:
            continue
        adr_id = m.group(1)
        prefix = adr_id.split("-")[0]
        tier_m = re.search(r"^Tier: ([SABCD])", text, re.M)
        tier = tier_m.group(1) if tier_m else "D"
        # Relationship line(s)
        refs: list[str] = []
        supersedes: list[str] = []
        is_root = False
        related_block = re.search(r"^## Related\s*\n(.+?)(?=\n##|\Z)", text, re.M | re.S)
        if related_block:
            for line in related_block.group(1).splitlines():
                line = line.strip()
                if not line:
                    continue
                # Multiple verbs can coexist on one line separated by `|`
                for piece in [p.strip() for p in line.split("|")]:
                    if piece.startswith("Root:"):
                        is_root = True
                    elif piece.startswith("References:"):
                        ids = re.findall(r"[A-Z]{3}-\d{4}", piece)
                        refs.extend(ids)
                    elif piece.startswith("Supersedes:"):
                        ids = re.findall(r"[A-Z]{3}-\d{4}", piece)
                        supersedes.extend(ids)
        adrs[adr_id] = Adr(
            id=adr_id,
            prefix=prefix,
            tier=tier,
            path=path,
            refs=refs,
            supersedes=supersedes,
            is_root=is_root,
            fan_in=[],
        )
    # Build fan-in
    for adr in adrs.values():
        for r in adr.refs:
            if r in adrs:
                adrs[r].fan_in.append(adr.id)
    return adrs


def is_domain_root(adr_id: str) -> bool:
    prefix = adr_id.split("-")[0]
    roots = DOMAIN_ROOTS.get(prefix)
    if isinstance(roots, list):
        return adr_id in roots
    return adr_id == roots


def bucket(focal: Adr, ref_id: str, ref: Adr | None) -> tuple[int, int, int]:
    """Return (bucket, tier_rank_secondary, num_tiebreak).

    Priority (broad → narrow):
      0  Own-domain root (most specific paradigm anchor for the focal)
      1  Foundation root (cross-cutting paradigm)
      2  Own-domain S-tier non-root (paradigm peers)
      3  Foundation S-tier non-root
      4  Cross-domain (non-foundation) S-tier
      5  Own-domain A-tier
      6  Foundation A-tier
      7  Cross-domain A-tier
      8  Own-domain B-tier
      9  Foundation B-tier
      10 Cross-domain B-tier
      11 Own-domain C-tier
      12 Foundation C-tier
      13 Cross-domain C-tier
      14 Own-domain D-tier
      15 Foundation D-tier
      16 Cross-domain D-tier
    """
    if ref is None:
        return (99, 0, int(ref_id.split("-")[1]))
    ref_prefix = ref.prefix
    same_domain = ref_prefix == focal.prefix
    is_foundation = ref_prefix in FOUNDATION_PREFIXES
    focal_is_foundation = focal.prefix in FOUNDATION_PREFIXES
    ref_is_root = is_domain_root(ref_id)
    ref_tier_rank = TIER_RANK[ref.tier]
    num = int(ref_id.split("-")[1])

    # Roots first (own then foundation)
    if ref_is_root and same_domain:
        return (0, ref_tier_rank, num)
    if ref_is_root and is_foundation and not focal_is_foundation:
        return (1, ref_tier_rank, num)
    # If focal is foundation, treat foundation refs as own-domain
    own_or_foundation_eq = same_domain or (focal_is_foundation and is_foundation)

    # Tier band: 3 sub-buckets per tier (own, foundation, cross-domain)
    # Tier base offset: S=2, A=5, B=8, C=11, D=14
    tier_base = 2 + ref_tier_rank * 3
    if own_or_foundation_eq:
        b = tier_base + 0
    elif is_foundation:
        b = tier_base + 1
    else:
        b = tier_base + 2

    return (b, ref_tier_rank, num)


def rank_refs(focal: Adr, adrs: dict[str, Adr]) -> list[tuple[str, tuple[int, int, int]]]:
    scored = []
    for r in focal.refs:
        ref = adrs.get(r)
        scored.append((r, bucket(focal, r, ref)))
    scored.sort(key=lambda x: x[1])
    return scored


def diff_summary(current: list[str], proposed: list[str]) -> str:
    if current == proposed:
        return "no change"
    return f"{', '.join(current)}  →  {', '.join(proposed)}"


def main() -> None:
    adrs = parse_adrs()
    # Group by domain
    by_domain: dict[str, list[Adr]] = defaultdict(list)
    for adr in adrs.values():
        by_domain[adr.prefix].append(adr)
    domain_order = ["COM", "RST", "SEC", "CHE", "PAR", "GEN", "AFM"]

    print("# ADR Reference-Ordering Optimization Report\n")
    print("Goal: rank `References:` broad → narrow so the first reference is the most significant relation, optimizing LLM `--context` consumption.\n")
    print("**Bucket priority (earlier = broader):**\n")
    print("0. Own-domain root  •  1. Foundation root  •  then by tier S→D, within each tier ordered: own-domain → foundation → cross-domain non-foundation.\n")
    print("Within bucket: tier ascending, then ADR number ascending.\n")
    print("Rationale: own-domain S-tier rules ARE the focal's paradigm; cross-domain foundation B-tier rules are tactical mechanisms — paradigm precedes mechanism.\n")
    print("---\n")

    total = 0
    changed = 0
    for prefix in domain_order:
        domain_adrs = sorted(by_domain.get(prefix, []), key=lambda a: a.id)
        if not domain_adrs:
            continue
        print(f"## Domain: {prefix}\n")
        print("| ADR | Tier | Current `References:` | Proposed | Δ |")
        print("|-----|------|------------------------|----------|---|")
        for adr in domain_adrs:
            total += 1
            if adr.is_root:
                print(f"| {adr.id} | {adr.tier} | _root_ | _root_ | — |")
                continue
            if not adr.refs:
                print(f"| {adr.id} | {adr.tier} | _none_ | _none_ | — |")
                continue
            scored = rank_refs(adr, adrs)
            proposed = [r for r, _ in scored]
            delta = "✓" if adr.refs == proposed else "**reorder**"
            if adr.refs != proposed:
                changed += 1
            cur = ", ".join(adr.refs)
            new = ", ".join(proposed)
            print(f"| {adr.id} | {adr.tier} | {cur} | {new} | {delta} |")
        print()
    print(f"\n**Summary:** {changed} of {total} ADRs reordered (root/empty excluded).\n")


if __name__ == "__main__":
    main()
