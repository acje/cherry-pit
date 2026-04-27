//! Configuration loading from `adr-fmt.toml`.
//!
//! The config file is the single machine-readable source for domain
//! definitions, crate mappings, stale directory path, and validation
//! rules with optional parameters.

use std::collections::HashMap;
use std::path::Path;

use serde::Deserialize;

/// Top-level configuration.
#[derive(Debug, Deserialize)]
pub struct Config {
    pub stale: StaleConfig,
    pub domains: Vec<DomainConfig>,
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
    pub crates: Vec<String>,
    /// Foundation domains are included with every domain query.
    /// COM is the canonical foundation domain.
    #[serde(default)]
    pub foundation: bool,
}

/// Rule definition with optional parameters.
#[derive(Debug, Deserialize)]
pub struct RuleConfig {
    pub id: String,
    pub category: String,
    pub description: String,
    /// Optional rule parameters (e.g., `min_words = 10`).
    #[serde(default)]
    pub params: HashMap<String, toml::Value>,
    /// Internal rules are self-checks, not user-facing governance.
    #[serde(default)]
    pub internal: bool,
}

impl Config {
    /// Look up a rule parameter by rule ID and key.
    ///
    /// Returns `None` if the rule or key does not exist.
    pub fn rule_param_u64(&self, rule_id: &str, key: &str) -> Option<u64> {
        self.rules
            .iter()
            .find(|r| r.id == rule_id)
            .and_then(|r| r.params.get(key))
            .and_then(|v| v.as_integer())
            .map(|v| v as u64)
    }
}

/// Load configuration from `adr-fmt.toml` in the ADR root directory.
///
/// Returns an error string if the file is missing or malformed.
/// Config errors are hard failures — the tool cannot lint without
/// valid configuration.
pub fn load(adr_root: &Path) -> Result<Config, String> {
    let config_path = adr_root.join("adr-fmt.toml");

    let content = std::fs::read_to_string(&config_path).map_err(|e| {
        format!(
            "cannot read {}: {e}\n       adr-fmt.toml is required",
            config_path.display()
        )
    })?;

    toml::from_str(&content).map_err(|e| {
        format!(
            "failed to parse {}: {e}",
            config_path.display()
        )
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
        assert!(config.rules[0].params.is_empty());
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
    fn parse_rule_with_params() {
        let toml_str = r#"
[stale]
directory = "stale"

[[domains]]
prefix = "CHE"
name = "Cherry"
directory = "cherry"
description = "Test"
crates = []

[[rules]]
id = "T015"
category = "template"
description = "Section minimum word count"
params = { min_words = 10 }
"#;
        let config: Config = toml::from_str(toml_str).unwrap();
        assert_eq!(config.rules[0].id, "T015");
        let min_words = config.rule_param_u64("T015", "min_words");
        assert_eq!(min_words, Some(10));
    }

    #[test]
    fn rule_param_missing_returns_none() {
        let toml_str = r#"
[stale]
directory = "stale"

[[domains]]
prefix = "CHE"
name = "Cherry"
directory = "cherry"
description = "Test"
crates = []

[[rules]]
id = "T001"
category = "template"
description = "H1 title present"
"#;
        let config: Config = toml::from_str(toml_str).unwrap();
        assert_eq!(config.rule_param_u64("T001", "min_words"), None);
        assert_eq!(config.rule_param_u64("MISSING", "key"), None);
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

    #[test]
    fn foundation_flag_defaults_to_false() {
        let toml_str = r#"
[stale]
directory = "stale"

[[domains]]
prefix = "CHE"
name = "Cherry"
directory = "cherry"
description = "Test"
crates = []

[[rules]]
id = "T001"
category = "template"
description = "H1 title present"
"#;
        let config: Config = toml::from_str(toml_str).unwrap();
        assert!(!config.domains[0].foundation);
    }

    #[test]
    fn foundation_flag_true_deserializes() {
        let toml_str = r#"
[stale]
directory = "stale"

[[domains]]
prefix = "COM"
name = "Common"
directory = "common"
description = "Cross-cutting"
crates = []
foundation = true

[[rules]]
id = "T001"
category = "template"
description = "H1 title present"
"#;
        let config: Config = toml::from_str(toml_str).unwrap();
        assert!(config.domains[0].foundation);
    }

    #[test]
    fn internal_flag_defaults_to_false() {
        let toml_str = r#"
[stale]
directory = "stale"

[[domains]]
prefix = "CHE"
name = "Cherry"
directory = "cherry"
description = "Test"
crates = []

[[rules]]
id = "T001"
category = "template"
description = "H1 title present"
"#;
        let config: Config = toml::from_str(toml_str).unwrap();
        assert!(!config.rules[0].internal);
    }

    #[test]
    fn internal_flag_true_deserializes() {
        let toml_str = r#"
[stale]
directory = "stale"

[[domains]]
prefix = "CHE"
name = "Cherry"
directory = "cherry"
description = "Test"
crates = []

[[rules]]
id = "I001"
category = "index"
description = "ADR file exists but not in README"
internal = true
"#;
        let config: Config = toml::from_str(toml_str).unwrap();
        assert!(config.rules[0].internal);
    }

    #[test]
    fn backward_compat_no_params_field() {
        // Old-style TOML without params field still parses
        let toml_str = r#"
[stale]
directory = "stale"

[[domains]]
prefix = "CHE"
name = "Cherry"
directory = "cherry"
description = "Test"
crates = []

[[rules]]
id = "T001"
category = "template"
description = "H1 title present"

[[rules]]
id = "T002"
category = "template"
description = "Date field present"
"#;
        let config: Config = toml::from_str(toml_str).unwrap();
        assert_eq!(config.rules.len(), 2);
        assert!(config.rules[0].params.is_empty());
        assert!(config.rules[1].params.is_empty());
    }
}
