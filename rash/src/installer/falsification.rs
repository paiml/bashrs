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

include!("falsification_falsificationgenerator.rs");
