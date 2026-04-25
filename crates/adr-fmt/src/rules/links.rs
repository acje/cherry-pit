//! Link integrity rules (L001–L004).
//!
//! L001: Dangling link — target ADR file does not exist
//! L002: Missing backlink — A links to B, but B lacks the reverse link back to A
//! L003: Symmetric verb mismatch — `Contrasts with` must be mirrored
//! L004: Cross-domain reference to unmigrated ADR (warning only)

use std::collections::{HashMap, HashSet};

use crate::model::{AdrId, AdrRecord, RelVerb, Relationship};
use crate::report::Diagnostic;

/// Known prefixes for domains that exist in this workspace.
const KNOWN_PREFIXES: &[&str] = &["CHE", "PAR", "GEN"];

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
            return; // Don't check backlinks for unmigrated ADRs
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
        return; // Can't check backlinks for non-existent targets
    }

    // L002: Missing backlink — target should have reverse verb → source
    // Skip for symmetric verbs — L003 handles those exclusively.
    let reverse_verb = rel.verb.reverse();
    if !rel.verb.is_symmetric() {
        let has_backlink = link_set.contains(&(target_id, reverse_verb, &source.id));

        if !has_backlink {
            diags.push(Diagnostic::error(
                "L002",
                &source.file_path,
                rel.line,
                format!(
                    "{} -[{}]→ {target_id}: missing backlink \
                     (expected `{reverse_verb}: {}` in {target_id})",
                    source.id, rel.verb, source.id,
                ),
            ));
        }
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
                "docs/adr/framework/{prefix}-{num:04}-test.md"
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
        }
    }

    #[test]
    fn matching_forward_and_back_links_no_errors() {
        let records = vec![
            make_record_with_rels("CHE", 1, vec![(RelVerb::DependsOn, make_id("CHE", 2))]),
            make_record_with_rels("CHE", 2, vec![(RelVerb::Informs, make_id("CHE", 1))]),
        ];
        let mut diags = Vec::new();
        check(&records, &mut diags);
        assert!(diags.is_empty(), "expected no diags, got: {diags:?}");
    }

    #[test]
    fn missing_backlink_produces_l002() {
        let records = vec![
            make_record_with_rels("CHE", 1, vec![(RelVerb::DependsOn, make_id("CHE", 2))]),
            make_record_with_rels("CHE", 2, vec![]), // no Informs back
        ];
        let mut diags = Vec::new();
        check(&records, &mut diags);
        assert!(
            diags.iter().any(|d| d.rule == "L002"),
            "expected L002, got: {diags:?}"
        );
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
        // Should be warning, not error
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
            make_record_with_rels("CHE", 2, vec![]), // no ContrastsWith back
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
}
