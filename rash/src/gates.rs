use crate::models::{Error, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct GateConfig {
    pub metadata: Option<Metadata>,
    pub gates: Gates,
    #[serde(default)]
    pub tiers: Tiers,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Metadata {
    pub version: String,
    pub tool: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Gates {
    #[serde(default)]
    pub run_clippy: bool,
    #[serde(default)]
    pub clippy_strict: bool,
    #[serde(default)]
    pub run_tests: bool,
    #[serde(default = "default_test_timeout")]
    pub test_timeout: u64,
    #[serde(default)]
    pub check_coverage: bool,
    #[serde(default = "default_min_coverage")]
    pub min_coverage: f64,
    #[serde(default)]
    pub check_complexity: bool,
    #[serde(default = "default_max_complexity")]
    pub max_complexity: usize,

    #[serde(default)]
    pub satd: Option<SatdGate>,
    #[serde(default)]
    pub mutation: Option<MutationGate>,
    #[serde(default)]
    pub security: Option<SecurityGate>,
}

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct SatdGate {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default)]
    pub max_count: usize,
    #[serde(default)]
    pub patterns: Vec<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct MutationGate {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default)]
    pub min_score: f64,
}

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct SecurityGate {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default)]
    pub max_unsafe_blocks: usize,
}

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct Tiers {
    #[serde(default)]
    pub tier1_gates: Vec<String>,
    #[serde(default)]
    pub tier2_gates: Vec<String>,
    #[serde(default)]
    pub tier3_gates: Vec<String>,
}

fn default_test_timeout() -> u64 {
    300
}
fn default_min_coverage() -> f64 {
    80.0
}
fn default_max_complexity() -> usize {
    10
}

impl GateConfig {
    pub fn load() -> Result<Self> {
        let filename = ".pmat-gates.toml";

        // Find config file in current or parent directories
        let mut current_dir = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        let config_path = loop {
            let candidate = current_dir.join(filename);
            if candidate.exists() {
                break Some(candidate);
            }
            if !current_dir.pop() {
                break None;
            }
        };

        let path = config_path.ok_or_else(|| {
            Error::Internal(format!(
                "Configuration file '{}' not found in current or parent directories.",
                filename
            ))
        })?;

        let content = fs::read_to_string(&path).map_err(Error::Io)?;
        let config: GateConfig = toml::from_str(&content)
            .map_err(|e| Error::Internal(format!("Failed to parse {}: {}", path.display(), e)))?;

        // Validate configuration
        if config.gates.min_coverage < 0.0 || config.gates.min_coverage > 100.0 {
            return Err(Error::Validation(format!(
                "min_coverage must be between 0.0 and 100.0, got {}",
                config.gates.min_coverage
            )));
        }

        Ok(config)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_values() {
        let toml = r#"
            [gates]
            run_clippy = true
        "#;
        let config: GateConfig = toml::from_str(toml).unwrap();
        assert_eq!(config.gates.test_timeout, 300);
        assert_eq!(config.gates.min_coverage, 80.0);
    }

    #[test]
    fn test_validation_error() {
        let toml = r#"
            [gates]
            min_coverage = 150.0
        "#;
        let config: Result<GateConfig> = toml::from_str::<GateConfig>(toml)
            .map_err(|e| Error::Internal(e.to_string()))
            .and_then(|mut c| {
                if c.gates.min_coverage > 100.0 {
                    Err(Error::Validation("Coverage too high".into()))
                } else {
                    Ok(c)
                }
            });

        assert!(config.is_err());
    }
}
