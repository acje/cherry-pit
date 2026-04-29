//! Unified output formatter — Alternative 4 markdown format.
//!
//! All modes emit concatenated markdown with structured header blocks
//! using `◆`/`◇` markers and `---` separators.
//! Optimized for LLM token efficiency.

use std::collections::HashMap;
use std::fmt::Write as _;

use crate::config::Config;
use crate::model::{AdrId, AdrRecord, DomainDir, RelVerb, Tier};
use crate::nav::ChildEntry;
use crate::report::Diagnostic;

// ── Output block types ─────────────────────────────────────────────

/// Header metadata for an output block.
#[derive(Debug)]
pub struct HeaderMeta {
    pub id: AdrId,
    pub tier: Option<Tier>,
    pub status: String,
    pub domain: String,
    pub crates: Vec<String>,
    pub fan_out: Vec<String>,
    pub fan_in: Vec<String>,
}

/// An output block in the Alternative 4 format.
#[derive(Debug)]
pub enum OutputBlock {
    /// Focal ADR block — the target of a `--critique` query.
    Focal { meta: HeaderMeta, content: String },
    /// Connected ADR block — transitively reachable from focal.
    Connected {
        meta: HeaderMeta,
        content: String,
        path: String,
    },
}

/// A group of rules emitted under a single root ADR in `--context` mode.
#[derive(Debug)]
pub struct RootGroup {
    pub root_id: AdrId,
    pub root_title: String,
    pub rules: Vec<EmittedRule>,
}

/// A single rule positioned in root-grouped context output.
#[derive(Debug)]
pub struct EmittedRule {
    pub adr_id: AdrId,
    pub rule_id: String,
    pub text: String,
    pub layer: u8,
    #[allow(dead_code)] // Used in sort key, kept for future rendering
    pub depth: u16,
}

// ── Block rendering ────────────────────────────────────────────────

/// Render output blocks to Alternative 4 markdown.
#[must_use]
pub fn render_blocks(blocks: &[OutputBlock]) -> String {
    let mut out = String::new();

    for (i, block) in blocks.iter().enumerate() {
        if i > 0 {
            out.push_str("\n---\n\n");
        }

        match block {
            OutputBlock::Focal { meta, content } => {
                render_focal_header(&mut out, meta);
                out.push('\n');
                out.push_str(content);
                out.push('\n');
            }
            OutputBlock::Connected {
                meta,
                content,
                path,
            } => {
                render_connected_header(&mut out, meta, path);
                out.push('\n');
                out.push_str(content);
                out.push('\n');
            }
        }
    }

    out
}

fn render_focal_header(out: &mut String, meta: &HeaderMeta) {
    let tier = meta.tier.map_or_else(|| "?".into(), |t| format!("{t}"));
    writeln!(
        out,
        "## ◆ FOCAL: {} | Tier: {} | Status: {}",
        meta.id, tier, meta.status
    )
    .unwrap();

    let crates_str = if meta.crates.is_empty() {
        String::new()
    } else {
        format!(" | Crates: {}", meta.crates.join(", "))
    };
    writeln!(out, "## Domain: {}{crates_str}", meta.domain).unwrap();

    if !meta.fan_out.is_empty() {
        writeln!(out, "## Fan-out: {}", meta.fan_out.join(", ")).unwrap();
    }
    if !meta.fan_in.is_empty() {
        writeln!(out, "## Fan-in: {}", meta.fan_in.join(", ")).unwrap();
    }
}

fn render_connected_header(out: &mut String, meta: &HeaderMeta, path: &str) {
    let tier = meta.tier.map_or_else(|| "?".into(), |t| format!("{t}"));
    writeln!(
        out,
        "## ◇ CONNECTED: {} | Tier: {} | Status: {}",
        meta.id, tier, meta.status
    )
    .unwrap();
    writeln!(out, "## Path: {path}").unwrap();
}

// ── Diagnostic rendering ───────────────────────────────────────────

/// Render diagnostics as Alternative 4 markdown blocks to stdout.
#[must_use]
pub fn render_diagnostics(diagnostics: &[Diagnostic], record_count: usize) -> String {
    let mut out = String::new();

    let mut errors = 0u32;
    let mut warnings = 0u32;

    for d in diagnostics {
        if d.internal {
            continue;
        }
        match d.severity {
            crate::report::Severity::Error => errors += 1,
            crate::report::Severity::Warning => warnings += 1,
        }

        let location = if d.line > 0 {
            format!("{}:{}", d.file, d.line)
        } else {
            d.file.clone()
        };

        writeln!(
            out,
            "- **{}[{}]** {}: {}",
            d.severity, d.rule, location, d.message
        )
        .unwrap();
    }

    if out.is_empty() {
        writeln!(
            out,
            "## Diagnostics: 0 error(s), 0 warning(s) across {record_count} ADR(s)"
        )
        .unwrap();
    } else {
        let header = format!(
            "## Diagnostics: {errors} error(s), {warnings} warning(s) across {record_count} ADR(s)\n\n"
        );
        out.insert_str(0, &header);
    }

    out
}

// ── Tree rendering (--tree mode) ───────────────────────────────────

/// Render root-grouped context output with preamble.
///
/// Rules are grouped by root ADR subtree. Each root with rules gets a
/// `### ROOT-ID. Title` heading. Rule lines use `- {text} [{ADR_ID}:{RULE_ID}:L{layer}]`
/// format with the anchoring ID at the end.
///
/// Groups with no rules after dedup are skipped. An optional "Unclaimed Rules"
/// section appears if any eligible rules were not reached by any root's BFS.
#[must_use]
pub fn render_root_groups(crate_name: &str, groups: &[RootGroup]) -> String {
    let mut out = String::new();

    // Preamble
    writeln!(out, "# Architecture Rules").unwrap();
    writeln!(out).unwrap();
    writeln!(
        out,
        "These rules are mandatory constraints for all code in crate `{crate_name}`."
    )
    .unwrap();
    writeln!(out, "Follow every rule without exception.").unwrap();

    for group in groups {
        if group.rules.is_empty() {
            continue;
        }

        writeln!(out).unwrap();
        writeln!(out, "### {}. {}", group.root_id, group.root_title).unwrap();

        for rule in &group.rules {
            writeln!(
                out,
                "- {} [{}:{}:L{}]",
                rule.text, rule.adr_id, rule.rule_id, rule.layer
            )
            .unwrap();
        }
    }

    out
}

// ── Tree rendering (--tree mode, domain overview) ──────────────────

/// Render the domain tree with box-drawing to stdout.
#[must_use]
pub fn render_tree(
    records: &[AdrRecord],
    domain_dirs: &[DomainDir],
    config: &Config,
    domain_filter: Option<&str>,
) -> String {
    let mut out = String::new();

    // Group records by domain prefix
    let mut by_prefix: HashMap<&str, Vec<&AdrRecord>> = HashMap::new();
    for record in records {
        if !record.is_stale {
            by_prefix.entry(&record.id.prefix).or_default().push(record);
        }
    }

    // Filter by domain if requested
    let dirs: Vec<&DomainDir> = if let Some(filter) = domain_filter {
        domain_dirs.iter().filter(|d| d.prefix == filter).collect()
    } else {
        domain_dirs.iter().collect()
    };

    if dirs.is_empty() {
        if let Some(f) = domain_filter {
            writeln!(out, "No domain found matching '{f}'").unwrap();
        }
        return out;
    }

    for dir in &dirs {
        let domain_name = &dir.name;
        let foundation = config
            .domains
            .iter()
            .find(|d| d.prefix == dir.prefix)
            .is_some_and(|d| d.foundation);
        let foundation_marker = if foundation { " [foundation]" } else { "" };

        writeln!(
            out,
            "## {} ({}){foundation_marker}",
            domain_name, dir.prefix
        )
        .unwrap();

        if let Some(domain_records) = by_prefix.get(dir.prefix.as_str()) {
            let mut sorted = domain_records.clone();
            sorted.sort_by_key(|r| r.id.number);

            for record in &sorted {
                let title = record.title.as_deref().unwrap_or("(untitled)");
                let tier = record.tier.map_or_else(|| "?".into(), |t| format!("{t}"));
                let status = record
                    .status
                    .as_ref()
                    .map_or_else(|| "?".into(), super::model::Status::short_display);
                writeln!(out, "  {} {title} [{tier}] {status}", record.id).unwrap();
            }
        }

        // Stale count for this domain
        let stale_count = records
            .iter()
            .filter(|r| r.is_stale && r.id.prefix == dir.prefix)
            .count();
        if stale_count > 0 {
            writeln!(out, "  ({stale_count} stale)").unwrap();
        }

        out.push('\n');
    }

    out
}

// ── Helpers ────────────────────────────────────────────────────────

/// Build `HeaderMeta` for a record, resolving domain name from config.
#[must_use]
pub fn build_header_meta(
    record: &AdrRecord,
    config: &Config,
    children: &HashMap<AdrId, Vec<ChildEntry>>,
) -> HeaderMeta {
    let domain = config
        .domains
        .iter()
        .find(|d| d.prefix == record.id.prefix)
        .map_or_else(|| record.id.prefix.clone(), |d| d.name.clone());

    // Fan-out: forward relationships
    let fan_out: Vec<String> = record
        .relationships
        .iter()
        .filter(|r| !(r.verb.is_reverse() || r.verb == RelVerb::Root && r.target == record.id))
        .map(|r| format!("{} {}", r.verb, r.target))
        .collect();

    // Fan-in: reverse links from children
    let fan_in: Vec<String> = children
        .get(&record.id)
        .map(|entries| {
            entries
                .iter()
                .map(|e| format!("{} ← {}", e.verb, e.child))
                .collect()
        })
        .unwrap_or_default();

    let status = record
        .status
        .as_ref()
        .map_or_else(|| "?".into(), super::model::Status::short_display);

    HeaderMeta {
        id: record.id.clone(),
        tier: record.tier,
        status,
        domain,
        crates: record.crates.clone(),
        fan_out,
        fan_in,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::AdrId;

    fn make_id(prefix: &str, num: u16) -> AdrId {
        AdrId {
            prefix: prefix.into(),
            number: num,
        }
    }

    #[test]
    fn render_blocks_focal_only() {
        let blocks = vec![OutputBlock::Focal {
            meta: HeaderMeta {
                id: make_id("CHE", 42),
                tier: Some(Tier::A),
                status: "Accepted".into(),
                domain: "Cherry Domain".into(),
                crates: vec!["cherry-pit-core".into()],
                fan_out: vec!["References CHE-0001".into()],
                fan_in: vec!["References ← CHE-0050".into()],
            },
            content: "# CHE-0042 · Title\n\nContent here.".into(),
        }];

        let output = render_blocks(&blocks);
        assert!(output.contains("## ◆ FOCAL: CHE-0042"), "output:\n{output}");
        assert!(output.contains("Tier: A"), "output:\n{output}");
        assert!(output.contains("cherry-pit-core"), "output:\n{output}");
        assert!(output.contains("Fan-out:"), "output:\n{output}");
        assert!(output.contains("Fan-in:"), "output:\n{output}");
    }

    #[test]
    fn render_blocks_with_connected() {
        let blocks = vec![
            OutputBlock::Focal {
                meta: HeaderMeta {
                    id: make_id("CHE", 42),
                    tier: Some(Tier::A),
                    status: "Accepted".into(),
                    domain: "Cherry".into(),
                    crates: vec![],
                    fan_out: vec![],
                    fan_in: vec![],
                },
                content: "focal content".into(),
            },
            OutputBlock::Connected {
                meta: HeaderMeta {
                    id: make_id("CHE", 1),
                    tier: Some(Tier::S),
                    status: "Accepted".into(),
                    domain: "Cherry".into(),
                    crates: vec![],
                    fan_out: vec![],
                    fan_in: vec![],
                },
                content: "connected content".into(),
                path: "CHE-0042 → References → CHE-0001".into(),
            },
        ];

        let output = render_blocks(&blocks);
        assert!(output.contains("## ◆ FOCAL:"), "output:\n{output}");
        assert!(output.contains("---"), "output:\n{output}");
        assert!(output.contains("## ◇ CONNECTED:"), "output:\n{output}");
        assert!(output.contains("## Path:"), "output:\n{output}");
    }

    #[test]
    fn render_diagnostics_clean() {
        let output = render_diagnostics(&[], 5);
        assert!(output.contains("0 error(s), 0 warning(s)"));
    }

    #[test]
    fn render_diagnostics_with_warnings() {
        let diags = vec![Diagnostic::warning(
            "T001",
            &std::path::PathBuf::from("test.md"),
            1,
            "missing title".into(),
        )];
        let output = render_diagnostics(&diags, 1);
        assert!(output.contains("1 warning(s)"));
        assert!(output.contains("T001"));
    }

    // ── render_root_groups tests ────────────────────────────────────

    #[test]
    fn render_root_groups_basic() {
        let groups = vec![RootGroup {
            root_id: make_id("COM", 1),
            root_title: "Foundation Principle".into(),
            rules: vec![EmittedRule {
                adr_id: make_id("COM", 1),
                rule_id: "R1".into(),
                text: "All modules must log errors.".into(),
                layer: 5,
                depth: 0,
            }],
        }];
        let output = render_root_groups("cherry-pit-core", &groups);
        // Preamble
        assert!(output.contains("# Architecture Rules"), "output:\n{output}");
        assert!(
            output.contains("crate `cherry-pit-core`"),
            "output:\n{output}"
        );
        assert!(
            output.contains("Follow every rule without exception"),
            "output:\n{output}"
        );
        // Root header
        assert!(
            output.contains("### COM-0001. Foundation Principle"),
            "output:\n{output}"
        );
        // Rule line with ID and layer
        assert!(
            output.contains("- All modules must log errors. [COM-0001:R1:L5]"),
            "output:\n{output}"
        );
    }

    #[test]
    fn render_root_groups_empty_group_skipped() {
        let groups = vec![
            RootGroup {
                root_id: make_id("COM", 1),
                root_title: "Empty Root".into(),
                rules: vec![],
            },
            RootGroup {
                root_id: make_id("CHE", 1),
                root_title: "Non-empty Root".into(),
                rules: vec![EmittedRule {
                    adr_id: make_id("CHE", 2),
                    rule_id: "R1".into(),
                    text: "Rule here.".into(),
                    layer: 3,
                    depth: 1,
                }],
            },
        ];
        let output = render_root_groups("test", &groups);
        assert!(
            !output.contains("Empty Root"),
            "empty group should be skipped:\n{output}"
        );
        assert!(
            output.contains("### CHE-0001. Non-empty Root"),
            "non-empty group should appear:\n{output}"
        );
    }

    #[test]
    fn render_root_groups_multiple_roots_ordering() {
        let groups = vec![
            RootGroup {
                root_id: make_id("COM", 1),
                root_title: "Foundation".into(),
                rules: vec![EmittedRule {
                    adr_id: make_id("COM", 1),
                    rule_id: "R1".into(),
                    text: "Foundation rule.".into(),
                    layer: 1,
                    depth: 0,
                }],
            },
            RootGroup {
                root_id: make_id("CHE", 1),
                root_title: "Domain Root".into(),
                rules: vec![EmittedRule {
                    adr_id: make_id("CHE", 5),
                    rule_id: "R1".into(),
                    text: "Domain rule.".into(),
                    layer: 7,
                    depth: 1,
                }],
            },
        ];
        let output = render_root_groups("test", &groups);
        let com_pos = output
            .find("### COM-0001. Foundation")
            .expect("COM header missing");
        let che_pos = output
            .find("### CHE-0001. Domain Root")
            .expect("CHE header missing");
        assert!(
            com_pos < che_pos,
            "Groups should render in order given:\n{output}"
        );
    }

    #[test]
    fn render_root_groups_all_empty_produces_preamble_only() {
        let groups = vec![RootGroup {
            root_id: make_id("COM", 1),
            root_title: "Empty".into(),
            rules: vec![],
        }];
        let output = render_root_groups("test", &groups);
        assert!(output.contains("# Architecture Rules"));
        assert!(
            !output.contains("###"),
            "no root headers for empty groups:\n{output}"
        );
    }

    #[test]
    fn render_root_groups_multiple_adrs_under_one_root() {
        let groups = vec![RootGroup {
            root_id: make_id("CHE", 1),
            root_title: "Design Priority".into(),
            rules: vec![
                EmittedRule {
                    adr_id: make_id("CHE", 1),
                    rule_id: "R1".into(),
                    text: "Root rule from the root itself.".into(),
                    layer: 2,
                    depth: 0,
                },
                EmittedRule {
                    adr_id: make_id("CHE", 5),
                    rule_id: "R1".into(),
                    text: "Child rule from CHE-0005.".into(),
                    layer: 5,
                    depth: 1,
                },
                EmittedRule {
                    adr_id: make_id("CHE", 10),
                    rule_id: "R1".into(),
                    text: "Grandchild rule from CHE-0010.".into(),
                    layer: 7,
                    depth: 2,
                },
            ],
        }];
        let output = render_root_groups("cherry-pit-core", &groups);
        // Single root header
        assert!(
            output.contains("### CHE-0001. Design Priority"),
            "root header missing:\n{output}"
        );
        // All three rules present under that header
        assert!(
            output.contains("[CHE-0001:R1:L2]"),
            "root's own rule missing:\n{output}"
        );
        assert!(
            output.contains("[CHE-0005:R1:L5]"),
            "child rule missing:\n{output}"
        );
        assert!(
            output.contains("[CHE-0010:R1:L7]"),
            "grandchild rule missing:\n{output}"
        );
        // Verify ordering: L2 before L5 before L7
        let pos_l2 = output.find("[CHE-0001:R1:L2]").unwrap();
        let pos_l5 = output.find("[CHE-0005:R1:L5]").unwrap();
        let pos_l7 = output.find("[CHE-0010:R1:L7]").unwrap();
        assert!(
            pos_l2 < pos_l5 && pos_l5 < pos_l7,
            "rules should appear in layer order:\n{output}"
        );
    }
}
