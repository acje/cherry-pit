//! ADR file parser — extracts metadata from markdown files.

use std::fs;
use std::path::Path;

use regex::Regex;

use crate::model::{
    parse_adr_id_from_str, AdrId, AdrRecord, DomainDir, RelVerb, Relationship, Status, Tier,
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

        if let Some(record) = parse_adr_file(&entry.path(), &dir.prefix) {
            records.push(record);
        }
    }

    records.sort_by_key(|r| r.id.number);
    records
}

/// Parse a single ADR file into an `AdrRecord`.
pub fn parse_adr_file(path: &Path, expected_prefix: &str) -> Option<AdrRecord> {
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
    let (relationships, has_related) = find_relationships(&lines);

    // --- Required sections ---
    let has_context = has_heading(&lines, "Context");
    let has_decision = has_heading(&lines, "Decision");
    let has_consequences = has_heading(&lines, "Consequences");

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
    })
}

/// Parse the H1 title line: `# PREFIX-NNNN. Title text`.
fn parse_title(lines: &[&str], expected_prefix: &str) -> Option<(AdrId, String, usize)> {
    for (i, line) in lines.iter().enumerate() {
        if let Some(rest) = line.strip_prefix("# ") {
            // Expected format: "PREFIX-NNNN. Title text"
            if let Some(dot_pos) = rest.find(". ") {
                let id_part = &rest[..dot_pos];
                if let Some(id) = parse_adr_id_from_str(id_part) {
                    if id.prefix == expected_prefix {
                        let title = rest[dot_pos + 2..].to_owned();
                        return Some((id, title, i + 1));
                    }
                }
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
fn find_relationships(lines: &[&str]) -> (Vec<Relationship>, bool) {
    let mut rels = Vec::new();
    let mut in_related = false;
    let mut found_section = false;

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
            // Parse "- Verb: TARGET1, TARGET2"
            if let Some(rest) = line.strip_prefix("- ") {
                if let Some(colon_pos) = rest.find(": ") {
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
    }

    (rels, found_section)
}

/// Strip trailing annotations like ` (indirect)` or ` (\`#[non_exhaustive]\`)`.
fn strip_annotation(s: &str) -> &str {
    let s = s.trim();
    if let Some(paren_start) = s.find(" (") {
        s[..paren_start].trim()
    } else {
        s
    }
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
        let lines = vec!["# Title", "", "Date: 2026-04-25", "Last-reviewed: 2026-04-25"];
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
    fn find_relationships_parses_multi_target() {
        let lines = vec![
            "## Related",
            "",
            "- Depends on: CHE-0006, CHE-0032",
            "- Referenced by: CHE-0036 (indirect), CHE-0043",
            "",
            "## Context",
        ];
        let (rels, found) = find_relationships(&lines);
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
        let lines = vec!["## Status", "", "Accepted", "", "## Context", "", "Some text."];
        assert!(has_heading(&lines, "Context"));
        assert!(!has_heading(&lines, "Decision"));
    }
}
