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
//! cargo run -p adr-fmt [-- [--report | --guidelines] [path/to/adr]]
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

use config::Config;
use model::DomainDir;
use report::Severity;

/// CLI mode of operation.
enum Mode {
    Lint,
    Report,
    Guidelines,
}

fn main() {
    let (adr_root, mode) = resolve_args();

    let config = match config::load(&adr_root) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("error: {e}");
            process::exit(1);
        }
    };

    // --guidelines: print guidelines to stdout and exit
    if matches!(mode, Mode::Guidelines) {
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
    if matches!(mode, Mode::Report) {
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

/// Parse CLI arguments. Returns (adr_root, mode).
///
/// Extracts `--report` / `--guidelines` flags and optional positional path.
/// `--report` and `--guidelines` are mutually exclusive.
/// No dependency on clap — manual flag extraction.
fn resolve_args() -> (PathBuf, Mode) {
    let args: Vec<String> = std::env::args().collect();

    let mut mode = Mode::Lint;
    let mut positional: Option<String> = None;

    for arg in &args[1..] {
        match arg.as_str() {
            "--help" | "-h" => {
                print_help();
                process::exit(0);
            }
            "--report" => {
                if matches!(mode, Mode::Guidelines) {
                    eprintln!("error: --report and --guidelines are mutually exclusive");
                    process::exit(1);
                }
                mode = Mode::Report;
            }
            "--guidelines" => {
                if matches!(mode, Mode::Report) {
                    eprintln!("error: --report and --guidelines are mutually exclusive");
                    process::exit(1);
                }
                mode = Mode::Guidelines;
            }
            _ => {
                if positional.is_some() {
                    eprintln!("error: unexpected argument: {arg}");
                    process::exit(1);
                }
                positional = Some(arg.clone());
            }
        }
    }

    let adr_root = if let Some(path_str) = positional {
        let p = PathBuf::from(&path_str);
        if p.is_dir() {
            p
        } else {
            eprintln!("error: {} is not a directory", p.display());
            process::exit(1);
        }
    } else {
        resolve_adr_root_auto()
    };

    (adr_root, mode)
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

fn print_help() {
    eprintln!("adr-fmt — ADR template and link-integrity validator for cherry-pit");
    eprintln!();
    eprintln!("USAGE:");
    eprintln!("    cargo run -p adr-fmt [-- [OPTIONS] [<adr-directory>]]");
    eprintln!();
    eprintln!("OPTIONS:");
    eprintln!("    --report            Print computed children report (reverse-link index)");
    eprintln!("    --guidelines        Print complete ADR guidelines (plain text to stdout)");
    eprintln!("    -h, --help          Print this help message");
    eprintln!();
    eprintln!("    --report and --guidelines are mutually exclusive.");
    eprintln!();
    eprintln!("ARGS:");
    eprintln!("    <adr-directory>    Path to ADR root (default: auto-discover docs/adr/)");
    eprintln!();
    eprintln!("EXIT CODES:");
    eprintln!("    0    Lint complete (warnings may be present)");
    eprintln!("    1    Infrastructure error (missing config, no domains)");
    eprintln!();
    eprintln!("RULES:");
    eprintln!("    T001-T015   Template compliance");
    eprintln!("    L001,L003   Link and relationship integrity");
    eprintln!("    L006-L009   Verb vocabulary and Root validation");
    eprintln!("    N001-N004   File naming conventions");
    eprintln!("    S004-S006   Structure and stale archive rules");
    eprintln!("    I001-I003   README index consistency (internal)");
}
