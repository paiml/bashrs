//! Falsification Test Generator (#121)
//!
//! Implements Karl Popper's falsificationism for installer testing.
//!
//! A claim is only scientific if it can be proven false. This module generates
//! tests designed to DISPROVE specific claims about installer behavior:
//!
//! - Idempotency: Running twice produces same state
//! - Determinism: Same inputs produce same outputs
//! - Rollback completeness: Undo restores original state
//! - Dry-run accuracy: Predictions match execution
//!
//! # Philosophy
//!
//! Traditional testing asks "does it work?" Falsificationism asks "can I break it?"
//!
//! # Example
//!
//! ```ignore
//! use bashrs::installer::{FalsificationGenerator, FalsificationHypothesis};
//!
//! let generator = FalsificationGenerator::new();
//! let hypotheses = generator.analyze_installer(&spec);
//!
//! for h in hypotheses {
//!     println!("Testing: {}", h.claim);
//!     let test = h.generate_test();
//!     // Run test and check if hypothesis is falsified
//! }
//! ```

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A falsifiable hypothesis about installer behavior
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FalsificationHypothesis {
    /// Unique ID for this hypothesis
    pub id: String,
    /// Human-readable claim being tested
    pub claim: String,
    /// Category of the hypothesis
    pub category: HypothesisCategory,
    /// How to falsify this claim
    pub falsification_method: String,
    /// Steps involved in testing
    pub step_ids: Vec<String>,
    /// Expected evidence if hypothesis holds
    pub expected_evidence: String,
    /// Evidence that would falsify the hypothesis
    pub falsifying_evidence: String,
    /// Test priority (higher = more critical)
    pub priority: u8,
}

/// Categories of falsifiable claims
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HypothesisCategory {
    /// Step produces same state when run twice
    Idempotency,
    /// Same inputs always produce same outputs
    Determinism,
    /// Rollback fully restores previous state
    RollbackCompleteness,
    /// Dry-run predictions match actual execution
    DryRunAccuracy,
    /// Postconditions hold after step completion
    PostconditionValidity,
    /// Preconditions prevent invalid execution
    PreconditionGuard,
    /// Step completes within expected time
    PerformanceBound,
    /// Resource usage stays within limits
    ResourceLimit,
}

impl HypothesisCategory {
    /// Get human-readable description
    pub fn description(&self) -> &'static str {
        match self {
            Self::Idempotency => "Running the operation twice produces identical state",
            Self::Determinism => "Same inputs always produce identical outputs",
            Self::RollbackCompleteness => "Rollback fully restores the original state",
            Self::DryRunAccuracy => "Dry-run predictions match actual execution",
            Self::PostconditionValidity => "Postconditions hold after step completion",
            Self::PreconditionGuard => "Preconditions prevent invalid execution",
            Self::PerformanceBound => "Operation completes within time limit",
            Self::ResourceLimit => "Resource usage stays within specified limits",
        }
    }
}

/// A generated test case for falsification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FalsificationTest {
    /// Test name
    pub name: String,
    /// Hypothesis being tested
    pub hypothesis_id: String,
    /// Setup actions before test
    pub setup: Vec<TestAction>,
    /// The test action
    pub action: TestAction,
    /// Verification steps
    pub verification: Vec<Verification>,
    /// Cleanup after test
    pub cleanup: Vec<TestAction>,
}

/// An action in a test case
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TestAction {
    /// Execute a step
    ExecuteStep { step_id: String },
    /// Execute step with specific environment
    ExecuteStepWithEnv {
        step_id: String,
        env: HashMap<String, String>,
    },
    /// Capture system state
    CaptureState { label: String },
    /// Execute rollback
    Rollback { step_id: String },
    /// Execute dry-run
    DryRun { step_id: String },
    /// Wait for duration
    Wait { duration_ms: u64 },
    /// Create file with content
    CreateFile { path: String, content: String },
    /// Remove file
    RemoveFile { path: String },
}

/// Verification to check after action
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Verification {
    /// Compare two captured states
    StatesEqual { state_a: String, state_b: String },
    /// States must differ
    StatesDiffer { state_a: String, state_b: String },
    /// File exists
    FileExists { path: String },
    /// File does not exist
    FileNotExists { path: String },
    /// File has content
    FileContains { path: String, content: String },
    /// Command returns exit code 0
    CommandSucceeds { command: String },
    /// Command returns non-zero
    CommandFails { command: String },
    /// Duration is below threshold
    DurationBelow { max_ms: u64 },
    /// Dry-run matches execution
    DryRunMatchesExecution { step_id: String },
}

/// Result of running a falsification test
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FalsificationResult {
    /// Test that was run
    pub test_name: String,
    /// Hypothesis being tested
    pub hypothesis_id: String,
    /// Whether the hypothesis was falsified (i.e., the test found a bug)
    pub falsified: bool,
    /// Evidence collected
    pub evidence: Vec<Evidence>,
    /// Error message if test failed to run
    pub error: Option<String>,
    /// Duration of the test
    pub duration_ms: u64,
}

/// Evidence collected during test
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Evidence {
    /// What was observed
    pub observation: String,
    /// Expected value (if applicable)
    pub expected: Option<String>,
    /// Actual value (if applicable)
    pub actual: Option<String>,
    /// Whether this evidence supports or falsifies the hypothesis
    pub supports_hypothesis: bool,
}

/// Generator for falsification tests
#[derive(Debug, Default)]
pub struct FalsificationGenerator {
    /// Configuration options
    config: FalsificationConfig,
}

/// Configuration for test generation
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FalsificationConfig {
    /// Generate idempotency tests
    pub test_idempotency: bool,
    /// Generate determinism tests
    pub test_determinism: bool,
    /// Generate rollback tests
    pub test_rollback: bool,
    /// Generate dry-run tests
    pub test_dry_run: bool,
    /// Generate postcondition tests
    pub test_postconditions: bool,
    /// Generate precondition tests
    pub test_preconditions: bool,
    /// Generate performance tests
    pub test_performance: bool,
    /// Maximum number of tests per category
    pub max_tests_per_category: usize,
}

impl FalsificationConfig {
    /// Create config with all tests enabled
    pub fn all() -> Self {
        Self {
            test_idempotency: true,
            test_determinism: true,
            test_rollback: true,
            test_dry_run: true,
            test_postconditions: true,
            test_preconditions: true,
            test_performance: true,
            max_tests_per_category: 10,
        }
    }

    /// Create minimal config for quick testing
    pub fn minimal() -> Self {
        Self {
            test_idempotency: true,
            test_determinism: true,
            test_rollback: false,
            test_dry_run: false,
            test_postconditions: false,
            test_preconditions: false,
            test_performance: false,
            max_tests_per_category: 5,
        }
    }
}

impl FalsificationGenerator {
    /// Create new generator with default config
    pub fn new() -> Self {
        Self {
            config: FalsificationConfig::all(),
        }
    }

    /// Create generator with custom config
    pub fn with_config(config: FalsificationConfig) -> Self {
        Self { config }
    }

    /// Generate hypotheses for an installer specification
    pub fn generate_hypotheses(&self, spec: &InstallerInfo) -> Vec<FalsificationHypothesis> {
        let mut hypotheses = Vec::new();

        for step in &spec.steps {
            // Idempotency hypothesis
            if self.config.test_idempotency {
                hypotheses.push(FalsificationHypothesis {
                    id: format!("IDEM-{}", step.id),
                    claim: format!("Step '{}' is idempotent", step.name),
                    category: HypothesisCategory::Idempotency,
                    falsification_method: "Execute step twice, compare final states".to_string(),
                    step_ids: vec![step.id.clone()],
                    expected_evidence: "State after first run equals state after second run"
                        .to_string(),
                    falsifying_evidence: "States differ after repeated execution".to_string(),
                    priority: 9,
                });
            }

            // Determinism hypothesis
            if self.config.test_determinism {
                hypotheses.push(FalsificationHypothesis {
                    id: format!("DET-{}", step.id),
                    claim: format!("Step '{}' is deterministic", step.name),
                    category: HypothesisCategory::Determinism,
                    falsification_method: "Execute step with same inputs twice, compare outputs"
                        .to_string(),
                    step_ids: vec![step.id.clone()],
                    expected_evidence: "Outputs are byte-identical across runs".to_string(),
                    falsifying_evidence: "Outputs differ between runs with same inputs".to_string(),
                    priority: 9,
                });
            }

            // Rollback hypothesis
            if self.config.test_rollback && step.has_rollback {
                hypotheses.push(FalsificationHypothesis {
                    id: format!("ROLL-{}", step.id),
                    claim: format!("Rollback for '{}' is complete", step.name),
                    category: HypothesisCategory::RollbackCompleteness,
                    falsification_method:
                        "Capture state, execute step, rollback, compare to original state"
                            .to_string(),
                    step_ids: vec![step.id.clone()],
                    expected_evidence: "State after rollback equals state before execution"
                        .to_string(),
                    falsifying_evidence: "State differs after rollback".to_string(),
                    priority: 8,
                });
            }

            // Dry-run accuracy hypothesis
            if self.config.test_dry_run {
                hypotheses.push(FalsificationHypothesis {
                    id: format!("DRY-{}", step.id),
                    claim: format!("Dry-run for '{}' accurately predicts changes", step.name),
                    category: HypothesisCategory::DryRunAccuracy,
                    falsification_method:
                        "Run dry-run, capture prediction, execute, compare to actual".to_string(),
                    step_ids: vec![step.id.clone()],
                    expected_evidence: "Dry-run prediction matches actual execution".to_string(),
                    falsifying_evidence: "Prediction differs from actual changes".to_string(),
                    priority: 7,
                });
            }

            // Postcondition validity
            if self.config.test_postconditions && !step.postconditions.is_empty() {
                for (i, pc) in step.postconditions.iter().enumerate() {
                    hypotheses.push(FalsificationHypothesis {
                        id: format!("POST-{}-{}", step.id, i),
                        claim: format!("Postcondition '{}' holds after '{}'", pc, step.name),
                        category: HypothesisCategory::PostconditionValidity,
                        falsification_method: "Execute step, verify postcondition".to_string(),
                        step_ids: vec![step.id.clone()],
                        expected_evidence: format!("Postcondition '{}' is true", pc),
                        falsifying_evidence: format!("Postcondition '{}' is false", pc),
                        priority: 8,
                    });
                }
            }

            // Precondition guard
            if self.config.test_preconditions && !step.preconditions.is_empty() {
                for (i, pre) in step.preconditions.iter().enumerate() {
                    hypotheses.push(FalsificationHypothesis {
                        id: format!("PRE-{}-{}", step.id, i),
                        claim: format!(
                            "Precondition '{}' prevents invalid execution of '{}'",
                            pre, step.name
                        ),
                        category: HypothesisCategory::PreconditionGuard,
                        falsification_method:
                            "Violate precondition, attempt execution, verify failure".to_string(),
                        step_ids: vec![step.id.clone()],
                        expected_evidence: format!(
                            "Step fails when precondition '{}' is not met",
                            pre
                        ),
                        falsifying_evidence: format!(
                            "Step succeeds despite precondition '{}' being false",
                            pre
                        ),
                        priority: 7,
                    });
                }
            }

            // Performance bounds
            if self.config.test_performance {
                if let Some(max_duration) = step.max_duration_ms {
                    hypotheses.push(FalsificationHypothesis {
                        id: format!("PERF-{}", step.id),
                        claim: format!("Step '{}' completes within {}ms", step.name, max_duration),
                        category: HypothesisCategory::PerformanceBound,
                        falsification_method: "Execute step, measure duration".to_string(),
                        step_ids: vec![step.id.clone()],
                        expected_evidence: format!(
                            "Execution completes in under {}ms",
                            max_duration
                        ),
                        falsifying_evidence: format!("Execution exceeds {}ms", max_duration),
                        priority: 5,
                    });
                }
            }
        }

        // Sort by priority (highest first)
        hypotheses.sort_by(|a, b| b.priority.cmp(&a.priority));
        hypotheses
    }

    /// Generate test cases for hypotheses
    pub fn generate_tests(&self, hypotheses: &[FalsificationHypothesis]) -> Vec<FalsificationTest> {
        hypotheses
            .iter()
            .map(|h| self.generate_test_for_hypothesis(h))
            .collect()
    }

    /// Generate a test for a specific hypothesis
    fn generate_test_for_hypothesis(
        &self,
        hypothesis: &FalsificationHypothesis,
    ) -> FalsificationTest {
        let step_id = hypothesis
            .step_ids
            .first()
            .cloned()
            .unwrap_or_else(|| "unknown".to_string());

        match hypothesis.category {
            HypothesisCategory::Idempotency => FalsificationTest {
                name: format!("test_falsify_idempotency_{}", step_id),
                hypothesis_id: hypothesis.id.clone(),
                setup: vec![TestAction::CaptureState {
                    label: "initial".to_string(),
                }],
                action: TestAction::ExecuteStep {
                    step_id: step_id.clone(),
                },
                verification: vec![Verification::StatesEqual {
                    state_a: "after_first".to_string(),
                    state_b: "after_second".to_string(),
                }],
                cleanup: vec![],
            },
            HypothesisCategory::Determinism => FalsificationTest {
                name: format!("test_falsify_determinism_{}", step_id),
                hypothesis_id: hypothesis.id.clone(),
                setup: vec![TestAction::CaptureState {
                    label: "initial".to_string(),
                }],
                action: TestAction::ExecuteStep {
                    step_id: step_id.clone(),
                },
                verification: vec![Verification::StatesEqual {
                    state_a: "output_1".to_string(),
                    state_b: "output_2".to_string(),
                }],
                cleanup: vec![],
            },
            HypothesisCategory::RollbackCompleteness => FalsificationTest {
                name: format!("test_falsify_rollback_{}", step_id),
                hypothesis_id: hypothesis.id.clone(),
                setup: vec![TestAction::CaptureState {
                    label: "before".to_string(),
                }],
                action: TestAction::ExecuteStep {
                    step_id: step_id.clone(),
                },
                verification: vec![Verification::StatesEqual {
                    state_a: "before".to_string(),
                    state_b: "after_rollback".to_string(),
                }],
                cleanup: vec![TestAction::Rollback {
                    step_id: step_id.clone(),
                }],
            },
            HypothesisCategory::DryRunAccuracy => FalsificationTest {
                name: format!("test_falsify_dry_run_{}", step_id),
                hypothesis_id: hypothesis.id.clone(),
                setup: vec![TestAction::DryRun {
                    step_id: step_id.clone(),
                }],
                action: TestAction::ExecuteStep {
                    step_id: step_id.clone(),
                },
                verification: vec![Verification::DryRunMatchesExecution {
                    step_id: step_id.clone(),
                }],
                cleanup: vec![],
            },
            HypothesisCategory::PostconditionValidity => FalsificationTest {
                name: format!("test_falsify_postcondition_{}", hypothesis.id),
                hypothesis_id: hypothesis.id.clone(),
                setup: vec![],
                action: TestAction::ExecuteStep {
                    step_id: step_id.clone(),
                },
                verification: vec![Verification::CommandSucceeds {
                    command: format!("verify_postcondition_{}", hypothesis.id),
                }],
                cleanup: vec![],
            },
            HypothesisCategory::PreconditionGuard => FalsificationTest {
                name: format!("test_falsify_precondition_{}", hypothesis.id),
                hypothesis_id: hypothesis.id.clone(),
                setup: vec![],
                action: TestAction::ExecuteStep {
                    step_id: step_id.clone(),
                },
                verification: vec![Verification::CommandFails {
                    command: format!("execute_step_{}", step_id),
                }],
                cleanup: vec![],
            },
            HypothesisCategory::PerformanceBound => FalsificationTest {
                name: format!("test_falsify_performance_{}", step_id),
                hypothesis_id: hypothesis.id.clone(),
                setup: vec![],
                action: TestAction::ExecuteStep {
                    step_id: step_id.clone(),
                },
                verification: vec![Verification::DurationBelow {
                    max_ms: 60000, // Default 1 minute
                }],
                cleanup: vec![],
            },
            HypothesisCategory::ResourceLimit => FalsificationTest {
                name: format!("test_falsify_resources_{}", step_id),
                hypothesis_id: hypothesis.id.clone(),
                setup: vec![],
                action: TestAction::ExecuteStep {
                    step_id: step_id.clone(),
                },
                verification: vec![],
                cleanup: vec![],
            },
        }
    }

    /// Generate Rust test code for hypotheses
    pub fn generate_rust_tests(&self, hypotheses: &[FalsificationHypothesis]) -> String {
        let mut code = String::new();

        code.push_str("//! Auto-generated falsification tests\n");
        code.push_str("//! Generated by bashrs falsification test generator\n\n");
        code.push_str("#[cfg(test)]\n");
        code.push_str("mod falsification_tests {\n");
        code.push_str("    use super::*;\n\n");

        for h in hypotheses {
            code.push_str(&format!("    /// FALSIFIABLE: \"{}\"\n", h.claim));
            code.push_str(&format!("    /// DISPROOF: {}\n", h.falsifying_evidence));
            code.push_str("    #[test]\n");
            code.push_str(&format!(
                "    fn test_falsify_{}() {{\n",
                h.id.to_lowercase().replace('-', "_")
            ));
            code.push_str("        // Placeholder: implement with step execution\n");
            code.push_str(&format!("        // Method: {}\n", h.falsification_method));
            code.push_str(&format!("        // Expected: {}\n", h.expected_evidence));
            code.push_str("        assert!(true, \"Implement falsification test\");\n");
            code.push_str("    }\n\n");
        }

        code.push_str("}\n");
        code
    }
}

/// Minimal installer info for test generation
#[derive(Debug, Clone, Default)]
pub struct InstallerInfo {
    /// Installer name
    pub name: String,
    /// Installer version
    pub version: String,
    /// Steps in the installer
    pub steps: Vec<StepInfo>,
}

/// Minimal step info for test generation
#[derive(Debug, Clone, Default)]
pub struct StepInfo {
    /// Step ID
    pub id: String,
    /// Step name
    pub name: String,
    /// Whether step has rollback
    pub has_rollback: bool,
    /// Preconditions
    pub preconditions: Vec<String>,
    /// Postconditions
    pub postconditions: Vec<String>,
    /// Max duration in ms
    pub max_duration_ms: Option<u64>,
}

/// Summary report of falsification testing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FalsificationReport {
    /// Installer tested
    pub installer_name: String,
    /// Total hypotheses tested
    pub total_hypotheses: usize,
    /// Hypotheses that were falsified (bugs found)
    pub falsified_count: usize,
    /// Hypotheses that held (no bugs found)
    pub validated_count: usize,
    /// Tests that failed to run
    pub error_count: usize,
    /// Results by category
    pub by_category: HashMap<String, CategorySummary>,
    /// All results
    pub results: Vec<FalsificationResult>,
}

/// Summary for a category
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CategorySummary {
    /// Total tests in category
    pub total: usize,
    /// Tests that falsified hypothesis
    pub falsified: usize,
    /// Tests that validated hypothesis
    pub validated: usize,
}

impl FalsificationReport {
    /// Create report from results
    pub fn from_results(
        installer_name: &str,
        results: Vec<FalsificationResult>,
        hypotheses: &[FalsificationHypothesis],
    ) -> Self {
        let mut by_category: HashMap<String, CategorySummary> = HashMap::new();

        let mut falsified_count = 0;
        let mut validated_count = 0;
        let mut error_count = 0;

        for result in &results {
            if result.error.is_some() {
                error_count += 1;
                continue;
            }

            if result.falsified {
                falsified_count += 1;
            } else {
                validated_count += 1;
            }

            // Find hypothesis category
            if let Some(h) = hypotheses.iter().find(|h| h.id == result.hypothesis_id) {
                let cat = format!("{:?}", h.category);
                let entry = by_category.entry(cat).or_default();
                entry.total += 1;
                if result.falsified {
                    entry.falsified += 1;
                } else {
                    entry.validated += 1;
                }
            }
        }

        Self {
            installer_name: installer_name.to_string(),
            total_hypotheses: results.len(),
            falsified_count,
            validated_count,
            error_count,
            by_category,
            results,
        }
    }

    /// Format as human-readable report
    pub fn format(&self) -> String {
        let mut report = String::new();

        report.push_str(&format!("Falsification Report: {}\n", self.installer_name));
        report.push_str(&"=".repeat(50));
        report.push('\n');

        report.push_str(&format!(
            "Total hypotheses tested: {}\n",
            self.total_hypotheses
        ));
        report.push_str(&format!(
            "  ✓ Validated: {} (no bugs found)\n",
            self.validated_count
        ));
        report.push_str(&format!(
            "  ✗ Falsified: {} (bugs found!)\n",
            self.falsified_count
        ));
        if self.error_count > 0 {
            report.push_str(&format!("  ⚠ Errors: {}\n", self.error_count));
        }

        report.push_str("\nBy Category:\n");
        for (cat, summary) in &self.by_category {
            report.push_str(&format!(
                "  {}: {}/{} validated\n",
                cat, summary.validated, summary.total
            ));
        }

        if self.falsified_count > 0 {
            report.push_str("\nFalsified Hypotheses (Bugs Found):\n");
            for result in &self.results {
                if result.falsified {
                    report.push_str(&format!("  - {}\n", result.hypothesis_id));
                    for evidence in &result.evidence {
                        if !evidence.supports_hypothesis {
                            report.push_str(&format!("    → {}\n", evidence.observation));
                        }
                    }
                }
            }
        }

        report
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_installer() -> InstallerInfo {
        InstallerInfo {
            name: "test-installer".to_string(),
            version: "1.0.0".to_string(),
            steps: vec![
                StepInfo {
                    id: "install-deps".to_string(),
                    name: "Install Dependencies".to_string(),
                    has_rollback: true,
                    preconditions: vec!["network_available".to_string()],
                    postconditions: vec!["deps_installed".to_string()],
                    max_duration_ms: Some(120000),
                },
                StepInfo {
                    id: "configure".to_string(),
                    name: "Configure Application".to_string(),
                    has_rollback: true,
                    preconditions: vec![],
                    postconditions: vec!["config_valid".to_string()],
                    max_duration_ms: None,
                },
            ],
        }
    }

    #[test]
    fn test_FALSIFY_001_generator_new() {
        let gen = FalsificationGenerator::new();
        assert!(gen.config.test_idempotency);
        assert!(gen.config.test_determinism);
    }

    #[test]
    fn test_FALSIFY_002_config_all() {
        let config = FalsificationConfig::all();
        assert!(config.test_idempotency);
        assert!(config.test_rollback);
        assert!(config.test_performance);
        assert_eq!(config.max_tests_per_category, 10);
    }

    #[test]
    fn test_FALSIFY_003_config_minimal() {
        let config = FalsificationConfig::minimal();
        assert!(config.test_idempotency);
        assert!(config.test_determinism);
        assert!(!config.test_rollback);
        assert!(!config.test_performance);
    }

    #[test]
    fn test_FALSIFY_004_generate_hypotheses() {
        let gen = FalsificationGenerator::new();
        let installer = sample_installer();
        let hypotheses = gen.generate_hypotheses(&installer);

        assert!(!hypotheses.is_empty());

        // Should have idempotency hypotheses for each step
        let idem_count = hypotheses
            .iter()
            .filter(|h| h.category == HypothesisCategory::Idempotency)
            .count();
        assert_eq!(idem_count, 2);
    }

    #[test]
    fn test_FALSIFY_005_generate_determinism_hypotheses() {
        let gen = FalsificationGenerator::new();
        let installer = sample_installer();
        let hypotheses = gen.generate_hypotheses(&installer);

        let det_count = hypotheses
            .iter()
            .filter(|h| h.category == HypothesisCategory::Determinism)
            .count();
        assert_eq!(det_count, 2);
    }

    #[test]
    fn test_FALSIFY_006_generate_rollback_hypotheses() {
        let gen = FalsificationGenerator::new();
        let installer = sample_installer();
        let hypotheses = gen.generate_hypotheses(&installer);

        let roll_count = hypotheses
            .iter()
            .filter(|h| h.category == HypothesisCategory::RollbackCompleteness)
            .count();
        assert_eq!(roll_count, 2); // Both steps have rollback
    }

    #[test]
    fn test_FALSIFY_007_generate_precondition_hypotheses() {
        let gen = FalsificationGenerator::new();
        let installer = sample_installer();
        let hypotheses = gen.generate_hypotheses(&installer);

        let pre_count = hypotheses
            .iter()
            .filter(|h| h.category == HypothesisCategory::PreconditionGuard)
            .count();
        assert_eq!(pre_count, 1); // Only first step has preconditions
    }

    #[test]
    fn test_FALSIFY_008_generate_postcondition_hypotheses() {
        let gen = FalsificationGenerator::new();
        let installer = sample_installer();
        let hypotheses = gen.generate_hypotheses(&installer);

        let post_count = hypotheses
            .iter()
            .filter(|h| h.category == HypothesisCategory::PostconditionValidity)
            .count();
        assert_eq!(post_count, 2); // Both steps have postconditions
    }

    #[test]
    fn test_FALSIFY_009_generate_performance_hypotheses() {
        let gen = FalsificationGenerator::new();
        let installer = sample_installer();
        let hypotheses = gen.generate_hypotheses(&installer);

        let perf_count = hypotheses
            .iter()
            .filter(|h| h.category == HypothesisCategory::PerformanceBound)
            .count();
        assert_eq!(perf_count, 1); // Only first step has max_duration
    }

    #[test]
    fn test_FALSIFY_010_hypothesis_priority() {
        let gen = FalsificationGenerator::new();
        let installer = sample_installer();
        let hypotheses = gen.generate_hypotheses(&installer);

        // Hypotheses should be sorted by priority (highest first)
        let priorities: Vec<u8> = hypotheses.iter().map(|h| h.priority).collect();
        for i in 1..priorities.len() {
            assert!(priorities[i - 1] >= priorities[i]);
        }
    }

    #[test]
    fn test_FALSIFY_011_generate_tests() {
        let gen = FalsificationGenerator::new();
        let installer = sample_installer();
        let hypotheses = gen.generate_hypotheses(&installer);
        let tests = gen.generate_tests(&hypotheses);

        assert_eq!(tests.len(), hypotheses.len());
    }

    #[test]
    fn test_FALSIFY_012_generate_rust_tests() {
        let gen = FalsificationGenerator::new();
        let installer = sample_installer();
        let hypotheses = gen.generate_hypotheses(&installer);
        let code = gen.generate_rust_tests(&hypotheses);

        assert!(code.contains("#[test]"));
        assert!(code.contains("test_falsify_"));
        assert!(code.contains("FALSIFIABLE"));
    }

    #[test]
    fn test_FALSIFY_013_hypothesis_category_description() {
        assert!(!HypothesisCategory::Idempotency.description().is_empty());
        assert!(!HypothesisCategory::Determinism.description().is_empty());
        assert!(!HypothesisCategory::RollbackCompleteness
            .description()
            .is_empty());
    }

    #[test]
    fn test_FALSIFY_014_falsification_report() {
        let results = vec![
            FalsificationResult {
                test_name: "test_1".to_string(),
                hypothesis_id: "IDEM-install-deps".to_string(),
                falsified: false,
                evidence: vec![],
                error: None,
                duration_ms: 100,
            },
            FalsificationResult {
                test_name: "test_2".to_string(),
                hypothesis_id: "DET-configure".to_string(),
                falsified: true,
                evidence: vec![Evidence {
                    observation: "Output differs".to_string(),
                    expected: Some("hash1".to_string()),
                    actual: Some("hash2".to_string()),
                    supports_hypothesis: false,
                }],
                error: None,
                duration_ms: 200,
            },
        ];

        let hypotheses = vec![
            FalsificationHypothesis {
                id: "IDEM-install-deps".to_string(),
                claim: "Step is idempotent".to_string(),
                category: HypothesisCategory::Idempotency,
                falsification_method: "Execute twice".to_string(),
                step_ids: vec!["install-deps".to_string()],
                expected_evidence: "States equal".to_string(),
                falsifying_evidence: "States differ".to_string(),
                priority: 9,
            },
            FalsificationHypothesis {
                id: "DET-configure".to_string(),
                claim: "Step is deterministic".to_string(),
                category: HypothesisCategory::Determinism,
                falsification_method: "Execute twice".to_string(),
                step_ids: vec!["configure".to_string()],
                expected_evidence: "Outputs equal".to_string(),
                falsifying_evidence: "Outputs differ".to_string(),
                priority: 9,
            },
        ];

        let report = FalsificationReport::from_results("test", results, &hypotheses);

        assert_eq!(report.total_hypotheses, 2);
        assert_eq!(report.validated_count, 1);
        assert_eq!(report.falsified_count, 1);
    }

    #[test]
    fn test_FALSIFY_015_report_format() {
        let results = vec![FalsificationResult {
            test_name: "test_1".to_string(),
            hypothesis_id: "IDEM-step1".to_string(),
            falsified: false,
            evidence: vec![],
            error: None,
            duration_ms: 100,
        }];

        let report = FalsificationReport::from_results("test-installer", results, &[]);
        let formatted = report.format();

        assert!(formatted.contains("Falsification Report"));
        assert!(formatted.contains("test-installer"));
        assert!(formatted.contains("Validated"));
    }
}
