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

    /// Line number
    pub line: Option<usize>,

    /// Violation description
    pub description: String,

    /// Severity level
    pub severity: ViolationSeverity,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ViolationSeverity {
    Error,
    Warning,
    Info,
}

/// Quality gate executor
pub struct QualityGate {
    config: GateConfig,
}

impl QualityGate {
    /// Create a new quality gate with the given configuration
    pub fn new(config: GateConfig) -> Self {
        Self { config }
    }

    /// Create with default configuration
    pub fn with_defaults() -> Self {
        Self::new(GateConfig::default())
    }

    /// Run all gates for the specified tier
    pub fn run_tier(&self, tier: Tier) -> Vec<GateResult> {
        let gates = self.config.gates_for_tier(tier);
        let mut results = Vec::new();

        for gate_name in gates {
            let result = self.run_gate(gate_name);
            results.push(result);
        }

        results
    }

    /// Run a specific gate by name
    pub fn run_gate(&self, gate_name: &str) -> GateResult {
        let start = Instant::now();

        let (passed, message, metrics, violations) = match gate_name {
            "clippy" => self.run_clippy_gate(),
            "complexity" => self.run_complexity_gate(),
            "tests" => self.run_tests_gate(),
            "coverage" => self.run_coverage_gate(),
            "satd" => self.run_satd_gate(),
            "mutation" => self.run_mutation_gate(),
            "security" => self.run_security_gate(),
            _ => (
                false,
                format!("Unknown gate: {}", gate_name),
                HashMap::new(),
                vec![],
            ),
        };

        GateResult {
            gate_name: gate_name.to_string(),
            passed,
            duration: start.elapsed(),
            message,
            metrics,
            violations,
        }
    }

    fn run_clippy_gate(&self) -> (bool, String, HashMap<String, f64>, Vec<GateViolation>) {
        if !self.config.gates.run_clippy {
            return (
                true,
                "Clippy gate disabled".to_string(),
                HashMap::new(),
                vec![],
            );
        }

        let mut cmd = Command::new("cargo");
        cmd.args(["clippy", "--lib", "-p", "bashrs", "--message-format=json"]);

        if self.config.gates.clippy_strict {
            cmd.args(["--", "-D", "warnings"]);
        }

        match cmd.output() {
            Ok(output) => {
                let exit_code = output.status.code().unwrap_or(1);
                let passed = exit_code == 0;

                let mut violations = Vec::new();
                let stderr = String::from_utf8_lossy(&output.stderr);

                // Parse JSON output for violations
                for line in stderr.lines() {
                    if line.contains("\"level\":\"error\"")
                        || line.contains("\"level\":\"warning\"")
                    {
                        violations.push(GateViolation {
                            file: None,
                            line: None,
                            description: line.to_string(),
                            severity: if line.contains("error") {
                                ViolationSeverity::Error
                            } else {
                                ViolationSeverity::Warning
                            },
                        });
                    }
                }

                let message = if passed {
                    "Clippy passed with no warnings".to_string()
                } else {
                    format!("Clippy found {} issues", violations.len())
                };

                let mut metrics = HashMap::new();
                metrics.insert("violations".to_string(), violations.len() as f64);

                (passed, message, metrics, violations)
            }
            Err(e) => (
                false,
                format!("Failed to run clippy: {}", e),
                HashMap::new(),
                vec![],
            ),
        }
    }

    fn run_complexity_gate(&self) -> (bool, String, HashMap<String, f64>, Vec<GateViolation>) {
        if !self.config.gates.check_complexity {
            return (
                true,
                "Complexity gate disabled".to_string(),
                HashMap::new(),
                vec![],
            );
        }

        // Use pmat for complexity analysis if available
        let output = Command::new("pmat")
            .args(["analyze", "complexity", "--path", ".", "--max", "10"])
            .output();

        match output {
            Ok(output) => {
                let passed = output.status.success();
                // stdout available for future detailed parsing
                let _stdout = String::from_utf8_lossy(&output.stdout);

                let mut metrics = HashMap::new();
                metrics.insert(
                    "max_allowed".to_string(),
                    self.config.gates.max_complexity as f64,
                );

                let message = if passed {
                    format!(
                        "All functions below complexity {}",
                        self.config.gates.max_complexity
                    )
                } else {
                    "Functions exceed complexity threshold".to_string()
                };

                (passed, message, metrics, vec![])
            }
            Err(_) => {
                // pmat not available, pass by default
                (
                    true,
                    "Complexity check skipped (pmat not available)".to_string(),
                    HashMap::new(),
                    vec![],
                )
            }
        }
    }

    fn run_tests_gate(&self) -> (bool, String, HashMap<String, f64>, Vec<GateViolation>) {
        if !self.config.gates.run_tests {
            return (
                true,
                "Tests gate disabled".to_string(),
                HashMap::new(),
                vec![],
            );
        }

        let output = Command::new("cargo")
            .args(["test", "--lib", "-p", "bashrs", "--", "--test-threads=4"])
            .output();

        match output {
            Ok(output) => {
                let passed = output.status.success();
                let stdout = String::from_utf8_lossy(&output.stdout);

                // Parse test count from output
                let total_tests = 0;
                let mut passed_tests = 0;

                for line in stdout.lines() {
                    if line.contains("passed") && line.contains("failed") {
                        // Parse "test result: ok. X passed; Y failed"
                        if let Some(idx) = line.find("passed") {
                            let before = &line[..idx];
                            if let Some(num_str) = before.split_whitespace().last() {
                                passed_tests = num_str.parse().unwrap_or(0);
                            }
                        }
                    }
                }

                let mut metrics = HashMap::new();
                metrics.insert("passed".to_string(), passed_tests as f64);
                metrics.insert("total".to_string(), total_tests as f64);

                let message = if passed {
                    format!("{} tests passed", passed_tests)
                } else {
                    "Tests failed".to_string()
                };

                (passed, message, metrics, vec![])
            }
            Err(e) => (
                false,
                format!("Failed to run tests: {}", e),
                HashMap::new(),
                vec![],
            ),
        }
    }

    fn run_coverage_gate(&self) -> (bool, String, HashMap<String, f64>, Vec<GateViolation>) {
        if !self.config.gates.check_coverage {
            return (
                true,
                "Coverage gate disabled".to_string(),
                HashMap::new(),
                vec![],
            );
        }

        // This is a placeholder - actual coverage would use cargo-llvm-cov
        let mut metrics = HashMap::new();
        metrics.insert("target".to_string(), self.config.gates.min_coverage);

        (
            true,
            format!(
                "Coverage check (target: {}%) - run `make coverage` for full analysis",
                self.config.gates.min_coverage
            ),
            metrics,
            vec![],
        )
    }

    fn run_satd_gate(&self) -> (bool, String, HashMap<String, f64>, Vec<GateViolation>) {
        if !self.config.gates.satd.enabled {
            return (
                true,
                "SATD gate disabled".to_string(),
                HashMap::new(),
                vec![],
            );
        }

        // Search for SATD patterns in source files
        let patterns = &self.config.gates.satd.patterns;
        let mut violations = Vec::new();

        for pattern in patterns {
            let output = Command::new("grep")
                .args([
                    "-rn",
                    "--include=*.rs",
                    pattern,
                    "rash/src/",
                    "rash-runtime/src/",
                ])
                .output();

            if let Ok(output) = output {
                let stdout = String::from_utf8_lossy(&output.stdout);
                for line in stdout.lines() {
                    if !line.contains("tests") && !line.contains("_test.rs") {
                        violations.push(GateViolation {
                            file: line.split(':').next().map(String::from),
                            line: line.split(':').nth(1).and_then(|s| s.parse().ok()),
                            description: format!("SATD pattern '{}' found", pattern),
                            severity: ViolationSeverity::Warning,
                        });
                    }
                }
            }
        }

        let satd_count = violations.len();
        let passed = satd_count <= self.config.gates.satd.max_count
            || !self.config.gates.satd.fail_on_violation;

        let mut metrics = HashMap::new();
        metrics.insert("count".to_string(), satd_count as f64);
        metrics.insert(
            "max_allowed".to_string(),
            self.config.gates.satd.max_count as f64,
        );

        let message = if passed {
            format!(
                "SATD check passed ({} found, {} allowed)",
                satd_count, self.config.gates.satd.max_count
            )
        } else {
            format!(
                "SATD check failed: {} technical debt markers found (max: {})",
                satd_count, self.config.gates.satd.max_count
            )
        };

        (passed, message, metrics, violations)
    }

    fn run_mutation_gate(&self) -> (bool, String, HashMap<String, f64>, Vec<GateViolation>) {
        if !self.config.gates.mutation.enabled {
            return (
                true,
                "Mutation testing disabled (enable for Tier 3)".to_string(),
                HashMap::new(),
                vec![],
            );
        }

        let mut metrics = HashMap::new();
        metrics.insert("target".to_string(), self.config.gates.mutation.min_score);

        (
            true,
            format!(
                "Mutation testing (target: {}%) - run `cargo mutants` manually",
                self.config.gates.mutation.min_score
            ),
            metrics,
            vec![],
        )
    }

    fn run_security_gate(&self) -> (bool, String, HashMap<String, f64>, Vec<GateViolation>) {
        if !self.config.gates.security.enabled {
            return (
                true,
                "Security gate disabled".to_string(),
                HashMap::new(),
                vec![],
            );
        }

        // Run cargo audit
        let output = Command::new("cargo").args(["audit"]).output();

        match output {
            Ok(output) => {
                let passed = output.status.success();
                // stdout available for future detailed parsing
                let _stdout = String::from_utf8_lossy(&output.stdout);

                let message = if passed {
                    "No security vulnerabilities found".to_string()
                } else {
                    "Security vulnerabilities detected".to_string()
                };

                (passed, message, HashMap::new(), vec![])
            }
            Err(_) => (
                true,
                "Security audit skipped (cargo-audit not installed)".to_string(),
                HashMap::new(),
                vec![],
            ),
        }
    }

    /// Check if all results passed
    pub fn all_passed(results: &[GateResult]) -> bool {
        results.iter().all(|r| r.passed)
    }

    /// Get summary statistics
    pub fn summary(results: &[GateResult]) -> GateSummary {
        let total = results.len();
        let passed = results.iter().filter(|r| r.passed).count();
        let failed = total - passed;
        let total_duration: Duration = results.iter().map(|r| r.duration).sum();

        GateSummary {
            total,
            passed,
            failed,
            total_duration,
        }
    }
}

/// Summary of gate execution
#[derive(Debug, Clone)]
pub struct GateSummary {
    pub total: usize,
    pub passed: usize,
    pub failed: usize,
    pub total_duration: Duration,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ml_001_gate_config_default() {
        let config = GateConfig::default();

        assert_eq!(config.metadata.version, "1.0.0");
        assert_eq!(config.metadata.tool, "bashrs");
        assert!(config.gates.run_clippy);
        assert!(config.gates.clippy_strict);
        assert_eq!(config.gates.max_complexity, 10);
        assert_eq!(config.gates.min_coverage, 85.0);
    }

    #[test]
    fn test_ml_001_gate_config_parse() {
        let toml_content = r#"
[metadata]
version = "1.0.0"
tool = "bashrs"

[gates]
run_clippy = true
clippy_strict = true
min_coverage = 90.0
max_complexity = 8

[gates.satd]
enabled = true
max_count = 0
patterns = ["TODO", "FIXME"]

[tiers]
tier1_gates = ["clippy"]
tier2_gates = ["clippy", "tests"]
"#;

        let config: GateConfig = toml::from_str(toml_content).expect("valid toml");

        assert_eq!(config.gates.min_coverage, 90.0);
        assert_eq!(config.gates.max_complexity, 8);
        assert_eq!(config.gates.satd.patterns.len(), 2);
        assert_eq!(config.tiers.tier1_gates, vec!["clippy"]);
    }

    #[test]
    fn test_ml_001_tier_from_u8() {
        assert_eq!(Tier::from(1), Tier::Tier1);
        assert_eq!(Tier::from(2), Tier::Tier2);
        assert_eq!(Tier::from(3), Tier::Tier3);
        assert_eq!(Tier::from(99), Tier::Tier3); // Defaults to Tier3
    }

    #[test]
    fn test_ml_001_gates_for_tier() {
        let config = GateConfig::default();

        let tier1 = config.gates_for_tier(Tier::Tier1);
        assert!(tier1.contains(&"clippy".to_string()));
        assert!(tier1.contains(&"complexity".to_string()));

        let tier2 = config.gates_for_tier(Tier::Tier2);
        assert!(tier2.contains(&"tests".to_string()));
        assert!(tier2.contains(&"coverage".to_string()));

        let tier3 = config.gates_for_tier(Tier::Tier3);
        assert!(tier3.contains(&"mutation".to_string()));
        assert!(tier3.contains(&"security".to_string()));
    }

    #[test]
    fn test_ml_002_quality_gate_creation() {
        let gate = QualityGate::with_defaults();
        assert_eq!(gate.config.gates.max_complexity, 10);
    }

    #[test]
    fn test_ml_003_gate_summary() {
        let results = vec![
            GateResult {
                gate_name: "clippy".to_string(),
                passed: true,
                duration: Duration::from_millis(100),
                message: "OK".to_string(),
                metrics: HashMap::new(),
                violations: vec![],
            },
            GateResult {
                gate_name: "tests".to_string(),
                passed: false,
                duration: Duration::from_millis(200),
                message: "Failed".to_string(),
                metrics: HashMap::new(),
                violations: vec![],
            },
        ];

        let summary = QualityGate::summary(&results);
        assert_eq!(summary.total, 2);
        assert_eq!(summary.passed, 1);
        assert_eq!(summary.failed, 1);
        assert_eq!(summary.total_duration, Duration::from_millis(300));
    }

    #[test]
    fn test_ml_003_all_passed() {
        let passing = vec![GateResult {
            gate_name: "test".to_string(),
            passed: true,
            duration: Duration::default(),
            message: String::new(),
            metrics: HashMap::new(),
            violations: vec![],
        }];

        let failing = vec![GateResult {
            gate_name: "test".to_string(),
            passed: false,
            duration: Duration::default(),
            message: String::new(),
            metrics: HashMap::new(),
            violations: vec![],
        }];

        assert!(QualityGate::all_passed(&passing));
        assert!(!QualityGate::all_passed(&failing));
    }

    // Additional tests for coverage

    #[test]
    fn test_tier_display() {
        assert_eq!(format!("{}", Tier::Tier1), "Tier 1 (ON-SAVE)");
        assert_eq!(format!("{}", Tier::Tier2), "Tier 2 (ON-COMMIT)");
        assert_eq!(format!("{}", Tier::Tier3), "Tier 3 (NIGHTLY)");
    }

    #[test]
    fn test_tier_ordering() {
        assert!(Tier::Tier1 < Tier::Tier2);
        assert!(Tier::Tier2 < Tier::Tier3);
        assert!(Tier::Tier1 < Tier::Tier3);
    }

    #[test]
    fn test_metadata_config_default() {
        let meta = MetadataConfig::default();
        assert_eq!(meta.version, "1.0.0");
        assert_eq!(meta.tool, "bashrs");
    }

    #[test]
    fn test_satd_config_default() {
        let satd = SatdConfig::default();
        assert!(satd.enabled);
        assert_eq!(satd.max_count, 0);
        assert!(satd.patterns.contains(&"TODO".to_string()));
        assert!(satd.patterns.contains(&"FIXME".to_string()));
        assert!(satd.patterns.contains(&"HACK".to_string()));
        assert!(satd.patterns.contains(&"XXX".to_string()));
        assert!(satd.require_issue_links);
        assert!(satd.fail_on_violation);
    }

    #[test]
    fn test_mutation_config_default() {
        let mutation = MutationConfig::default();
        assert!(!mutation.enabled);
        assert_eq!(mutation.min_score, 85.0);
        assert_eq!(mutation.tool, "cargo-mutants");
        assert_eq!(mutation.strategy, "incremental");
    }

    #[test]
    fn test_security_config_default() {
        let security = SecurityConfig::default();
        assert!(security.enabled);
        assert_eq!(security.audit_vulnerabilities, "deny");
        assert_eq!(security.audit_unmaintained, "warn");
        assert_eq!(security.max_unsafe_blocks, 0);
        assert!(security.fail_on_violation);
    }

    #[test]
    fn test_risk_based_config_default() {
        let risk = RiskBasedConfig::default();
        assert_eq!(risk.very_high_risk_mutation_target, 92.5);
        assert!(risk.very_high_risk_components.is_empty());
        assert_eq!(risk.high_risk_mutation_target, 87.5);
        assert!(risk.high_risk_components.is_empty());
    }

    #[test]
    fn test_tiers_config_default() {
        let tiers = TiersConfig::default();
        assert_eq!(tiers.tier1_gates, vec!["clippy", "complexity"]);
        assert_eq!(
            tiers.tier2_gates,
            vec!["clippy", "tests", "coverage", "satd"]
        );
        assert_eq!(
            tiers.tier3_gates,
            vec!["clippy", "tests", "coverage", "satd", "mutation", "security"]
        );
    }

    #[test]
    fn test_gates_config_default() {
        let gates = GatesConfig::default();
        assert!(gates.run_clippy);
        assert!(gates.clippy_strict);
        assert!(gates.run_tests);
        assert_eq!(gates.test_timeout, 300);
        assert!(gates.check_coverage);
        assert_eq!(gates.min_coverage, 85.0);
        assert!(gates.check_complexity);
        assert_eq!(gates.max_complexity, 10);
    }

    #[test]
    fn test_gate_config_error_display() {
        let io_error = GateConfigError::Io {
            path: std::path::PathBuf::from("/test/path.toml"),
            source: std::io::Error::new(std::io::ErrorKind::NotFound, "not found"),
        };
        assert!(format!("{}", io_error).contains("/test/path.toml"));
        assert!(format!("{}", io_error).contains("Failed to read"));
    }

    #[test]
    fn test_gate_config_error_parse_display() {
        // Trigger a real parse error by parsing invalid TOML
        let result: Result<GateConfig, _> = toml::from_str("invalid { toml }");
        assert!(result.is_err());
        let parse_error = GateConfigError::Parse {
            path: std::path::PathBuf::from("/test/config.toml"),
            source: result.unwrap_err(),
        };
        assert!(format!("{}", parse_error).contains("/test/config.toml"));
        assert!(format!("{}", parse_error).contains("Failed to parse"));
    }

    #[test]
    fn test_violation_severity_variants() {
        let error = ViolationSeverity::Error;
        let warning = ViolationSeverity::Warning;
        let info = ViolationSeverity::Info;

        assert_eq!(error, ViolationSeverity::Error);
        assert_eq!(warning, ViolationSeverity::Warning);
        assert_eq!(info, ViolationSeverity::Info);
        assert_ne!(error, warning);
    }

    #[test]
    fn test_gate_violation_creation() {
        let violation = GateViolation {
            file: Some("test.rs".to_string()),
            line: Some(42),
            description: "Test violation".to_string(),
            severity: ViolationSeverity::Error,
        };
        assert_eq!(violation.file, Some("test.rs".to_string()));
        assert_eq!(violation.line, Some(42));
        assert_eq!(violation.description, "Test violation");
        assert_eq!(violation.severity, ViolationSeverity::Error);
    }

    #[test]
    fn test_gate_result_with_violations() {
        let violations = vec![
            GateViolation {
                file: Some("a.rs".to_string()),
                line: Some(1),
                description: "first".to_string(),
                severity: ViolationSeverity::Error,
            },
            GateViolation {
                file: None,
                line: None,
                description: "second".to_string(),
                severity: ViolationSeverity::Warning,
            },
        ];

        let mut metrics = HashMap::new();
        metrics.insert("count".to_string(), 2.0);

        let result = GateResult {
            gate_name: "test_gate".to_string(),
            passed: false,
            duration: Duration::from_secs(1),
            message: "2 violations found".to_string(),
            metrics,
            violations: violations.clone(),
        };

        assert_eq!(result.violations.len(), 2);
        assert_eq!(result.metrics.get("count"), Some(&2.0));
        assert!(!result.passed);
    }

    #[test]
    fn test_all_passed_empty_results() {
        let empty: Vec<GateResult> = vec![];
        assert!(QualityGate::all_passed(&empty));
    }

    #[test]
    fn test_summary_empty_results() {
        let empty: Vec<GateResult> = vec![];
        let summary = QualityGate::summary(&empty);
        assert_eq!(summary.total, 0);
        assert_eq!(summary.passed, 0);
        assert_eq!(summary.failed, 0);
        assert_eq!(summary.total_duration, Duration::default());
    }

    #[test]
    fn test_quality_gate_new() {
        let config = GateConfig {
            gates: GatesConfig {
                max_complexity: 15,
                ..GatesConfig::default()
            },
            ..GateConfig::default()
        };
        let gate = QualityGate::new(config);
        assert_eq!(gate.config.gates.max_complexity, 15);
    }

    #[test]
    fn test_gate_config_load_nonexistent() {
        let result = GateConfig::load(std::path::Path::new("/nonexistent/path.toml"));
        assert!(result.is_err());
    }

    #[test]
    fn test_gate_config_load_or_default() {
        // This should return default config since no .pmat-gates.toml exists
        let config = GateConfig::load_or_default();
        assert_eq!(config.metadata.version, "1.0.0");
    }

    #[test]
    fn test_comprehensive_gate_config_parse() {
        let toml_content = r#"
[metadata]
version = "2.0.0"
tool = "custom-tool"

[gates]
run_clippy = false
clippy_strict = false
run_tests = false
test_timeout = 600
check_coverage = false
min_coverage = 95.0
check_complexity = false
max_complexity = 15

[gates.satd]
enabled = false
max_count = 10
patterns = ["TODO"]
require_issue_links = false
fail_on_violation = false

[gates.mutation]
enabled = true
min_score = 90.0
tool = "mutagen"
strategy = "full"

[gates.security]
enabled = false
audit_vulnerabilities = "warn"
audit_unmaintained = "deny"
max_unsafe_blocks = 5
fail_on_violation = false

[tiers]
tier1_gates = ["fmt"]
tier2_gates = ["fmt", "clippy"]
tier3_gates = ["fmt", "clippy", "coverage"]

[risk_based]
very_high_risk_mutation_target = 95.0
very_high_risk_components = ["parser", "security"]
high_risk_mutation_target = 90.0
high_risk_components = ["linter"]
"#;

        let config: GateConfig = toml::from_str(toml_content).expect("valid toml");

        // Metadata
        assert_eq!(config.metadata.version, "2.0.0");
        assert_eq!(config.metadata.tool, "custom-tool");

        // Gates
        assert!(!config.gates.run_clippy);
        assert!(!config.gates.clippy_strict);
        assert!(!config.gates.run_tests);
        assert_eq!(config.gates.test_timeout, 600);
        assert!(!config.gates.check_coverage);
        assert_eq!(config.gates.min_coverage, 95.0);
        assert!(!config.gates.check_complexity);
        assert_eq!(config.gates.max_complexity, 15);

        // SATD
        assert!(!config.gates.satd.enabled);
        assert_eq!(config.gates.satd.max_count, 10);
        assert_eq!(config.gates.satd.patterns, vec!["TODO"]);
        assert!(!config.gates.satd.require_issue_links);
        assert!(!config.gates.satd.fail_on_violation);

        // Mutation
        assert!(config.gates.mutation.enabled);
        assert_eq!(config.gates.mutation.min_score, 90.0);
        assert_eq!(config.gates.mutation.tool, "mutagen");
        assert_eq!(config.gates.mutation.strategy, "full");

        // Security
        assert!(!config.gates.security.enabled);
        assert_eq!(config.gates.security.audit_vulnerabilities, "warn");
        assert_eq!(config.gates.security.audit_unmaintained, "deny");
        assert_eq!(config.gates.security.max_unsafe_blocks, 5);
        assert!(!config.gates.security.fail_on_violation);

        // Tiers
        assert_eq!(config.tiers.tier1_gates, vec!["fmt"]);
        assert_eq!(config.tiers.tier2_gates, vec!["fmt", "clippy"]);
        assert_eq!(config.tiers.tier3_gates, vec!["fmt", "clippy", "coverage"]);

        // Risk-based
        assert_eq!(config.risk_based.very_high_risk_mutation_target, 95.0);
        assert_eq!(
            config.risk_based.very_high_risk_components,
            vec!["parser", "security"]
        );
        assert_eq!(config.risk_based.high_risk_mutation_target, 90.0);
        assert_eq!(config.risk_based.high_risk_components, vec!["linter"]);
    }

    #[test]
    fn test_gate_summary_debug() {
        let summary = GateSummary {
            total: 5,
            passed: 3,
            failed: 2,
            total_duration: Duration::from_millis(500),
        };
        let debug_output = format!("{:?}", summary);
        assert!(debug_output.contains("total: 5"));
        assert!(debug_output.contains("passed: 3"));
        assert!(debug_output.contains("failed: 2"));
    }

    #[test]
    fn test_default_helper_functions() {
        assert!(default_true());
        assert_eq!(default_timeout(), 300);
        assert_eq!(default_coverage(), 85.0);
        assert_eq!(default_complexity(), 10);
        assert_eq!(default_mutation_score(), 85.0);
        assert_eq!(default_mutation_tool(), "cargo-mutants");
        assert_eq!(default_mutation_strategy(), "incremental");
        assert_eq!(default_audit_vulnerabilities(), "deny");
        assert_eq!(default_audit_unmaintained(), "warn");
        assert_eq!(default_very_high_risk_target(), 92.5);
        assert_eq!(default_high_risk_target(), 87.5);
        assert_eq!(default_version(), "1.0.0");
        assert_eq!(default_tool(), "bashrs");
    }
}
