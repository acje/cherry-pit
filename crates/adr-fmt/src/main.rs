//! ADR template and link-integrity validator for cherry-pit.
//!
//! Validates Architecture Decision Records against the governance
//! rules defined in `docs/adr/GOVERNANCE.md`:
//!
//! - Template compliance (required fields, sections, tier, status format,
//!   code block length)
//! - Forward link integrity (7-verb relationship vocabulary, no reverse verbs)
//! - File naming conventions (`{PREFIX}-{NNNN}-kebab-slug.md`)
//! - README index ↔ filesystem consistency
//!
//! # Usage
//!
//! ```text
//! cargo run -p adr-fmt [-- [--report] [path/to/adr]]
//! ```
//!
//! Exits 0 if all checks pass. Exits 1 if any errors are found.
//! Cross-domain references to unmigrated PAR/GEN ADRs emit warnings,
//! not errors.
//!
//! Use `--report` to print a computed children index (reverse-link
//! navigation without stored backlinks).

#![forbid(unsafe_code)]

mod model;
mod nav;
mod parser;
mod report;
mod rules;

use std::path::{Path, PathBuf};
use std::process;

use model::DomainDir;
use report::Severity;

fn main() {
    let (adr_root, report_mode) = resolve_args();
    let domain_dirs = discover_domains(&adr_root);

    if domain_dirs.is_empty() {
        eprintln!("error: no domain directories found in {}", adr_root.display());
        process::exit(1);
    }

    let mut all_records = Vec::new();
    for dir in &domain_dirs {
        let records = parser::parse_domain(dir);
        all_records.extend(records);
    }

    // Report mode: compute and print children index
    if report_mode {
        let children = nav::compute_children(&all_records);
        nav::print_report(&all_records, &children);
    }

    let diagnostics = rules::run_all(&all_records, &domain_dirs);

    let mut errors = 0u32;
    let mut warnings = 0u32;

    for d in &diagnostics {
        match d.severity {
            Severity::Error => errors += 1,
            Severity::Warning => warnings += 1,
        }
        report::print_diagnostic(d);
    }

    if errors > 0 || warnings > 0 {
        eprintln!();
    }
    eprintln!(
        "adr-fmt: {errors} error(s), {warnings} warning(s) across {} ADR(s)",
        all_records.len()
    );

    if errors > 0 {
        process::exit(1);
    }
}

/// Parse CLI arguments. Returns (adr_root, report_mode).
///
/// Extracts `--report` flag and optional positional path argument.
/// No dependency on clap — manual flag extraction.
fn resolve_args() -> (PathBuf, bool) {
    let args: Vec<String> = std::env::args().collect();

    let mut report_mode = false;
    let mut positional: Option<String> = None;

    for arg in &args[1..] {
        match arg.as_str() {
            "--help" | "-h" => {
                print_help();
                process::exit(0);
            }
            "--report" => {
                report_mode = true;
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

    (adr_root, report_mode)
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

fn discover_domains(root: &Path) -> Vec<DomainDir> {
    // Hardcoded domain list matching GOVERNANCE.md §2 taxonomy.
    // Adding a new domain requires updating this list.
    let known = [
        ("common", "COM"),
        ("cherry", "CHE"),
        ("pardosa", "PAR"),
        ("genome", "GEN"),
    ];

    let mut dirs = Vec::new();
    for (dir_name, prefix) in &known {
        let path = root.join(dir_name);
        if path.is_dir() {
            dirs.push(DomainDir {
                path,
                prefix: (*prefix).to_owned(),
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
    eprintln!("    -h, --help          Print this help message");
    eprintln!();
    eprintln!("ARGS:");
    eprintln!("    <adr-directory>    Path to ADR root (default: auto-discover docs/adr/)");
    eprintln!();
    eprintln!("EXIT CODES:");
    eprintln!("    0    All checks passed (warnings may be present)");
    eprintln!("    1    One or more errors found");
    eprintln!();
    eprintln!("RULES:");
    eprintln!("    T001-T011   Template compliance (fields, sections, status, code blocks)");
    eprintln!("    L001,L003-5 Link integrity (dangling, symmetric, reverse verb, unmigrated)");
    eprintln!("    N001-N003   File naming conventions");
    eprintln!("    I001-I002   README index consistency");
}
