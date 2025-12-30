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

    #[test]
    fn test_satd_gate_clone() {
        let gate = SatdGate {
            enabled: true,
            max_count: 5,
            patterns: vec!["TODO".to_string(), "FIXME".to_string()],
        };
        let cloned = gate.clone();
        assert_eq!(cloned.enabled, true);
        assert_eq!(cloned.max_count, 5);
        assert_eq!(cloned.patterns.len(), 2);
    }

    #[test]
    fn test_mutation_gate_clone() {
        let gate = MutationGate {
            enabled: true,
            min_score: 90.0,
        };
        let cloned = gate.clone();
        assert_eq!(cloned.enabled, true);
        assert_eq!(cloned.min_score, 90.0);
    }

    #[test]
    fn test_security_gate_clone() {
        let gate = SecurityGate {
            enabled: true,
            max_unsafe_blocks: 3,
        };
        let cloned = gate.clone();
        assert_eq!(cloned.enabled, true);
        assert_eq!(cloned.max_unsafe_blocks, 3);
    }

    #[test]
    fn test_tiers_clone() {
        let tiers = Tiers {
            tier1_gates: vec!["clippy".to_string()],
            tier2_gates: vec!["coverage".to_string()],
            tier3_gates: vec!["mutation".to_string()],
        };
        let cloned = tiers.clone();
        assert_eq!(cloned.tier1_gates, vec!["clippy"]);
        assert_eq!(cloned.tier2_gates, vec!["coverage"]);
        assert_eq!(cloned.tier3_gates, vec!["mutation"]);
    }

    #[test]
    fn test_metadata_clone() {
        let metadata = Metadata {
            version: "2.0".to_string(),
            tool: "bashrs".to_string(),
        };
        let cloned = metadata.clone();
        assert_eq!(cloned.version, "2.0");
        assert_eq!(cloned.tool, "bashrs");
    }

    #[test]
    fn test_gates_serialization() {
        let gates = Gates {
            run_clippy: true,
            clippy_strict: false,
            run_tests: true,
            test_timeout: 300,
            check_coverage: true,
            min_coverage: 85.0,
            check_complexity: true,
            max_complexity: 15,
            satd: None,
            mutation: None,
            security: None,
        };
        let serialized = toml::to_string(&gates).unwrap();
        assert!(serialized.contains("run_clippy = true"));
        assert!(serialized.contains("min_coverage = 85.0"));
    }

    #[test]
    fn test_gate_config_serialization() {
        let config = GateConfig {
            metadata: Some(Metadata {
                version: "1.0.0".to_string(),
                tool: "test".to_string(),
            }),
            gates: Gates {
                run_clippy: true,
                clippy_strict: false,
                run_tests: false,
                test_timeout: 300,
                check_coverage: false,
                min_coverage: 80.0,
                check_complexity: false,
                max_complexity: 10,
                satd: None,
                mutation: None,
                security: None,
            },
            tiers: Tiers::default(),
        };
        let serialized = toml::to_string(&config).unwrap();
        assert!(serialized.contains("[metadata]"));
        assert!(serialized.contains("[gates]"));
    }

    #[test]
    fn test_satd_gate_serialization() {
        let gate = SatdGate {
            enabled: true,
            max_count: 10,
            patterns: vec!["TODO".to_string(), "FIXME".to_string()],
        };
        let serialized = toml::to_string(&gate).unwrap();
        assert!(serialized.contains("enabled = true"));
        assert!(serialized.contains("max_count = 10"));
    }

    #[test]
    fn test_mutation_gate_serialization() {
        let gate = MutationGate {
            enabled: true,
            min_score: 80.0,
        };
        let serialized = toml::to_string(&gate).unwrap();
        assert!(serialized.contains("enabled = true"));
        assert!(serialized.contains("min_score = 80.0"));
    }

    #[test]
    fn test_security_gate_serialization() {
        let gate = SecurityGate {
            enabled: true,
            max_unsafe_blocks: 5,
        };
        let serialized = toml::to_string(&gate).unwrap();
        assert!(serialized.contains("enabled = true"));
        assert!(serialized.contains("max_unsafe_blocks = 5"));
    }

    #[test]
    fn test_tiers_serialization() {
        let tiers = Tiers {
            tier1_gates: vec!["clippy".to_string(), "tests".to_string()],
            tier2_gates: vec!["coverage".to_string()],
            tier3_gates: vec![],
        };
        let serialized = toml::to_string(&tiers).unwrap();
        assert!(serialized.contains("tier1_gates"));
        assert!(serialized.contains("clippy"));
    }

    #[test]
    fn test_gates_debug_all_fields() {
        let gates = Gates {
            run_clippy: true,
            clippy_strict: true,
            run_tests: true,
            test_timeout: 600,
            check_coverage: true,
            min_coverage: 90.0,
            check_complexity: true,
            max_complexity: 5,
            satd: Some(SatdGate::default()),
            mutation: Some(MutationGate::default()),
            security: Some(SecurityGate::default()),
        };
        let debug_str = format!("{:?}", gates);
        assert!(debug_str.contains("run_clippy"));
        assert!(debug_str.contains("clippy_strict"));
        assert!(debug_str.contains("run_tests"));
        assert!(debug_str.contains("satd"));
        assert!(debug_str.contains("mutation"));
        assert!(debug_str.contains("security"));
    }

    #[test]
    fn test_satd_gate_debug() {
        let gate = SatdGate {
            enabled: true,
            max_count: 5,
            patterns: vec!["TODO".to_string()],
        };
        let debug_str = format!("{:?}", gate);
        assert!(debug_str.contains("enabled"));
        assert!(debug_str.contains("max_count"));
        assert!(debug_str.contains("patterns"));
    }

    #[test]
    fn test_mutation_gate_debug() {
        let gate = MutationGate {
            enabled: false,
            min_score: 75.0,
        };
        let debug_str = format!("{:?}", gate);
        assert!(debug_str.contains("enabled"));
        assert!(debug_str.contains("min_score"));
    }

    #[test]
    fn test_security_gate_debug() {
        let gate = SecurityGate {
            enabled: true,
            max_unsafe_blocks: 2,
        };
        let debug_str = format!("{:?}", gate);
        assert!(debug_str.contains("enabled"));
        assert!(debug_str.contains("max_unsafe_blocks"));
    }

    #[test]
    fn test_tiers_debug() {
        let tiers = Tiers {
            tier1_gates: vec!["a".to_string()],
            tier2_gates: vec!["b".to_string()],
            tier3_gates: vec!["c".to_string()],
        };
        let debug_str = format!("{:?}", tiers);
        assert!(debug_str.contains("tier1_gates"));
        assert!(debug_str.contains("tier2_gates"));
        assert!(debug_str.contains("tier3_gates"));
    }

    #[test]
    fn test_gate_config_debug() {
        let config = GateConfig {
            metadata: None,
            gates: Gates {
                run_clippy: false,
                clippy_strict: false,
                run_tests: false,
                test_timeout: 300,
                check_coverage: false,
                min_coverage: 80.0,
                check_complexity: false,
                max_complexity: 10,
                satd: None,
                mutation: None,
                security: None,
            },
            tiers: Tiers::default(),
        };
        let debug_str = format!("{:?}", config);
        assert!(debug_str.contains("metadata"));
        assert!(debug_str.contains("gates"));
        assert!(debug_str.contains("tiers"));
    }

    #[test]
    fn test_config_with_all_optional_gates() {
        let toml = r#"
            [gates]
            run_clippy = true

            [gates.satd]
            enabled = false
            max_count = 0
            patterns = []

            [gates.mutation]
            enabled = false
            min_score = 0.0

            [gates.security]
            enabled = false
            max_unsafe_blocks = 0
        "#;
        let config: GateConfig = toml::from_str(toml).unwrap();
        assert!(config.gates.satd.is_some());
        assert!(config.gates.mutation.is_some());
        assert!(config.gates.security.is_some());
    }
}
