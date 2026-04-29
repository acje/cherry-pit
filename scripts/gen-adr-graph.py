#!/usr/bin/env python3
"""Generate Graphviz DOT graph of ADRs and their `References:` edges.

Source of truth: docs/adr/adr-fmt.toml (configured domains).
Excludes: docs/adr/stale, generated READMEs, TEMPLATE.md, GOVERNANCE.md.
Visual layout: explicit `Root:` ADRs at bottom; reference trees grow upward.
Within each domain, ADRs are stacked in rows of at most five nodes.
Vertical domain rows are ordered by root status, then tier S/A/B/C/D.
Edges: `References:` relationships rendered from referenced ADR to referencing ADR.
"""

from __future__ import annotations

import re
import sys
from pathlib import Path

try:
    import tomllib  # Python 3.11+
except ModuleNotFoundError:  # pragma: no cover — fallback for Python <3.11
    tomllib = None

ROOT = Path(__file__).resolve().parents[1]
ADR_DIR = ROOT / "docs" / "adr"
CFG = ADR_DIR / "adr-fmt.toml"
OUT = ADR_DIR / "adr-references.dot"

ADR_ID_RE = re.compile(r"^[A-Z]{3}-\d{4}$")
RELATED_HEADER_RE = re.compile(r"^## Related\s*$")
SECTION_HEADER_RE = re.compile(r"^## ")

# Tier color palette (Meadows-aligned tiers S/A/B/C/D)
TIER_COLORS = {
    "S": "#b91c1c",
    "A": "#c2410c",
    "B": "#a16207",
    "C": "#15803d",
    "D": "#1d4ed8",
}
TIER_ORDER = ("S", "A", "B", "C", "D")
MAX_DOMAIN_ROW_WIDTH = 5
DOMAIN_COLORS = {
    "COM": "#fef3c7",
    "CHE": "#fee2e2",
    "PAR": "#dcfce7",
    "GEN": "#dbeafe",
    "RST": "#ede9fe",
    "SEC": "#fce7f3",
    "AFM": "#e0f2fe",
}


def load_domains() -> list[dict]:
    if tomllib is not None:
        with CFG.open("rb") as fh:
            cfg = tomllib.load(fh)
        return cfg["domains"]
    # Minimal TOML fallback for adr-fmt.toml's `[[domains]]` array of tables.
    # Supports keys: prefix, name, directory (string scalars). Other keys ignored.
    text = CFG.read_text(encoding="utf-8")
    domains: list[dict] = []
    current: dict | None = None
    pending_key: str | None = None
    pending_buf: list[str] = []
    in_triple = False
    for raw in text.splitlines():
        line = raw.rstrip()
        stripped = line.strip()
        if in_triple:
            # Accumulate triple-quoted continuation until closing """.
            end = stripped.endswith('"""')
            content = stripped[:-3] if end else stripped
            # Drop line-continuation backslashes.
            if content.endswith("\\"):
                content = content[:-1]
            pending_buf.append(content)
            if end:
                if current is not None and pending_key is not None:
                    current[pending_key] = " ".join(p.strip() for p in pending_buf).strip()
                pending_key = None
                pending_buf = []
                in_triple = False
            continue
        if not stripped or stripped.startswith("#"):
            continue
        if stripped.startswith("[[") and stripped.endswith("]]"):
            if current is not None:
                domains.append(current)
            current = {} if stripped == "[[domains]]" else None
            continue
        if stripped.startswith("[") and stripped.endswith("]"):
            if current is not None:
                domains.append(current)
                current = None
            continue
        if current is None:
            continue
        if "=" not in stripped:
            continue
        key, _, value = stripped.partition("=")
        key = key.strip()
        value = value.strip()
        if value.startswith('"""'):
            rest = value[3:]
            if rest.endswith('"""') and len(rest) >= 3:
                current[key] = rest[:-3].strip()
            else:
                pending_key = key
                pending_buf = [rest[:-1] if rest.endswith("\\") else rest]
                in_triple = True
            continue
        if value.startswith('"') and value.endswith('"') and len(value) >= 2:
            current[key] = value[1:-1]
            continue
        # Other value shapes (lists, bools) are not needed downstream.
    if current is not None:
        domains.append(current)
    # Filter to entries that look like proper domain definitions.
    return [d for d in domains if "prefix" in d and "directory" in d and "name" in d]


def parse_adr(path: Path) -> dict | None:
    """Return {id, title, tier, domain, refs, is_root}. None if file is not an ADR."""
    text = path.read_text(encoding="utf-8")
    lines = text.splitlines()

    # Title line: # PREFIX-NNNN. Title
    m = re.match(r"^#\s+([A-Z]{3}-\d{4})\.\s*(.+?)\s*$", lines[0]) if lines else None
    if not m:
        print(f"warning: skipping {path.name}: no ADR title on first line", file=sys.stderr)
        return None
    adr_id, title = m.group(1), m.group(2)

    tier = ""
    in_related = False
    related_lines: list[str] = []
    for line in lines[1:]:
        if not in_related and line.startswith("Tier:") and not tier:
            parts = line.split(":", 1)[1].split()
            tier = parts[0] if parts else ""
        if RELATED_HEADER_RE.match(line):
            in_related = True
            continue
        if in_related:
            if SECTION_HEADER_RE.match(line):
                break
            stripped = line.strip()
            if stripped:
                related_lines.append(stripped)

    refs: list[str] = []
    is_root = False
    seen: set[str] = set()
    for raw in related_lines:
        # Allow pipe-separated relationship clauses on a single line.
        for clause in raw.split("|"):
            clause = clause.strip()
            clause_lc = clause.lower()
            if clause_lc.startswith("root:"):
                payload = clause.split(":", 1)[1]
                root_id = payload.strip()
                if root_id == adr_id:
                    is_root = True
                elif root_id:
                    print(
                        f"warning: {path.name}: Root target '{root_id}' does not match ADR id {adr_id}",
                        file=sys.stderr,
                    )
                continue
            if not clause_lc.startswith("references:"):
                continue
            payload = clause.split(":", 1)[1]
            for tok in payload.split(","):
                tok = tok.strip()
                if not tok:
                    continue
                if not ADR_ID_RE.match(tok):
                    print(
                        f"warning: {path.name}: invalid reference token '{tok}'",
                        file=sys.stderr,
                    )
                    continue
                if tok in seen:
                    continue
                seen.add(tok)
                refs.append(tok)

    return {"id": adr_id, "title": title, "tier": tier, "refs": refs, "is_root": is_root, "path": path}


def collect(domains: list[dict]) -> tuple[dict[str, dict], dict[str, list[str]]]:
    """Return (adrs by ID, list of ADR IDs per domain prefix)."""
    adrs: dict[str, dict] = {}
    by_domain: dict[str, list[str]] = {}
    prefix_re = re.compile(r"^[A-Z]{3}$")
    for d in domains:
        prefix = d["prefix"]
        if not prefix_re.match(prefix):
            print(f"error: invalid domain prefix '{prefix}' in adr-fmt.toml", file=sys.stderr)
            sys.exit(1)
        directory = ADR_DIR / d["directory"]
        by_domain[prefix] = []
        if not directory.is_dir():
            print(f"warning: domain directory missing: {directory}", file=sys.stderr)
            continue
        for path in sorted(directory.glob(f"{prefix}-*.md")):
            adr = parse_adr(path)
            if adr is None:
                continue
            if adr["id"] in adrs:
                print(f"warning: duplicate ADR id {adr['id']}", file=sys.stderr)
                continue
            adr["domain"] = prefix
            adrs[adr["id"]] = adr
            by_domain[prefix].append(adr["id"])
    return adrs, by_domain


def dot_escape(s: str) -> str:
    return s.replace("\\", "\\\\").replace('"', '\\"')


def chunked(items: list[str], size: int) -> list[list[str]]:
    return [items[i : i + size] for i in range(0, len(items), size)]


def dot_attrs(attrs: dict[str, str | int | float | bool]) -> str:
    rendered: list[str] = []
    for key, value in attrs.items():
        if isinstance(value, bool):
            rendered.append(f"{key}={'true' if value else 'false'}")
        elif isinstance(value, (int, float)):
            rendered.append(f"{key}={value}")
        else:
            rendered.append(f"{key}=\"{dot_escape(value)}\"")
    return ", ".join(rendered)


def domain_layout_rows(ids: list[str], adrs: dict[str, dict]) -> list[tuple[str, list[str]]]:
    """Return bottom-to-top layout rows for one domain.

    Rows are capped at MAX_DOMAIN_ROW_WIDTH so a single domain never becomes
    wider than five ADR nodes. Root rows come first, then non-root ADRs follow
    tier order so the bottom-up render reads Root, S, A, B, C, D.
    """
    rows: list[tuple[str, list[str]]] = []

    roots = sorted(adr_id for adr_id in ids if adrs[adr_id].get("is_root"))
    root_set = set(roots)
    non_roots = [adr_id for adr_id in ids if adr_id not in root_set]

    for index, chunk in enumerate(chunked(roots, MAX_DOMAIN_ROW_WIDTH)):
        rows.append((f"root_{index}", chunk))

    for tier in TIER_ORDER:
        tier_ids = sorted(adr_id for adr_id in non_roots if adrs[adr_id].get("tier") == tier)
        for index, chunk in enumerate(chunked(tier_ids, MAX_DOMAIN_ROW_WIDTH)):
            rows.append((f"tier_{tier}_{index}", chunk))

    unknown = sorted(adr_id for adr_id in non_roots if adrs[adr_id].get("tier") not in TIER_ORDER)
    for index, chunk in enumerate(chunked(unknown, MAX_DOMAIN_ROW_WIDTH)):
        rows.append((f"tier_unknown_{index}", chunk))

    return rows


def reference_edge_attrs(
    ref: str,
    adr_id: str,
    adrs: dict[str, dict],
    layout_by_id: dict[str, tuple[int, int]],
    domain_index: dict[str, int],
) -> dict[str, str | int | float | bool]:
    """Return geometry-aware rendering attributes for one References edge."""
    refs = adrs[adr_id]["refs"]
    primary = bool(refs) and refs[0] == ref
    attrs: dict[str, str | int | float | bool] = {
        "constraint": False,
        "weight": 0,
        "penwidth": 1.1 if primary else 0.75,
        "arrowsize": 0.55 if primary else 0.2,
        "color": "#334155b3" if primary else "#47556999",
        "style": "solid" if primary else "dashed",
        "arrowhead": "normal" if primary else "none",
    }

    if ref not in adrs or ref not in layout_by_id or adr_id not in layout_by_id:
        attrs.update({"tailport": "n", "headport": "s", "style": "dotted"})
        return attrs

    source_domain = adrs[ref]["domain"]
    target_domain = adrs[adr_id]["domain"]
    source_row, source_slot = layout_by_id[ref]
    target_row, target_slot = layout_by_id[adr_id]

    if source_domain != target_domain:
        left_to_right = domain_index[source_domain] < domain_index[target_domain]
        attrs.update(
            {
                "tailport": "e" if left_to_right else "w",
                "headport": "w" if left_to_right else "e",
                "style": "dashed" if primary else "dotted",
                "color": "#33415566" if primary else "#47556980",
            }
        )
        return attrs

    if target_row > source_row:
        if primary:
            attrs.update({"tailport": "n", "headport": "s"})
        else:
            left_to_right = source_slot <= target_slot
            attrs.update(
                {
                    "tailport": "e" if left_to_right else "w",
                    "headport": "w" if left_to_right else "e",
                }
            )
        return attrs

    if target_row < source_row:
        attrs.update(
            {
                "tailport": "s",
                "headport": "n",
                "style": "dotted",
                "color": "#b4530966" if primary else "#b4530980",
            }
        )
        return attrs

    left_to_right = source_slot <= target_slot
    attrs.update(
        {
            "tailport": "e" if left_to_right else "w",
            "headport": "w" if left_to_right else "e",
            "style": "dotted" if not primary else "solid",
        }
    )
    return attrs


def render(adrs: dict[str, dict], by_domain: dict[str, list[str]], domains: list[dict]) -> str:
    lines: list[str] = []
    lines.append("// Generated by scripts/gen-adr-graph.py — do not edit by hand.")
    lines.append("// Source: docs/adr/adr-fmt.toml configured domains.")
    lines.append("// Corpus: active ADRs only (docs/adr/stale excluded).")
    lines.append("// Layout: explicit `Root:` ADRs are anchored at the bottom; trees grow upward.")
    lines.append("// Layout: domain rows are capped at 5 ADRs wide and ordered Root, S, A, B, C, D.")
    lines.append("// Edges: `References:` relationships, geometry-routed from referenced ADR to referencing ADR.")
    lines.append("// Regenerate: python3 scripts/gen-adr-graph.py")
    lines.append("// Render:    dot -Tsvg docs/adr/adr-references.dot -o docs/adr/adr-references.svg")
    lines.append("")
    lines.append("digraph adr_references {")
    lines.append("  rankdir=BT;")
    lines.append(
        "  graph [splines=ortho, overlap=false, nodesep=0.18, ranksep=0.75, "
        "newrank=true, concentrate=false, outputorder=edgesfirst, fontname=\"Helvetica\"];"
    )
    lines.append("  node  [shape=box, style=\"rounded,filled\", fontname=\"Helvetica\", fontsize=10];")
    lines.append("  edge  [color=\"#64748b66\", arrowsize=0.5, penwidth=0.7];")
    lines.append("")

    domain_index = {d["prefix"]: index for index, d in enumerate(domains)}
    rows_by_domain = {d["prefix"]: domain_layout_rows(by_domain.get(d["prefix"], []), adrs) for d in domains}
    layout_by_id = {
        adr_id: (row_index, slot)
        for rows in rows_by_domain.values()
        for row_index, (_, row_ids) in enumerate(rows)
        for slot, adr_id in enumerate(row_ids)
    }

    # Clusters per configured domain.
    for d in domains:
        prefix = d["prefix"]
        ids = by_domain.get(prefix, [])
        rows = rows_by_domain[prefix]
        slot_by_id = {
            adr_id: slot
            for _, row_ids in rows
            for slot, adr_id in enumerate(row_ids)
        }
        cluster_name = f"cluster_{prefix}"
        fill = DOMAIN_COLORS.get(prefix, "#f1f5f9")
        lines.append(f"  subgraph \"{cluster_name}\" {{")
        lines.append(f"    label=\"{prefix} — {dot_escape(d['name'])}\";")
        lines.append("    style=\"rounded,filled\";")
        lines.append(f"    fillcolor=\"{fill}\";")
        lines.append("    color=\"#94a3b8\";")
        lines.append("    fontname=\"Helvetica\";")
        lines.append("    fontsize=11;")
        if not ids:
            lines.append(f"    \"_empty_{prefix}\" [label=\"(no ADRs)\", shape=plaintext, style=\"\"];")
        for adr_id in ids:
            adr = adrs[adr_id]
            tier = adr["tier"] or "?"
            color = TIER_COLORS.get(tier, "#475569")
            label = f"{adr_id}\\n{dot_escape(adr['title'])}"
            root_attrs = ", peripheries=2" if adr.get("is_root") else ""
            slot = slot_by_id.get(adr_id, 0)
            lines.append(
                f"    \"{adr_id}\" [label=\"{label}\", color=\"{color}\", "
                f"penwidth=2, fillcolor=\"white\", group=\"{prefix}_col_{slot}\"{root_attrs}];"
            )
        if rows:
            lines.append("    // Layout rows render bottom-to-top as Root, S, A, B, C, D; max 5 ADRs per row.")
        for row_index, (row_name, row_ids) in enumerate(rows):
            lines.append(f"    subgraph \"rank_{prefix}_{row_index:02d}_{row_name}\" {{")
            lines.append("      rank=same;")
            for adr_id in row_ids:
                lines.append(f"      \"{adr_id}\";")
            lines.append("    }")
            for left, right in zip(row_ids, row_ids[1:]):
                lines.append(
                    f"    \"{left}\" -> \"{right}\" "
                    "[style=invis, weight=100, constraint=false];"
                )
        for lower, upper in zip(rows, rows[1:]):
            for lower_anchor, upper_anchor in zip(lower[1], upper[1]):
                lines.append(
                    f"    \"{lower_anchor}\" -> \"{upper_anchor}\" "
                    "[style=invis, weight=1000, minlen=2];"
                )
        lines.append("  }")
        lines.append("")

    roots = sorted(adr_id for adr_id, adr in adrs.items() if adr.get("is_root"))
    if roots:
        lines.append("  // Explicit Root: ADRs sit on the bottom rank in the bottom-up layout.")
        lines.append("  { rank=source;")
        for adr_id in roots:
            lines.append(f"    \"{adr_id}\";")
        lines.append("  }")
        lines.append("")

    # Placeholder nodes for any References: target outside the active corpus.
    referenced = {ref for adr in adrs.values() for ref in adr["refs"]}
    missing = sorted(t for t in referenced if t not in adrs)
    if missing:
        lines.append("  // Unresolved references (not in active corpus)")
        lines.append("  subgraph \"cluster_unresolved\" {")
        lines.append("    label=\"Unresolved (stale or external)\";")
        lines.append("    style=\"dashed\";")
        lines.append("    color=\"#9ca3af\";")
        for tok in missing:
            lines.append(
                f"    \"{tok}\" [label=\"{tok}\\n(unresolved)\", "
                f"style=\"rounded,filled,dashed\", fillcolor=\"#f3f4f6\", color=\"#9ca3af\"];"
            )
        lines.append("  }")
        lines.append("")

    # Edges. Reverse the visual direction so references act as roots/foundations
    # and ADR trees grow upward from the bottom of the rendered image.
    edge_count = 0
    for adr_id in sorted(adrs):
        for ref in adrs[adr_id]["refs"]:
            attrs = reference_edge_attrs(ref, adr_id, adrs, layout_by_id, domain_index)
            lines.append(f"  \"{ref}\" -> \"{adr_id}\" [{dot_attrs(attrs)}];")
            edge_count += 1
    lines.append("")

    # Legend
    lines.append("  // Legend")
    lines.append("  subgraph \"cluster_legend\" {")
    lines.append("    label=\"Tier (border color)\";")
    lines.append("    style=\"rounded\";")
    lines.append("    color=\"#cbd5e1\";")
    lines.append("    fontsize=10;")
    for tier, color in TIER_COLORS.items():
        lines.append(
            f"    \"legend_{tier}\" [label=\"Tier {tier}\", color=\"{color}\", "
            f"penwidth=2, fillcolor=\"white\"];"
        )
    pairs = list(TIER_COLORS.keys())
    for a, b in zip(pairs, pairs[1:]):
        lines.append(f"    \"legend_{a}\" -> \"legend_{b}\" [style=invis];")
    lines.append("  }")

    lines.append("}")
    lines.append("")

    print(
        f"info: {len(adrs)} ADRs across {sum(1 for v in by_domain.values() if v)} domains; "
        f"{edge_count} edges; {len(missing)} unresolved targets",
        file=sys.stderr,
    )
    return "\n".join(lines)


def main() -> int:
    domains = load_domains()
    adrs, by_domain = collect(domains)
    OUT.write_text(render(adrs, by_domain, domains), encoding="utf-8")
    print(f"wrote {OUT.relative_to(ROOT)}", file=sys.stderr)
    return 0


if __name__ == "__main__":
    sys.exit(main())
