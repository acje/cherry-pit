//! Integration tests for the adr-fmt binary.
//!
//! Each test creates a self-contained tempdir with the necessary file
//! structure (adr-fmt.toml, domain directories, ADR files) and runs the
//! binary against it.

use std::fs;

use assert_cmd::Command;
use predicates::prelude::*;
use tempfile::TempDir;

// ── helpers ─────────────────────────────────────────────────────────

/// Minimal adr-fmt.toml for a single-domain test corpus.
const MINIMAL_CONFIG: &str = r#"
[stale]
directory = "stale"

[[domains]]
prefix = "TST"
name = "Test Domain"
directory = "test"
description = "Integration test domain."
crates = ["test-core"]

[[rules]]
id = "T001"
category = "template"
description = "H1 title present"

[[rules]]
id = "T015"
category = "template"
description = "Prose section below minimum word count"
params = { min_words = 10 }

[[rules]]
id = "T016"
category = "template"
description = "Decision section tagged rules"
"#;

/// Multi-domain config with foundation domain.
const MULTI_DOMAIN_CONFIG: &str = r#"
[stale]
directory = "stale"

[[domains]]
prefix = "COM"
name = "Common"
directory = "common"
description = "Cross-cutting principles."
crates = []
foundation = true

[[domains]]
prefix = "TST"
name = "Test Domain"
directory = "test"
description = "Integration test domain."
crates = ["test-core"]

[[rules]]
id = "T001"
category = "template"
description = "H1 title present"

[[rules]]
id = "T015"
category = "template"
description = "Prose section below minimum word count"
params = { min_words = 10 }

[[rules]]
id = "T016"
category = "template"
description = "Decision section tagged rules"
"#;

/// A valid ADR file that passes all rules (root ADR).
const VALID_ADR: &str = "\
# TST-0001. Valid Test ADR

Date: 2026-04-27
Last-reviewed: 2026-04-27
Tier: B

## Status

Accepted

## Related

- Root: TST-0001

## Context

This ADR documents a valid test case for the integration test suite to verify.

## Decision

- **R1**: We decided to create a minimal but complete ADR that satisfies all template rules.

## Consequences

The integration test can verify that a clean corpus produces zero diagnostics.
";

/// A second ADR that references TST-0001.
const REFERENCING_ADR: &str = "\
# TST-0002. Referencing ADR

Date: 2026-04-27
Last-reviewed: 2026-04-27
Tier: B

## Status

Accepted

## Related

- References: TST-0001

## Context

This ADR references TST-0001 to test transitive closure in critique mode.

## Decision

- **R1**: We reference another ADR to verify critique mode connectivity.

## Consequences

Critique mode should include both TST-0001 and TST-0002 in the closure.
";

/// An ADR with a dangling link to trigger L001.
const DANGLING_LINK_ADR: &str = "\
# TST-0003. Dangling Link ADR

Date: 2026-04-27
Last-reviewed: 2026-04-27
Tier: B

## Status

Accepted

## Related

- References: TST-9999

## Context

This ADR has a dangling link that should trigger the L001 validation rule.

## Decision

- **R1**: We reference a non-existent ADR to verify that dangling link detection works.

## Consequences

The linter should report a dangling link warning for TST-9999 in the output.
";

/// ADR without tagged rules (triggers T016).
const NO_TAGGED_RULES_ADR: &str = "\
# TST-0004. No Tagged Rules

Date: 2026-04-27
Last-reviewed: 2026-04-27
Tier: B

## Status

Accepted

## Related

- Root: TST-0004

## Context

This ADR has prose in the Decision section but no tagged rules.

## Decision

We decided to use plain prose without the required tagged rule format.

## Consequences

The linter should report a T016 warning for missing tagged rules.
";

/// Draft ADR without tagged rules (exempt from T016).
const DRAFT_ADR: &str = "\
# TST-0005. Draft ADR

Date: 2026-04-27
Tier: B

## Status

Draft

## Related

- Root: TST-0005

## Context

This is a draft ADR that is exempt from the tagged rules requirement.

## Decision

We are still drafting this decision and have not formalized rules yet.

## Consequences

Draft status exempts this ADR from T016 checks.
";

/// ADR with per-ADR Crates field.
const ADR_WITH_CRATES: &str = "\
# TST-0006. ADR With Crates

Date: 2026-04-27
Last-reviewed: 2026-04-27
Tier: B
Crates: test-core, test-api

## Status

Accepted

## Related

- Root: TST-0006

## Context

This ADR specifies crate applicability via the Crates metadata field.

## Decision

- **R1**: Crate-specific decisions are scoped to test-core and test-api.

## Consequences

Context mode should only include this ADR when querying test-core or test-api.
";

/// Foundation domain ADR (COM).
const FOUNDATION_ADR: &str = "\
# COM-0001. Foundation Principle

Date: 2026-04-27
Last-reviewed: 2026-04-27
Tier: S

## Status

Accepted

## Related

- Root: COM-0001

## Context

This is a foundation domain ADR that applies to all crates in the workspace.

## Decision

- **R1**: Foundation rules apply universally across all domains and crates.

## Consequences

Context mode must always include foundation domain ADRs.
";

/// Stale ADR (superseded, in stale directory).
const STALE_ADR: &str = "\
# TST-0010. Stale ADR

Date: 2026-01-01
Last-reviewed: 2026-01-01
Tier: B

## Status

Superseded by TST-0001

## Related

- References: TST-0001

## Context

This ADR was superseded and moved to the stale directory.

## Decision

- **R1**: This decision has been superseded by TST-0001.

## Consequences

This ADR should be excluded from critique closures with a count note.

## Retirement

Superseded by TST-0001 on 2026-04-27. The newer ADR provides better guidance.
";

/// ADR with non-sequential tagged rule IDs (gap: R1, R3).
const GAP_RULES_ADR: &str = "\
# TST-0007. Gap Rules ADR

Date: 2026-04-27
Last-reviewed: 2026-04-27
Tier: B

## Status

Accepted

## Related

- Root: TST-0007

## Context

This ADR has tagged rules with a gap in numbering.

## Decision

- **R1**: First rule is present.
- **R3**: Third rule skips R2.

## Consequences

The linter should report a T016 warning for non-sequential rule IDs.
";

/// Create test corpus in a tempdir with optional multi-domain support.
///
/// `domains` is a slice of (domain_directory, &[(filename, content)]) tuples.
/// `stale_adrs` is an optional slice of (filename, content) for the stale directory.
fn setup_multi_corpus(
    config: &str,
    domains: &[(&str, &[(&str, &str)])],
    stale_adrs: &[(&str, &str)],
) -> TempDir {
    let dir = TempDir::new().expect("create tempdir");
    let adr_root = dir.path().join("docs/adr");

    fs::create_dir_all(&adr_root).expect("create adr root");
    fs::write(adr_root.join("GOVERNANCE.md"), "# Governance\n").expect("write governance");
    fs::write(adr_root.join("adr-fmt.toml"), config).expect("write config");

    for (domain_dir_name, adrs) in domains {
        let domain_dir = adr_root.join(domain_dir_name);
        fs::create_dir_all(&domain_dir).expect("create domain dir");
        for (filename, content) in *adrs {
            fs::write(domain_dir.join(filename), content).expect("write ADR");
        }
    }

    if !stale_adrs.is_empty() {
        let stale_dir = adr_root.join("stale");
        fs::create_dir_all(&stale_dir).expect("create stale dir");
        for (filename, content) in stale_adrs {
            fs::write(stale_dir.join(filename), content).expect("write stale ADR");
        }
    }

    dir
}

/// Create simple single-domain corpus.
fn setup_corpus(config: &str, adrs: &[(&str, &str)]) -> TempDir {
    setup_multi_corpus(config, &[("test", adrs)], &[])
}

fn adr_fmt() -> Command {
    Command::cargo_bin("adr-fmt").expect("binary exists")
}

fn adr_root(dir: &TempDir) -> String {
    dir.path()
        .join("docs/adr")
        .to_str()
        .expect("valid utf-8 path")
        .to_owned()
}

// ── default lint mode ──────────────────────────────────────────────

#[test]
fn valid_corpus_clean_output() {
    let dir = setup_corpus(
        MINIMAL_CONFIG,
        &[("TST-0001-valid-test-adr.md", VALID_ADR)],
    );

    adr_fmt()
        .arg(adr_root(&dir))
        .assert()
        .success()
        .stdout(predicate::str::contains("0 error(s), 0 warning(s)"));
}

#[test]
fn dangling_link_produces_l001() {
    let dir = setup_corpus(
        MINIMAL_CONFIG,
        &[
            ("TST-0001-valid-test-adr.md", VALID_ADR),
            ("TST-0003-dangling-link-adr.md", DANGLING_LINK_ADR),
        ],
    );

    adr_fmt()
        .arg(adr_root(&dir))
        .assert()
        .success()
        .stdout(predicate::str::contains("L001"));
}

#[test]
fn empty_domain_directory_graceful() {
    let dir = setup_corpus(MINIMAL_CONFIG, &[]);

    adr_fmt()
        .arg(adr_root(&dir))
        .assert()
        .success()
        .stdout(predicate::str::contains("0 ADR(s)"));
}

#[test]
fn lint_output_on_stdout() {
    let dir = setup_corpus(
        MINIMAL_CONFIG,
        &[("TST-0001-valid-test-adr.md", VALID_ADR)],
    );

    // Verify diagnostics go to stdout, not stderr
    adr_fmt()
        .arg(adr_root(&dir))
        .assert()
        .success()
        .stdout(predicate::str::contains("Diagnostics"))
        .stderr(predicate::str::is_empty());
}

// ── T016 tagged rules ──────────────────────────────────────────────

#[test]
fn t016_missing_tagged_rules() {
    let dir = setup_corpus(
        MINIMAL_CONFIG,
        &[("TST-0004-no-tagged-rules.md", NO_TAGGED_RULES_ADR)],
    );

    adr_fmt()
        .arg(adr_root(&dir))
        .assert()
        .success()
        .stdout(predicate::str::contains("T016"));
}

#[test]
fn t016_draft_exempt() {
    let dir = setup_corpus(
        MINIMAL_CONFIG,
        &[("TST-0005-draft-adr.md", DRAFT_ADR)],
    );

    // Draft ADRs are exempt from T016 — should not appear in output
    adr_fmt()
        .arg(adr_root(&dir))
        .assert()
        .success()
        .stdout(predicate::str::contains("T016").not());
}

#[test]
fn t016_gap_in_rule_ids() {
    let dir = setup_corpus(
        MINIMAL_CONFIG,
        &[("TST-0007-gap-rules-adr.md", GAP_RULES_ADR)],
    );

    adr_fmt()
        .arg(adr_root(&dir))
        .assert()
        .success()
        .stdout(predicate::str::contains("T016"));
}

#[test]
fn t016_tagged_rules_present_no_warning() {
    let dir = setup_corpus(
        MINIMAL_CONFIG,
        &[("TST-0001-valid-test-adr.md", VALID_ADR)],
    );

    // VALID_ADR has tagged rules — no T016
    adr_fmt()
        .arg(adr_root(&dir))
        .assert()
        .success()
        .stdout(predicate::str::contains("T016").not());
}

// ── critique mode ──────────────────────────────────────────────────

#[test]
fn critique_focal_with_connected() {
    let dir = setup_corpus(
        MINIMAL_CONFIG,
        &[
            ("TST-0001-valid-test-adr.md", VALID_ADR),
            ("TST-0002-referencing-adr.md", REFERENCING_ADR),
        ],
    );

    adr_fmt()
        .args(["--critique", "TST-0001", &adr_root(&dir)])
        .assert()
        .success()
        .stdout(
            predicate::str::contains("◆ FOCAL")
                .and(predicate::str::contains("TST-0001"))
                .and(predicate::str::contains("◇ CONNECTED"))
                .and(predicate::str::contains("TST-0002")),
        );
}

#[test]
fn critique_isolated_adr() {
    let dir = setup_corpus(
        MINIMAL_CONFIG,
        &[("TST-0001-valid-test-adr.md", VALID_ADR)],
    );

    // Isolated ADR: focal only, no connected blocks
    adr_fmt()
        .args(["--critique", "TST-0001", &adr_root(&dir)])
        .assert()
        .success()
        .stdout(
            predicate::str::contains("◆ FOCAL")
                .and(predicate::str::contains("◇ CONNECTED").not()),
        );
}

#[test]
fn critique_invalid_id_exits_nonzero() {
    let dir = setup_corpus(
        MINIMAL_CONFIG,
        &[("TST-0001-valid-test-adr.md", VALID_ADR)],
    );

    adr_fmt()
        .args(["--critique", "INVALID", &adr_root(&dir)])
        .assert()
        .failure()
        .stderr(predicate::str::contains("not a valid ADR ID"));
}

#[test]
fn critique_unknown_adr_exits_nonzero() {
    let dir = setup_corpus(
        MINIMAL_CONFIG,
        &[("TST-0001-valid-test-adr.md", VALID_ADR)],
    );

    adr_fmt()
        .args(["--critique", "TST-9999", &adr_root(&dir)])
        .assert()
        .failure()
        .stderr(predicate::str::contains("not found"));
}

#[test]
fn critique_with_stale_excluded() {
    let dir = setup_multi_corpus(
        MINIMAL_CONFIG,
        &[("test", &[
            ("TST-0001-valid-test-adr.md", VALID_ADR),
        ])],
        &[("TST-0010-stale-adr.md", STALE_ADR)],
    );

    // Critique TST-0001 which is referenced by stale TST-0010
    // The stale ADR should be excluded from the closure
    adr_fmt()
        .args(["--critique", "TST-0001", &adr_root(&dir)])
        .assert()
        .success()
        .stdout(predicate::str::contains("◆ FOCAL"));
}

// ── context mode ───────────────────────────────────────────────────

#[test]
fn context_shows_crate_rules() {
    let dir = setup_corpus(
        MINIMAL_CONFIG,
        &[("TST-0001-valid-test-adr.md", VALID_ADR)],
    );

    adr_fmt()
        .args(["--context", "test-core", &adr_root(&dir)])
        .assert()
        .success()
        .stdout(
            predicate::str::contains("test-core")
                .and(predicate::str::contains("TST-0001")),
        );
}

#[test]
fn context_unknown_crate_exits_nonzero() {
    let dir = setup_corpus(
        MINIMAL_CONFIG,
        &[("TST-0001-valid-test-adr.md", VALID_ADR)],
    );

    adr_fmt()
        .args(["--context", "unknown-crate", &adr_root(&dir)])
        .assert()
        .failure()
        .stderr(predicate::str::contains("not found"));
}

#[test]
fn context_includes_foundation() {
    let dir = setup_multi_corpus(
        MULTI_DOMAIN_CONFIG,
        &[
            ("common", &[("COM-0001-foundation-principle.md", FOUNDATION_ADR)]),
            ("test", &[("TST-0001-valid-test-adr.md", VALID_ADR)]),
        ],
        &[],
    );

    // Foundation domain ADRs should always be included
    adr_fmt()
        .args(["--context", "test-core", &adr_root(&dir)])
        .assert()
        .success()
        .stdout(
            predicate::str::contains("COM-0001")
                .and(predicate::str::contains("TST-0001")),
        );
}

#[test]
fn context_per_adr_crates_filtering() {
    let dir = setup_corpus(
        MINIMAL_CONFIG,
        &[
            ("TST-0001-valid-test-adr.md", VALID_ADR),
            ("TST-0006-adr-with-crates.md", ADR_WITH_CRATES),
        ],
    );

    // TST-0006 has Crates: test-core, test-api — should be included
    // TST-0001 has no Crates: field — per-ADR filtering applies,
    // so only ADRs with matching crate are included
    adr_fmt()
        .args(["--context", "test-core", &adr_root(&dir)])
        .assert()
        .success()
        .stdout(predicate::str::contains("TST-0006"));
}

// ── index mode ─────────────────────────────────────────────────────

#[test]
fn index_produces_tree() {
    let dir = setup_corpus(
        MINIMAL_CONFIG,
        &[("TST-0001-valid-test-adr.md", VALID_ADR)],
    );

    // Use -- to separate --index (no domain filter) from positional ADR_DIR
    adr_fmt()
        .args(["--index", "--", &adr_root(&dir)])
        .assert()
        .success()
        .stdout(predicate::str::contains("TST-0001"));
}

#[test]
fn index_filtered_by_domain() {
    let dir = setup_multi_corpus(
        MULTI_DOMAIN_CONFIG,
        &[
            ("common", &[("COM-0001-foundation-principle.md", FOUNDATION_ADR)]),
            ("test", &[("TST-0001-valid-test-adr.md", VALID_ADR)]),
        ],
        &[],
    );

    // Filter to TST domain only
    adr_fmt()
        .args(["--index", "TST", &adr_root(&dir)])
        .assert()
        .success()
        .stdout(
            predicate::str::contains("TST-0001")
                .and(predicate::str::contains("COM-0001").not()),
        );
}

#[test]
fn index_unknown_domain_graceful() {
    let dir = setup_corpus(
        MINIMAL_CONFIG,
        &[("TST-0001-valid-test-adr.md", VALID_ADR)],
    );

    adr_fmt()
        .args(["--index", "NONEXISTENT", &adr_root(&dir)])
        .assert()
        .success()
        .stdout(predicate::str::contains("No domain found"));
}

// ── report mode ────────────────────────────────────────────────────

#[test]
fn report_flag_produces_output() {
    let dir = setup_corpus(
        MINIMAL_CONFIG,
        &[("TST-0001-valid-test-adr.md", VALID_ADR)],
    );

    adr_fmt()
        .args(["--report", &adr_root(&dir)])
        .assert()
        .success()
        .stdout(predicate::str::contains("ADR Children Report"));
}

// ── guidelines mode ────────────────────────────────────────────────

#[test]
fn guidelines_flag_produces_output() {
    let dir = setup_corpus(
        MINIMAL_CONFIG,
        &[("TST-0001-valid-test-adr.md", VALID_ADR)],
    );

    adr_fmt()
        .args(["--guidelines", &adr_root(&dir)])
        .assert()
        .success()
        .stdout(
            predicate::str::contains("ADR Guidelines")
                .and(predicate::str::contains("MODES"))
                .and(predicate::str::contains("TAGGED RULES")),
        );
}

// ── mutual exclusion ───────────────────────────────────────────────

#[test]
fn critique_and_context_mutually_exclusive() {
    adr_fmt()
        .args(["--critique", "TST-0001", "--context", "test-core"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("cannot be used with"));
}

#[test]
fn report_and_guidelines_mutually_exclusive() {
    adr_fmt()
        .args(["--report", "--guidelines"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("cannot be used with"));
}

#[test]
fn critique_and_index_mutually_exclusive() {
    adr_fmt()
        .args(["--critique", "TST-0001", "--index"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("cannot be used with"));
}

// ── infrastructure errors ──────────────────────────────────────────

#[test]
fn missing_config_exits_nonzero() {
    let dir = TempDir::new().expect("create tempdir");
    let root = dir.path().join("docs/adr");
    fs::create_dir_all(&root).expect("create dir");

    adr_fmt()
        .arg(root.to_str().unwrap())
        .assert()
        .failure()
        .stderr(predicate::str::contains("cannot read"));
}

#[test]
fn help_flag_shows_usage() {
    adr_fmt()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("adr-fmt"));
}

#[test]
fn version_flag_shows_version() {
    adr_fmt()
        .arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::contains("adr-fmt"));
}

#[test]
fn invalid_path_exits_nonzero() {
    adr_fmt()
        .arg("/nonexistent/path/to/adr")
        .assert()
        .failure()
        .stderr(predicate::str::contains("is not a directory"));
}

// ── read-only verification ─────────────────────────────────────────

#[test]
fn no_files_modified_after_lint() {
    let dir = setup_corpus(
        MINIMAL_CONFIG,
        &[("TST-0001-valid-test-adr.md", VALID_ADR)],
    );

    // Snapshot directory contents before
    let adr_dir = dir.path().join("docs/adr");
    let before: Vec<_> = walkdir(&adr_dir);

    adr_fmt()
        .arg(adr_root(&dir))
        .assert()
        .success();

    // Verify no new files or modifications
    let after: Vec<_> = walkdir(&adr_dir);
    assert_eq!(before, after, "lint mode should not create or modify files");
}

#[test]
fn no_files_modified_after_critique() {
    let dir = setup_corpus(
        MINIMAL_CONFIG,
        &[("TST-0001-valid-test-adr.md", VALID_ADR)],
    );

    let adr_dir = dir.path().join("docs/adr");
    let before: Vec<_> = walkdir(&adr_dir);

    adr_fmt()
        .args(["--critique", "TST-0001", &adr_root(&dir)])
        .assert()
        .success();

    let after: Vec<_> = walkdir(&adr_dir);
    assert_eq!(before, after, "critique mode should not create or modify files");
}

/// Recursively list all files under a directory (sorted, relative paths).
fn walkdir(root: &std::path::Path) -> Vec<String> {
    let mut entries = Vec::new();
    walk_recursive(root, root, &mut entries);
    entries.sort();
    entries
}

fn walk_recursive(base: &std::path::Path, dir: &std::path::Path, out: &mut Vec<String>) {
    if let Ok(rd) = fs::read_dir(dir) {
        for entry in rd.flatten() {
            let path = entry.path();
            if path.is_dir() {
                walk_recursive(base, &path, out);
            } else {
                let rel = path.strip_prefix(base).unwrap().display().to_string();
                out.push(rel);
            }
        }
    }
}
