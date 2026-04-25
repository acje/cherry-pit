//! Template compliance rules (T001–T010).
//!
//! T001: H1 title present
//! T002: Date field present
//! T003: Last-reviewed field (error for S/A-tier, warning for others)
//! T004: Tier field present
//! T005: Status section present
//! T006: Status value valid (no parentheticals, known variant)
//! T007: Related section present
//! T008: Context section present
//! T009: Decision section present
//! T010: Consequences section present

use crate::model::{AdrRecord, Status};
use crate::report::Diagnostic;

pub fn check(record: &AdrRecord, diags: &mut Vec<Diagnostic>) {
    // T001: H1 title
    if record.title.is_none() {
        diags.push(Diagnostic::error(
            "T001",
            &record.file_path,
            1,
            "missing H1 title line (expected `# PREFIX-NNNN. Title`)".into(),
        ));
    }

    // T002: Date
    if record.date.is_none() {
        diags.push(Diagnostic::error(
            "T002",
            &record.file_path,
            0,
            "missing `Date:` field".into(),
        ));
    }

    // T003: Last-reviewed (severity depends on tier)
    if record.last_reviewed.is_none() {
        let requires = record
            .tier
            .map_or(false, |t| t.requires_last_reviewed());
        if requires {
            diags.push(Diagnostic::error(
                "T003",
                &record.file_path,
                0,
                format!(
                    "missing `Last-reviewed:` field (required for tier {:?})",
                    record.tier.unwrap()
                ),
            ));
        } else {
            diags.push(Diagnostic::warning(
                "T003",
                &record.file_path,
                0,
                "missing `Last-reviewed:` field".into(),
            ));
        }
    }

    // T004: Tier
    if record.tier.is_none() {
        diags.push(Diagnostic::error(
            "T004",
            &record.file_path,
            0,
            "missing `Tier:` field".into(),
        ));
    }

    // T005: Status section
    if record.status.is_none() {
        diags.push(Diagnostic::error(
            "T005",
            &record.file_path,
            0,
            "missing `## Status` section or status line".into(),
        ));
    }

    // T006: Status value validity
    if let Some(ref raw) = record.status_raw {
        if Status::has_parenthetical(raw) {
            diags.push(Diagnostic::error(
                "T006",
                &record.file_path,
                record.status_line,
                format!(
                    "status line contains parenthetical annotation: `{raw}` — \
                     use `Amended YYYY-MM-DD — note` format instead"
                ),
            ));
        }
        if let Some(Status::Invalid(ref s)) = record.status {
            diags.push(Diagnostic::error(
                "T006",
                &record.file_path,
                record.status_line,
                format!(
                    "unrecognized status: `{s}` — expected one of: \
                     Draft, Proposed, Accepted, Amended [date — note], \
                     Deprecated, Superseded by PREFIX-NNNN"
                ),
            ));
        }
    }

    // T007: Related section
    if !record.has_related {
        diags.push(Diagnostic::warning(
            "T007",
            &record.file_path,
            0,
            "missing `## Related` section".into(),
        ));
    }

    // T008: Context section
    if !record.has_context {
        diags.push(Diagnostic::error(
            "T008",
            &record.file_path,
            0,
            "missing `## Context` section".into(),
        ));
    }

    // T009: Decision section
    if !record.has_decision {
        diags.push(Diagnostic::error(
            "T009",
            &record.file_path,
            0,
            "missing `## Decision` section".into(),
        ));
    }

    // T010: Consequences section
    if !record.has_consequences {
        diags.push(Diagnostic::error(
            "T010",
            &record.file_path,
            0,
            "missing `## Consequences` section".into(),
        ));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{AdrId, Tier};
    use std::path::PathBuf;

    fn make_record() -> AdrRecord {
        AdrRecord {
            id: AdrId {
                prefix: "CHE".into(),
                number: 1,
            },
            file_path: PathBuf::from("test.md"),
            title: Some("Test".into()),
            title_line: 1,
            date: Some("2026-04-25".into()),
            date_line: 3,
            last_reviewed: Some("2026-04-25".into()),
            last_reviewed_line: 4,
            tier: Some(Tier::S),
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
    fn valid_record_produces_no_diagnostics() {
        let record = make_record();
        let mut diags = Vec::new();
        check(&record, &mut diags);
        assert!(diags.is_empty(), "expected no diags, got: {diags:?}");
    }

    #[test]
    fn missing_tier_produces_t004() {
        let mut record = make_record();
        record.tier = None;
        let mut diags = Vec::new();
        check(&record, &mut diags);
        assert!(diags.iter().any(|d| d.rule == "T004"));
    }

    #[test]
    fn missing_last_reviewed_s_tier_is_error() {
        let mut record = make_record();
        record.tier = Some(Tier::S);
        record.last_reviewed = None;
        let mut diags = Vec::new();
        check(&record, &mut diags);
        let t003 = diags.iter().find(|d| d.rule == "T003").unwrap();
        assert_eq!(t003.severity, crate::report::Severity::Error);
    }

    #[test]
    fn missing_last_reviewed_c_tier_is_warning() {
        let mut record = make_record();
        record.tier = Some(Tier::C);
        record.last_reviewed = None;
        let mut diags = Vec::new();
        check(&record, &mut diags);
        let t003 = diags.iter().find(|d| d.rule == "T003").unwrap();
        assert_eq!(t003.severity, crate::report::Severity::Warning);
    }

    #[test]
    fn parenthetical_status_produces_t006() {
        let mut record = make_record();
        record.status_raw = Some("Accepted (supersedes original u64 design)".into());
        record.status = Some(Status::Invalid(
            "Accepted (supersedes original u64 design)".into(),
        ));
        let mut diags = Vec::new();
        check(&record, &mut diags);
        assert!(
            diags.iter().any(|d| d.rule == "T006"),
            "expected T006, got: {diags:?}"
        );
    }
}
