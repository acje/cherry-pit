//! ADR template and link-integrity validator for cherry-pit.
//!
//! Read-only analysis tool. Single source of truth for all invariant
//! ADR governance rules.
//!
//! # Modes
//!
//! ```text
//! adr-forge [<ADR_DIR>]                     # default: print governance guidelines
//! adr-forge --lint [<ADR_DIR>]              # lint all ADRs
//! adr-forge --critique <ADR_ID> [<ADR_DIR>] # focal ADR + direct neighbors
//! adr-forge --context <CRATE> [<ADR_DIR>]   # decision rules for a crate
//! adr-forge --tree [DOMAIN] [<ADR_DIR>]     # domain tree overview
//! ```
//!
//! Exit codes:
//!   0 — Analysis complete (warnings only, or clean)
//!   1 — Infrastructure error or lint errors detected

#![forbid(unsafe_code)]

mod config;
mod containment;
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
#[command(name = "adr-forge", version)]
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

    // Resolve ADR root directory (may not exist for guidelines setup mode).
    // The user-supplied path is canonicalized so subsequent containment
    // checks operate against a stable, symlink-resolved root.
    let adr_root = match cli.adr_directory {
        Some(ref p) => {
            if !p.is_dir() {
                eprintln!("error: {} is not a directory", p.display());
                process::exit(1);
            }
            match std::fs::canonicalize(p) {
                Ok(canon) => Some(canon),
                Err(e) => {
                    eprintln!("error: cannot canonicalize {}: {e}", p.display());
                    process::exit(1);
                }
            }
        }
        None => match resolve_adr_root_optional() {
            Ok(opt) => opt,
            Err(e) => {
                eprintln!("error: {e}");
                process::exit(1);
            }
        },
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

    let domain_dirs = match discover_domains(&adr_root, &config) {
        Ok(dirs) => dirs,
        Err(e) => {
            eprintln!("error: {e}");
            process::exit(1);
        }
    };

    if domain_dirs.is_empty() {
        eprintln!(
            "error: no domain directories found in {}",
            adr_root.display()
        );
        process::exit(1);
    }

    let mut all_records = Vec::new();
    let mut parse_diagnostics = Vec::new();
    for dir in &domain_dirs {
        match parser::parse_domain(dir) {
            Ok(outcome) => {
                all_records.extend(outcome.records);
                parse_diagnostics.extend(outcome.diagnostics);
            }
            Err(e) => {
                eprintln!("error: {e}");
                process::exit(1);
            }
        }
    }

    // Parse stale directory (optional — may not exist in fresh repos)
    let stale_dir = match containment::contained_join_optional(&adr_root, &config.stale.directory)
    {
        Ok(opt) => opt,
        Err(e) => {
            eprintln!("error: stale directory in adr-forge.toml: {e}");
            process::exit(1);
        }
    };
    if let Some(stale_dir) = stale_dir
        && stale_dir.is_dir()
    {
        match parser::parse_stale(&stale_dir, &config) {
            Ok(outcome) => {
                all_records.extend(outcome.records);
                parse_diagnostics.extend(outcome.diagnostics);
            }
            Err(e) => {
                eprintln!("error: {e}");
                process::exit(1);
            }
        }
    }

    // Mode dispatch
    if let Some(ref adr_id_str) = cli.critique {
        // --critique mode
        let Some(focal_id) = parse_adr_id_from_str(adr_id_str) else {
            eprintln!(
                "error: {} is not a valid ADR ID (expected PREFIX-NNNN)",
                adr_id_str.escape_debug()
            );
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
        // --lint mode: advisory-only per AFM-0003 R1/R2. All rule findings
        // are warnings; exit 0 always when lint completes. Exit 1 is reserved
        // for infrastructure errors (missing config, unreadable files,
        // invalid configuration) handled earlier in this function via
        // eprintln! + process::exit(1).
        //
        // Parser-stage diagnostics (P### per AFM-0017) are merged with
        // rule-stage diagnostics so the user sees one unified list.
        let mut diagnostics = parse_diagnostics;
        diagnostics.extend(rules::run_all(&all_records, &config));
        print!(
            "{}",
            output::render_diagnostics(&diagnostics, all_records.len())
        );
    }
}

/// Walk up from CWD looking for `docs/adr/GOVERNANCE.md`.
///
/// Returns `Ok(Some(canonical_path))` when found and canonicalized,
/// `Ok(None)` when the marker file is not found in any ancestor,
/// or `Err(message)` when a candidate root was found but cannot be
/// canonicalized (permission denied, broken symlink, etc.). Errors
/// surface as infrastructure failures per AFM-0003 R1.
fn resolve_adr_root_optional() -> Result<Option<PathBuf>, String> {
    let Ok(cwd) = std::env::current_dir() else {
        return Ok(None);
    };
    let mut dir = cwd.as_path();
    loop {
        let candidate = dir.join("docs/adr/GOVERNANCE.md");
        if candidate.is_file() {
            let target = dir.join("docs/adr");
            return std::fs::canonicalize(&target).map(Some).map_err(|e| {
                format!("cannot canonicalize discovered ADR root {}: {e}", target.display())
            });
        }
        match dir.parent() {
            Some(parent) => dir = parent,
            None => break,
        }
    }
    Ok(None)
}

/// Build domain directories from config, applying strict containment.
///
/// Each `domain.directory` from `adr-forge.toml` is joined to `root`
/// via [`containment::contained_join_optional`]: absolute paths and
/// `..` components are rejected, and the canonical target must be a
/// descendant of the canonical ADR root. Containment failures abort
/// the run as infrastructure errors per AFM-0003 R1.
///
/// A configured directory that does not exist on disk is silently
/// skipped (returns `None` from the optional join); the caller emits
/// a diagnostic when zero domains resolve.
fn discover_domains(root: &Path, config: &Config) -> Result<Vec<DomainDir>, String> {
    let mut dirs = Vec::new();
    for domain in &config.domains {
        let resolved = containment::contained_join_optional(root, &domain.directory)
            .map_err(|e| format!("domain '{}' directory: {e}", domain.prefix))?;
        if let Some(path) = resolved
            && path.is_dir()
        {
            dirs.push(DomainDir {
                path,
                prefix: domain.prefix.clone(),
                name: domain.name.clone(),
            });
        }
    }
    Ok(dirs)
}
