//! Template compliance rules (T001–T015) and structure rules (S004–S006).
//!
//! T001: H1 title present
//! T002: Date field present
//! T003: Last-reviewed field required (all tiers)
//! T004: Tier field present
//! T005: Status section present
//! T006: Status value valid (no parentheticals, known variant)
//! T007: Related section with at least one relationship
//! T008: Context section present
//! T009: Decision section present
//! T010: Consequences section present
//! T011: Code block exceeds 20 lines (warning)
//! T012: Amendment date ≥ Date (amendment cannot predate ADR creation)
//! T013: (reserved — Retirement section covers all terminal states)
//! T014: Section ordering — H2 sections in canonical order
//! T015: Section minimum word count (parameterized via TOML)
//! S004: Stale ADR missing Retirement section
//! S005: Active ADR has Retirement section (location/status mismatch)
//! S006: Terminal-status ADR not in stale directory

use crate::config::Config;
use crate::model::{AdrRecord, Status};
use crate::report::Diagnostic;

/// Maximum lines in a single fenced code block before T011 fires.
const MAX_CODE_BLOCK_LINES: usize = 20;

/// Default minimum word count for prose sections.
const DEFAULT_MIN_WORDS: u64 = 10;

/// Canonical H2 section order for active ADRs.
const ACTIVE_SECTION_ORDER: &[&str] = &[
    "Status",
    "Related",
    "Context",
    "Decision",
    "Consequences",
];

/// Canonical H2 section order for stale ADRs (Retirement at end).
const STALE_SECTION_ORDER: &[&str] = &[
    "Status",
    "Related",
    "Context",
    "Decision",
    "Consequences",
    "Retirement",
];

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

pub fn check(record: &AdrRecord, config: &Config, diags: &mut Vec<Diagnostic>) {
    // T001: H1 title
    if record.title.is_none() {
        diags.push(Diagnostic::warning(
            "T001",
            &record.file_path,
            1,
            "missing H1 title line (expected `# PREFIX-NNNN. Title`)".into(),
        ));
    }

    // T002: Date
    if record.date.is_none() {
        diags.push(Diagnostic::warning(
            "T002",
            &record.file_path,
            0,
            "missing `Date:` field".into(),
        ));
    }

    // T003: Last-reviewed — required for all tiers
    if record.last_reviewed.is_none() {
        diags.push(Diagnostic::warning(
            "T003",
            &record.file_path,
            0,
            "missing `Last-reviewed:` field (required for all tiers)".into(),
        ));
    }

    // T004: Tier
    if record.tier.is_none() {
        diags.push(Diagnostic::warning(
            "T004",
            &record.file_path,
            0,
            "missing `Tier:` field".into(),
        ));
    }

    // T005: Status section
    if record.status.is_none() {
        diags.push(Diagnostic::warning(
            "T005",
            &record.file_path,
            0,
            "missing `## Status` section or status line".into(),
        ));
    }

    // T006: Status value validity
    if let Some(ref raw) = record.status_raw {
        if Status::has_parenthetical(raw) {
            diags.push(Diagnostic::warning(
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
            diags.push(Diagnostic::warning(
                "T006",
                &record.file_path,
                record.status_line,
                format!(
                    "unrecognized status: `{s}` — expected one of: \
                     Draft, Proposed, Accepted, Amended [date — note], \
                     Rejected, Deprecated, Superseded by PREFIX-NNNN"
                ),
            ));
        }
    }

    // T007: Related section — must have at least one relationship
    if !record.has_related {
        diags.push(Diagnostic::warning(
            "T007",
            &record.file_path,
            0,
            "missing `## Related` section".into(),
        ));
    } else if record.relationships.is_empty() {
        diags.push(Diagnostic::warning(
            "T007",
            &record.file_path,
            0,
            "Related section has no relationships — every ADR must \
             have at least one relation (use `- Root: ID` for tree roots)"
                .into(),
        ));
    }

    // T008: Context section
    if !record.has_context {
        diags.push(Diagnostic::warning(
            "T008",
            &record.file_path,
            0,
            "missing `## Context` section".into(),
        ));
    }

    // T009: Decision section
    if !record.has_decision {
        diags.push(Diagnostic::warning(
            "T009",
            &record.file_path,
            0,
            "missing `## Decision` section".into(),
        ));
    }

    // T010: Consequences section
    if !record.has_consequences {
        diags.push(Diagnostic::warning(
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
                diags.push(Diagnostic::warning(
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

    // T014: Section ordering
    check_section_order(record, diags);

    // T015: Section minimum word count
    let min_words = config.rule_param_u64("T015", "min_words").unwrap_or(DEFAULT_MIN_WORDS);
    check_section_word_counts(record, min_words, diags);

    // T016: Tagged rules in Decision section
    check_tagged_rules(record, diags);

    // S004: Stale ADR must have Retirement section
    if record.is_stale && !record.has_retirement {
        diags.push(Diagnostic::warning(
            "S004",
            &record.file_path,
            0,
            "stale ADR missing `## Retirement` section — explain why \
             this ADR was retired"
                .into(),
        ));
    }

    // S005: Active ADR must NOT have Retirement section
    if !record.is_stale && record.has_retirement {
        diags.push(Diagnostic::warning(
            "S005",
            &record.file_path,
            0,
            "active ADR has `## Retirement` section — Retirement is \
             only for stale ADRs"
                .into(),
        ));
    }

    // S006: Terminal-status ADR not in stale directory
    if let Some(ref status) = record.status {
        if status.is_terminal() && !record.is_stale {
            let status_display = match status {
                Status::Rejected => "Rejected".to_string(),
                Status::Deprecated => "Deprecated".to_string(),
                Status::SupersededBy(id) => format!("Superseded by {id}"),
                _ => format!("{status:?}"),
            };
            diags.push(Diagnostic::warning(
                "S006",
                &record.file_path,
                record.status_line,
                format!(
                    "{} has terminal status '{status_display}' but is not in the \
                     stale directory. Action: move this file to {stale_dir}/ and add a \
                     `## Retirement` section (≥{min_words} words) explaining why this \
                     ADR left active service.",
                    record.id,
                    stale_dir = config.stale.directory,
                ),
            ));
        }
    }
}

/// T014: H2 sections must appear in canonical order.
///
/// Only validates the relative ordering of known canonical sections.
/// Extra subsections (e.g., `### Rules`) within a section are ignored.
fn check_section_order(record: &AdrRecord, diags: &mut Vec<Diagnostic>) {
    let expected = if record.is_stale {
        STALE_SECTION_ORDER
    } else {
        ACTIVE_SECTION_ORDER
    };

    // Filter section_order to only canonical sections
    let actual: Vec<&str> = record
        .section_order
        .iter()
        .map(String::as_str)
        .filter(|s| expected.contains(s))
        .collect();

    // Check that canonical sections appear in order
    let mut expected_iter = expected.iter();
    for actual_section in &actual {
        // Advance expected_iter to find this section
        let mut found = false;
        for expected_section in expected_iter.by_ref() {
            if actual_section == expected_section {
                found = true;
                break;
            }
        }
        if !found {
            diags.push(Diagnostic::warning(
                "T014",
                &record.file_path,
                0,
                format!(
                    "section `## {actual_section}` is out of canonical order — \
                     expected: {}",
                    expected.join(" → "),
                ),
            ));
            return; // One diagnostic is enough
        }
    }
}

/// T015: Prose sections must meet minimum word count.
fn check_section_word_counts(record: &AdrRecord, min_words: u64, diags: &mut Vec<Diagnostic>) {
    let prose_sections = ["Context", "Decision", "Consequences"];

    for section in &prose_sections {
        if let Some(&count) = record.section_word_counts.get(*section) {
            if (count as u64) < min_words {
                diags.push(Diagnostic::warning(
                    "T015",
                    &record.file_path,
                    0,
                    format!(
                        "`## {section}` has {count} word(s) (minimum {min_words}) — \
                         provide meaningful content"
                    ),
                ));
            }
        }
    }

    // Retirement section also requires min_words if present
    if record.has_retirement {
        if let Some(&count) = record.section_word_counts.get("Retirement") {
            if (count as u64) < min_words {
                diags.push(Diagnostic::warning(
                    "S004",
                    &record.file_path,
                    0,
                    format!(
                        "`## Retirement` has {count} word(s) (minimum {min_words}) — \
                         explain why this ADR was retired"
                    ),
                ));
            }
        }
    }
}

/// T016: Decision section should have tagged rules (`- **RN**: text`).
///
/// Two diagnostic variants:
/// - No tagged rules (decision_rules empty or sole entry is R0)
/// - Non-sequential IDs (gap in R-number sequence)
///
/// Exempt: `Status::Draft` and `Status::Proposed`.
fn check_tagged_rules(record: &AdrRecord, diags: &mut Vec<Diagnostic>) {
    // Exempt Draft and Proposed
    if let Some(ref status) = record.status {
        if matches!(status, Status::Draft | Status::Proposed) {
            return;
        }
    }

    // Check for missing tagged rules
    let has_real_rules = !record.decision_rules.is_empty()
        && !(record.decision_rules.len() == 1 && record.decision_rules[0].id == "R0");

    if !has_real_rules {
        diags.push(Diagnostic::warning(
            "T016",
            &record.file_path,
            0,
            "Decision section lacks tagged rules (- **RN**: pattern)".into(),
        ));
        return;
    }

    // Check for non-sequential IDs
    let mut nums: Vec<u32> = Vec::new();
    for rule in &record.decision_rules {
        if let Some(num_str) = rule.id.strip_prefix('R') {
            if let Ok(num) = num_str.parse::<u32>() {
                nums.push(num);
            }
        }
    }

    nums.sort();
    for (i, &num) in nums.iter().enumerate() {
        let expected = (i as u32) + 1;
        if num != expected {
            let prev = if i > 0 {
                format!("R{}", nums[i - 1])
            } else {
                "start".into()
            };
            diags.push(Diagnostic::warning(
                "T016",
                &record.file_path,
                0,
                format!("Tagged rule IDs not sequential (gap after {prev})"),
            ));
            return;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{AdrId, Tier};
    use std::collections::HashMap;
    use std::path::PathBuf;

    fn make_config() -> Config {
        toml::from_str(
            r#"
[stale]
directory = "stale"

[[domains]]
prefix = "CHE"
name = "Cherry"
directory = "cherry"
description = "Test"
crates = []

[[rules]]
id = "T015"
category = "template"
description = "Section minimum word count"
params = { min_words = 10 }
"#,
        )
        .unwrap()
    }

    fn make_record() -> AdrRecord {
        let mut word_counts = HashMap::new();
        word_counts.insert("Context".into(), 15);
        word_counts.insert("Decision".into(), 15);
        word_counts.insert("Consequences".into(), 15);

        AdrRecord {
            id: AdrId {
                prefix: "CHE".into(),
                number: 1,
            },
            file_path: PathBuf::from("test.md"),
            title: Some("Test".into()),
            title_line: 1,
            date: Some("2026-04-25".into()),
            last_reviewed: Some("2026-04-25".into()),
            tier: Some(Tier::S),
            status: Some(Status::Accepted),
            status_line: 8,
            status_raw: Some("Accepted".into()),
            has_related: true,
            has_context: true,
            has_decision: true,
            has_consequences: true,
            section_order: vec![
                "Status".into(),
                "Related".into(),
                "Context".into(),
                "Decision".into(),
                "Consequences".into(),
            ],
            section_word_counts: word_counts,
            ..AdrRecord::default()
        }
    }

    #[test]
    fn valid_record_produces_no_diagnostics() {
        // Add a relationship to avoid T007
        use crate::model::{RelVerb, Relationship, TaggedRule};
        let mut record = make_record();
        record.relationships = vec![Relationship {
            verb: RelVerb::Root,
            target: record.id.clone(),
            line: 10,
        }];
        record.is_self_referencing = true;
        record.decision_rules = vec![
            TaggedRule { id: "R1".into(), text: "Test rule".into(), line: 10 },
        ];

        let config = make_config();
        let mut diags = Vec::new();
        check(&record, &config, &mut diags);
        assert!(diags.is_empty(), "expected no diags, got: {diags:?}");
    }

    #[test]
    fn missing_tier_produces_t004() {
        let mut record = make_record();
        record.tier = None;
        let config = make_config();
        let mut diags = Vec::new();
        check(&record, &config, &mut diags);
        assert!(diags.iter().any(|d| d.rule == "T004"));
    }

    #[test]
    fn missing_last_reviewed_all_tiers_is_warning() {
        for tier in [Tier::S, Tier::A, Tier::B, Tier::C, Tier::D] {
            let mut record = make_record();
            record.tier = Some(tier);
            record.last_reviewed = None;
            let config = make_config();
            let mut diags = Vec::new();
            check(&record, &config, &mut diags);
            assert!(
                diags.iter().any(|d| d.rule == "T003"),
                "expected T003 for tier {tier:?}"
            );
        }
    }

    #[test]
    fn parenthetical_status_produces_t006() {
        let mut record = make_record();
        record.status_raw = Some("Accepted (supersedes original u64 design)".into());
        record.status = Some(Status::Invalid(
            "Accepted (supersedes original u64 design)".into(),
        ));
        let config = make_config();
        let mut diags = Vec::new();
        check(&record, &config, &mut diags);
        assert!(
            diags.iter().any(|d| d.rule == "T006"),
            "expected T006, got: {diags:?}"
        );
    }

    #[test]
    fn empty_related_produces_t007() {
        let mut record = make_record();
        record.has_related = true;
        record.relationships = vec![];
        record.related_has_placeholder = true;
        let config = make_config();
        let mut diags = Vec::new();
        check(&record, &config, &mut diags);
        assert!(
            diags.iter().any(|d| d.rule == "T007"),
            "empty Related should trigger T007, got: {diags:?}"
        );
    }

    #[test]
    fn related_with_relationship_no_t007() {
        use crate::model::{RelVerb, Relationship};
        let mut record = make_record();
        record.relationships = vec![Relationship {
            verb: RelVerb::Root,
            target: record.id.clone(),
            line: 10,
        }];
        let config = make_config();
        let mut diags = Vec::new();
        check(&record, &config, &mut diags);
        assert!(
            !diags.iter().any(|d| d.rule == "T007"),
            "Related with relationship should not trigger T007"
        );
    }

    #[test]
    fn code_block_at_limit_no_t011() {
        let mut record = make_record();
        record.max_code_block_lines = 20;
        let config = make_config();
        let mut diags = Vec::new();
        check(&record, &config, &mut diags);
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
        let config = make_config();
        let mut diags = Vec::new();
        check(&record, &config, &mut diags);
        let t011 = diags.iter().find(|d| d.rule == "T011");
        assert!(t011.is_some(), "expected T011, got: {diags:?}");
        assert_eq!(t011.unwrap().line, 42, "T011 should point to opening fence");
    }

    #[test]
    fn amendment_date_before_creation_produces_t012() {
        let mut record = make_record();
        record.date = Some("2026-04-25".into());
        record.amendment_dates = vec![("2026-04-01".into(), 12)];
        let config = make_config();
        let mut diags = Vec::new();
        check(&record, &config, &mut diags);
        let t012 = diags.iter().find(|d| d.rule == "T012");
        assert!(t012.is_some(), "expected T012, got: {diags:?}");
    }

    #[test]
    fn section_out_of_order_produces_t014() {
        let mut record = make_record();
        record.section_order = vec![
            "Status".into(),
            "Context".into(),  // out of order — Related should come first
            "Related".into(),
            "Decision".into(),
            "Consequences".into(),
        ];
        let config = make_config();
        let mut diags = Vec::new();
        check(&record, &config, &mut diags);
        assert!(
            diags.iter().any(|d| d.rule == "T014"),
            "out-of-order sections should trigger T014, got: {diags:?}"
        );
    }

    #[test]
    fn section_correct_order_no_t014() {
        let record = make_record();
        let config = make_config();
        let mut diags = Vec::new();
        check(&record, &config, &mut diags);
        assert!(
            !diags.iter().any(|d| d.rule == "T014"),
            "correct order should not trigger T014, got: {diags:?}"
        );
    }

    #[test]
    fn section_too_few_words_produces_t015() {
        let mut record = make_record();
        record.section_word_counts.insert("Context".into(), 5);
        let config = make_config();
        let mut diags = Vec::new();
        check(&record, &config, &mut diags);
        assert!(
            diags.iter().any(|d| d.rule == "T015"),
            "5 words should trigger T015, got: {diags:?}"
        );
    }

    #[test]
    fn section_enough_words_no_t015() {
        let record = make_record(); // all sections have 15 words
        let config = make_config();
        let mut diags = Vec::new();
        check(&record, &config, &mut diags);
        assert!(
            !diags.iter().any(|d| d.rule == "T015"),
            "15 words should not trigger T015, got: {diags:?}"
        );
    }

    #[test]
    fn stale_adr_without_retirement_produces_s004() {
        let mut record = make_record();
        record.is_stale = true;
        record.has_retirement = false;
        let config = make_config();
        let mut diags = Vec::new();
        check(&record, &config, &mut diags);
        assert!(
            diags.iter().any(|d| d.rule == "S004"),
            "stale without Retirement should trigger S004, got: {diags:?}"
        );
    }

    #[test]
    fn stale_adr_with_retirement_no_s004() {
        let mut record = make_record();
        record.is_stale = true;
        record.has_retirement = true;
        record.section_word_counts.insert("Retirement".into(), 15);
        record.section_order.push("Retirement".into());
        let config = make_config();
        let mut diags = Vec::new();
        check(&record, &config, &mut diags);
        // Filter S004 — should not appear for existence check
        let s004_existence: Vec<_> = diags
            .iter()
            .filter(|d| d.rule == "S004" && d.message.contains("missing"))
            .collect();
        assert!(
            s004_existence.is_empty(),
            "stale with Retirement should not trigger S004 existence check"
        );
    }

    #[test]
    fn active_adr_with_retirement_produces_s005() {
        let mut record = make_record();
        record.is_stale = false;
        record.has_retirement = true;
        let config = make_config();
        let mut diags = Vec::new();
        check(&record, &config, &mut diags);
        assert!(
            diags.iter().any(|d| d.rule == "S005"),
            "active with Retirement should trigger S005, got: {diags:?}"
        );
    }

    #[test]
    fn rejected_in_active_dir_produces_s006() {
        let mut record = make_record();
        record.status = Some(Status::Rejected);
        record.status_raw = Some("Rejected".into());
        record.is_stale = false;
        let config = make_config();
        let mut diags = Vec::new();
        check(&record, &config, &mut diags);
        let s006 = diags.iter().find(|d| d.rule == "S006");
        assert!(s006.is_some(), "Rejected in active dir should trigger S006");
        assert!(
            s006.unwrap().message.contains("Action:"),
            "S006 message must contain actionable instructions"
        );
    }

    #[test]
    fn deprecated_in_active_dir_produces_s006() {
        let mut record = make_record();
        record.status = Some(Status::Deprecated);
        record.status_raw = Some("Deprecated".into());
        record.is_stale = false;
        let config = make_config();
        let mut diags = Vec::new();
        check(&record, &config, &mut diags);
        assert!(
            diags.iter().any(|d| d.rule == "S006"),
            "Deprecated in active dir should trigger S006"
        );
    }

    #[test]
    fn superseded_in_active_dir_produces_s006() {
        let mut record = make_record();
        record.status = Some(Status::SupersededBy(AdrId {
            prefix: "CHE".into(),
            number: 99,
        }));
        record.status_raw = Some("Superseded by CHE-0099".into());
        record.is_stale = false;
        let config = make_config();
        let mut diags = Vec::new();
        check(&record, &config, &mut diags);
        let s006 = diags.iter().find(|d| d.rule == "S006");
        assert!(s006.is_some(), "Superseded in active dir should trigger S006");
        assert!(
            s006.unwrap().message.contains("Superseded by CHE-0099"),
            "S006 message should name the superseding ADR"
        );
    }

    #[test]
    fn accepted_in_active_dir_no_s006() {
        let record = make_record(); // status = Accepted, is_stale = false
        let config = make_config();
        let mut diags = Vec::new();
        check(&record, &config, &mut diags);
        assert!(
            !diags.iter().any(|d| d.rule == "S006"),
            "Accepted in active dir should NOT trigger S006"
        );
    }

    #[test]
    fn rejected_in_stale_dir_no_s006() {
        let mut record = make_record();
        record.status = Some(Status::Rejected);
        record.status_raw = Some("Rejected".into());
        record.is_stale = true;
        record.has_retirement = true;
        record.section_word_counts.insert("Retirement".into(), 15);
        record.section_order.push("Retirement".into());
        let config = make_config();
        let mut diags = Vec::new();
        check(&record, &config, &mut diags);
        assert!(
            !diags.iter().any(|d| d.rule == "S006"),
            "Rejected in stale dir should NOT trigger S006"
        );
    }

    #[test]
    fn s006_message_is_llm_actionable() {
        let mut record = make_record();
        record.status = Some(Status::Rejected);
        record.status_raw = Some("Rejected".into());
        record.is_stale = false;
        let config = make_config();
        let mut diags = Vec::new();
        check(&record, &config, &mut diags);
        let s006 = diags.iter().find(|d| d.rule == "S006").unwrap();
        assert!(s006.message.contains("move this file to"), "must say what to do");
        assert!(s006.message.contains("stale/"), "must name target directory");
        assert!(s006.message.contains("## Retirement"), "must name required section");
        assert!(s006.message.contains("≥10 words"), "must specify word count");
    }

    #[test]
    fn tagged_rules_present_no_t016() {
        use crate::model::TaggedRule;
        let mut record = make_record();
        record.decision_rules = vec![
            TaggedRule { id: "R1".into(), text: "Rule one".into(), line: 10 },
            TaggedRule { id: "R2".into(), text: "Rule two".into(), line: 11 },
        ];
        let config = make_config();
        let mut diags = Vec::new();
        check(&record, &config, &mut diags);
        assert!(
            !diags.iter().any(|d| d.rule == "T016"),
            "tagged rules should not trigger T016, got: {diags:?}"
        );
    }

    #[test]
    fn no_tagged_rules_produces_t016() {
        let mut record = make_record();
        record.decision_rules = vec![]; // no rules at all
        let config = make_config();
        let mut diags = Vec::new();
        check(&record, &config, &mut diags);
        assert!(
            diags.iter().any(|d| d.rule == "T016"),
            "missing tagged rules should trigger T016, got: {diags:?}"
        );
    }

    #[test]
    fn r0_fallback_produces_t016() {
        use crate::model::TaggedRule;
        let mut record = make_record();
        record.decision_rules = vec![TaggedRule {
            id: "R0".into(),
            text: "Full decision text".into(),
            line: 0,
        }];
        let config = make_config();
        let mut diags = Vec::new();
        check(&record, &config, &mut diags);
        assert!(
            diags.iter().any(|d| d.rule == "T016"),
            "R0 fallback should trigger T016, got: {diags:?}"
        );
    }

    #[test]
    fn draft_exempt_from_t016() {
        let mut record = make_record();
        record.status = Some(Status::Draft);
        record.status_raw = Some("Draft".into());
        record.decision_rules = vec![];
        let config = make_config();
        let mut diags = Vec::new();
        check(&record, &config, &mut diags);
        assert!(
            !diags.iter().any(|d| d.rule == "T016"),
            "Draft should be exempt from T016, got: {diags:?}"
        );
    }

    #[test]
    fn proposed_exempt_from_t016() {
        let mut record = make_record();
        record.status = Some(Status::Proposed);
        record.status_raw = Some("Proposed".into());
        record.decision_rules = vec![];
        let config = make_config();
        let mut diags = Vec::new();
        check(&record, &config, &mut diags);
        assert!(
            !diags.iter().any(|d| d.rule == "T016"),
            "Proposed should be exempt from T016"
        );
    }

    #[test]
    fn non_sequential_ids_produces_t016() {
        use crate::model::TaggedRule;
        let mut record = make_record();
        record.decision_rules = vec![
            TaggedRule { id: "R1".into(), text: "Rule one".into(), line: 10 },
            TaggedRule { id: "R3".into(), text: "Rule three".into(), line: 12 },
        ];
        let config = make_config();
        let mut diags = Vec::new();
        check(&record, &config, &mut diags);
        let t016 = diags.iter().find(|d| d.rule == "T016");
        assert!(t016.is_some(), "gap in IDs should trigger T016");
        assert!(
            t016.unwrap().message.contains("gap after R1"),
            "should identify the gap location"
        );
    }
}
