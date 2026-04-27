//! Template compliance rules (T001–T016) and structure rules (S004–S006).
//!
//! T001: H1 title present
//! T002: Date field present
//! T003: Last-reviewed field required (all tiers)
//! T004: Tier field present
//! T005: Status section present
//! T006: Status value valid (strict keyword, no parentheticals)
//! T007: Related section with at least one relationship
//! T008: Context section present
//! T009: Decision section present
//! T010: Consequences section present
//! T011: Code block exceeds 20 lines (warning)
//! T014: Section ordering — H2 sections in canonical order
//! T015: Section word count range (min 7, max 50; configurable)
//! T016: Tagged rules validation (exist, sequential, max 10, 7-50 words each)
//! S004: Stale ADR missing Retirement section
//! S005: Active ADR has Retirement section (location/status mismatch)
//! S006: Terminal-status ADR not in stale directory

use crate::config::Config;
use crate::model::{AdrRecord, Status};
use crate::report::Diagnostic;

/// Maximum lines in a single fenced code block before T011 fires.
const MAX_CODE_BLOCK_LINES: usize = 20;

/// Default minimum word count for prose sections.
const DEFAULT_MIN_WORDS: u64 = 7;

/// Default maximum word count for prose sections.
const DEFAULT_MAX_WORDS: u64 = 50;

/// Default maximum number of tagged rules per ADR.
const DEFAULT_MAX_RULES: u64 = 10;

/// Default minimum words per tagged rule.
const DEFAULT_MIN_RULE_WORDS: u64 = 7;

/// Default maximum words per tagged rule.
const DEFAULT_MAX_RULE_WORDS: u64 = 50;

/// Canonical H2 section order for active ADRs.
const ACTIVE_SECTION_ORDER: &[&str] = &["Status", "Related", "Context", "Decision", "Consequences"];

/// Canonical H2 section order for stale ADRs (Retirement at end).
const STALE_SECTION_ORDER: &[&str] = &[
    "Status",
    "Related",
    "Context",
    "Decision",
    "Consequences",
    "Retirement",
];

#[allow(clippy::too_many_lines)]
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

    // T006: Status value validity — strict keyword
    if let Some(ref raw) = record.status_raw {
        if Status::has_parenthetical(raw) {
            diags.push(Diagnostic::warning(
                "T006",
                &record.file_path,
                record.status_line,
                format!(
                    "status line contains parenthetical annotation: `{raw}` — \
                     remove annotations, use a valid status keyword"
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
                     Draft, Proposed, Accepted, Rejected, Deprecated, \
                     Superseded by PREFIX-NNNN"
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

    // T014: Section ordering
    check_section_order(record, diags);

    // T015: Section word count range (applies to Context, Consequences, Retirement)
    let min_words = config
        .rule_param_u64("T015", "min_words")
        .unwrap_or(DEFAULT_MIN_WORDS);
    let max_words = config
        .rule_param_u64("T015", "max_words")
        .unwrap_or(DEFAULT_MAX_WORDS);
    check_section_word_counts(record, min_words, max_words, diags);

    // T016: Tagged rules in Decision section
    let max_rules = config
        .rule_param_u64("T016", "max_rules")
        .unwrap_or(DEFAULT_MAX_RULES);
    let min_rule_words = config
        .rule_param_u64("T016", "min_rule_words")
        .unwrap_or(DEFAULT_MIN_RULE_WORDS);
    let max_rule_words = config
        .rule_param_u64("T016", "max_rule_words")
        .unwrap_or(DEFAULT_MAX_RULE_WORDS);
    check_tagged_rules(record, max_rules, min_rule_words, max_rule_words, diags);

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
    if let Some(ref status) = record.status
        && status.is_terminal()
        && !record.is_stale
    {
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

/// T015: Prose sections must meet word count range.
///
/// Applies to Context, Consequences, and Retirement only.
/// Decision section is validated by T016 (rule count, not word count).
fn check_section_word_counts(
    record: &AdrRecord,
    min_words: u64,
    max_words: u64,
    diags: &mut Vec<Diagnostic>,
) {
    let prose_sections = ["Context", "Consequences"];

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
            } else if (count as u64) > max_words {
                diags.push(Diagnostic::warning(
                    "T015",
                    &record.file_path,
                    0,
                    format!(
                        "`## {section}` has {count} word(s) (maximum {max_words}) — \
                         be concise, split into multiple ADRs if needed"
                    ),
                ));
            }
        }
    }

    // Retirement section word count range (if present)
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
            } else if (count as u64) > max_words {
                diags.push(Diagnostic::warning(
                    "T015",
                    &record.file_path,
                    0,
                    format!(
                        "`## Retirement` has {count} word(s) (maximum {max_words}) — \
                         be concise"
                    ),
                ));
            }
        }
    }
}

/// T016: Tagged rules validation in Decision section.
///
/// Checks:
/// - At least one tagged rule present (unless Draft/Proposed)
/// - Sequential IDs (R1, R2, R3 — no gaps)
/// - Maximum rule count (default 10)
/// - Word count per rule (default 7-50)
fn check_tagged_rules(
    record: &AdrRecord,
    max_rules: u64,
    min_rule_words: u64,
    max_rule_words: u64,
    diags: &mut Vec<Diagnostic>,
) {
    // Exempt Draft and Proposed
    if let Some(ref status) = record.status
        && matches!(status, Status::Draft | Status::Proposed)
    {
        return;
    }

    // Check for missing tagged rules
    let has_real_rules = !(record.decision_rules.is_empty()
        || record.decision_rules.len() == 1 && record.decision_rules[0].id == "R0");

    if !has_real_rules {
        diags.push(Diagnostic::warning(
            "T016",
            &record.file_path,
            0,
            "Decision section lacks tagged rules (- **RN**: pattern)".into(),
        ));
        return;
    }

    // Check maximum rule count
    if record.decision_rules.len() as u64 > max_rules {
        diags.push(Diagnostic::warning(
            "T016",
            &record.file_path,
            0,
            format!(
                "Decision section has {} tagged rules (maximum {max_rules}) — \
                 split into multiple ADRs",
                record.decision_rules.len(),
            ),
        ));
    }

    // Check per-rule word bounds
    for rule in &record.decision_rules {
        if rule.id == "R0" {
            continue;
        }
        let word_count = rule.text.split_whitespace().count() as u64;
        if word_count < min_rule_words {
            diags.push(Diagnostic::warning(
                "T016",
                &record.file_path,
                rule.line,
                format!(
                    "Rule {id} has {word_count} word(s) (minimum {min_rule_words})",
                    id = rule.id,
                ),
            ));
        } else if word_count > max_rule_words {
            diags.push(Diagnostic::warning(
                "T016",
                &record.file_path,
                rule.line,
                format!(
                    "Rule {id} has {word_count} word(s) (maximum {max_rule_words}) — be concise",
                    id = rule.id,
                ),
            ));
        }
    }

    // Check for non-sequential IDs
    let mut nums: Vec<u32> = Vec::new();
    for rule in &record.decision_rules {
        if let Some(num_str) = rule.id.strip_prefix('R')
            && let Ok(num) = num_str.parse::<u32>()
        {
            nums.push(num);
        }
    }

    nums.sort_unstable();
    for (i, &num) in nums.iter().enumerate() {
        let expected = u32::try_from(i).expect("rule count fits u32") + 1;
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
params = { min_words = 7, max_words = 50 }

[[rules]]
id = "T016"
params = { max_rules = 10, min_rule_words = 7, max_rule_words = 50 }
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
        use crate::model::{RelVerb, Relationship, TaggedRule};
        let mut record = make_record();
        record.relationships = vec![Relationship {
            verb: RelVerb::Root,
            target: record.id.clone(),
            line: 10,
        }];
        record.is_self_referencing = true;
        record.decision_rules = vec![TaggedRule {
            id: "R1".into(),
            text: "All events must be versioned with semantic version numbers".into(),
            line: 10,
        }];

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
    fn amended_status_produces_t006() {
        let mut record = make_record();
        record.status_raw = Some("Amended 2026-04-25 — note".into());
        record.status = Some(Status::Invalid(
            "Amended 2026-04-25 — note".into(),
        ));
        let config = make_config();
        let mut diags = Vec::new();
        check(&record, &config, &mut diags);
        assert!(
            diags.iter().any(|d| d.rule == "T006"),
            "Amended should trigger T006 as invalid, got: {diags:?}"
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
    fn section_out_of_order_produces_t014() {
        let mut record = make_record();
        record.section_order = vec![
            "Status".into(),
            "Context".into(), // out of order — Related should come first
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
        record.section_word_counts.insert("Context".into(), 3);
        let config = make_config();
        let mut diags = Vec::new();
        check(&record, &config, &mut diags);
        assert!(
            diags.iter().any(|d| d.rule == "T015"),
            "3 words should trigger T015, got: {diags:?}"
        );
    }

    #[test]
    fn section_too_many_words_produces_t015() {
        let mut record = make_record();
        record.section_word_counts.insert("Context".into(), 60);
        let config = make_config();
        let mut diags = Vec::new();
        check(&record, &config, &mut diags);
        let t015 = diags.iter().find(|d| d.rule == "T015" && d.message.contains("maximum"));
        assert!(
            t015.is_some(),
            "60 words should trigger T015 max, got: {diags:?}"
        );
    }

    #[test]
    fn section_within_range_no_t015() {
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
        assert!(
            s006.is_some(),
            "Superseded in active dir should trigger S006"
        );
    }

    #[test]
    fn accepted_in_active_dir_no_s006() {
        let record = make_record();
        let config = make_config();
        let mut diags = Vec::new();
        check(&record, &config, &mut diags);
        assert!(
            !diags.iter().any(|d| d.rule == "S006"),
            "Accepted in active dir should NOT trigger S006"
        );
    }

    #[test]
    fn tagged_rules_present_no_t016() {
        use crate::model::TaggedRule;
        let mut record = make_record();
        record.decision_rules = vec![
            TaggedRule {
                id: "R1".into(),
                text: "All events must be versioned with semantic version numbers always".into(),
                line: 10,
            },
            TaggedRule {
                id: "R2".into(),
                text: "Snapshots are created at one hundred event intervals minimum always".into(),
                line: 11,
            },
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
        record.decision_rules = vec![];
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
    fn too_many_rules_produces_t016() {
        use crate::model::TaggedRule;
        let mut record = make_record();
        record.decision_rules = (1..=11)
            .map(|i| TaggedRule {
                id: format!("R{i}"),
                text: "This rule has enough words to pass the minimum check here".into(),
                line: 10 + i,
            })
            .collect();
        let config = make_config();
        let mut diags = Vec::new();
        check(&record, &config, &mut diags);
        let t016_max = diags
            .iter()
            .find(|d| d.rule == "T016" && d.message.contains("maximum"));
        assert!(
            t016_max.is_some(),
            "11 rules should trigger T016 max, got: {diags:?}"
        );
    }

    #[test]
    fn rule_too_few_words_produces_t016() {
        use crate::model::TaggedRule;
        let mut record = make_record();
        record.decision_rules = vec![TaggedRule {
            id: "R1".into(),
            text: "Too short".into(), // 2 words
            line: 10,
        }];
        let config = make_config();
        let mut diags = Vec::new();
        check(&record, &config, &mut diags);
        let t016 = diags
            .iter()
            .find(|d| d.rule == "T016" && d.message.contains("minimum"));
        assert!(
            t016.is_some(),
            "2-word rule should trigger T016 min, got: {diags:?}"
        );
    }

    #[test]
    fn rule_too_many_words_produces_t016() {
        use crate::model::TaggedRule;
        let mut record = make_record();
        let long_text = (0..60).map(|_| "word").collect::<Vec<_>>().join(" ");
        record.decision_rules = vec![TaggedRule {
            id: "R1".into(),
            text: long_text,
            line: 10,
        }];
        let config = make_config();
        let mut diags = Vec::new();
        check(&record, &config, &mut diags);
        let t016 = diags
            .iter()
            .find(|d| d.rule == "T016" && d.message.contains("maximum"));
        assert!(
            t016.is_some(),
            "60-word rule should trigger T016 max, got: {diags:?}"
        );
    }

    #[test]
    fn non_sequential_ids_produces_t016() {
        use crate::model::TaggedRule;
        let mut record = make_record();
        record.decision_rules = vec![
            TaggedRule {
                id: "R1".into(),
                text: "This rule has enough words to pass the minimum check here".into(),
                line: 10,
            },
            TaggedRule {
                id: "R3".into(),
                text: "This rule also has enough words to pass the minimum check".into(),
                line: 12,
            },
        ];
        let config = make_config();
        let mut diags = Vec::new();
        check(&record, &config, &mut diags);
        let t016 = diags.iter().find(|d| d.rule == "T016" && d.message.contains("gap"));
        assert!(t016.is_some(), "gap in IDs should trigger T016");
    }
}
