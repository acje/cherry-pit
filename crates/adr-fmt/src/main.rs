//! ADR template and link-integrity validator for cherry-pit.
//!
//! Read-only, agent-first analysis tool. Single source of truth for
//! all invariant ADR governance rules.
//!
//! # Modes
//!
//! ```text
//! adr-fmt [<ADR_DIR>]                     # default: lint all ADRs
//! adr-fmt --critique <ADR_ID> [<ADR_DIR>] # focal ADR + transitive closure
//! adr-fmt --context <CRATE> [<ADR_DIR>]   # decision rules for a crate
//! adr-fmt --index [DOMAIN] [<ADR_DIR>]    # domain tree overview
//! adr-fmt --report [<ADR_DIR>]            # computed children report
//! adr-fmt --guidelines [<ADR_DIR>]        # print ADR guidelines
//! ```
//!
//! Exit codes:
//!   0 — Analysis complete
//!   1 — Infrastructure error (missing config, unknown ADR, unknown crate)

#![forbid(unsafe_code)]

mod config;
mod context;
mod critique;
mod guidelines;
mod model;
mod nav;
mod output;
mod parser;
mod report;
mod rules;

use std::path::{Path, PathBuf};
use std::process;

use clap::Parser;

use config::Config;
use model::{DomainDir, parse_adr_id_from_str};

/// ADR template and link-integrity validator for cherry-pit.
#[derive(Parser)]
#[command(name = "adr-fmt", version)]
struct Cli {
    /// Critique a focal ADR: show transitive closure with full content
    #[arg(long, value_name = "ADR_ID", group = "mode")]
    critique: Option<String>,

    /// Show decision rules applicable to a crate
    #[arg(long, value_name = "CRATE", group = "mode")]
    context: Option<String>,

    /// Print domain index tree (optionally filtered by domain prefix)
    #[arg(long, value_name = "DOMAIN", num_args = 0..=1, default_missing_value = "", group = "mode")]
    index: Option<String>,

    /// Print computed children report (reverse-link index)
    #[arg(long, group = "mode")]
    report: bool,

    /// Print complete ADR guidelines (plain text to stdout)
    #[arg(long, group = "mode")]
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
        eprintln!(
            "error: no domain directories found in {}",
            adr_root.display()
        );
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

    // Mode dispatch
    if let Some(ref adr_id_str) = cli.critique {
        // --critique mode
        let Some(focal_id) = parse_adr_id_from_str(adr_id_str) else {
            eprintln!("error: '{adr_id_str}' is not a valid ADR ID (expected PREFIX-NNNN)");
            process::exit(1);
        };
        let blocks = critique::critique(&focal_id, &all_records, &config);
        print!("{}", output::render_blocks(&blocks));
    } else if let Some(ref crate_name) = cli.context {
        // --context mode
        let rules = context::context(crate_name, &all_records, &config);
        print!("{}", output::render_rules(crate_name, &rules));
    } else if let Some(ref domain_filter) = cli.index {
        // --index mode
        let filter = if domain_filter.is_empty() {
            None
        } else {
            Some(domain_filter.as_str())
        };
        print!(
            "{}",
            output::render_index(&all_records, &domain_dirs, &config, filter)
        );
    } else if cli.report {
        // --report mode
        let children = nav::compute_children(&all_records);
        print!("{}", output::render_report(&all_records, &children));
    } else {
        // Default lint mode
        let diagnostics = rules::run_all(&all_records, &config);
        print!(
            "{}",
            output::render_diagnostics(&diagnostics, all_records.len())
        );
    }
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
