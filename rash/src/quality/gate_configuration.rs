//! Quality Gate Configuration and Enforcement (ML-001, ML-002, ML-003)
//!
//! Parses `.pmat-gates.toml` and enforces quality gates at different tiers.
//!
//! # Toyota Way Principles
//!
//! - **Heijunka** (Level the workload): Tiered gates distribute verification effort
//! - **Jidoka** (Automation with human touch): Gates automate checks, humans review results
//! - **Poka-yoke** (Error-proofing): Prevent quality regressions before they reach production

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::process::Command;
use std::time::{Duration, Instant};

/// Quality gate tier level
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum Tier {
    /// Tier 1: ON-SAVE checks (sub-second)
    /// Gates: clippy, complexity
    #[serde(rename = "tier1")]
    Tier1 = 1,

    /// Tier 2: ON-COMMIT checks (1-5 minutes)
    /// Gates: tests, coverage, SATD detection
    #[serde(rename = "tier2")]
    Tier2 = 2,

    /// Tier 3: NIGHTLY checks (hours)
    /// Gates: mutation testing, security audit
    #[serde(rename = "tier3")]
    Tier3 = 3,
}

impl From<u8> for Tier {
    fn from(value: u8) -> Self {
        match value {
            1 => Tier::Tier1,
            2 => Tier::Tier2,
            _ => Tier::Tier3,
        }
    }
}

impl std::fmt::Display for Tier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Tier::Tier1 => write!(f, "Tier 1 (ON-SAVE)"),
            Tier::Tier2 => write!(f, "Tier 2 (ON-COMMIT)"),
            Tier::Tier3 => write!(f, "Tier 3 (NIGHTLY)"),
        }
    }
}

/// Root configuration structure for `.pmat-gates.toml`
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct GateConfig {
    /// Metadata section
    #[serde(default)]
    pub metadata: MetadataConfig,

    /// Core gates configuration
    #[serde(default)]
    pub gates: GatesConfig,

    /// Tier definitions
    #[serde(default)]
    pub tiers: TiersConfig,

    /// Risk-based verification configuration
    #[serde(default)]
    pub risk_based: RiskBasedConfig,
}

/// Metadata configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetadataConfig {
    #[serde(default = "default_version")]
    pub version: String,
    #[serde(default = "default_tool")]
    pub tool: String,
}

fn default_version() -> String {
    "1.0.0".to_string()
}
fn default_tool() -> String {
    "bashrs".to_string()
}

impl Default for MetadataConfig {
    fn default() -> Self {
        Self {
            version: default_version(),
            tool: default_tool(),
        }
    }
}

/// Core gates configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GatesConfig {
    /// Run clippy linter
    #[serde(default = "default_true")]
    pub run_clippy: bool,

    /// Enforce strict clippy (-D warnings)
    #[serde(default = "default_true")]
    pub clippy_strict: bool,

    /// Run test suite
    #[serde(default = "default_true")]
    pub run_tests: bool,

    /// Test timeout in seconds
    #[serde(default = "default_timeout")]
    pub test_timeout: u64,

    /// Check code coverage
    #[serde(default = "default_true")]
    pub check_coverage: bool,

    /// Minimum coverage percentage
    #[serde(default = "default_coverage")]
    pub min_coverage: f64,

    /// Check cyclomatic complexity
    #[serde(default = "default_true")]
    pub check_complexity: bool,

    /// Maximum cyclomatic complexity per function (Toyota standard: 10)
    #[serde(default = "default_complexity")]
    pub max_complexity: u32,

    /// SATD (Self-Admitted Technical Debt) configuration
    #[serde(default)]
    pub satd: SatdConfig,

    /// Mutation testing configuration
    #[serde(default)]
    pub mutation: MutationConfig,

    /// Security audit configuration
    #[serde(default)]
    pub security: SecurityConfig,
}

fn default_true() -> bool {
    true
}
fn default_timeout() -> u64 {
    300
}
fn default_coverage() -> f64 {
    85.0
}
fn default_complexity() -> u32 {
    10
}

impl Default for GatesConfig {
    fn default() -> Self {
        Self {
            run_clippy: true,
            clippy_strict: true,
            run_tests: true,
            test_timeout: 300,
            check_coverage: true,
            min_coverage: 85.0,
            check_complexity: true,
            max_complexity: 10,
            satd: SatdConfig::default(),
            mutation: MutationConfig::default(),
            security: SecurityConfig::default(),
        }
    }
}

/// SATD (Self-Admitted Technical Debt) configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SatdConfig {
    #[serde(default = "default_true")]
    pub enabled: bool,

    #[serde(default)]
    pub max_count: usize,

    #[serde(default = "default_satd_patterns")]
    pub patterns: Vec<String>,

    #[serde(default)]
    pub require_issue_links: bool,

    #[serde(default = "default_true")]
    pub fail_on_violation: bool,
}

fn default_satd_patterns() -> Vec<String> {
    vec![
        "TODO".to_string(),
        "FIXME".to_string(),
        "HACK".to_string(),
        "XXX".to_string(),
    ]
}

impl Default for SatdConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            max_count: 0,
            patterns: default_satd_patterns(),
            require_issue_links: true,
            fail_on_violation: true,
        }
    }
}

/// Mutation testing configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MutationConfig {
    #[serde(default)]
    pub enabled: bool,

    #[serde(default = "default_mutation_score")]
    pub min_score: f64,

    #[serde(default = "default_mutation_tool")]
    pub tool: String,

    #[serde(default = "default_mutation_strategy")]
    pub strategy: String,
}

fn default_mutation_score() -> f64 {
    85.0
}
fn default_mutation_tool() -> String {
    "cargo-mutants".to_string()
}
fn default_mutation_strategy() -> String {
    "incremental".to_string()
}

impl Default for MutationConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            min_score: 85.0,
            tool: default_mutation_tool(),
            strategy: default_mutation_strategy(),
        }
    }
}

/// Security audit configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityConfig {
    #[serde(default = "default_true")]
    pub enabled: bool,

    #[serde(default = "default_audit_vulnerabilities")]
    pub audit_vulnerabilities: String,

    #[serde(default = "default_audit_unmaintained")]
    pub audit_unmaintained: String,

    #[serde(default)]
    pub max_unsafe_blocks: usize,

    #[serde(default = "default_true")]
    pub fail_on_violation: bool,
}

fn default_audit_vulnerabilities() -> String {
    "deny".to_string()
}
fn default_audit_unmaintained() -> String {
    "warn".to_string()
}

impl Default for SecurityConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            audit_vulnerabilities: default_audit_vulnerabilities(),
            audit_unmaintained: default_audit_unmaintained(),
            max_unsafe_blocks: 0,
            fail_on_violation: true,
        }
    }
}

/// Tier definitions configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TiersConfig {
    #[serde(default = "default_tier1_gates")]
    pub tier1_gates: Vec<String>,

    #[serde(default = "default_tier2_gates")]
    pub tier2_gates: Vec<String>,

    #[serde(default = "default_tier3_gates")]
    pub tier3_gates: Vec<String>,
}

fn default_tier1_gates() -> Vec<String> {
    vec!["clippy".to_string(), "complexity".to_string()]
}

fn default_tier2_gates() -> Vec<String> {
    vec![
        "clippy".to_string(),
        "tests".to_string(),
        "coverage".to_string(),
        "satd".to_string(),
    ]
}

fn default_tier3_gates() -> Vec<String> {
    vec![
        "clippy".to_string(),
        "tests".to_string(),
        "coverage".to_string(),
        "satd".to_string(),
        "mutation".to_string(),
        "security".to_string(),
    ]
}

impl Default for TiersConfig {
    fn default() -> Self {
        Self {
            tier1_gates: default_tier1_gates(),
            tier2_gates: default_tier2_gates(),
            tier3_gates: default_tier3_gates(),
        }
    }
}

/// Risk-based verification configuration (Pareto principle)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskBasedConfig {
    #[serde(default = "default_very_high_risk_target")]
    pub very_high_risk_mutation_target: f64,

    #[serde(default)]
    pub very_high_risk_components: Vec<String>,

    #[serde(default = "default_high_risk_target")]
    pub high_risk_mutation_target: f64,

    #[serde(default)]
    pub high_risk_components: Vec<String>,
}

fn default_very_high_risk_target() -> f64 {
    92.5
}
fn default_high_risk_target() -> f64 {
    87.5
}

impl Default for RiskBasedConfig {
    fn default() -> Self {
        Self {
            very_high_risk_mutation_target: 92.5,
            very_high_risk_components: vec![],
            high_risk_mutation_target: 87.5,
            high_risk_components: vec![],
        }
    }
}

impl GateConfig {
    /// Load configuration from `.pmat-gates.toml` file
    pub fn load(path: &Path) -> Result<Self, GateConfigError> {
        let content = fs::read_to_string(path).map_err(|e| GateConfigError::Io {
            path: path.to_path_buf(),
            source: e,
        })?;

        toml::from_str(&content).map_err(|e| GateConfigError::Parse {
            path: path.to_path_buf(),
            source: e,
        })
    }

    /// Load from default locations or return default config
    pub fn load_or_default() -> Self {
        // Try current directory first
        if let Ok(config) = Self::load(Path::new(".pmat-gates.toml")) {
            return config;
        }

        // Try project root (look for Cargo.toml)
        let mut current = std::env::current_dir().unwrap_or_default();
        loop {
            let candidate = current.join(".pmat-gates.toml");
            if candidate.exists() {
                if let Ok(config) = Self::load(&candidate) {
                    return config;
                }
            }
            if !current.pop() {
                break;
            }
        }

        // Return default configuration
        Self::default()
    }

    /// Get gates for a specific tier
    pub fn gates_for_tier(&self, tier: Tier) -> &[String] {
        match tier {
            Tier::Tier1 => &self.tiers.tier1_gates,
            Tier::Tier2 => &self.tiers.tier2_gates,
            Tier::Tier3 => &self.tiers.tier3_gates,
        }
    }
}

/// Errors that can occur during gate configuration
#[derive(Debug)]
pub enum GateConfigError {
    Io {
        path: std::path::PathBuf,
        source: std::io::Error,
    },
    Parse {
        path: std::path::PathBuf,
        source: toml::de::Error,
    },
}

impl std::fmt::Display for GateConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GateConfigError::Io { path, source } => {
                write!(f, "Failed to read {}: {}", path.display(), source)
            }
            GateConfigError::Parse { path, source } => {
                write!(f, "Failed to parse {}: {}", path.display(), source)
            }
        }
    }
}

impl std::error::Error for GateConfigError {}

/// Result of running a quality gate
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GateResult {
    /// Name of the gate
    pub gate_name: String,

    /// Whether the gate passed
    pub passed: bool,

    /// Duration of the check
    pub duration: Duration,

    /// Detailed message
    pub message: String,

    /// Metrics collected (e.g., coverage percentage)
    pub metrics: HashMap<String, f64>,

    /// Violations found
    pub violations: Vec<GateViolation>,
}

/// A specific violation found by a gate
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GateViolation {
    /// File where violation occurred
    pub file: Option<String>,


}

    include!("gates_part2_incl2.rs");
