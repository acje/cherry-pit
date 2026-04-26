//! ADR template and link-integrity validator for cherry-pit.
//!
//! Single source of truth for all invariant ADR governance rules.
//! Validates Architecture Decision Records against the rule catalog:
//!
//! - Template compliance (required fields, sections, tier, status format,
//!   code block length, section ordering, minimum word count)
//! - Relationship integrity (3-verb vocabulary: References, Supersedes,
//!   Root; legacy verb detection; supersedes-status consistency)
//! - File naming conventions (`{PREFIX}-{NNNN}-kebab-slug.md`)
//! - Structure rules (stale location/status, Retirement section)
//!
//! # Usage
//!
//! ```text
//! adr-fmt [--report | --guidelines] [<ADR_DIR>]
//! ```
//!
//! Exit codes:
//!   0 — Lint complete (warnings may be present)
//!   1 — Infrastructure error (missing config, unreadable directory)
//!
//! Use `--report` to print a computed children index (reverse-link
//! navigation without stored backlinks).
//!
//! Use `--guidelines` to print the complete ADR guidelines document
//! generated from the rule catalog and configuration.

#![forbid(unsafe_code)]

mod config;
mod generate;
mod guidelines;
mod model;
mod nav;
mod parser;
mod report;
mod rules;

use std::path::{Path, PathBuf};
use std::process;

use clap::Parser;

use config::Config;
use model::DomainDir;
use report::Severity;

/// ADR template and link-integrity validator for cherry-pit.
#[derive(Parser)]
#[command(name = "adr-fmt", version)]
struct Cli {
    /// Print computed children report (reverse-link index)
    #[arg(long, conflicts_with = "guidelines")]
    report: bool,

    /// Print complete ADR guidelines (plain text to stdout)
    #[arg(long, conflicts_with = "report")]
    guidelines: bool,

    /// Path to ADR root directory (default: auto-discover docs/adr/)
    #[arg(value_name = "ADR_DIR")]
    adr_directory: Option<PathBuf>,
}

fn main() {
    let cli = Cli::parse();

    let adr_root = match cli.adr_directory {
        Some(ref p) => {
            if p.is_dir() {
                p.clone()
            } else {
                eprintln!("error: {} is not a directory", p.display());
                process::exit(1);
            }
        }
        None => resolve_adr_root_auto(),
    };

    let config = match config::load(&adr_root) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("error: {e}");
            process::exit(1);
        }
    };

    // --guidelines: print guidelines to stdout and exit
    if cli.guidelines {
        guidelines::print(&config);
        return;
    }

    let domain_dirs = discover_domains(&adr_root, &config);

    if domain_dirs.is_empty() {
        eprintln!("error: no domain directories found in {}", adr_root.display());
        process::exit(1);
    }

    let mut all_records = Vec::new();
    for dir in &domain_dirs {
        let records = parser::parse_domain(dir);
        all_records.extend(records);
    }

    // Parse stale directory
    let stale_dir = adr_root.join(&config.stale.directory);
    if stale_dir.is_dir() {
        let stale_records = parser::parse_stale(&stale_dir, &config);
        all_records.extend(stale_records);
    }

    // Report mode: compute and print children index
    if cli.report {
        let children = nav::compute_children(&all_records);
        nav::print_report(&all_records, &children);
    }

    // Generate README files
    generate::generate_all(&all_records, &domain_dirs, &adr_root, &config);

    let diagnostics = rules::run_all(&all_records, &domain_dirs, &config);

    let mut errors = 0u32;
    let mut warnings = 0u32;
    let mut internal_count = 0u32;

    for d in &diagnostics {
        if d.internal {
            internal_count += 1;
            continue;
        }
        match d.severity {
            Severity::Error => errors += 1,
            Severity::Warning => warnings += 1,
        }
        report::print_diagnostic(d);
    }

    if errors > 0 || warnings > 0 {
        eprintln!();
    }
    let mut summary = format!(
        "adr-fmt: {errors} error(s), {warnings} warning(s) across {} ADR(s)",
        all_records.len()
    );
    if internal_count > 0 {
        summary.push_str(&format!(" [{internal_count} internal assertion(s)]"));
    }
    eprintln!("{summary}");

    // Advisory tool: always exit 0 for lint findings.
    // Only infrastructure errors (missing config, no domains) exit 1.
}

/// Walk up from CWD looking for `docs/adr/GOVERNANCE.md`.
fn resolve_adr_root_auto() -> PathBuf {
    if let Ok(cwd) = std::env::current_dir() {
        let mut dir = cwd.as_path();
        loop {
            let candidate = dir.join("docs/adr/GOVERNANCE.md");
            if candidate.is_file() {
                return dir.join("docs/adr");
            }
            match dir.parent() {
                Some(parent) => dir = parent,
                None => break,
            }
        }
    }

    eprintln!("error: could not find docs/adr/GOVERNANCE.md in any parent directory");
    eprintln!("       run from the workspace root or pass an explicit path");
    process::exit(1);
}

/// Build domain directories from config.
fn discover_domains(root: &Path, config: &Config) -> Vec<DomainDir> {
    let mut dirs = Vec::new();
    for domain in &config.domains {
        let path = root.join(&domain.directory);
        if path.is_dir() {
            dirs.push(DomainDir {
                path,
                prefix: domain.prefix.clone(),
                name: domain.name.clone(),
                description: domain.description.clone(),
            });
        }
    }
    dirs
}
