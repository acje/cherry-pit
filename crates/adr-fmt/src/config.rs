//! Configuration loading from `adr-fmt.toml`.
//!
//! The config file is the single machine-readable source for domain
//! definitions, crate mappings, stale directory path, and validation
//! rules.  See GOVERNANCE.md §13.

use std::path::Path;

use serde::Deserialize;

/// Top-level configuration.
#[derive(Debug, Deserialize)]
pub struct Config {
    pub stale: StaleConfig,
    pub domains: Vec<DomainConfig>,
    #[allow(dead_code)]
    pub rules: Vec<RuleConfig>,
}

/// Stale archive configuration.
#[derive(Debug, Deserialize)]
pub struct StaleConfig {
    pub directory: String,
}

/// Domain definition.
#[derive(Debug, Deserialize)]
pub struct DomainConfig {
    pub prefix: String,
    pub name: String,
    pub directory: String,
    pub description: String,
    #[allow(dead_code)]
    pub crates: Vec<String>,
}

/// Rule definition (informational — all rules are warnings).
#[derive(Debug, Deserialize)]
pub struct RuleConfig {
    #[allow(dead_code)]
    pub id: String,
    #[allow(dead_code)]
    pub category: String,
    #[allow(dead_code)]
    pub description: String,
}

/// Load configuration from `adr-fmt.toml` in the ADR root directory.
///
/// Fails with a clear error message if the file is missing or malformed.
pub fn load(adr_root: &Path) -> Config {
    let config_path = adr_root.join("adr-fmt.toml");

    let content = std::fs::read_to_string(&config_path).unwrap_or_else(|e| {
        eprintln!(
            "error: cannot read {}: {e}",
            config_path.display()
        );
        eprintln!(
            "       adr-fmt.toml is required — see GOVERNANCE.md §13"
        );
        std::process::exit(1);
    });

    toml::from_str(&content).unwrap_or_else(|e| {
        eprintln!(
            "error: failed to parse {}: {e}",
            config_path.display()
        );
        std::process::exit(1);
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_minimal_config() {
        let toml_str = r#"
[stale]
directory = "stale"

[[domains]]
prefix = "CHE"
name = "Cherry"
directory = "cherry"
description = "Test domain"
crates = ["cherry-pit-core"]

[[rules]]
id = "T001"
category = "template"
description = "H1 title present"
"#;
        let config: Config = toml::from_str(toml_str).unwrap();
        assert_eq!(config.stale.directory, "stale");
        assert_eq!(config.domains.len(), 1);
        assert_eq!(config.domains[0].prefix, "CHE");
        assert_eq!(config.domains[0].crates, vec!["cherry-pit-core"]);
        assert_eq!(config.rules.len(), 1);
        assert_eq!(config.rules[0].id, "T001");
    }

    #[test]
    fn parse_multi_domain_config() {
        let toml_str = r#"
[stale]
directory = "stale"

[[domains]]
prefix = "COM"
name = "Common"
directory = "common"
description = "Cross-cutting"
crates = []

[[domains]]
prefix = "CHE"
name = "Cherry"
directory = "cherry"
description = "Architecture"
crates = ["cherry-pit-core", "cherry-pit-gateway"]

[[rules]]
id = "T001"
category = "template"
description = "test"
"#;
        let config: Config = toml::from_str(toml_str).unwrap();
        assert_eq!(config.domains.len(), 2);
        assert_eq!(config.domains[0].prefix, "COM");
        assert!(config.domains[0].crates.is_empty());
        assert_eq!(config.domains[1].crates.len(), 2);
    }

    #[test]
    fn missing_required_field_fails() {
        let toml_str = r#"
[stale]
directory = "stale"

[[domains]]
prefix = "CHE"
name = "Cherry"
# missing directory and description
"#;
        let result: Result<Config, _> = toml::from_str(toml_str);
        assert!(result.is_err());
    }
}
