//! Rule modules — each validates one aspect of ADR compliance.

mod index;
mod links;
mod naming;
mod template;

use crate::config::Config;
use crate::model::{AdrRecord, DomainDir};
use crate::report::Diagnostic;

/// Run all rule modules and collect diagnostics.
pub fn run_all(records: &[AdrRecord], domain_dirs: &[DomainDir], config: &Config) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();

    let domain_prefixes: Vec<&str> = config.domains.iter().map(|d| d.prefix.as_str()).collect();

    // Per-file rules
    for record in records {
        template::check(record, config, &mut diagnostics);
        naming::check(record, &domain_prefixes, &mut diagnostics);
    }

    // Cross-file rules
    links::check(records, &domain_prefixes, &mut diagnostics);

    // Index consistency (per domain)
    for dir in domain_dirs {
        let domain_records: Vec<&AdrRecord> =
            records.iter().filter(|r| r.id.prefix == dir.prefix).collect();
        index::check(dir, &domain_records, &mut diagnostics);
    }

    // Sort by file, then line
    diagnostics.sort_by(|a, b| a.file.cmp(&b.file).then(a.line.cmp(&b.line)));
    diagnostics
}
