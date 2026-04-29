//! ADR template and link-integrity validator for cherry-pit.
//!
//! Read-only analysis tool. Single source of truth for all invariant
//! ADR governance rules.
//!
//! # Modes
//!
//! ```text
//! adr-fmt [<ADR_DIR>]                     # default: print governance guidelines
//! adr-fmt --lint [<ADR_DIR>]              # lint all ADRs
//! adr-fmt --critique <ADR_ID> [<ADR_DIR>] # focal ADR + direct neighbors
//! adr-fmt --context <CRATE> [<ADR_DIR>]   # decision rules for a crate
//! adr-fmt --tree [DOMAIN] [<ADR_DIR>]     # domain tree overview
//! ```
//!
//! Exit codes:
//!   0 — Analysis complete (warnings only, or clean)
//!   1 — Infrastructure error or lint errors detected

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
    /// Lint all ADRs, report diagnostics to stdout
    #[arg(long, group = "mode")]
    lint: bool,

    /// Critique a focal ADR: show direct neighbors with full content
    #[arg(long, value_name = "ADR_ID", group = "mode")]
    critique: Option<String>,

    /// Depth of transitive closure for critique (default: 1)
    #[arg(long, value_name = "N", default_value = "1", requires = "critique")]
    depth: usize,

    /// Show decision rules applicable to a crate
    #[arg(long, value_name = "CRATE", group = "mode")]
    context: Option<String>,

    /// Print domain tree (optionally filtered by domain prefix)
    #[arg(long, value_name = "DOMAIN", num_args = 0..=1, default_missing_value = "", group = "mode")]
    tree: Option<String>,

    /// Path to ADR root directory (default: auto-discover docs/adr/)
    #[arg(value_name = "ADR_DIR")]
    adr_directory: Option<PathBuf>,
}

#[allow(clippy::too_many_lines)]
fn main() {
    let cli = Cli::parse();

    // Resolve ADR root directory (may not exist for guidelines setup mode)
    let adr_root = match cli.adr_directory {
        Some(ref p) => {
            if p.is_dir() {
                Some(p.clone())
            } else {
                eprintln!("error: {} is not a directory", p.display());
                process::exit(1);
            }
        }
        None => resolve_adr_root_optional(),
    };

    // Default mode: guidelines
    // If no mode flag is specified and no config exists, show setup guide
    let is_non_default_mode =
        cli.lint || cli.critique.is_some() || cli.context.is_some() || cli.tree.is_some();

    if !is_non_default_mode {
        // Guidelines mode — handles both setup and governance display
        if let Some(root) = &adr_root {
            match config::try_load(root) {
                Ok(Some(config)) => {
                    guidelines::print_governance(&config);
                    return;
                }
                Ok(None) => {
                    guidelines::print_setup_guide();
                    return;
                }
                Err(e) => {
                    eprintln!("error: {e}");
                    process::exit(1);
                }
            }
        } else {
            guidelines::print_setup_guide();
            return;
        }
    }

    // Non-default modes require a valid root and config
    let Some(adr_root) = adr_root else {
        eprintln!("error: could not find docs/adr/ in any parent directory");
        eprintln!("       run from the workspace root or pass an explicit path");
        process::exit(1);
    };

    let config = match config::load(&adr_root) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("error: {e}");
            process::exit(1);
        }
    };

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
        let blocks = match critique::critique(&focal_id, &all_records, &config, cli.depth) {
            Ok(b) => b,
            Err(e) => {
                eprintln!("error: {e}");
                process::exit(1);
            }
        };
        print!("{}", output::render_blocks(&blocks));
    } else if let Some(ref crate_name) = cli.context {
        // --context mode
        let groups = match context::context_grouped(crate_name, &all_records, &config) {
            Ok(g) => g,
            Err(e) => {
                eprintln!("error: {e}");
                process::exit(1);
            }
        };
        print!("{}", output::render_root_groups(crate_name, &groups));
    } else if let Some(ref domain_filter) = cli.tree {
        // --tree mode
        let filter = if domain_filter.is_empty() {
            None
        } else {
            Some(domain_filter.as_str())
        };
        print!(
            "{}",
            output::render_tree(&all_records, &domain_dirs, &config, filter)
        );
    } else if cli.lint {
        // --lint mode
        let diagnostics = rules::run_all(&all_records, &config);
        print!(
            "{}",
            output::render_diagnostics(&diagnostics, all_records.len())
        );
        // Exit with error if any error-severity diagnostics exist
        if diagnostics
            .iter()
            .any(|d| d.severity == report::Severity::Error)
        {
            process::exit(1);
        }
    }
}

/// Walk up from CWD looking for `docs/adr/`. Returns None if not found.
fn resolve_adr_root_optional() -> Option<PathBuf> {
    let Ok(cwd) = std::env::current_dir() else {
        return None;
    };
    let mut dir = cwd.as_path();
    loop {
        let candidate = dir.join("docs/adr/GOVERNANCE.md");
        if candidate.is_file() {
            return Some(dir.join("docs/adr"));
        }
        match dir.parent() {
            Some(parent) => dir = parent,
            None => break,
        }
    }
    None
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
            });
        }
    }
    dirs
}
