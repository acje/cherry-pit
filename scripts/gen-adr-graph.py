#!/usr/bin/env python3
"""Generate Graphviz DOT graph of ADRs and their `References:` edges.

Source of truth: docs/adr/adr-fmt.toml (configured domains).
Excludes: docs/adr/stale, generated READMEs, TEMPLATE.md, GOVERNANCE.md.
Edges: only the `References:` relationship verb.
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
    """Return {id, title, tier, domain, refs}. None if file is not an ADR."""
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
    seen: set[str] = set()
    for raw in related_lines:
        # Allow pipe-separated relationship clauses on a single line.
        for clause in raw.split("|"):
            clause = clause.strip()
            if not clause.lower().startswith("references:"):
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

    return {"id": adr_id, "title": title, "tier": tier, "refs": refs, "path": path}


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


def render(adrs: dict[str, dict], by_domain: dict[str, list[str]], domains: list[dict]) -> str:
    lines: list[str] = []
    lines.append("// Generated by scripts/gen-adr-graph.py — do not edit by hand.")
    lines.append("// Source: docs/adr/adr-fmt.toml configured domains.")
    lines.append("// Corpus: active ADRs only (docs/adr/stale excluded).")
    lines.append("// Edges: `References:` relationships only.")
    lines.append("// Regenerate: python3 scripts/gen-adr-graph.py")
    lines.append("// Render:    dot -Tsvg docs/adr/adr-references.dot -o adr-references.svg")
    lines.append("")
    lines.append("digraph adr_references {")
    lines.append("  rankdir=LR;")
    lines.append("  graph [splines=true, overlap=false, nodesep=0.25, ranksep=0.6, fontname=\"Helvetica\"];")
    lines.append("  node  [shape=box, style=\"rounded,filled\", fontname=\"Helvetica\", fontsize=10];")
    lines.append("  edge  [color=\"#64748b\", arrowsize=0.7];")
    lines.append("")

    # Clusters per configured domain.
    for d in domains:
        prefix = d["prefix"]
        ids = by_domain.get(prefix, [])
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
            lines.append(
                f"    \"{adr_id}\" [label=\"{label}\", color=\"{color}\", "
                f"penwidth=2, fillcolor=\"white\"];"
            )
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

    # Edges
    edge_count = 0
    for adr_id in sorted(adrs):
        for ref in adrs[adr_id]["refs"]:
            lines.append(f"  \"{adr_id}\" -> \"{ref}\";")
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
