//! Template compliance rules (T001–T012).
//!
//! T001: H1 title present
//! T002: Date field present
//! T003: Last-reviewed field (error for S/A-tier, warning for others)
//! T004: Tier field present
//! T005: Status section present
//! T006: Status value valid (no parentheticals, known variant)
//! T007: Related section present (empty section requires `—` placeholder)
//! T008: Context section present
//! T009: Decision section present
//! T010: Consequences section present
//! T011: Code block exceeds 20 lines (warning)
//! T012: Amendment date ≥ Date (amendment cannot predate ADR creation)

use crate::model::{AdrRecord, Status};
use crate::report::Diagnostic;

/// Maximum lines in a single fenced code block before T011 fires.
const MAX_CODE_BLOCK_LINES: usize = 20;

/// ISO 8601 date format: YYYY-MM-DD.
fn is_valid_date_format(s: &str) -> bool {
    if s.len() != 10 {
        return false;
    }
    let bytes = s.as_bytes();
    bytes[4] == b'-'
        && bytes[7] == b'-'
        && bytes[..4].iter().all(|b| b.is_ascii_digit())
        && bytes[5..7].iter().all(|b| b.is_ascii_digit())
        && bytes[8..10].iter().all(|b| b.is_ascii_digit())
}

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
    } else if record.relationships.is_empty() && !record.related_has_placeholder {
        diags.push(Diagnostic::warning(
            "T007",
            &record.file_path,
            0,
            "Related section has no relationships and no `— ` placeholder — \
             add `- —` for ADRs with no dependencies"
                .into(),
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

    // T011: Code block length
    if record.max_code_block_lines > MAX_CODE_BLOCK_LINES {
        diags.push(Diagnostic::warning(
            "T011",
            &record.file_path,
            record.max_code_block_line,
            format!(
                "code block has {} lines (max {}). \
                 Use signatures or pseudocode; reference source files \
                 for full implementations.",
                record.max_code_block_lines, MAX_CODE_BLOCK_LINES,
            ),
        ));
    }

    // T012: Amendment date ≥ Date
    if let Some(ref date) = record.date {
        for (amendment_date, line) in &record.amendment_dates {
            if !is_valid_date_format(amendment_date) {
                diags.push(Diagnostic::warning(
                    "T012",
                    &record.file_path,
                    *line,
                    format!(
                        "amendment date `{amendment_date}` is not valid \
                         ISO 8601 (expected YYYY-MM-DD)"
                    ),
                ));
                continue;
            }
            if amendment_date.as_str() < date.as_str() {
                diags.push(Diagnostic::error(
                    "T012",
                    &record.file_path,
                    *line,
                    format!(
                        "amendment date `{amendment_date}` predates \
                         ADR creation date `{date}` — amendment dates \
                         must be ≥ Date"
                    ),
                ));
            }
        }
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
            max_code_block_lines: 0,
            max_code_block_line: 0,
            code_block_count: 0,
            amendment_dates: vec![],
            related_has_placeholder: true,
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

    #[test]
    fn code_block_at_limit_no_t011() {
        let mut record = make_record();
        record.max_code_block_lines = 20;
        let mut diags = Vec::new();
        check(&record, &mut diags);
        assert!(
            !diags.iter().any(|d| d.rule == "T011"),
            "20 lines should not trigger T011"
        );
    }

    #[test]
    fn code_block_over_limit_produces_t011() {
        let mut record = make_record();
        record.max_code_block_lines = 21;
        record.max_code_block_line = 42;
        let mut diags = Vec::new();
        check(&record, &mut diags);
        let t011 = diags.iter().find(|d| d.rule == "T011");
        assert!(t011.is_some(), "expected T011, got: {diags:?}");
        assert_eq!(
            t011.unwrap().severity,
            crate::report::Severity::Warning,
            "T011 should be warning, not error"
        );
        assert_eq!(t011.unwrap().line, 42, "T011 should point to opening fence");
    }

    #[test]
    fn empty_related_without_placeholder_produces_t007() {
        let mut record = make_record();
        record.has_related = true;
        record.relationships = vec![];
        record.related_has_placeholder = false;
        let mut diags = Vec::new();
        check(&record, &mut diags);
        let t007 = diags.iter().find(|d| d.rule == "T007");
        assert!(t007.is_some(), "expected T007, got: {diags:?}");
        assert_eq!(t007.unwrap().severity, crate::report::Severity::Warning);
    }

    #[test]
    fn empty_related_with_placeholder_no_t007() {
        let mut record = make_record();
        record.has_related = true;
        record.relationships = vec![];
        record.related_has_placeholder = true;
        let mut diags = Vec::new();
        check(&record, &mut diags);
        assert!(
            !diags.iter().any(|d| d.rule == "T007"),
            "placeholder should suppress T007"
        );
    }

    #[test]
    fn amendment_date_before_creation_produces_t012() {
        let mut record = make_record();
        record.date = Some("2026-04-25".into());
        record.amendment_dates = vec![("2026-04-01".into(), 12)];
        let mut diags = Vec::new();
        check(&record, &mut diags);
        let t012 = diags.iter().find(|d| d.rule == "T012");
        assert!(t012.is_some(), "expected T012, got: {diags:?}");
        assert_eq!(t012.unwrap().severity, crate::report::Severity::Error);
        assert_eq!(t012.unwrap().line, 12);
    }

    #[test]
    fn amendment_date_equal_to_creation_no_t012() {
        let mut record = make_record();
        record.date = Some("2026-04-25".into());
        record.amendment_dates = vec![("2026-04-25".into(), 12)];
        let mut diags = Vec::new();
        check(&record, &mut diags);
        assert!(
            !diags.iter().any(|d| d.rule == "T012"),
            "same-day amendment should not trigger T012"
        );
    }

    #[test]
    fn amendment_date_after_creation_no_t012() {
        let mut record = make_record();
        record.date = Some("2026-04-25".into());
        record.amendment_dates = vec![("2026-05-01".into(), 12)];
        let mut diags = Vec::new();
        check(&record, &mut diags);
        assert!(
            !diags.iter().any(|d| d.rule == "T012"),
            "future amendment should not trigger T012"
        );
    }

    #[test]
    fn malformed_amendment_date_produces_t012_warning() {
        let mut record = make_record();
        record.date = Some("2026-04-25".into());
        record.amendment_dates = vec![("not-a-date".into(), 12)];
        let mut diags = Vec::new();
        check(&record, &mut diags);
        let t012 = diags.iter().find(|d| d.rule == "T012");
        assert!(t012.is_some(), "expected T012 for malformed date");
        assert_eq!(t012.unwrap().severity, crate::report::Severity::Warning);
    }
}
