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
crates = []

[[rules]]
id = "T001"
category = "template"
description = "H1 title present"

[[rules]]
id = "T015"
category = "template"
description = "Prose section below minimum word count"
params = { min_words = 10 }
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

We decided to create a minimal but complete ADR that satisfies all template rules.

## Consequences

The integration test can verify that a clean corpus produces zero diagnostics.
";

/// An ADR with a dangling link to trigger L001.
const DANGLING_LINK_ADR: &str = "\
# TST-0002. Dangling Link ADR

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

We reference a non-existent ADR to verify that dangling link detection works.

## Consequences

The linter should report a dangling link warning for TST-9999 in the output.
";

/// Create the minimal test corpus in a tempdir.
///
/// Returns the tempdir (kept alive for the test duration).
fn setup_corpus(config: &str, adrs: &[(&str, &str)]) -> TempDir {
    let dir = TempDir::new().expect("create tempdir");
    let adr_root = dir.path().join("docs/adr");

    // GOVERNANCE.md (required for auto-discovery, but we pass path explicitly)
    fs::create_dir_all(&adr_root).expect("create adr root");
    fs::write(adr_root.join("GOVERNANCE.md"), "# Governance\n").expect("write governance");

    // Config
    fs::write(adr_root.join("adr-fmt.toml"), config).expect("write config");

    // Domain directory
    let domain_dir = adr_root.join("test");
    fs::create_dir_all(&domain_dir).expect("create domain dir");

    // ADR files
    for (filename, content) in adrs {
        fs::write(domain_dir.join(filename), content).expect("write ADR");
    }

    dir
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

// ── tests ───────────────────────────────────────────────────────────

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
        .stderr(predicate::str::contains("0 error(s), 0 warning(s)"));
}

#[test]
fn dangling_link_produces_l001() {
    let dir = setup_corpus(
        MINIMAL_CONFIG,
        &[
            ("TST-0001-valid-test-adr.md", VALID_ADR),
            ("TST-0002-dangling-link-adr.md", DANGLING_LINK_ADR),
        ],
    );

    adr_fmt()
        .arg(adr_root(&dir))
        .assert()
        .success() // Advisory: always exit 0 for lint findings
        .stderr(predicate::str::contains("L001"));
}

#[test]
fn empty_domain_directory_graceful() {
    let dir = setup_corpus(MINIMAL_CONFIG, &[]);

    adr_fmt()
        .arg(adr_root(&dir))
        .assert()
        .success()
        .stderr(predicate::str::contains("0 ADR(s)"));
}

#[test]
fn missing_config_exits_nonzero() {
    let dir = TempDir::new().expect("create tempdir");
    let root = dir.path().join("docs/adr");
    fs::create_dir_all(&root).expect("create dir");
    // No adr-fmt.toml written — config::load should fail

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
fn report_and_guidelines_mutually_exclusive() {
    adr_fmt()
        .args(["--report", "--guidelines"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("cannot be used with"));
}

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
        .stdout(predicate::str::contains("ADR Guidelines"));
}

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

#[test]
fn invalid_path_exits_nonzero() {
    adr_fmt()
        .arg("/nonexistent/path/to/adr")
        .assert()
        .failure()
        .stderr(predicate::str::contains("is not a directory"));
}
