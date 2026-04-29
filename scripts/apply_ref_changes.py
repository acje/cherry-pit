#!/usr/bin/env python3
"""Apply the reference-ordering changes from REFERENCE_ORDERING_REPORT.md.

Three change classes:
  1. REORDER  — replace `References:` line with re-sorted list (same set).
  2. ADD_ROOT — prepend domain root to `References:` list.
  3. DEMOTE   — remove a specific ref entirely.

Idempotent: reads the current line, computes target, writes only if different.
"""

from __future__ import annotations

import re
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
from rank_adr_refs import parse_adrs, rank_refs  # noqa: E402

ROOT = Path(__file__).resolve().parents[1]

# Manual change list — sourced from REFERENCE_ORDERING_REPORT.md Sections 2 & 3.

ADD_ROOTS: dict[str, str] = {
    "AFM-0006": "AFM-0001",
    "AFM-0012": "AFM-0001",
    "AFM-0014": "AFM-0001",
    "COM-0020": "COM-0001",
    "COM-0021": "COM-0001",
    "COM-0022": "COM-0001",
    "COM-0023": "COM-0001",
    "COM-0024": "COM-0001",
    "GEN-0034": "GEN-0001",
    "PAR-0006": "PAR-0001",
}

# Demotions: focal -> ref to remove
DEMOTE: dict[str, list[str]] = {
    "CHE-0040": ["CHE-0037"],
    "GEN-0009": ["GEN-0031"],
}


def apply_reference_line(path: Path, new_refs: list[str]) -> bool:
    """Replace the `References: ...` line in path with the new ordering.

    Preserves any `Root:` or `Supersedes:` segments on the same line.
    Returns True if file was modified.
    """
    text = path.read_text()
    pattern = re.compile(r"^(?P<prefix>(?:Root: [A-Z]{3}-\d{4} \| )?(?:Supersedes: [A-Z]{3}-\d{4}(?:, [A-Z]{3}-\d{4})* \| )?)References: (?P<refs>[A-Z]{3}-\d{4}(?:, [A-Z]{3}-\d{4})*)(?P<suffix>(?: \| Supersedes: [A-Z]{3}-\d{4}(?:, [A-Z]{3}-\d{4})*)?)$", re.M)
    m = pattern.search(text)
    if not m:
        # Fallback: maybe Supersedes is on its own line above References (typical pattern in this corpus)
        simple = re.compile(r"^References: (?P<refs>[A-Z]{3}-\d{4}(?:, [A-Z]{3}-\d{4})*)$", re.M)
        m2 = simple.search(text)
        if not m2:
            return False
        new_line = f"References: {', '.join(new_refs)}"
        new_text = text[:m2.start()] + new_line + text[m2.end():]
    else:
        new_line = f"{m.group('prefix')}References: {', '.join(new_refs)}{m.group('suffix')}"
        new_text = text[:m.start()] + new_line + text[m.end():]
    if new_text == text:
        return False
    path.write_text(new_text)
    return True


def main() -> None:
    adrs = parse_adrs()
    changes = []

    for adr_id, adr in sorted(adrs.items()):
        if adr.is_root or not adr.refs:
            continue
        current = list(adr.refs)
        new = list(current)

        # 1. Demotions
        if adr_id in DEMOTE:
            for r in DEMOTE[adr_id]:
                if r in new:
                    new.remove(r)

        # 2. Root additions
        if adr_id in ADD_ROOTS:
            root_id = ADD_ROOTS[adr_id]
            if root_id not in new:
                new.append(root_id)  # will be sorted to front by ranker

        # 3. Re-rank using the same algorithm as the report
        # Build a synthetic Adr with updated refs for ranking
        adr.refs = new
        scored = rank_refs(adr, adrs)
        ordered = [r for r, _ in scored]
        adr.refs = current  # restore for any later reads

        if ordered != current:
            changes.append((adr_id, current, ordered))

    print(f"Applying {len(changes)} ADR changes...\n")
    applied = 0
    for adr_id, current, ordered in changes:
        path = adrs[adr_id].path
        if apply_reference_line(path, ordered):
            applied += 1
            print(f"  ✓ {adr_id}: {', '.join(current)}  →  {', '.join(ordered)}")
        else:
            print(f"  ✗ {adr_id}: FAILED to rewrite {path}")
    print(f"\nApplied {applied}/{len(changes)} changes.")


if __name__ == "__main__":
    main()
