//! File naming rules (N001–N003).
//!
//! N001: Filename must match `PREFIX-NNNN-kebab-slug.md`
//! N002: Number in filename must match H1 title ID
//! N003: Slug must be lowercase kebab-case (a-z0-9, hyphens)

use regex::Regex;

use crate::model::{parse_adr_id_from_str, AdrRecord};
use crate::report::Diagnostic;

pub fn check(record: &AdrRecord, diags: &mut Vec<Diagnostic>) {
    let Some(file_name) = record.file_path.file_name().and_then(|f| f.to_str()) else {
        return;
    };

    // N001: Overall pattern
    let pattern = Regex::new(r"^[A-Z]{2,4}-\d{4}-[a-z0-9]+(?:-[a-z0-9]+)*\.md$")
        .expect("valid regex");

    if !pattern.is_match(file_name) {
        diags.push(Diagnostic::error(
            "N001",
            &record.file_path,
            0,
            format!(
                "filename `{file_name}` does not match pattern \
                 `PREFIX-NNNN-kebab-slug.md`"
            ),
        ));
        return; // N002 and N003 depend on valid filename structure
    }

    // N002: Number in filename matches H1 ID
    if let Some(file_id) = parse_adr_id_from_str(&file_name[..file_name.len() - 3]) {
        // We need the first segment PREFIX-NNNN
        if file_id.prefix != record.id.prefix || file_id.number != record.id.number {
            diags.push(Diagnostic::error(
                "N002",
                &record.file_path,
                record.title_line,
                format!(
                    "filename ID `{file_id}` does not match H1 title ID `{}`",
                    record.id
                ),
            ));
        }
    }

    // N003: Slug is kebab-case
    // Extract slug portion after "PREFIX-NNNN-"
    // "CHE-0001-" → prefix.len() (3) + 1 (-) + 4 (digits) + 1 (-) = len+6
    let prefix_len = record.id.prefix.len() + 6;
    let slug_with_ext = &file_name[prefix_len..];
    let slug = slug_with_ext.strip_suffix(".md").unwrap_or(slug_with_ext);

    let kebab = Regex::new(r"^[a-z0-9]+(?:-[a-z0-9]+)*$").expect("valid regex");
    if !kebab.is_match(slug) {
        diags.push(Diagnostic::error(
            "N003",
            &record.file_path,
            0,
            format!("slug `{slug}` is not valid kebab-case (a-z0-9, hyphens only)"),
        ));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{AdrId, Status, Tier};
    use std::path::PathBuf;

    fn make_record(filename: &str, prefix: &str, num: u16) -> AdrRecord {
        AdrRecord {
            id: AdrId {
                prefix: prefix.into(),
                number: num,
            },
            file_path: PathBuf::from(format!("docs/adr/framework/{filename}")),
            title: Some("Test".into()),
            title_line: 1,
            date: Some("2026-04-25".into()),
            date_line: 3,
            last_reviewed: Some("2026-04-25".into()),
            last_reviewed_line: 4,
            tier: Some(Tier::B),
            tier_line: 5,
            status: Some(Status::Accepted),
            status_line: 8,
            status_raw: Some("Accepted".into()),
            relationships: vec![],
            has_related: true,
            has_context: true,
            has_decision: true,
            has_consequences: true,
        }
    }

    #[test]
    fn valid_filename_no_diagnostics() {
        let record = make_record("CHE-0001-design-priority-ordering.md", "CHE", 1);
        let mut diags = Vec::new();
        check(&record, &mut diags);
        assert!(diags.is_empty(), "expected no diags, got: {diags:?}");
    }

    #[test]
    fn uppercase_slug_produces_n001() {
        let record = make_record("CHE-0001-Design-Priority.md", "CHE", 1);
        let mut diags = Vec::new();
        check(&record, &mut diags);
        assert!(
            diags.iter().any(|d| d.rule == "N001"),
            "expected N001, got: {diags:?}"
        );
    }

    #[test]
    fn mismatched_number_produces_n002() {
        let record = make_record("CHE-0099-test.md", "CHE", 1);
        let mut diags = Vec::new();
        check(&record, &mut diags);
        assert!(
            diags.iter().any(|d| d.rule == "N002"),
            "expected N002, got: {diags:?}"
        );
    }
}
