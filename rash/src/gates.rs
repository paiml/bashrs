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
            .and_then(|c| {
                if c.gates.min_coverage > 100.0 {
                    Err(Error::Validation("Coverage too high".into()))
                } else {
                    Ok(c)
                }
            });

        assert!(config.is_err());
    }

    #[test]
    fn test_default_test_timeout() {
        assert_eq!(default_test_timeout(), 300);
    }

    #[test]
    fn test_default_min_coverage() {
        assert_eq!(default_min_coverage(), 80.0);
    }

    #[test]
    fn test_default_max_complexity() {
        assert_eq!(default_max_complexity(), 10);
    }

    #[test]
    fn test_full_gate_config() {
        let toml = r#"
            [metadata]
            version = "1.0.0"
            tool = "bashrs"

            [gates]
            run_clippy = true
            clippy_strict = true
            run_tests = true
            test_timeout = 600
            check_coverage = true
            min_coverage = 85.0
            check_complexity = true
            max_complexity = 15

            [gates.satd]
            enabled = true
            max_count = 10
            patterns = ["TODO", "FIXME"]

            [gates.mutation]
            enabled = true
            min_score = 80.0

            [gates.security]
            enabled = true
            max_unsafe_blocks = 5

            [tiers]
            tier1_gates = ["clippy", "tests"]
            tier2_gates = ["coverage", "complexity"]
            tier3_gates = ["mutation", "security"]
        "#;
        let config: GateConfig = toml::from_str(toml).unwrap();

        // Check metadata
        assert!(config.metadata.is_some());
        let metadata = config.metadata.unwrap();
        assert_eq!(metadata.version, "1.0.0");
        assert_eq!(metadata.tool, "bashrs");

        // Check gates
        assert!(config.gates.run_clippy);
        assert!(config.gates.clippy_strict);
        assert!(config.gates.run_tests);
        assert_eq!(config.gates.test_timeout, 600);
        assert!(config.gates.check_coverage);
        assert_eq!(config.gates.min_coverage, 85.0);
        assert!(config.gates.check_complexity);
        assert_eq!(config.gates.max_complexity, 15);

        // Check SATD gate
        let satd = config.gates.satd.unwrap();
        assert!(satd.enabled);
        assert_eq!(satd.max_count, 10);
        assert_eq!(satd.patterns, vec!["TODO", "FIXME"]);

        // Check mutation gate
        let mutation = config.gates.mutation.unwrap();
        assert!(mutation.enabled);
        assert_eq!(mutation.min_score, 80.0);

        // Check security gate
        let security = config.gates.security.unwrap();
        assert!(security.enabled);
        assert_eq!(security.max_unsafe_blocks, 5);

        // Check tiers
        assert_eq!(config.tiers.tier1_gates, vec!["clippy", "tests"]);
        assert_eq!(config.tiers.tier2_gates, vec!["coverage", "complexity"]);
        assert_eq!(config.tiers.tier3_gates, vec!["mutation", "security"]);
    }

    #[test]
    fn test_minimal_config() {
        let toml = r#"
            [gates]
        "#;
        let config: GateConfig = toml::from_str(toml).unwrap();

        // All booleans should default to false
        assert!(!config.gates.run_clippy);
        assert!(!config.gates.clippy_strict);
        assert!(!config.gates.run_tests);
        assert!(!config.gates.check_coverage);
        assert!(!config.gates.check_complexity);

        // Check defaults
        assert_eq!(config.gates.test_timeout, 300);
        assert_eq!(config.gates.min_coverage, 80.0);
        assert_eq!(config.gates.max_complexity, 10);

        // Optional fields should be None
        assert!(config.gates.satd.is_none());
        assert!(config.gates.mutation.is_none());
        assert!(config.gates.security.is_none());
        assert!(config.metadata.is_none());
    }

    #[test]
    fn test_satd_gate_defaults() {
        let gate = SatdGate::default();
        assert!(!gate.enabled);
        assert_eq!(gate.max_count, 0);
        assert!(gate.patterns.is_empty());
    }

    #[test]
    fn test_mutation_gate_defaults() {
        let gate = MutationGate::default();
        assert!(!gate.enabled);
        assert_eq!(gate.min_score, 0.0);
    }

    #[test]
    fn test_security_gate_defaults() {
        let gate = SecurityGate::default();
        assert!(!gate.enabled);
        assert_eq!(gate.max_unsafe_blocks, 0);
    }

    #[test]
    fn test_tiers_defaults() {
        let tiers = Tiers::default();
        assert!(tiers.tier1_gates.is_empty());
        assert!(tiers.tier2_gates.is_empty());
        assert!(tiers.tier3_gates.is_empty());
    }

    #[test]
    fn test_config_clone() {
        let toml = r#"
            [gates]
            run_clippy = true
        "#;
        let config: GateConfig = toml::from_str(toml).unwrap();
        let cloned = config.clone();
        assert_eq!(config.gates.run_clippy, cloned.gates.run_clippy);
    }

    #[test]
    fn test_negative_coverage_handling() {
        let toml = r#"
            [gates]
            min_coverage = -10.0
        "#;
        let config: GateConfig = toml::from_str(toml).unwrap();
        // The validation happens in load(), not parse
        // So we test that the value is parsed correctly
        assert_eq!(config.gates.min_coverage, -10.0);
    }

    #[test]
    fn test_gates_debug_format() {
        let gates = Gates {
            run_clippy: true,
            clippy_strict: false,
            run_tests: true,
            test_timeout: 300,
            check_coverage: false,
            min_coverage: 80.0,
            check_complexity: false,
            max_complexity: 10,
            satd: None,
            mutation: None,
            security: None,
        };
        let debug_str = format!("{:?}", gates);
        assert!(debug_str.contains("run_clippy: true"));
    }

    #[test]
    fn test_metadata_debug_format() {
        let metadata = Metadata {
            version: "1.0".to_string(),
            tool: "test".to_string(),
        };
        let debug_str = format!("{:?}", metadata);
        assert!(debug_str.contains("version"));
        assert!(debug_str.contains("tool"));
    }
}
