//! README index consistency rules (I001–I003).
//!
//! These are internal assertions verifying that adr-fmt's own README
//! generation produced consistent output. They are not user-facing
//! governance rules — the generated READMEs are the source of truth.
//!
//! I001: ADR file exists on disk but is missing from README.md index
//! I002: README.md references an ADR that does not exist on disk
//! I003: README.md hyperlink target does not resolve to a file on disk

use std::fs;

use regex::Regex;

use crate::model::{parse_adr_id_from_str, AdrId, AdrRecord, DomainDir};
use crate::report::Diagnostic;

pub fn check(dir: &DomainDir, records: &[&AdrRecord], diags: &mut Vec<Diagnostic>) {
    let readme_path = dir.path.join("README.md");

    let Ok(content) = fs::read_to_string(&readme_path) else {
        // No README.md — not an error for this rule set (but odd)
        return;
    };

    let readme_ids = extract_ids_from_readme(&content, &dir.prefix);
    let file_ids: Vec<&AdrId> = records.iter().map(|r| &r.id).collect();

    // I001: file exists but not in README
    for record in records {
        if !readme_ids.contains(&record.id) {
            diags.push(Diagnostic::internal_warning(
                "I001",
                &record.file_path,
                0,
                format!(
                    "{} exists on disk but is not listed in {}",
                    record.id,
                    readme_path.display()
                ),
            ));
        }
    }

    // I002: README references non-existent ADR
    for (readme_id, line) in &readme_ids {
        if !file_ids.contains(&readme_id) {
            diags.push(Diagnostic::internal_warning(
                "I002",
                &readme_path,
                *line,
                format!(
                    "{readme_id} listed in README but no matching ADR file on disk"
                ),
            ));
        }
    }

    // I003: Hyperlink targets in README resolve to files
    let link_targets = extract_link_targets_from_readme(&content);
    for (target, line) in &link_targets {
        // Skip cross-domain relative paths (e.g., ../genome/GEN-0002-...)
        if target.starts_with("../") {
            continue;
        }
        let target_path = dir.path.join(target);
        if !target_path.exists() {
            diags.push(Diagnostic::internal_error(
                "I003",
                &readme_path,
                *line,
                format!(
                    "link target `{target}` does not resolve to a file in {}",
                    dir.path.display()
                ),
            ));
        }
    }
}

/// Extract ADR IDs referenced in the README, along with the line number.
fn extract_ids_from_readme(content: &str, prefix: &str) -> Vec<(AdrId, usize)> {
    let pattern = Regex::new(&format!(r"\b{}-\d{{4}}\b", regex::escape(prefix)))
        .expect("valid regex");

    let mut ids = Vec::new();
    let mut seen = std::collections::HashSet::new();

    for (i, line) in content.lines().enumerate() {
        for m in pattern.find_iter(line) {
            if let Some(id) = parse_adr_id_from_str(m.as_str()) {
                if seen.insert(id.clone()) {
                    ids.push((id, i + 1));
                }
            }
        }
    }
    ids
}

/// Trait to allow `Vec<(AdrId, usize)>` to be checked for containment by id.
trait ContainsId {
    fn contains(&self, id: &AdrId) -> bool;
}

impl ContainsId for Vec<(AdrId, usize)> {
    fn contains(&self, id: &AdrId) -> bool {
        self.iter().any(|(i, _)| i == id)
    }
}

/// Extract Markdown link targets from README index table rows.
///
/// Matches patterns like `[CHE-0001](CHE-0001-design-priority-ordering.md)`
/// and returns the target filename with its 1-indexed line number.
fn extract_link_targets_from_readme(content: &str) -> Vec<(String, usize)> {
    let link_re = Regex::new(r"\[(?:CHE|PAR|GEN|COM)-\d{4}\]\(([^)]+\.md)\)")
        .expect("valid regex");

    let mut targets = Vec::new();
    for (i, line) in content.lines().enumerate() {
        // Only scan table rows (start with `|`)
        if !line.starts_with('|') {
            continue;
        }
        for cap in link_re.captures_iter(line) {
            if let Some(m) = cap.get(1) {
                targets.push((m.as_str().to_owned(), i + 1));
            }
        }
    }
    targets
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extract_ids_finds_che_refs() {
        let content = "\
| CHE-0001 | Design Priority Ordering | S | Accepted |
| CHE-0002 | Illegal States | S | Accepted |
Some text mentioning CHE-0042 inline.
";
        let ids = extract_ids_from_readme(content, "CHE");
        assert_eq!(ids.len(), 3);
        assert_eq!(ids[0].0.number, 1);
        assert_eq!(ids[1].0.number, 2);
        assert_eq!(ids[2].0.number, 42);
    }

    #[test]
    fn extract_ids_deduplicates() {
        let content = "\
CHE-0001 appears here.
And CHE-0001 appears again.
";
        let ids = extract_ids_from_readme(content, "CHE");
        assert_eq!(ids.len(), 1);
    }

    #[test]
    fn extract_link_targets_finds_md_links() {
        let content = "\
| [CHE-0001](CHE-0001-design-priority-ordering.md) | Design Priority | S | Accepted |
| [CHE-0002](CHE-0002-illegal-states.md) | Illegal States | S | Accepted |
Some text with [CHE-0003](CHE-0003-foo.md) not in table.
";
        let targets = extract_link_targets_from_readme(content);
        assert_eq!(targets.len(), 2, "only table rows should match");
        assert_eq!(targets[0].0, "CHE-0001-design-priority-ordering.md");
        assert_eq!(targets[0].1, 1);
        assert_eq!(targets[1].0, "CHE-0002-illegal-states.md");
        assert_eq!(targets[1].1, 2);
    }

    #[test]
    fn extract_link_targets_empty_for_unlinked_table() {
        let content = "\
| CHE-0001 | Design Priority | S | Accepted |
";
        let targets = extract_link_targets_from_readme(content);
        assert!(targets.is_empty(), "no hyperlinks should produce no targets");
    }
}
