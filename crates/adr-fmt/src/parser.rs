//! ADR file parser — extracts metadata from markdown files.

use std::collections::HashMap;
use std::fs;
use std::path::Path;

use regex::Regex;

use crate::config::Config;
use crate::model::{
    AdrId, AdrRecord, DomainDir, RelVerb, Relationship, Status, TaggedRule, Tier,
    parse_adr_id_from_str,
};

/// Parse all ADR files in a domain directory.
pub fn parse_domain(dir: &DomainDir) -> Vec<AdrRecord> {
    let mut records = Vec::new();

    let Ok(entries) = fs::read_dir(&dir.path) else {
        return records;
    };

    let filename_re = Regex::new(&format!(
        r"^{}-(\d{{4}})-[a-z0-9]+(?:-[a-z0-9]+)*\.md$",
        regex::escape(&dir.prefix)
    ))
    .expect("valid regex");

    for entry in entries.flatten() {
        let file_name = entry.file_name();
        let name = file_name.to_string_lossy();

        if !filename_re.is_match(&name) {
            continue;
        }

        if let Some(record) = parse_adr_file(&entry.path(), &dir.prefix, false) {
            records.push(record);
        }
    }

    records.sort_by_key(|r| r.id.number);
    records
}

/// Parse all ADR files in the stale directory.
///
/// Stale files may belong to any domain, so we try all configured
/// domain prefixes.
pub fn parse_stale(stale_dir: &Path, config: &Config) -> Vec<AdrRecord> {
    let mut records = Vec::new();

    let Ok(entries) = fs::read_dir(stale_dir) else {
        return records;
    };

    let prefixes: Vec<(&str, Regex)> = config
        .domains
        .iter()
        .map(|d| {
            let pattern = format!(
                r"^{}-\d{{4}}-[a-z0-9]+(?:-[a-z0-9]+)*\.md$",
                regex::escape(&d.prefix),
            );
            (
                d.prefix.as_str(),
                Regex::new(&pattern).expect("valid regex"),
            )
        })
        .collect();

    for entry in entries.flatten() {
        let file_name = entry.file_name();
        let name = file_name.to_string_lossy();

        if !name.ends_with(".md") || name == "README.md" {
            continue;
        }

        // Try each prefix to find which domain this stale ADR belongs to
        for (prefix, re) in &prefixes {
            if re.is_match(&name) {
                if let Some(record) = parse_adr_file(&entry.path(), prefix, true) {
                    records.push(record);
                }
                break;
            }
        }
    }

    records.sort_by_key(|r| r.id.number);
    records
}

/// Parse a single ADR file into an `AdrRecord`.
pub fn parse_adr_file(path: &Path, expected_prefix: &str, is_stale: bool) -> Option<AdrRecord> {
    let content = fs::read_to_string(path).ok()?;
    let lines: Vec<&str> = content.lines().collect();

    if lines.is_empty() {
        return None;
    }

    // --- H1: title and ID ---
    let (id, title, title_line) = parse_title(&lines, expected_prefix)?;

    // --- Metadata fields ---
    let (date, date_line) = find_field(&lines, "Date:");
    let (last_reviewed, last_reviewed_line) = find_field(&lines, "Last-reviewed:");
    let (tier, tier_line) = find_tier_field(&lines);

    // --- Status ---
    let (status, status_line, status_raw) = find_status(&lines);

    // --- Related ---
    let (relationships, has_related, related_has_placeholder) = find_relationships(&lines);

    // --- Self-referencing detection (Root verb targeting own ID) ---
    let is_self_referencing = relationships
        .iter()
        .any(|rel| rel.verb == RelVerb::Root && rel.target == id);

    // --- Required sections ---
    let has_context = has_heading(&lines, "Context");
    let has_decision = has_heading(&lines, "Decision");
    let has_consequences = has_heading(&lines, "Consequences");
    let has_retirement = has_heading(&lines, "Retirement");
    let has_rejection_rationale = has_heading(&lines, "Rejection Rationale");

    // --- Section ordering and word counts ---
    let (section_order, section_word_counts) = analyze_sections(&lines);

    // --- Crates field ---
    let crates = find_crates_field(&lines);

    // --- Decision section content and tagged rules ---
    let decision_content = extract_decision_content(&lines);
    let decision_rules = extract_tagged_rules(&lines, decision_content.as_ref());

    // --- Code block metrics ---
    let (max_code_block_lines, code_block_count, max_code_block_line) = measure_code_blocks(&lines);

    Some(AdrRecord {
        id,
        file_path: path.to_owned(),
        title: Some(title),
        title_line,
        date,
        date_line,
        last_reviewed,
        last_reviewed_line,
        tier,
        tier_line,
        status,
        status_line,
        status_raw,
        relationships,
        has_related,
        has_context,
        has_decision,
        has_consequences,
        has_retirement,
        has_rejection_rationale,
        is_stale,
        is_self_referencing,
        max_code_block_lines,
        max_code_block_line,
        code_block_count,
        related_has_placeholder,
        section_order,
        section_word_counts,
        crates,
        decision_rules,
        decision_content,
    })
}

/// Parse the H1 title line: `# PREFIX-NNNN. Title text`.
fn parse_title(lines: &[&str], expected_prefix: &str) -> Option<(AdrId, String, usize)> {
    for (i, line) in lines.iter().enumerate() {
        if let Some(rest) = line.strip_prefix("# ") {
            // Expected format: "PREFIX-NNNN. Title text"
            if let Some(dot_pos) = rest.find(". ")
                && let Some(id) = parse_adr_id_from_str(&rest[..dot_pos])
                && id.prefix == expected_prefix
            {
                let title = rest[dot_pos + 2..].to_owned();
                return Some((id, title, i + 1));
            }
        }
    }
    None
}

/// Find a simple `Key: value` field.
fn find_field(lines: &[&str], key: &str) -> (Option<String>, usize) {
    for (i, line) in lines.iter().enumerate() {
        if let Some(value) = line.strip_prefix(key) {
            let value = value.trim();
            if !value.is_empty() {
                return (Some(value.to_owned()), i + 1);
            }
        }
    }
    (None, 0)
}

/// Find `Tier:` field.
fn find_tier_field(lines: &[&str]) -> (Option<Tier>, usize) {
    for (i, line) in lines.iter().enumerate() {
        if let Some(value) = line.strip_prefix("Tier:") {
            let value = value.trim();
            return (Tier::parse(value), i + 1);
        }
    }
    (None, 0)
}

/// Find the `## Status` section and parse the status line.
fn find_status(lines: &[&str]) -> (Option<Status>, usize, Option<String>) {
    let mut in_status = false;
    for (i, line) in lines.iter().enumerate() {
        if *line == "## Status" {
            in_status = true;
            continue;
        }
        if in_status {
            // Skip blank lines between heading and content
            if line.is_empty() {
                continue;
            }
            // Stop at next heading
            if line.starts_with("## ") {
                break;
            }
            let raw = (*line).to_owned();
            let status = Status::parse(line);
            return (Some(status), i + 1, Some(raw));
        }
    }
    (None, 0, None)
}

/// Find all relationships in the `## Related` section.
fn find_relationships(lines: &[&str]) -> (Vec<Relationship>, bool, bool) {
    let mut rels = Vec::new();
    let mut in_related = false;
    let mut found_section = false;
    let mut has_placeholder = false;

    for (i, line) in lines.iter().enumerate() {
        if *line == "## Related" {
            in_related = true;
            found_section = true;
            continue;
        }
        if in_related {
            if line.is_empty() {
                continue;
            }
            if line.starts_with("## ") {
                break;
            }
            // Detect `—` or `- —` placeholder for empty Related
            let trimmed = line.trim();
            if trimmed == "—" || trimmed == "- —" {
                has_placeholder = true;
                continue;
            }
            // Parse "- Verb: TARGET1, TARGET2"
            if let Some(rest) = line.strip_prefix("- ")
                && let Some(colon_pos) = rest.find(": ")
            {
                let verb_str = &rest[..colon_pos];
                let targets_str = &rest[colon_pos + 2..];

                if let Some(verb) = RelVerb::parse(verb_str) {
                    for target_str in targets_str.split(", ") {
                        // Strip annotations like "(indirect)" or "(`#[non_exhaustive]`)"
                        let clean = strip_annotation(target_str);
                        if let Some(target_id) = parse_adr_id_from_str(clean) {
                            rels.push(Relationship {
                                verb,
                                target: target_id,
                                line: i + 1,
                            });
                        }
                    }
                }
            }
        }
    }

    (rels, found_section, has_placeholder)
}

/// Analyze H2 sections: extract ordering and word counts.
///
/// Word counts exclude fenced code blocks.
fn analyze_sections(lines: &[&str]) -> (Vec<String>, HashMap<String, usize>) {
    let mut order = Vec::new();
    let mut word_counts: HashMap<String, usize> = HashMap::new();

    let mut current_section: Option<String> = None;
    let mut current_words = 0usize;
    let mut in_code_block = false;

    for line in lines {
        // Track code block boundaries
        if line.starts_with("```") {
            in_code_block = !in_code_block;
            continue;
        }

        if in_code_block {
            continue;
        }

        if let Some(heading) = line.strip_prefix("## ") {
            // Flush previous section
            if let Some(ref section) = current_section {
                word_counts.insert(section.clone(), current_words);
            }

            let name = heading.trim().to_owned();
            order.push(name.clone());
            current_section = Some(name);
            current_words = 0;
        } else if current_section.is_some() && !line.is_empty() {
            // Count words in non-empty, non-heading, non-code lines
            current_words += line.split_whitespace().count();
        }
    }

    // Flush last section
    if let Some(ref section) = current_section {
        word_counts.insert(section.clone(), current_words);
    }

    (order, word_counts)
}

/// Parse `Crates: crate-a, crate-b` from the metadata preamble.
///
/// Returns an empty vec if the field is absent or empty.
fn find_crates_field(lines: &[&str]) -> Vec<String> {
    for line in lines {
        if let Some(value) = line.strip_prefix("Crates:") {
            let value = value.trim();
            if value.is_empty() {
                return Vec::new();
            }
            return value.split(',').map(|s| s.trim().to_owned()).collect();
        }
        // Stop searching at first H2 — metadata preamble is above sections
        if line.starts_with("## ") {
            break;
        }
    }
    Vec::new()
}

/// Extract the full text of the Decision section (for R0 fallback).
fn extract_decision_content(lines: &[&str]) -> Option<String> {
    let mut in_decision = false;
    let mut content = Vec::new();

    for line in lines {
        if *line == "## Decision" {
            in_decision = true;
            continue;
        }
        if in_decision {
            if line.starts_with("## ") {
                break;
            }
            content.push(*line);
        }
    }

    if content.is_empty() {
        return None;
    }

    // Trim leading/trailing blank lines
    let text = content.join("\n").trim().to_owned();
    if text.is_empty() { None } else { Some(text) }
}

/// Extract tagged rules from the Decision section.
///
/// Matches `- **RN**: text` pattern within the Decision section.
/// When no tagged rules are found, produces a single R0 fallback
/// with the full decision content.
fn extract_tagged_rules(lines: &[&str], decision_content: Option<&String>) -> Vec<TaggedRule> {
    let rule_re = Regex::new(r"^\s*-\s*\*\*R(\d+)\*\*:\s*(.+)").expect("valid regex");
    let mut rules = Vec::new();
    let mut in_decision = false;

    for (i, line) in lines.iter().enumerate() {
        if *line == "## Decision" {
            in_decision = true;
            continue;
        }
        if in_decision {
            if line.starts_with("## ") {
                break;
            }
            if let Some(caps) = rule_re.captures(line) {
                let num = caps.get(1).unwrap().as_str();
                let text = caps.get(2).unwrap().as_str().trim().to_owned();
                rules.push(TaggedRule {
                    id: format!("R{num}"),
                    text,
                    line: i + 1,
                });
            }
        }
    }

    // R0 fallback: when no tagged rules found, use full decision text
    if rules.is_empty()
        && let Some(content) = decision_content
    {
        rules.push(TaggedRule {
            id: "R0".into(),
            text: content.clone(),
            line: 0,
        });
    }

    rules
}

/// Strip trailing annotations like ` (indirect)` or `` (`#[non_exhaustive]`) ``.
fn strip_annotation(s: &str) -> &str {
    let s = s.trim();
    if let Some(paren_start) = s.find(" (") {
        s[..paren_start].trim()
    } else {
        s
    }
}

/// Measure fenced code blocks (triple-backtick delimiters).
///
/// Returns `(max_lines, block_count, max_block_start_line)` where
/// `max_block_start_line` is the 1-indexed line number of the opening
/// fence of the largest block (0 if no blocks or all blocks are empty).
/// Fence lines themselves are excluded from the count. Language
/// annotations on opening fences (e.g., ` ```rust `) are ignored.
///
/// Known limitation: nested backticks (markdown documenting markdown)
/// cause false open/close toggling. Acceptable for ADR content.
fn measure_code_blocks(lines: &[&str]) -> (usize, usize, usize) {
    let mut in_block = false;
    let mut current_lines = 0usize;
    let mut current_start = 0usize; // 1-indexed
    let mut max_lines = 0usize;
    let mut max_start = 0usize;
    let mut block_count = 0usize;

    for (i, line) in lines.iter().enumerate() {
        if line.starts_with("```") {
            if in_block {
                // Closing fence
                if current_lines > max_lines {
                    max_lines = current_lines;
                    max_start = current_start;
                }
                in_block = false;
            } else {
                // Opening fence
                in_block = true;
                current_lines = 0;
                current_start = i + 1; // 1-indexed
                block_count += 1;
            }
        } else if in_block {
            current_lines += 1;
        }
    }

    // Unclosed block — count what we have
    if in_block && current_lines > max_lines {
        max_lines = current_lines;
        max_start = current_start;
    }

    (max_lines, block_count, max_start)
}

/// Check if a `## Heading` exists.
fn has_heading(lines: &[&str], name: &str) -> bool {
    let target = format!("## {name}");
    lines.iter().any(|line| *line == target)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_title_extracts_id_and_text() {
        let lines = vec![
            "# CHE-0042. Event Envelope Construction Invariants",
            "",
            "Date: 2026-04-25",
        ];
        let (id, title, line) = parse_title(&lines, "CHE").unwrap();
        assert_eq!(id.prefix, "CHE");
        assert_eq!(id.number, 42);
        assert_eq!(title, "Event Envelope Construction Invariants");
        assert_eq!(line, 1);
    }

    #[test]
    fn parse_title_wrong_prefix_returns_none() {
        let lines = vec!["# PAR-0001. Some Title"];
        assert!(parse_title(&lines, "CHE").is_none());
    }

    #[test]
    fn find_field_extracts_date() {
        let lines = vec![
            "# Title",
            "",
            "Date: 2026-04-25",
            "Last-reviewed: 2026-04-25",
        ];
        let (date, line) = find_field(&lines, "Date:");
        assert_eq!(date.as_deref(), Some("2026-04-25"));
        assert_eq!(line, 3);
    }

    #[test]
    fn find_status_parses_accepted() {
        let lines = vec!["## Status", "", "Accepted", "", "## Related"];
        let (status, line, raw) = find_status(&lines);
        assert_eq!(status, Some(Status::Accepted));
        assert_eq!(line, 3);
        assert_eq!(raw.as_deref(), Some("Accepted"));
    }

    #[test]
    fn find_status_parses_rejected() {
        let lines = vec!["## Status", "", "Rejected", "", "## Related"];
        let (status, _, _) = find_status(&lines);
        assert_eq!(status, Some(Status::Rejected));
    }

    #[test]
    fn find_relationships_parses_multi_target() {
        let lines = vec![
            "## Related",
            "",
            "- Depends on: CHE-0006, CHE-0032",
            "- Referenced by: CHE-0036 (indirect), CHE-0043",
            "",
            "## Context",
        ];
        let (rels, found, _placeholder) = find_relationships(&lines);
        assert!(found);
        assert_eq!(rels.len(), 4);
        assert_eq!(rels[0].verb, RelVerb::DependsOn);
        assert_eq!(rels[0].target, parse_adr_id_from_str("CHE-0006").unwrap());
        assert_eq!(rels[1].target, parse_adr_id_from_str("CHE-0032").unwrap());
        assert_eq!(rels[2].verb, RelVerb::ReferencedBy);
        assert_eq!(rels[2].target, parse_adr_id_from_str("CHE-0036").unwrap());
        assert_eq!(rels[3].target, parse_adr_id_from_str("CHE-0043").unwrap());
    }

    #[test]
    fn find_relationships_parses_root_verb() {
        let lines = vec!["## Related", "", "- Root: CHE-0001", "", "## Context"];
        let (rels, found, _) = find_relationships(&lines);
        assert!(found);
        assert_eq!(rels.len(), 1);
        assert_eq!(rels[0].verb, RelVerb::Root);
        assert_eq!(rels[0].target.prefix, "CHE");
        assert_eq!(rels[0].target.number, 1);
    }

    #[test]
    fn strip_annotation_removes_parenthetical() {
        assert_eq!(strip_annotation("CHE-0036 (indirect)"), "CHE-0036");
        assert_eq!(strip_annotation("CHE-0021"), "CHE-0021");
        assert_eq!(
            strip_annotation("CHE-0021 (`#[non_exhaustive]`)"),
            "CHE-0021"
        );
    }

    #[test]
    fn has_heading_finds_section() {
        let lines = vec![
            "## Status",
            "",
            "Accepted",
            "",
            "## Context",
            "",
            "Some text.",
        ];
        assert!(has_heading(&lines, "Context"));
        assert!(!has_heading(&lines, "Decision"));
    }

    #[test]
    fn has_heading_finds_retirement() {
        let lines = vec!["## Retirement", "", "Deprecated because reasons."];
        assert!(has_heading(&lines, "Retirement"));
    }

    #[test]
    fn measure_code_blocks_counts_lines() {
        let lines = vec![
            "some text",
            "```rust",
            "fn main() {}",
            "let x = 1;",
            "let y = 2;",
            "```",
            "more text",
        ];
        let (max, count, start) = measure_code_blocks(&lines);
        assert_eq!(max, 3, "3 lines between fences");
        assert_eq!(count, 1);
        assert_eq!(start, 2, "opening fence is line 2 (1-indexed)");
    }

    #[test]
    fn measure_code_blocks_multiple_blocks() {
        let lines = vec![
            "```", "line1", "```", "text", "```rust", "a", "b", "c", "d", "e", "```",
        ];
        let (max, count, start) = measure_code_blocks(&lines);
        assert_eq!(max, 5, "second block has 5 lines");
        assert_eq!(count, 2);
        assert_eq!(start, 5, "second block opens at line 5 (1-indexed)");
    }

    #[test]
    fn measure_code_blocks_empty_block() {
        let lines = vec!["```", "```"];
        let (max, count, start) = measure_code_blocks(&lines);
        assert_eq!(max, 0);
        assert_eq!(count, 1);
        // Empty block has 0 content lines — no "largest block" to point to
        assert_eq!(start, 0);
    }

    #[test]
    fn measure_code_blocks_no_blocks() {
        let lines = vec!["some text", "more text"];
        let (max, count, start) = measure_code_blocks(&lines);
        assert_eq!(max, 0);
        assert_eq!(count, 0);
        assert_eq!(start, 0, "no blocks means start line 0");
    }

    #[test]
    fn measure_code_blocks_fence_lines_excluded() {
        // Opening and closing fence lines should not be counted
        let lines = vec!["```", "only_this", "```"];
        let (max, count, start) = measure_code_blocks(&lines);
        assert_eq!(max, 1, "only content line counted, not fences");
        assert_eq!(count, 1);
        assert_eq!(start, 1);
    }

    #[test]
    fn measure_code_blocks_unclosed_block() {
        let lines = vec!["text", "```rust", "fn main() {}", "let x = 1;"];
        let (max, count, start) = measure_code_blocks(&lines);
        assert_eq!(max, 2, "unclosed block has 2 content lines");
        assert_eq!(count, 1);
        assert_eq!(start, 2, "opening fence at line 2 (1-indexed)");
    }

    #[test]
    fn find_relationships_detects_placeholder() {
        let lines = vec!["## Related", "", "- —", "", "## Context"];
        let (rels, found, placeholder) = find_relationships(&lines);
        assert!(found);
        assert!(rels.is_empty());
        assert!(placeholder, "should detect `- —` as placeholder");
    }

    #[test]
    fn find_relationships_detects_bare_dash_placeholder() {
        let lines = vec!["## Related", "", "—", "", "## Context"];
        let (rels, found, placeholder) = find_relationships(&lines);
        assert!(found);
        assert!(rels.is_empty());
        assert!(placeholder, "should detect bare `—` as placeholder");
    }

    #[test]
    fn find_relationships_no_placeholder_with_rels() {
        let lines = vec!["## Related", "", "- Depends on: CHE-0001", "", "## Context"];
        let (rels, found, placeholder) = find_relationships(&lines);
        assert!(found);
        assert_eq!(rels.len(), 1);
        assert!(
            !placeholder,
            "should not detect placeholder when rels exist"
        );
    }

    #[test]
    fn analyze_sections_correct_order() {
        let lines = vec![
            "# CHE-0001. Title",
            "",
            "## Status",
            "",
            "Accepted",
            "",
            "## Related",
            "",
            "- Root: CHE-0001",
            "",
            "## Context",
            "",
            "This is the context with enough words to pass validation easily.",
            "",
            "## Decision",
            "",
            "We decided to do this thing because it makes sense to us.",
            "",
            "## Consequences",
            "",
            "This makes testing easier and code more maintainable overall.",
        ];
        let (order, counts) = analyze_sections(&lines);
        assert_eq!(
            order,
            vec!["Status", "Related", "Context", "Decision", "Consequences"]
        );
        assert_eq!(counts["Context"], 11);
        assert_eq!(counts["Decision"], 12);
        assert_eq!(counts["Consequences"], 9);
    }

    #[test]
    fn analyze_sections_excludes_code_blocks() {
        let lines = vec![
            "## Decision",
            "",
            "We decided to use this approach.",
            "",
            "```rust",
            "fn main() {",
            "    println!(\"hello\");",
            "}",
            "```",
            "",
            "That is all.",
        ];
        let (_, counts) = analyze_sections(&lines);
        // Only prose words counted: "We decided to use this approach." (6) + "That is all." (3) = 9
        assert_eq!(counts["Decision"], 9);
    }

    #[test]
    fn analyze_sections_with_retirement() {
        let lines = vec![
            "## Status",
            "",
            "Deprecated",
            "",
            "## Retirement",
            "",
            "Deprecated because the transport layer moved to a different protocol entirely.",
        ];
        let (order, counts) = analyze_sections(&lines);
        assert!(order.contains(&"Retirement".to_owned()));
        assert_eq!(counts["Retirement"], 11);
    }

    #[test]
    fn self_referencing_detected() {
        let lines = vec!["## Related", "", "- Root: CHE-0001", "", "## Context"];
        let (rels, _, _) = find_relationships(&lines);
        let id = AdrId {
            prefix: "CHE".into(),
            number: 1,
        };
        let is_self_ref = rels
            .iter()
            .any(|rel| rel.verb == RelVerb::Root && rel.target == id);
        assert!(is_self_ref);
    }

    #[test]
    fn self_referencing_wrong_id_not_detected() {
        let lines = vec!["## Related", "", "- Root: CHE-0002", "", "## Context"];
        let (rels, _, _) = find_relationships(&lines);
        let id = AdrId {
            prefix: "CHE".into(),
            number: 1,
        };
        let is_self_ref = rels
            .iter()
            .any(|rel| rel.verb == RelVerb::Root && rel.target == id);
        assert!(!is_self_ref);
    }

    #[test]
    fn find_crates_field_present() {
        let lines = vec![
            "# CHE-0042. Title",
            "",
            "Date: 2026-04-25",
            "Crates: cherry-pit-core, cherry-pit-gateway",
            "Tier: A",
            "",
            "## Status",
        ];
        let crates = find_crates_field(&lines);
        assert_eq!(crates, vec!["cherry-pit-core", "cherry-pit-gateway"]);
    }

    #[test]
    fn find_crates_field_empty() {
        let lines = vec!["# CHE-0042. Title", "", "Crates:", "", "## Status"];
        let crates = find_crates_field(&lines);
        assert!(crates.is_empty());
    }

    #[test]
    fn find_crates_field_absent() {
        let lines = vec!["# CHE-0042. Title", "", "Date: 2026-04-25", "", "## Status"];
        let crates = find_crates_field(&lines);
        assert!(crates.is_empty());
    }

    #[test]
    fn extract_decision_content_basic() {
        let lines = vec![
            "## Decision",
            "",
            "We decided to use event sourcing.",
            "This provides full auditability.",
            "",
            "## Consequences",
        ];
        let content = extract_decision_content(&lines);
        assert_eq!(
            content.as_deref(),
            Some("We decided to use event sourcing.\nThis provides full auditability.")
        );
    }

    #[test]
    fn extract_decision_content_absent() {
        let lines = vec!["## Context", "", "Some context.", "", "## Consequences"];
        let content = extract_decision_content(&lines);
        assert!(content.is_none());
    }

    #[test]
    fn extract_tagged_rules_normal() {
        let lines = vec![
            "## Decision",
            "",
            "- **R1**: All events must be versioned",
            "- **R2**: Snapshots at 100-event intervals",
            "",
            "## Consequences",
        ];
        let decision_content = extract_decision_content(&lines);
        let rules = extract_tagged_rules(&lines, decision_content.as_ref());
        assert_eq!(rules.len(), 2);
        assert_eq!(rules[0].id, "R1");
        assert_eq!(rules[0].text, "All events must be versioned");
        assert_eq!(rules[1].id, "R2");
        assert_eq!(rules[1].text, "Snapshots at 100-event intervals");
    }

    #[test]
    fn extract_tagged_rules_r0_fallback() {
        let lines = vec![
            "## Decision",
            "",
            "We use event sourcing for persistence.",
            "",
            "## Consequences",
        ];
        let decision_content = extract_decision_content(&lines);
        let rules = extract_tagged_rules(&lines, decision_content.as_ref());
        assert_eq!(rules.len(), 1);
        assert_eq!(rules[0].id, "R0");
        assert_eq!(rules[0].text, "We use event sourcing for persistence.");
    }

    #[test]
    fn extract_tagged_rules_mixed_with_prose() {
        let lines = vec![
            "## Decision",
            "",
            "We adopt the following rules:",
            "",
            "- **R1**: Events are append-only",
            "Some prose between rules.",
            "- **R2**: Snapshots are optional",
            "",
            "## Consequences",
        ];
        let decision_content = extract_decision_content(&lines);
        let rules = extract_tagged_rules(&lines, decision_content.as_ref());
        assert_eq!(rules.len(), 2);
        assert_eq!(rules[0].id, "R1");
        assert_eq!(rules[1].id, "R2");
    }

    #[test]
    fn extract_tagged_rules_malformed_ignored() {
        let lines = vec![
            "## Decision",
            "",
            "- **Rfoo**: Not a valid rule tag",
            "- **R1**: Valid rule",
            "",
            "## Consequences",
        ];
        let decision_content = extract_decision_content(&lines);
        let rules = extract_tagged_rules(&lines, decision_content.as_ref());
        assert_eq!(rules.len(), 1);
        assert_eq!(rules[0].id, "R1");
    }
}
