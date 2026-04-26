//! Link integrity rules (L001, L003–L005).
//!
//! L001: Dangling link — target ADR file does not exist
//! L003: Symmetric verb mismatch — `Contrasts with` must be mirrored
//! L004: Cross-domain reference to unmigrated ADR (warning only)
//! L005: Reverse verb used — only forward (root-direction) verbs permitted

use std::collections::{HashMap, HashSet};

use crate::model::{AdrId, AdrRecord, RelVerb, Relationship};
use crate::report::Diagnostic;

/// Known prefixes for domains that exist in this workspace.
const KNOWN_PREFIXES: &[&str] = &["COM", "CHE", "PAR", "GEN"];

pub fn check(records: &[AdrRecord], diags: &mut Vec<Diagnostic>) {
    // Build a lookup: AdrId → &AdrRecord
    let by_id: HashMap<&AdrId, &AdrRecord> = records.iter().map(|r| (&r.id, r)).collect();

    // Build a set of all (source, verb, target) triples for reverse lookup
    let mut link_set: HashSet<(&AdrId, RelVerb, &AdrId)> = HashSet::new();
    for record in records {
        for rel in &record.relationships {
            link_set.insert((&record.id, rel.verb, &rel.target));
        }
    }

    for record in records {
        for rel in &record.relationships {
            check_single_link(record, rel, &by_id, &link_set, diags);
        }
    }
}

fn check_single_link(
    source: &AdrRecord,
    rel: &Relationship,
    by_id: &HashMap<&AdrId, &AdrRecord>,
    link_set: &HashSet<(&AdrId, RelVerb, &AdrId)>,
    diags: &mut Vec<Diagnostic>,
) {
    let target_id = &rel.target;

    // L005: Reverse verb — only forward (root-direction) verbs permitted
    if rel.verb.is_reverse() {
        diags.push(Diagnostic::error(
            "L005",
            &source.file_path,
            rel.line,
            format!(
                "{} -[{}]→ {target_id}: reverse verb not permitted — \
                 use the forward verb in the target ADR instead",
                source.id, rel.verb,
            ),
        ));
        return; // Skip further checks on reverse verbs
    }

    // L004: Cross-domain reference to unmigrated domain
    if target_id.prefix != source.id.prefix {
        if KNOWN_PREFIXES.contains(&target_id.prefix.as_str()) && !by_id.contains_key(target_id) {
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
    }

    // L001: Dangling link — target file not found
    if !by_id.contains_key(target_id) {
        diags.push(Diagnostic::error(
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

    // L003: Symmetric verb check — `Contrasts with` must be mirrored with same verb
    if rel.verb.is_symmetric() {
        let has_symmetric = link_set.contains(&(target_id, rel.verb, &source.id));
        if !has_symmetric {
            diags.push(Diagnostic::error(
                "L003",
                &source.file_path,
                rel.line,
                format!(
                    "{} -[{}]→ {target_id}: symmetric verb requires matching \
                     `{}: {}` in {target_id}",
                    source.id, rel.verb, rel.verb, source.id,
                ),
            ));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{AdrId, Status, Tier};
    use std::path::PathBuf;

    fn make_id(prefix: &str, num: u16) -> AdrId {
        AdrId {
            prefix: prefix.into(),
            number: num,
        }
    }

    fn make_record_with_rels(
        prefix: &str,
        num: u16,
        rels: Vec<(RelVerb, AdrId)>,
    ) -> AdrRecord {
        let relationships = rels
            .into_iter()
            .enumerate()
            .map(|(i, (verb, target))| Relationship {
                verb,
                target,
                line: 10 + i,
            })
            .collect();

        AdrRecord {
            id: make_id(prefix, num),
            file_path: PathBuf::from(format!(
                "docs/adr/cherry/{prefix}-{num:04}-test.md"
            )),
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
            relationships,
            has_related: true,
            has_context: true,
            has_decision: true,
            has_consequences: true,
            max_code_block_lines: 0,
            max_code_block_line: 0,
            code_block_count: 0,
            amendment_dates: vec![],
            related_has_placeholder: false,
        }
    }

    #[test]
    fn forward_link_without_backlink_no_errors() {
        // After removing bidirectional enforcement, a forward link
        // with no reciprocal backlink is accepted.
        let records = vec![
            make_record_with_rels("CHE", 1, vec![(RelVerb::DependsOn, make_id("CHE", 2))]),
            make_record_with_rels("CHE", 2, vec![]),
        ];
        let mut diags = Vec::new();
        check(&records, &mut diags);
        assert!(diags.is_empty(), "expected no diags, got: {diags:?}");
    }

    #[test]
    fn dangling_link_produces_l001() {
        let records = vec![make_record_with_rels(
            "CHE",
            1,
            vec![(RelVerb::DependsOn, make_id("CHE", 99))],
        )];
        let mut diags = Vec::new();
        check(&records, &mut diags);
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
        check(&records, &mut diags);
        assert!(
            diags.iter().any(|d| d.rule == "L004"),
            "expected L004, got: {diags:?}"
        );
        let l004 = diags.iter().find(|d| d.rule == "L004").unwrap();
        assert_eq!(l004.severity, crate::report::Severity::Warning);
    }

    #[test]
    fn symmetric_contrasts_with_requires_mirror() {
        // Only one direction → L003
        let records = vec![
            make_record_with_rels(
                "CHE",
                1,
                vec![(RelVerb::ContrastsWith, make_id("CHE", 2))],
            ),
            make_record_with_rels("CHE", 2, vec![]),
        ];
        let mut diags = Vec::new();
        check(&records, &mut diags);
        assert!(
            diags.iter().any(|d| d.rule == "L003"),
            "expected L003, got: {diags:?}"
        );
    }

    #[test]
    fn symmetric_contrasts_with_both_directions_ok() {
        let records = vec![
            make_record_with_rels(
                "CHE",
                1,
                vec![(RelVerb::ContrastsWith, make_id("CHE", 2))],
            ),
            make_record_with_rels(
                "CHE",
                2,
                vec![(RelVerb::ContrastsWith, make_id("CHE", 1))],
            ),
        ];
        let mut diags = Vec::new();
        check(&records, &mut diags);
        assert!(diags.is_empty(), "expected no diags, got: {diags:?}");
    }

    #[test]
    fn reverse_verb_produces_l005() {
        let reverse_verbs = [
            RelVerb::Informs,
            RelVerb::ExtendedBy,
            RelVerb::IllustratedBy,
            RelVerb::ReferencedBy,
            RelVerb::SupersededBy,
            RelVerb::Scopes,
        ];
        for verb in reverse_verbs {
            let records = vec![
                make_record_with_rels("CHE", 1, vec![(verb, make_id("CHE", 2))]),
                make_record_with_rels("CHE", 2, vec![]),
            ];
            let mut diags = Vec::new();
            check(&records, &mut diags);
            assert!(
                diags.iter().any(|d| d.rule == "L005"),
                "expected L005 for {verb}, got: {diags:?}"
            );
        }
    }

    #[test]
    fn forward_verbs_do_not_produce_l005() {
        let forward_verbs = [
            RelVerb::DependsOn,
            RelVerb::Extends,
            RelVerb::Illustrates,
            RelVerb::References,
            RelVerb::Supersedes,
            RelVerb::ScopedBy,
        ];
        for verb in forward_verbs {
            let records = vec![
                make_record_with_rels("CHE", 1, vec![(verb, make_id("CHE", 2))]),
                make_record_with_rels("CHE", 2, vec![]),
            ];
            let mut diags = Vec::new();
            check(&records, &mut diags);
            assert!(
                !diags.iter().any(|d| d.rule == "L005"),
                "forward verb {verb} should not trigger L005, got: {diags:?}"
            );
        }
    }

    #[test]
    fn cross_domain_forward_link_no_errors() {
        let mut com = make_record_with_rels(
            "CHE",
            5,
            vec![(RelVerb::Illustrates, make_id("COM", 2))],
        );
        com.file_path = "docs/adr/cherry/CHE-0005-test.md".into();

        let mut com_target = make_record_with_rels("COM", 2, vec![]);
        com_target.file_path = "docs/adr/common/COM-0002-test.md".into();

        let records = vec![com, com_target];
        let mut diags = Vec::new();
        check(&records, &mut diags);
        assert!(diags.is_empty(), "expected no diags, got: {diags:?}");
    }
}
