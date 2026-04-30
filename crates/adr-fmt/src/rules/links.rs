//! Link and relationship rules (L001, L003, L006–L017).
//!
//! L001: Dangling link — target ADR file does not exist
//! L003: Supersedes-status consistency — if A supersedes B, B's
//!       status must be `Superseded by A`
//! L006: Legacy relationship verb — verb is parsed for recognition
//!       but deprecated per AFM-0009
//! L007: Stale reference — target ADR is in stale archive
//! L008: Root self-reference mismatch — Root target must match own ID
//! L009: Root + References coexistence — Root and References cannot
//!       appear in the same Related section
//!
//! Tree-structure rules (parent-edge model, advisory):
//!
//! L010: Missing parent — non-root ADR has no `References:` target
//! L011: Cross-domain parent — first `References:` target is in a
//!       different domain (suppressed by `Parent-cross-domain:`
//!       preamble field)
//! L012: Non-Accepted parent — first `References:` target is not in
//!       `Accepted` status (advisory only; chain still flows through
//!       per draft-waypoint policy)
//! L013: Parent-edge cycle — first-references graph contains a cycle
//! L014: Unreachable from root — non-root ADR's parent chain does
//!       not terminate at any root
//! L015: Heuristic — first-position parent is a root while later
//!       references include same-domain Accepted non-root candidates
//!       (suspicious flat-tree authoring)
//! L016: Heuristic — parent ADR's tier is lower (further from S)
//!       than child's tier
//! L017: Superseded parent — first `References:` target has been
//!       superseded; redirect to the successor
//!
//! Diagnostics are independent: a single relationship may emit
//! multiple codes (e.g. L006 + L001 for a legacy verb pointing to
//! a missing target; L006 + L007 for a legacy verb pointing to a
//! stale target). Each rule encodes one concern; suppression is
//! the author's job after fixing the underlying issue.
//!
//! Cycle dominance: when L013 fires for a record (parent-edge graph
//! contains a cycle through it), the per-record parent-edge checks
//! L011/L012/L014/L016/L017 are suppressed for that record. Rationale:
//! "parent" is not well-defined inside a cycle — once the cycle is
//! broken, the remaining diagnostics will re-evaluate against a
//! well-formed graph. L010 cannot fire for cycle members (they have
//! a parent edge by definition). L015 still fires because it
//! evaluates other References slots, not the parent edge.
//!
//! Stale source: ADRs in the stale archive (`is_stale`) are exempt
//! from L010–L017 entirely.

use std::collections::HashMap;

use crate::model::{AdrId, AdrRecord, RelVerb, Relationship, Status};
use crate::nav::{compute_parent_edges, walk_parent_chain};
use crate::report::Diagnostic;

pub fn check(records: &[AdrRecord], diags: &mut Vec<Diagnostic>) {
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

        // L006: Legacy verb deprecation (per-relationship)
        for rel in &record.relationships {
            check_legacy_verb(record, rel, diags);
        }

        for rel in &record.relationships {
            check_single_link(record, rel, &by_id, diags);
        }
    }

    // L003: Supersedes-status consistency (cross-file)
    check_supersedes_consistency(records, &by_id, diags);

    // L010–L017: tree-structure diagnostics (cross-file, parent-edge graph)
    check_tree_structure(records, &by_id, diags);
}

fn check_single_link(
    source: &AdrRecord,
    rel: &Relationship,
    by_id: &HashMap<&AdrId, &AdrRecord>,
    diags: &mut Vec<Diagnostic>,
) {
    let target_id = &rel.target;

    // Skip further link validation for Root self-references
    if rel.verb == RelVerb::Root && rel.target == source.id {
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
    // Exempt Supersedes relationships: they inherently target stale ADRs.
    if let Some(target_record) = by_id.get(target_id)
        && target_record.is_stale
        && !source.is_stale
        && rel.verb != RelVerb::Supersedes
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

/// L006: Legacy relationship verb. AFM-0009 R1 restricts the vocabulary
/// to Root, References, Supersedes; any other parsed verb is legacy
/// and emits a deprecation warning with migration guidance.
///
/// `RelVerb::migration()` in model.rs is the single source of truth
/// for the legacy/permitted partition: it returns `Some(_)` exactly
/// when a verb is legacy. Adding or retiring a verb requires only
/// updating that helper.
fn check_legacy_verb(source: &AdrRecord, rel: &Relationship, diags: &mut Vec<Diagnostic>) {
    if let Some(migration) = rel.verb.migration() {
        diags.push(Diagnostic::warning(
            "L006",
            &source.file_path,
            rel.line,
            format!(
                "{}: legacy relationship verb `{}` → {} — {migration} \
                 (per AFM-0009)",
                source.id, rel.verb, rel.target,
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

/// L010–L017: parent-edge tree-structure diagnostics.
///
/// Operates on the parent-edge projection (see `nav::compute_parent_edges`)
/// rather than the full citation graph. Stale source ADRs are excluded
/// from these checks — orphaned ancestry is expected for retired ADRs.
fn check_tree_structure(
    records: &[AdrRecord],
    by_id: &HashMap<&AdrId, &AdrRecord>,
    diags: &mut Vec<Diagnostic>,
) {
    let parent_edges = compute_parent_edges(records);

    // Detect cycles once (L013) — emit at the lowest-numbered ADR
    // in each cycle to make output deterministic.
    let cycle_members = detect_cycle_members(&parent_edges);
    emit_cycle_diagnostics(records, &cycle_members, diags);

    for record in records {
        if record.is_stale {
            continue;
        }
        let is_root = record.is_root();

        // L010: non-root ADR has no References-based parent.
        // Root + Supersedes (root with predecessor) is exempt — roots
        // do not need a parent edge regardless of other relationships.
        if !is_root && !parent_edges.contains_key(&record.id) {
            // Find a sensible line: the Related section's first relationship,
            // or fall back to status_line, or 0.
            let line = record
                .relationships
                .first()
                .map_or(record.status_line, |r| r.line);
            diags.push(Diagnostic::warning(
                "L010",
                &record.file_path,
                line,
                format!(
                    "{}: non-root ADR has no `References:` target — \
                     every non-root ADR needs a structural parent",
                    record.id,
                ),
            ));
            continue;
        }

        let Some(parent_id) = parent_edges.get(&record.id) else {
            continue; // root — no further checks
        };

        // Cycle members have a structurally invalid parent edge. L013
        // already reports the cycle; piling on L011/L012/L016/L017
        // (all of which evaluate parent-edge quality) adds noise
        // without new signal — the user must break the cycle first,
        // after which the remaining diagnostics will re-evaluate
        // against a well-formed graph. L015 (root-first ordering)
        // evaluates other References, not the parent edge, so it
        // keeps firing.
        let in_cycle = cycle_members.contains(&record.id);

        // Find the parent's relationship line for diagnostic location
        let parent_rel_line = record
            .relationships
            .iter()
            .find(|r| r.verb == RelVerb::References && r.target == *parent_id)
            .map_or(0, |r| r.line);

        // L011: cross-domain parent (suppressed by Parent-cross-domain field)
        // Skip when parent target is dangling — L001 already covers that defect;
        // emitting L011 too would double-report.
        // Skip when in cycle — L013 already reports the structural defect.
        if !in_cycle && parent_id.prefix != record.id.prefix && by_id.contains_key(parent_id) {
            let suppressed = record
                .parent_cross_domain
                .as_ref()
                .is_some_and(|allowed| allowed == parent_id);
            if !suppressed {
                diags.push(Diagnostic::warning(
                    "L011",
                    &record.file_path,
                    parent_rel_line,
                    format!(
                        "{} → {parent_id}: cross-domain parent edge — \
                         add `Parent-cross-domain: {parent_id} — <reason>` \
                         to the preamble to suppress, or pick a same-domain parent",
                        record.id,
                    ),
                ));
            }
        }

        // Look up parent record for status/tier checks.
        // L012/L016/L017 all evaluate parent-edge quality and are
        // suppressed for cycle members (L013 dominates).
        if !in_cycle && let Some(parent_record) = by_id.get(parent_id) {
            // L012 / L017: non-Accepted parent
            match &parent_record.status {
                Some(Status::Accepted) => {}
                Some(Status::SupersededBy(succ)) => {
                    diags.push(Diagnostic::warning(
                        "L017",
                        &record.file_path,
                        parent_rel_line,
                        format!(
                            "{} → {parent_id}: parent edge points at a superseded ADR \
                             (succeeded by {succ}) — redirect to the successor",
                            record.id,
                        ),
                    ));
                }
                Some(other) => {
                    diags.push(Diagnostic::warning(
                        "L012",
                        &record.file_path,
                        parent_rel_line,
                        format!(
                            "{} → {parent_id}: parent edge target is `{}`, not `Accepted` — \
                             advisory only; chain still flows through",
                            record.id,
                            other.short_display(),
                        ),
                    ));
                }
                None => {
                    diags.push(Diagnostic::warning(
                        "L012",
                        &record.file_path,
                        parent_rel_line,
                        format!(
                            "{} → {parent_id}: parent edge target has no status — \
                             advisory only; chain still flows through",
                            record.id,
                        ),
                    ));
                }
            }

            // L016: parent tier lower-leverage than child tier
            // (rank: S=0 strongest leverage, D=4 weakest; parent should
            // be ≤ child's rank)
            if let (Some(parent_tier), Some(child_tier)) = (parent_record.tier, record.tier)
                && parent_tier.rank() > child_tier.rank()
            {
                diags.push(Diagnostic::warning(
                    "L016",
                    &record.file_path,
                    parent_rel_line,
                    format!(
                        "{} ({}) → {parent_id} ({}): parent tier is weaker leverage \
                         than child — heuristic, may be intentional",
                        record.id, child_tier, parent_tier,
                    ),
                ));
            }
        }

        // L015: heuristic — parent is a root while same-domain Accepted
        // non-root candidates appear later in References. Suspicious flat
        // structure: a more specific parent is probably available.
        if let Some(parent_record) = by_id.get(parent_id)
            && parent_record.is_root()
        {
            let has_better_candidate = record
                .relationships
                .iter()
                .filter(|r| r.verb == RelVerb::References && r.target != *parent_id)
                .any(|r| {
                    by_id.get(&r.target).is_some_and(|cand| {
                        cand.id.prefix == record.id.prefix
                            && !cand.is_root()
                            && cand.status.as_ref() == Some(&Status::Accepted)
                    })
                });
            if has_better_candidate {
                diags.push(Diagnostic::warning(
                    "L015",
                    &record.file_path,
                    parent_rel_line,
                    format!(
                        "{} → {parent_id}: first reference is a root while later \
                         References include same-domain non-root candidates — \
                         consider promoting one to first position",
                        record.id,
                    ),
                ));
            }
        }
    }

    // L014: non-root ADR unreachable from any root (separate pass after
    // cycle detection so cycle members are not double-reported).
    for record in records {
        if record.is_stale || record.is_root() {
            continue;
        }
        if !parent_edges.contains_key(&record.id) {
            continue; // already handled by L010
        }
        if cycle_members.contains(&record.id) {
            continue; // L013 already covers this
        }
        match walk_parent_chain(&record.id, &parent_edges) {
            Ok(terminal) => {
                // Skip when terminal is dangling — L001 already covers that
                // defect; L014 would double-report on the same root cause.
                if !by_id.contains_key(&terminal) {
                    continue;
                }
                let reaches_root = by_id
                    .get(&terminal)
                    .is_some_and(|t| t.is_root());
                if !reaches_root {
                    let line = record
                        .relationships
                        .first()
                        .map_or(record.status_line, |r| r.line);
                    diags.push(Diagnostic::warning(
                        "L014",
                        &record.file_path,
                        line,
                        format!(
                            "{}: parent chain ends at {terminal}, which is not a root — \
                             non-root ADR unreachable from any root",
                            record.id,
                        ),
                    ));
                }
            }
            Err(_) => {
                // Cycle — already handled
            }
        }
    }
}

/// Identify all ADR IDs participating in a parent-edge cycle.
///
/// Walks each child once with a visited-set. Members of any detected
/// cycle are added to the returned set.
fn detect_cycle_members(parent_edges: &HashMap<AdrId, AdrId>) -> std::collections::HashSet<AdrId> {
    use std::collections::HashSet;

    let mut cycle_set: HashSet<AdrId> = HashSet::new();
    let mut globally_seen: HashSet<AdrId> = HashSet::new();

    for start in parent_edges.keys() {
        if globally_seen.contains(start) {
            continue;
        }
        let mut path: Vec<AdrId> = Vec::new();
        let mut path_set: HashSet<AdrId> = HashSet::new();
        let mut current = start.clone();
        loop {
            if path_set.contains(&current) {
                // Found cycle — add every node from `current` onward to cycle_set
                if let Some(start_idx) = path.iter().position(|id| id == &current) {
                    for id in &path[start_idx..] {
                        cycle_set.insert(id.clone());
                    }
                }
                break;
            }
            if globally_seen.contains(&current) {
                // Already explored from another start; if it was in a cycle,
                // cycle_set already contains it
                break;
            }
            path.push(current.clone());
            path_set.insert(current.clone());
            match parent_edges.get(&current) {
                Some(parent) => current = parent.clone(),
                None => break,
            }
        }
        for id in path {
            globally_seen.insert(id);
        }
    }

    cycle_set
}

fn emit_cycle_diagnostics(
    records: &[AdrRecord],
    cycle_members: &std::collections::HashSet<AdrId>,
    diags: &mut Vec<Diagnostic>,
) {
    if cycle_members.is_empty() {
        return;
    }
    // Emit one diagnostic per ADR in the cycle, anchored at the
    // first References line.
    for record in records {
        if !cycle_members.contains(&record.id) || record.is_stale {
            continue;
        }
        let line = record
            .relationships
            .iter()
            .find(|r| r.verb == RelVerb::References)
            .map_or(record.status_line, |r| r.line);
        diags.push(Diagnostic::warning(
            "L013",
            &record.file_path,
            line,
            format!(
                "{}: parent-edge graph contains a cycle through this ADR — \
                 break the cycle by re-rooting one of the participants",
                record.id,
            ),
        ));
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
        check(&records, &mut diags);
        assert!(diags.is_empty(), "expected no diags, got: {diags:?}");
    }

    #[test]
    fn dangling_link_produces_l001() {
        let records = vec![make_record_with_rels(
            "CHE",
            1,
            vec![(RelVerb::References, make_id("CHE", 99))],
        )];
        let mut diags = Vec::new();
        check(&records, &mut diags);
        assert!(
            diags.iter().any(|d| d.rule == "L001"),
            "expected L001, got: {diags:?}"
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
        check(&records, &mut diags);
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
        check(&records, &mut diags);
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
        check(&records, &mut diags);
        assert!(
            diags.iter().any(|d| d.rule == "L009"),
            "Root + References should trigger L009, got: {diags:?}"
        );
    }

    #[test]
    fn root_and_supersedes_no_l009() {
        let record = make_record_with_rels(
            "CHE",
            2,
            vec![
                (RelVerb::Root, make_id("CHE", 2)),
                (RelVerb::Supersedes, make_id("CHE", 1)),
            ],
        );
        let mut target = make_record_with_rels("CHE", 1, vec![(RelVerb::Root, make_id("CHE", 1))]);
        target.status = Some(Status::SupersededBy(make_id("CHE", 2)));
        target.status_raw = Some("Superseded by CHE-0002".into());

        let records = vec![record, target];
        let mut diags = Vec::new();
        check(&records, &mut diags);
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
        ];
        let mut diags = Vec::new();
        check(&records, &mut diags);
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
        check(&records, &mut diags);
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
        check(&records, &mut diags);
        assert!(
            diags.iter().any(|d| d.rule == "L007"),
            "expected L007, got: {diags:?}"
        );
    }

    #[test]
    fn supersedes_stale_no_l007() {
        let mut target = make_record_with_rels("CHE", 1, vec![(RelVerb::Root, make_id("CHE", 1))]);
        target.is_stale = true;
        target.status = Some(Status::SupersededBy(make_id("CHE", 2)));
        target.status_raw = Some("Superseded by CHE-0002".into());

        let records = vec![
            make_record_with_rels("CHE", 2, vec![(RelVerb::Supersedes, make_id("CHE", 1))]),
            target,
        ];
        let mut diags = Vec::new();
        check(&records, &mut diags);
        assert!(
            !diags.iter().any(|d| d.rule == "L007"),
            "Supersedes→stale should not trigger L007, got: {diags:?}"
        );
    }

    #[test]
    fn stale_source_references_stale_no_l007() {
        let mut source =
            make_record_with_rels("CHE", 2, vec![(RelVerb::References, make_id("CHE", 1))]);
        source.is_stale = true;

        let mut target = make_record_with_rels("CHE", 1, vec![(RelVerb::Root, make_id("CHE", 1))]);
        target.is_stale = true;

        let records = vec![source, target];
        let mut diags = Vec::new();
        check(&records, &mut diags);
        assert!(
            !diags.iter().any(|d| d.rule == "L007"),
            "stale→stale should not trigger L007, got: {diags:?}"
        );
    }

    #[test]
    fn legacy_forward_verb_produces_l006() {
        // "Depends on" is a legacy forward verb → L006 with "use References".
        let records = vec![
            make_record_with_rels("CHE", 1, vec![(RelVerb::DependsOn, make_id("CHE", 2))]),
            make_record_with_rels("CHE", 2, vec![(RelVerb::Root, make_id("CHE", 2))]),
        ];
        let mut diags = Vec::new();
        check(&records, &mut diags);
        let l006: Vec<_> = diags.iter().filter(|d| d.rule == "L006").collect();
        assert_eq!(l006.len(), 1, "expected exactly one L006, got: {diags:?}");
        assert!(
            l006[0].message.contains("Depends on"),
            "L006 message should name the legacy verb, got: {}",
            l006[0].message
        );
        assert!(
            l006[0].message.contains("use References"),
            "L006 message should include migration guidance, got: {}",
            l006[0].message
        );
    }

    #[test]
    fn legacy_reverse_verb_produces_l006_with_remove_guidance() {
        // "Informs" is a legacy reverse verb → L006 with "remove (reverse verb)".
        let records = vec![
            make_record_with_rels("CHE", 1, vec![(RelVerb::Informs, make_id("CHE", 2))]),
            make_record_with_rels("CHE", 2, vec![(RelVerb::Root, make_id("CHE", 2))]),
        ];
        let mut diags = Vec::new();
        check(&records, &mut diags);
        let l006: Vec<_> = diags.iter().filter(|d| d.rule == "L006").collect();
        assert_eq!(l006.len(), 1, "expected exactly one L006, got: {diags:?}");
        assert!(
            l006[0].message.contains("remove (reverse verb)"),
            "reverse verb should suggest removal, got: {}",
            l006[0].message
        );
    }

    #[test]
    fn permitted_verbs_no_l006() {
        // Root, References, Supersedes — all permitted, no L006.
        let mut target = make_record_with_rels("CHE", 1, vec![(RelVerb::Root, make_id("CHE", 1))]);
        target.status = Some(Status::SupersededBy(make_id("CHE", 3)));
        target.status_raw = Some("Superseded by CHE-0003".into());

        let records = vec![
            make_record_with_rels(
                "CHE",
                3,
                vec![
                    (RelVerb::Root, make_id("CHE", 3)),
                    (RelVerb::Supersedes, make_id("CHE", 1)),
                ],
            ),
            make_record_with_rels("CHE", 2, vec![(RelVerb::References, make_id("CHE", 3))]),
            target,
        ];
        let mut diags = Vec::new();
        check(&records, &mut diags);
        assert!(
            !diags.iter().any(|d| d.rule == "L006"),
            "permitted verbs should not trigger L006, got: {diags:?}"
        );
    }

    #[test]
    fn legacy_verb_with_dangling_target_emits_both_l006_and_l001() {
        // A legacy verb pointing to a missing target should produce
        // both diagnostics — they are independent concerns.
        let records = vec![make_record_with_rels(
            "CHE",
            1,
            vec![(RelVerb::Extends, make_id("CHE", 999))],
        )];
        let mut diags = Vec::new();
        check(&records, &mut diags);
        assert!(
            diags.iter().any(|d| d.rule == "L006"),
            "expected L006 (legacy verb), got: {diags:?}"
        );
        assert!(
            diags.iter().any(|d| d.rule == "L001"),
            "expected L001 (dangling), got: {diags:?}"
        );
    }

    #[test]
    fn legacy_verb_to_stale_target_emits_l006_and_l007() {
        // A legacy verb pointing to a stale (but existing) target
        // produces both L006 (verb deprecation) and L007 (stale ref).
        // Pins the policy that lint rules co-emit on a single rel.
        let mut target = make_record_with_rels("CHE", 1, vec![(RelVerb::Root, make_id("CHE", 1))]);
        target.is_stale = true;

        let records = vec![
            make_record_with_rels("CHE", 2, vec![(RelVerb::DependsOn, make_id("CHE", 1))]),
            target,
        ];
        let mut diags = Vec::new();
        check(&records, &mut diags);
        assert!(
            diags.iter().any(|d| d.rule == "L006"),
            "expected L006 (legacy verb), got: {diags:?}"
        );
        assert!(
            diags.iter().any(|d| d.rule == "L007"),
            "expected L007 (stale ref), got: {diags:?}"
        );
    }

    #[test]
    fn every_legacy_verb_triggers_l006() {
        // Bind RelVerb::legacy() to L006 emission. If a future verb
        // joins the legacy set without a matching migration() arm,
        // this test catches it.
        for &verb in RelVerb::legacy() {
            let records = vec![
                make_record_with_rels("CHE", 1, vec![(verb, make_id("CHE", 2))]),
                make_record_with_rels("CHE", 2, vec![(RelVerb::Root, make_id("CHE", 2))]),
            ];
            let mut diags = Vec::new();
            check(&records, &mut diags);
            assert!(
                diags.iter().any(|d| d.rule == "L006"),
                "legacy verb {verb:?} should trigger L006, got: {diags:?}"
            );
        }
    }

    #[test]
    fn no_permitted_verb_triggers_l006() {
        // Bind RelVerb::permitted() to absence of L006. Catches the
        // inverse drift: a permitted verb accidentally returning
        // Some(_) from migration().
        for &verb in RelVerb::permitted() {
            // Use self-Root for Root verb, otherwise point at CHE-0002.
            let target = if verb == RelVerb::Root {
                make_id("CHE", 1)
            } else {
                make_id("CHE", 2)
            };
            let mut other = make_record_with_rels("CHE", 2, vec![(RelVerb::Root, make_id("CHE", 2))]);
            // Supersedes requires the target's status to be set, else L003 fires
            // (independent of L006). Pre-set it to keep diags clean.
            if verb == RelVerb::Supersedes {
                other.status = Some(Status::SupersededBy(make_id("CHE", 1)));
                other.status_raw = Some("Superseded by CHE-0001".into());
            }
            let records = vec![
                make_record_with_rels("CHE", 1, vec![(verb, target)]),
                other,
            ];
            let mut diags = Vec::new();
            check(&records, &mut diags);
            assert!(
                !diags.iter().any(|d| d.rule == "L006"),
                "permitted verb {verb:?} should not trigger L006, got: {diags:?}"
            );
        }
    }

    // ── Tree-structure diagnostics (L010–L017) ─────────────────────

    #[test]
    fn non_root_without_references_produces_l010() {
        // CHE-0002 has no References — should trigger L010.
        let records = vec![
            make_record_with_rels("CHE", 1, vec![(RelVerb::Root, make_id("CHE", 1))]),
            make_record_with_rels("CHE", 2, vec![]),
        ];
        let mut diags = Vec::new();
        check(&records, &mut diags);
        assert!(
            diags.iter().any(|d| d.rule == "L010"),
            "expected L010, got: {diags:?}"
        );
    }

    #[test]
    fn root_without_references_no_l010() {
        // Root ADR with no References — exempt from L010.
        let records = vec![make_record_with_rels(
            "CHE",
            1,
            vec![(RelVerb::Root, make_id("CHE", 1))],
        )];
        let mut diags = Vec::new();
        check(&records, &mut diags);
        assert!(
            !diags.iter().any(|d| d.rule == "L010"),
            "root should be exempt from L010, got: {diags:?}"
        );
    }

    #[test]
    fn root_with_supersedes_only_no_l010() {
        // Root + Supersedes (root replacing predecessor) — still a root,
        // exempt from L010 even though no References:.
        let mut predecessor = make_record_with_rels(
            "CHE",
            1,
            vec![(RelVerb::Root, make_id("CHE", 1))],
        );
        predecessor.status = Some(Status::SupersededBy(make_id("CHE", 2)));
        predecessor.status_raw = Some("Superseded by CHE-0002".into());

        let new_root = make_record_with_rels(
            "CHE",
            2,
            vec![
                (RelVerb::Root, make_id("CHE", 2)),
                (RelVerb::Supersedes, make_id("CHE", 1)),
            ],
        );

        let records = vec![predecessor, new_root];
        let mut diags = Vec::new();
        check(&records, &mut diags);
        assert!(
            !diags.iter().any(|d| d.rule == "L010"),
            "Root + Supersedes should not trigger L010, got: {diags:?}"
        );
    }

    #[test]
    fn cross_domain_parent_produces_l011() {
        // CHE-0002's first References is COM-0001 (different domain).
        let mut com_root =
            make_record_with_rels("COM", 1, vec![(RelVerb::Root, make_id("COM", 1))]);
        com_root.file_path = PathBuf::from("docs/adr/common/COM-0001-test.md");

        let che = make_record_with_rels(
            "CHE",
            2,
            vec![(RelVerb::References, make_id("COM", 1))],
        );

        let records = vec![com_root, che];
        let mut diags = Vec::new();
        check(&records, &mut diags);
        assert!(
            diags.iter().any(|d| d.rule == "L011"),
            "expected L011, got: {diags:?}"
        );
    }

    #[test]
    fn cross_domain_parent_suppressed_by_field() {
        // Same as above but Parent-cross-domain field allows it.
        let mut com_root =
            make_record_with_rels("COM", 1, vec![(RelVerb::Root, make_id("COM", 1))]);
        com_root.file_path = PathBuf::from("docs/adr/common/COM-0001-test.md");

        let mut che = make_record_with_rels(
            "CHE",
            2,
            vec![(RelVerb::References, make_id("COM", 1))],
        );
        che.parent_cross_domain = Some(make_id("COM", 1));
        che.parent_cross_domain_reason = "boundary ADR".into();

        let records = vec![com_root, che];
        let mut diags = Vec::new();
        check(&records, &mut diags);
        assert!(
            !diags.iter().any(|d| d.rule == "L011"),
            "Parent-cross-domain field should suppress L011, got: {diags:?}"
        );
    }

    #[test]
    fn cross_domain_suppression_only_for_named_target() {
        // Field allows COM-0001 but parent is COM-0002 — must still warn.
        let mut com1 = make_record_with_rels("COM", 1, vec![(RelVerb::Root, make_id("COM", 1))]);
        com1.file_path = PathBuf::from("docs/adr/common/COM-0001-test.md");
        let mut com2 = make_record_with_rels("COM", 2, vec![(RelVerb::References, make_id("COM", 1))]);
        com2.file_path = PathBuf::from("docs/adr/common/COM-0002-test.md");

        let mut che = make_record_with_rels(
            "CHE",
            5,
            vec![(RelVerb::References, make_id("COM", 2))],
        );
        che.parent_cross_domain = Some(make_id("COM", 1)); // wrong allowance
        let records = vec![com1, com2, che];
        let mut diags = Vec::new();
        check(&records, &mut diags);
        assert!(
            diags.iter().any(|d| d.rule == "L011"),
            "suppression must match actual parent target, got: {diags:?}"
        );
    }

    #[test]
    fn non_accepted_parent_produces_l012() {
        let mut parent = make_record_with_rels("CHE", 1, vec![(RelVerb::Root, make_id("CHE", 1))]);
        parent.status = Some(Status::Draft);
        parent.status_raw = Some("Draft".into());

        let child = make_record_with_rels(
            "CHE",
            2,
            vec![(RelVerb::References, make_id("CHE", 1))],
        );
        let records = vec![parent, child];
        let mut diags = Vec::new();
        check(&records, &mut diags);
        assert!(
            diags.iter().any(|d| d.rule == "L012"),
            "expected L012 for Draft parent, got: {diags:?}"
        );
    }

    #[test]
    fn superseded_parent_produces_l017_not_l012() {
        let mut parent = make_record_with_rels("CHE", 1, vec![(RelVerb::Root, make_id("CHE", 1))]);
        parent.status = Some(Status::SupersededBy(make_id("CHE", 9)));
        parent.status_raw = Some("Superseded by CHE-0009".into());

        let mut succ = make_record_with_rels(
            "CHE",
            9,
            vec![
                (RelVerb::Root, make_id("CHE", 9)),
                (RelVerb::Supersedes, make_id("CHE", 1)),
            ],
        );
        succ.status = Some(Status::Accepted);

        let child = make_record_with_rels(
            "CHE",
            2,
            vec![(RelVerb::References, make_id("CHE", 1))],
        );
        let records = vec![parent, succ, child];
        let mut diags = Vec::new();
        check(&records, &mut diags);
        assert!(
            diags.iter().any(|d| d.rule == "L017"),
            "expected L017, got: {diags:?}"
        );
        assert!(
            !diags.iter().any(|d| d.rule == "L012"),
            "L017 supersedes L012 for superseded parent, got: {diags:?}"
        );
    }

    #[test]
    fn parent_edge_cycle_produces_l013() {
        // CHE-0002 → CHE-0003 → CHE-0002 cycle (no root reachable)
        let a = make_record_with_rels(
            "CHE",
            2,
            vec![(RelVerb::References, make_id("CHE", 3))],
        );
        let b = make_record_with_rels(
            "CHE",
            3,
            vec![(RelVerb::References, make_id("CHE", 2))],
        );
        let mut diags = Vec::new();
        check(&[a, b], &mut diags);
        let l013_count = diags.iter().filter(|d| d.rule == "L013").count();
        assert_eq!(l013_count, 2, "expected L013 for both cycle members, got: {diags:?}");
    }

    #[test]
    fn secondary_reference_cycle_does_not_trigger_l013() {
        // CHE-0002 references CHE-0001 (parent) AND CHE-0003 (secondary).
        // CHE-0003 references CHE-0001 (parent) AND CHE-0002 (secondary).
        // No parent-edge cycle exists — only a secondary citation cycle.
        let root = make_record_with_rels("CHE", 1, vec![(RelVerb::Root, make_id("CHE", 1))]);
        let a = make_record_with_rels(
            "CHE",
            2,
            vec![
                (RelVerb::References, make_id("CHE", 1)),
                (RelVerb::References, make_id("CHE", 3)),
            ],
        );
        let b = make_record_with_rels(
            "CHE",
            3,
            vec![
                (RelVerb::References, make_id("CHE", 1)),
                (RelVerb::References, make_id("CHE", 2)),
            ],
        );
        let mut diags = Vec::new();
        check(&[root, a, b], &mut diags);
        assert!(
            !diags.iter().any(|d| d.rule == "L013"),
            "secondary cycles must not trigger L013, got: {diags:?}"
        );
    }

    #[test]
    fn unreachable_from_root_produces_l014() {
        // Three-ADR chain CHE-0002 → CHE-0003 → CHE-0004, none of which is
        // a root. Terminal CHE-0004 exists but `is_root()` is false, so
        // L014 fires. (Dangling terminals are suppressed to avoid double-
        // reporting with L001.)
        let a = make_record_with_rels(
            "CHE",
            2,
            vec![(RelVerb::References, make_id("CHE", 3))],
        );
        let b = make_record_with_rels(
            "CHE",
            3,
            vec![(RelVerb::References, make_id("CHE", 4))],
        );
        // CHE-0004: not a root (no Root self-ref), no parent edge — chain
        // terminates here via walk_parent_chain's "no edge" exit.
        let c = make_record_with_rels(
            "CHE",
            4,
            vec![(RelVerb::Supersedes, make_id("CHE", 99))], // forward but not References
        );
        let mut diags = Vec::new();
        check(&[a, b, c], &mut diags);
        assert!(
            diags.iter().any(|d| d.rule == "L014"),
            "expected L014, got: {diags:?}"
        );
    }

    #[test]
    fn dangling_terminal_does_not_double_report_l014() {
        // CHE-0002 → CHE-0099 (dangling). L001 already covers the dangling
        // reference; L014 must NOT fire to avoid double-reporting.
        let a = make_record_with_rels(
            "CHE",
            2,
            vec![(RelVerb::References, make_id("CHE", 99))],
        );
        let mut diags = Vec::new();
        check(&[a], &mut diags);
        assert!(
            !diags.iter().any(|d| d.rule == "L014"),
            "L014 must not fire on dangling terminal, got: {diags:?}"
        );
    }

    #[test]
    fn dangling_cross_domain_parent_does_not_double_report_l011() {
        // PAR-0001 → CHE-0099 (dangling cross-domain). L001 covers it;
        // L011 must not fire (would be a misleading second diagnostic
        // for the same root cause).
        let a = make_record_with_rels(
            "PAR",
            1,
            vec![(RelVerb::References, make_id("CHE", 99))],
        );
        let mut diags = Vec::new();
        check(&[a], &mut diags);
        assert!(
            !diags.iter().any(|d| d.rule == "L011"),
            "L011 must not fire on dangling cross-domain target, got: {diags:?}"
        );
    }

    #[test]
    fn reachable_from_root_no_l014() {
        let root = make_record_with_rels("CHE", 1, vec![(RelVerb::Root, make_id("CHE", 1))]);
        let mid = make_record_with_rels(
            "CHE",
            2,
            vec![(RelVerb::References, make_id("CHE", 1))],
        );
        let leaf = make_record_with_rels(
            "CHE",
            3,
            vec![(RelVerb::References, make_id("CHE", 2))],
        );
        let mut diags = Vec::new();
        check(&[root, mid, leaf], &mut diags);
        assert!(
            !diags.iter().any(|d| d.rule == "L014"),
            "chain reaching root must not trigger L014, got: {diags:?}"
        );
    }

    #[test]
    fn root_first_with_local_candidate_produces_l015() {
        // CHE-0003 references root CHE-0001 first, and same-domain
        // Accepted non-root CHE-0002 second → L015.
        let root = make_record_with_rels("CHE", 1, vec![(RelVerb::Root, make_id("CHE", 1))]);
        let mid = make_record_with_rels(
            "CHE",
            2,
            vec![(RelVerb::References, make_id("CHE", 1))],
        );
        let leaf = make_record_with_rels(
            "CHE",
            3,
            vec![
                (RelVerb::References, make_id("CHE", 1)),
                (RelVerb::References, make_id("CHE", 2)),
            ],
        );
        let mut diags = Vec::new();
        check(&[root, mid, leaf], &mut diags);
        assert!(
            diags.iter().any(|d| d.rule == "L015"),
            "expected L015, got: {diags:?}"
        );
    }

    #[test]
    fn root_first_no_other_candidates_no_l015() {
        // Root is genuine parent — no later References to consider.
        let root = make_record_with_rels("CHE", 1, vec![(RelVerb::Root, make_id("CHE", 1))]);
        let leaf = make_record_with_rels(
            "CHE",
            3,
            vec![(RelVerb::References, make_id("CHE", 1))],
        );
        let mut diags = Vec::new();
        check(&[root, leaf], &mut diags);
        assert!(
            !diags.iter().any(|d| d.rule == "L015"),
            "no other candidates means no L015, got: {diags:?}"
        );
    }

    #[test]
    fn l015_ignores_non_accepted_candidates() {
        // CHE-0002 is Draft, CHE-0001 is the root parent — Draft must
        // not be flagged as a "better candidate".
        let root = make_record_with_rels("CHE", 1, vec![(RelVerb::Root, make_id("CHE", 1))]);
        let mut mid = make_record_with_rels(
            "CHE",
            2,
            vec![(RelVerb::References, make_id("CHE", 1))],
        );
        mid.status = Some(Status::Draft);
        let leaf = make_record_with_rels(
            "CHE",
            3,
            vec![
                (RelVerb::References, make_id("CHE", 1)),
                (RelVerb::References, make_id("CHE", 2)),
            ],
        );
        let mut diags = Vec::new();
        check(&[root, mid, leaf], &mut diags);
        assert!(
            !diags.iter().any(|d| d.rule == "L015"),
            "Draft candidate must not trigger L015, got: {diags:?}"
        );
    }

    #[test]
    fn lower_tier_parent_produces_l016() {
        // Parent is D-tier (rank 4), child is B-tier (rank 2) → parent
        // is lower leverage than child → L016.
        let mut parent = make_record_with_rels("CHE", 1, vec![(RelVerb::Root, make_id("CHE", 1))]);
        parent.tier = Some(Tier::D);
        let mut child = make_record_with_rels(
            "CHE",
            2,
            vec![(RelVerb::References, make_id("CHE", 1))],
        );
        child.tier = Some(Tier::B);
        let mut diags = Vec::new();
        check(&[parent, child], &mut diags);
        assert!(
            diags.iter().any(|d| d.rule == "L016"),
            "expected L016, got: {diags:?}"
        );
    }

    #[test]
    fn same_or_higher_tier_parent_no_l016() {
        let mut parent = make_record_with_rels("CHE", 1, vec![(RelVerb::Root, make_id("CHE", 1))]);
        parent.tier = Some(Tier::S);
        let mut child = make_record_with_rels(
            "CHE",
            2,
            vec![(RelVerb::References, make_id("CHE", 1))],
        );
        child.tier = Some(Tier::B);
        let mut diags = Vec::new();
        check(&[parent, child], &mut diags);
        assert!(
            !diags.iter().any(|d| d.rule == "L016"),
            "higher-tier parent should not trigger L016, got: {diags:?}"
        );
    }

    #[test]
    fn l012_l007_co_emission_for_stale_non_accepted_parent() {
        // Pin co-emission: stale Draft parent emits both L007 and L012.
        let mut parent = make_record_with_rels("CHE", 1, vec![(RelVerb::Root, make_id("CHE", 1))]);
        parent.status = Some(Status::Draft);
        parent.is_stale = true;

        let child = make_record_with_rels(
            "CHE",
            2,
            vec![(RelVerb::References, make_id("CHE", 1))],
        );
        let mut diags = Vec::new();
        check(&[parent, child], &mut diags);
        assert!(
            diags.iter().any(|d| d.rule == "L007"),
            "expected L007 for stale ref, got: {diags:?}"
        );
        assert!(
            diags.iter().any(|d| d.rule == "L012"),
            "expected L012 for non-Accepted parent, got: {diags:?}"
        );
    }

    #[test]
    fn stale_source_skipped_for_tree_structure() {
        // Stale source ADRs are exempt from L010-L017.
        let mut stale = make_record_with_rels("CHE", 2, vec![]);
        stale.is_stale = true;
        let mut diags = Vec::new();
        check(&[stale], &mut diags);
        assert!(
            !diags.iter().any(|d| matches!(&*d.rule, "L010" | "L011" | "L012" | "L013" | "L014" | "L015" | "L016" | "L017")),
            "stale source should be exempt from tree-structure rules, got: {diags:?}"
        );
    }

    // ── Step 8 gap-filling tests ──────────────────────────────────────

    #[test]
    fn cross_domain_suppression_independent_of_reason_text() {
        // L011 suppression checks the parent_cross_domain ID only;
        // the reason text is documentation for human reviewers and
        // is never inspected by the rule. Verifies that an empty
        // reason still suppresses L011 — i.e. the rule does not
        // require non-empty reason text.
        let root = make_record_with_rels("COM", 1, vec![(RelVerb::Root, make_id("COM", 1))]);
        let mut child = make_record_with_rels(
            "CHE",
            5,
            vec![(RelVerb::References, make_id("COM", 1))],
        );
        child.parent_cross_domain = Some(make_id("COM", 1));
        child.parent_cross_domain_reason = String::new(); // empty reason
        let mut diags = Vec::new();
        check(&[root, child], &mut diags);
        assert!(
            !diags.iter().any(|d| d.rule == "L011"),
            "empty-reason Parent-cross-domain must still suppress L011, got: {diags:?}"
        );
    }

    #[test]
    fn l017_takes_precedence_over_l012_in_cycle() {
        // Cycle CHE-0002 ↔ CHE-0003 where CHE-0003 is `Superseded by`.
        // L013 (cycle) fires for both members. L017 should NOT fire on
        // top of L013 for CHE-0002, since cycle membership is the
        // dominant defect (cycle members are excluded from per-record
        // status checks via the cycle_members short-circuit).
        let mut a = make_record_with_rels(
            "CHE",
            2,
            vec![(RelVerb::References, make_id("CHE", 3))],
        );
        a.status = Some(Status::Accepted);
        let mut b = make_record_with_rels(
            "CHE",
            3,
            vec![(RelVerb::References, make_id("CHE", 2))],
        );
        b.status = Some(Status::SupersededBy(make_id("CHE", 99)));
        let mut diags = Vec::new();
        check(&[a, b], &mut diags);
        // Both members should produce L013
        let l013s: Vec<_> = diags.iter().filter(|d| d.rule == "L013").collect();
        assert_eq!(l013s.len(), 2, "expected 2× L013, got: {diags:?}");
        // L017 should NOT fire for CHE-0002 (its parent CHE-0003 is in the cycle)
        // We accept either: no L017 at all, OR L017 only for non-cycle-members.
        // Since both members are in the cycle, L017 should not fire here.
        assert!(
            !diags.iter().any(|d| d.rule == "L017"),
            "L017 should not fire for cycle-member parents, got: {diags:?}"
        );
    }

    #[test]
    fn l015_does_not_fire_when_no_root_first() {
        // CHE-0005's first ref is a non-root ADR. L015 only fires when
        // the first ref IS a root and a same-domain non-root sibling
        // exists later — neither condition holds here.
        let parent = make_record_with_rels("CHE", 2, vec![(RelVerb::References, make_id("CHE", 1))]);
        let candidate = make_record_with_rels("CHE", 7, vec![(RelVerb::References, make_id("CHE", 1))]);
        let child = make_record_with_rels(
            "CHE",
            5,
            vec![
                (RelVerb::References, make_id("CHE", 2)),  // first ref: non-root
                (RelVerb::References, make_id("CHE", 7)),  // later: also non-root
            ],
        );
        let root = make_record_with_rels("CHE", 1, vec![(RelVerb::Root, make_id("CHE", 1))]);
        let mut diags = Vec::new();
        check(&[root, parent, candidate, child], &mut diags);
        assert!(
            !diags.iter().any(|d| d.rule == "L015"),
            "L015 must not fire when first ref is not a Root, got: {diags:?}"
        );
    }
}
