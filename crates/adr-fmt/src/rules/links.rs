//! Link and relationship rules (L001, L003, L004, L006–L009).
//!
//! L001: Dangling link — target ADR file does not exist
//! L003: Supersedes-status consistency — if A supersedes B, B's
//!       status must be `Superseded by A`
//! L004: Cross-domain reference to unmigrated ADR (warning only)
//! L006: Legacy verb used — only References, Supersedes, Root permitted
//! L007: Stale reference — target ADR is in stale archive
//! L008: Root self-reference mismatch — Root target must match own ID
//! L009: Root + References coexistence — Root and References cannot
//!       appear in the same Related section

use std::collections::HashMap;

use crate::model::{AdrId, AdrRecord, RelVerb, Relationship};
use crate::report::Diagnostic;

pub fn check(records: &[AdrRecord], domain_prefixes: &[&str], diags: &mut Vec<Diagnostic>) {
    // Build a lookup: AdrId → &AdrRecord
    let by_id: HashMap<&AdrId, &AdrRecord> = records.iter().map(|r| (&r.id, r)).collect();

    for record in records {
        // L009: Root + References coexistence check (per-record)
        check_root_references_coexistence(record, diags);

        // L008: Root self-reference mismatch (per-relationship)
        for rel in &record.relationships {
            if rel.verb == RelVerb::Root {
                check_root_self_reference(record, rel, diags);
            }
        }

        for rel in &record.relationships {
            check_single_link(record, rel, &by_id, domain_prefixes, diags);
        }
    }

    // L003: Supersedes-status consistency (cross-file)
    check_supersedes_consistency(records, &by_id, diags);
}

fn check_single_link(
    source: &AdrRecord,
    rel: &Relationship,
    by_id: &HashMap<&AdrId, &AdrRecord>,
    domain_prefixes: &[&str],
    diags: &mut Vec<Diagnostic>,
) {
    let target_id = &rel.target;

    // L006: Legacy verb — only References, Supersedes, Root permitted
    if !rel.verb.is_permitted() {
        diags.push(Diagnostic::warning(
            "L006",
            &source.file_path,
            rel.line,
            format!(
                "{} -[{}]→ {target_id}: legacy verb — use References, \
                 Supersedes, or Root instead",
                source.id, rel.verb,
            ),
        ));
        // Continue checking other rules — the link target may still be valid
    }

    // Skip further link validation for Root self-references
    if rel.verb == RelVerb::Root && rel.target == source.id {
        return;
    }

    // L004: Cross-domain reference to unmigrated domain
    if target_id.prefix != source.id.prefix
        && domain_prefixes.contains(&target_id.prefix.as_str())
        && !by_id.contains_key(target_id)
    {
        diags.push(Diagnostic::warning(
            "L004",
            &source.file_path,
            rel.line,
            format!(
                "{} → {target_id}: cross-domain reference to unmigrated ADR",
                source.id,
            ),
        ));
        return;
    }

    // L001: Dangling link — target file not found
    if !by_id.contains_key(target_id) {
        diags.push(Diagnostic::warning(
            "L001",
            &source.file_path,
            rel.line,
            format!(
                "{} → {target_id}: dangling link (target ADR not found)",
                source.id,
            ),
        ));
        return;
    }

    // L007: Stale reference — target is in stale archive
    if let Some(target_record) = by_id.get(target_id)
        && target_record.is_stale
        && !source.is_stale
    {
        diags.push(Diagnostic::warning(
            "L007",
            &source.file_path,
            rel.line,
            format!("{} → {target_id}: reference to stale ADR", source.id),
        ));
    }
}

/// L003: If A has `Supersedes: B`, then B's status must be
/// `Superseded by A`. Warns on inconsistency.
fn check_supersedes_consistency(
    records: &[AdrRecord],
    by_id: &HashMap<&AdrId, &AdrRecord>,
    diags: &mut Vec<Diagnostic>,
) {
    for record in records {
        for rel in &record.relationships {
            if rel.verb != RelVerb::Supersedes {
                continue;
            }

            let target_id = &rel.target;
            if let Some(target_record) = by_id.get(target_id) {
                let status_matches = matches!(
                    &target_record.status,
                    Some(crate::model::Status::SupersededBy(by_id)) if *by_id == record.id
                );

                if !status_matches {
                    diags.push(Diagnostic::warning(
                        "L003",
                        &record.file_path,
                        rel.line,
                        format!(
                            "{} supersedes {target_id}, but {target_id}'s status \
                             is not `Superseded by {}` — update the target's status",
                            record.id, record.id,
                        ),
                    ));
                }
            }
        }
    }
}

/// L008: Root self-reference mismatch.
fn check_root_self_reference(source: &AdrRecord, rel: &Relationship, diags: &mut Vec<Diagnostic>) {
    debug_assert_eq!(rel.verb, RelVerb::Root);
    if rel.target != source.id {
        diags.push(Diagnostic::warning(
            "L008",
            &source.file_path,
            rel.line,
            format!(
                "{}: Root target `{}` does not match own ID — \
                 Root must be a self-reference (`- Root: {}`)",
                source.id, rel.target, source.id,
            ),
        ));
    }
}

/// L009: Root and References cannot coexist in the same Related section.
fn check_root_references_coexistence(source: &AdrRecord, diags: &mut Vec<Diagnostic>) {
    let has_root = source.relationships.iter().any(|r| r.verb == RelVerb::Root);
    let has_references = source
        .relationships
        .iter()
        .any(|r| r.verb == RelVerb::References);

    if has_root && has_references {
        // Find line of first References entry for diagnostic location
        let ref_line = source
            .relationships
            .iter()
            .find(|r| r.verb == RelVerb::References)
            .map_or(0, |r| r.line);

        diags.push(Diagnostic::warning(
            "L009",
            &source.file_path,
            ref_line,
            format!(
                "{}: Root and References cannot coexist — \
                 a root ADR stands alone structurally",
                source.id,
            ),
        ));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{AdrId, Status, Tier};
    use std::path::PathBuf;

    const TEST_PREFIXES: &[&str] = &["COM", "CHE", "PAR", "GEN"];

    fn make_id(prefix: &str, num: u16) -> AdrId {
        AdrId {
            prefix: prefix.into(),
            number: num,
        }
    }

    fn make_record_with_rels(prefix: &str, num: u16, rels: Vec<(RelVerb, AdrId)>) -> AdrRecord {
        let id = make_id(prefix, num);
        let relationships: Vec<Relationship> = rels
            .into_iter()
            .enumerate()
            .map(|(i, (verb, target))| Relationship {
                verb,
                target,
                line: 10 + i,
            })
            .collect();

        let is_self_referencing = relationships
            .iter()
            .any(|rel| rel.verb == RelVerb::Root && rel.target == id);

        AdrRecord {
            id,
            file_path: PathBuf::from(format!("docs/adr/cherry/{prefix}-{num:04}-test.md")),
            title: Some("Test".into()),
            title_line: 1,
            date: Some("2026-04-25".into()),
            last_reviewed: Some("2026-04-25".into()),
            tier: Some(Tier::B),
            status: Some(Status::Accepted),
            status_line: 8,
            status_raw: Some("Accepted".into()),
            relationships,
            has_related: true,
            has_context: true,
            has_decision: true,
            has_consequences: true,
            is_self_referencing,
            ..AdrRecord::default()
        }
    }

    #[test]
    fn forward_link_no_errors() {
        let records = vec![
            make_record_with_rels("CHE", 1, vec![(RelVerb::References, make_id("CHE", 2))]),
            make_record_with_rels("CHE", 2, vec![(RelVerb::Root, make_id("CHE", 2))]),
        ];
        let mut diags = Vec::new();
        check(&records, TEST_PREFIXES, &mut diags);
        // Filter out L006 — we're testing link integrity, not verb vocabulary
        let non_l006: Vec<_> = diags.iter().filter(|d| d.rule != "L006").collect();
        assert!(
            non_l006.is_empty(),
            "expected no diags (excl L006), got: {non_l006:?}"
        );
    }

    #[test]
    fn dangling_link_produces_l001() {
        let records = vec![make_record_with_rels(
            "CHE",
            1,
            vec![(RelVerb::References, make_id("CHE", 99))],
        )];
        let mut diags = Vec::new();
        check(&records, TEST_PREFIXES, &mut diags);
        assert!(
            diags.iter().any(|d| d.rule == "L001"),
            "expected L001, got: {diags:?}"
        );
    }

    #[test]
    fn cross_domain_unmigrated_is_warning() {
        let records = vec![make_record_with_rels(
            "CHE",
            1,
            vec![(RelVerb::References, make_id("PAR", 5))],
        )];
        let mut diags = Vec::new();
        check(&records, TEST_PREFIXES, &mut diags);
        assert!(
            diags.iter().any(|d| d.rule == "L004"),
            "expected L004, got: {diags:?}"
        );
        let l004 = diags.iter().find(|d| d.rule == "L004").unwrap();
        assert_eq!(l004.severity, crate::report::Severity::Warning);
    }

    #[test]
    fn legacy_verb_produces_l006() {
        let records = vec![
            make_record_with_rels("CHE", 1, vec![(RelVerb::DependsOn, make_id("CHE", 2))]),
            make_record_with_rels("CHE", 2, vec![(RelVerb::Root, make_id("CHE", 2))]),
        ];
        let mut diags = Vec::new();
        check(&records, TEST_PREFIXES, &mut diags);
        assert!(
            diags.iter().any(|d| d.rule == "L006"),
            "expected L006, got: {diags:?}"
        );
    }

    #[test]
    fn permitted_verb_no_l006() {
        let records = vec![
            make_record_with_rels("CHE", 1, vec![(RelVerb::References, make_id("CHE", 2))]),
            make_record_with_rels("CHE", 2, vec![(RelVerb::Root, make_id("CHE", 2))]),
        ];
        let mut diags = Vec::new();
        check(&records, TEST_PREFIXES, &mut diags);
        assert!(
            !diags.iter().any(|d| d.rule == "L006"),
            "permitted verb should not trigger L006, got: {diags:?}"
        );
    }

    #[test]
    fn root_verb_no_l006() {
        let records = vec![make_record_with_rels(
            "CHE",
            1,
            vec![(RelVerb::Root, make_id("CHE", 1))],
        )];
        let mut diags = Vec::new();
        check(&records, TEST_PREFIXES, &mut diags);
        assert!(
            !diags.iter().any(|d| d.rule == "L006"),
            "Root should not trigger L006"
        );
    }

    #[test]
    fn root_self_reference_match_no_l008() {
        let records = vec![make_record_with_rels(
            "CHE",
            1,
            vec![(RelVerb::Root, make_id("CHE", 1))],
        )];
        let mut diags = Vec::new();
        check(&records, TEST_PREFIXES, &mut diags);
        assert!(
            !diags.iter().any(|d| d.rule == "L008"),
            "correct Root self-ref should not trigger L008"
        );
    }

    #[test]
    fn root_wrong_id_produces_l008() {
        let records = vec![make_record_with_rels(
            "CHE",
            1,
            vec![(RelVerb::Root, make_id("CHE", 2))],
        )];
        let mut diags = Vec::new();
        check(&records, TEST_PREFIXES, &mut diags);
        assert!(
            diags.iter().any(|d| d.rule == "L008"),
            "Root pointing to wrong ID should trigger L008, got: {diags:?}"
        );
    }

    #[test]
    fn root_and_references_produces_l009() {
        let records = vec![
            make_record_with_rels(
                "CHE",
                1,
                vec![
                    (RelVerb::Root, make_id("CHE", 1)),
                    (RelVerb::References, make_id("CHE", 2)),
                ],
            ),
            make_record_with_rels("CHE", 2, vec![(RelVerb::Root, make_id("CHE", 2))]),
        ];
        let mut diags = Vec::new();
        check(&records, TEST_PREFIXES, &mut diags);
        assert!(
            diags.iter().any(|d| d.rule == "L009"),
            "Root + References should trigger L009, got: {diags:?}"
        );
    }

    #[test]
    fn root_and_supersedes_no_l009() {
        let mut record = make_record_with_rels(
            "CHE",
            2,
            vec![
                (RelVerb::Root, make_id("CHE", 2)),
                (RelVerb::Supersedes, make_id("CHE", 1)),
            ],
        );
        // Ensure target has correct superseded status
        let mut target = make_record_with_rels("CHE", 1, vec![(RelVerb::Root, make_id("CHE", 1))]);
        target.status = Some(Status::SupersededBy(make_id("CHE", 2)));
        target.status_raw = Some("Superseded by CHE-0002".into());

        record.is_self_referencing = true;

        let records = vec![record, target];
        let mut diags = Vec::new();
        check(&records, TEST_PREFIXES, &mut diags);
        assert!(
            !diags.iter().any(|d| d.rule == "L009"),
            "Root + Supersedes should not trigger L009, got: {diags:?}"
        );
    }

    #[test]
    fn supersedes_without_target_status_produces_l003() {
        let records = vec![
            make_record_with_rels("CHE", 2, vec![(RelVerb::Supersedes, make_id("CHE", 1))]),
            make_record_with_rels("CHE", 1, vec![(RelVerb::Root, make_id("CHE", 1))]),
            // CHE-0001 status is Accepted, not SupersededBy CHE-0002
        ];
        let mut diags = Vec::new();
        check(&records, TEST_PREFIXES, &mut diags);
        assert!(
            diags.iter().any(|d| d.rule == "L003"),
            "expected L003, got: {diags:?}"
        );
    }

    #[test]
    fn supersedes_with_correct_target_status_no_l003() {
        let mut target = make_record_with_rels("CHE", 1, vec![(RelVerb::Root, make_id("CHE", 1))]);
        target.status = Some(Status::SupersededBy(make_id("CHE", 2)));
        target.status_raw = Some("Superseded by CHE-0002".into());

        let records = vec![
            make_record_with_rels("CHE", 2, vec![(RelVerb::Supersedes, make_id("CHE", 1))]),
            target,
        ];
        let mut diags = Vec::new();
        check(&records, TEST_PREFIXES, &mut diags);
        assert!(
            !diags.iter().any(|d| d.rule == "L003"),
            "correct supersedes-status should not trigger L003, got: {diags:?}"
        );
    }

    #[test]
    fn stale_reference_produces_l007() {
        let mut target = make_record_with_rels("CHE", 1, vec![(RelVerb::Root, make_id("CHE", 1))]);
        target.is_stale = true;

        let records = vec![
            make_record_with_rels("CHE", 2, vec![(RelVerb::References, make_id("CHE", 1))]),
            target,
        ];
        let mut diags = Vec::new();
        check(&records, TEST_PREFIXES, &mut diags);
        assert!(
            diags.iter().any(|d| d.rule == "L007"),
            "expected L007, got: {diags:?}"
        );
    }

    #[test]
    fn cross_domain_forward_link_no_errors() {
        let mut che =
            make_record_with_rels("CHE", 5, vec![(RelVerb::References, make_id("COM", 2))]);
        che.file_path = "docs/adr/cherry/CHE-0005-test.md".into();

        let mut com_target =
            make_record_with_rels("COM", 2, vec![(RelVerb::Root, make_id("COM", 2))]);
        com_target.file_path = "docs/adr/common/COM-0002-test.md".into();

        let records = vec![che, com_target];
        let mut diags = Vec::new();
        check(&records, TEST_PREFIXES, &mut diags);
        // Filter L006 — testing link integrity only
        let non_l006: Vec<_> = diags.iter().filter(|d| d.rule != "L006").collect();
        assert!(
            non_l006.is_empty(),
            "expected no diags (excl L006), got: {non_l006:?}"
        );
    }

    #[test]
    fn contrasts_with_produces_l006() {
        let records = vec![
            make_record_with_rels("CHE", 1, vec![(RelVerb::ContrastsWith, make_id("CHE", 2))]),
            make_record_with_rels("CHE", 2, vec![(RelVerb::Root, make_id("CHE", 2))]),
        ];
        let mut diags = Vec::new();
        check(&records, TEST_PREFIXES, &mut diags);
        assert!(
            diags.iter().any(|d| d.rule == "L006"),
            "ContrastsWith should trigger L006"
        );
    }
}
