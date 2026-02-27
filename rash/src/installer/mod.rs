//! TDD-First Installer Framework (v2.0.0)
//!
//! This module provides a declarative installer framework that generates
//! pure Rust installers with:
//!
//! - TDD by default - Tests exist before implementation
//! - Checkpointed - Resume from any failure point
//! - Observable - Visual progress, structured logging, tracing
//! - Deterministic - Same inputs always produce same outputs
//! - Falsifiable - Every claim can be empirically tested
//! - Cryptographically Verified - Ed25519 signatures on all artifacts
//! - Hermetically Reproducible - Bit-for-bit identical builds across machines
//! - Container-Native - First-class multi-distro testing in isolation
//!
//! # Philosophy
//!
//! Apply Toyota Production System (TPS) principles and Karl Popper's
//! falsificationism to installer engineering.
//!
//! # Usage
//!
//! ```bash
//! bashrs installer init my-app-installer
//! bashrs installer validate ./my-installer
//! bashrs installer run ./my-installer --dry-run --diff
//! ```

#[allow(clippy::expect_used)] // Audit uses expect() for internal invariants
mod audit;
mod checkpoint;
mod container;
#[allow(clippy::expect_used)] // Distributed uses expect() for internal invariants
mod distributed;
mod dry_run;
#[allow(clippy::expect_used)] // Executor uses expect() for internal invariants
mod executor;
mod falsification;
mod from_bash;
#[allow(clippy::expect_used)] // Golden trace uses expect() for internal invariants
mod golden_trace;
mod hermetic;
#[allow(clippy::expect_used)] // Metrics uses expect() for internal invariants
mod metrics;
mod plan;
mod progress;
mod rollback;
mod signature;
#[allow(clippy::expect_used)] // Spec uses expect() for internal invariants
mod spec;
mod tracing;

#[cfg(test)]
mod tests;

#[cfg(test)]
#[path = "audit_tests.rs"]
mod audit_tests;

#[cfg(test)]
#[path = "checkpoint_tests.rs"]
mod checkpoint_tests;

#[cfg(test)]
#[path = "dry_run_tests.rs"]
mod dry_run_tests;

#[cfg(test)]
#[path = "from_bash_coverage_tests.rs"]
mod from_bash_coverage_tests;

#[cfg(test)]
#[path = "installer_tests.rs"]
mod installer_tests;

#[cfg(test)]
#[path = "executor_cov_tests.rs"]
mod executor_cov_tests;

pub use audit::{
    AuditCategory, AuditContext, AuditFinding, AuditMetadata, AuditReport, AuditSeverity,
};
pub use checkpoint::{
    CheckpointStore, InstallerRun, RunStatus, StateFile, StepCheckpoint as CheckpointEntry,
    StepStatus,
};
pub use container::{
    Architecture, ContainerConfig, ContainerRuntime, ContainerTestMatrix, MatrixConfig,
    MatrixSummary, Platform, PlatformResult, ResourceLimits, StepTestResult, TestStatus,
};
pub use distributed::{
    format_execution_plan, CacheStats, DistributedConfig, ExecutionPlan, ExecutionWave, GraphNode,
    InstallerGraph, OptimizationConfig, RemoteExecutor, SccacheClient,
};
pub use dry_run::{
    DiffPreview, DryRunContext, DryRunSummary, FileChange, FileChangeType, PackageOperation,
    ServiceOperation, SimulationEntry, UserGroupOperation,
};
pub use executor::{ExecutorConfig, PostconditionResult, StepExecutionResult, StepExecutor};
pub use falsification::{
    CategorySummary, Evidence, FalsificationConfig, FalsificationGenerator,
    FalsificationHypothesis, FalsificationReport, FalsificationResult, FalsificationTest,
    HypothesisCategory, InstallerInfo as FalsificationInstallerInfo,
    StepInfo as FalsificationStepInfo, TestAction, Verification,
};
pub use from_bash::{
    convert_bash_to_installer, convert_file_to_project, ConversionResult, ConversionStats,
};
pub use golden_trace::{
    ComparisonMetadata, GoldenTrace, GoldenTraceConfig, GoldenTraceManager,
    SimulatedTraceCollector, TraceComparison, TraceEvent, TraceEventType, TraceResult,
};
pub use hermetic::{
    HermeticContext, LockedArtifact, Lockfile, LockfileEnvironment, LOCKFILE_VERSION,
};
pub use metrics::{
    format_metrics_report, AggregatedMetrics, EnvironmentInfo, InstallerMetrics, KaizenReport,
    MetricsAggregator, MetricsCollector, StepAggregate, StepMetrics, StepOutcome,
};
pub use plan::InstallerPlan;
pub use progress::{
    generate_summary, ExecutionMode, InstallationSummary, InstallerProgress, JsonRenderer,
    LiveProgress, ProgressRenderer, ProgressStyle, StepInfo, StepResult, StepState,
    TerminalRenderer,
};
pub use rollback::{RollbackAction, RollbackManager, RollbackPlan, StateFileBackup, StepRollback};
pub use signature::{
    compute_sha256, create_test_signature, verify_sha256, verify_signature, ArtifactSpec, Keyring,
    PublicKey, Sha256Hash, Signature, TrustDecision, TrustedKey, VerificationResult,
};
pub use spec::{
    Action, Artifact, Environment, InstallerSecurity, InstallerSpec, Postcondition, Precondition,
    Requirements, Step, StepCheckpoint, StepTiming,
};
pub use tracing::{
    AttributeValue, LogEntry, Logger, Span, SpanEvent, SpanKind, SpanStatus, TraceExporter,
    TraceLevel, TraceSummary, TracingContext,
};

use crate::models::{Error, Result};
use std::path::Path;

/// Initialize a new installer project with TDD-first test harness.
///
/// Creates the following structure:
/// ```text
/// <name>/
/// ├── installer.toml     # Declarative installer specification
/// ├── tests/
/// │   ├── mod.rs         # Test module
/// │   └── falsification.rs # Popper-style falsification tests
/// └── templates/         # Optional template files
/// ```
pub fn init_project(name: &Path, description: Option<&str>) -> Result<InstallerProject> {
    // Validate project name
    let project_name = name
        .file_name()
        .and_then(|n| n.to_str())
        .ok_or_else(|| Error::Validation("Invalid project name".to_string()))?;

    if project_name.is_empty() {
        return Err(Error::Validation(
            "Project name cannot be empty".to_string(),
        ));
    }

    // Create project directory
    std::fs::create_dir_all(name).map_err(|e| {
        Error::Io(std::io::Error::new(
            e.kind(),
            format!("Failed to create project directory: {e}"),
        ))
    })?;

    // Create tests directory
    std::fs::create_dir_all(name.join("tests")).map_err(|e| {
        Error::Io(std::io::Error::new(
            e.kind(),
            format!("Failed to create tests directory: {e}"),
        ))
    })?;

    // Create templates directory
    std::fs::create_dir_all(name.join("templates")).map_err(|e| {
        Error::Io(std::io::Error::new(
            e.kind(),
            format!("Failed to create templates directory: {e}"),
        ))
    })?;

    // Generate installer.toml
    let installer_toml = generate_installer_toml(project_name, description);
    std::fs::write(name.join("installer.toml"), installer_toml).map_err(|e| {
        Error::Io(std::io::Error::new(
            e.kind(),
            format!("Failed to write installer.toml: {e}"),
        ))
    })?;

    // Generate test files
    let test_mod = generate_test_mod(project_name);
    std::fs::write(name.join("tests").join("mod.rs"), test_mod).map_err(|e| {
        Error::Io(std::io::Error::new(
            e.kind(),
            format!("Failed to write tests/mod.rs: {e}"),
        ))
    })?;

    let falsification_tests = generate_falsification_tests(project_name);
    std::fs::write(
        name.join("tests").join("falsification.rs"),
        falsification_tests,
    )
    .map_err(|e| {
        Error::Io(std::io::Error::new(
            e.kind(),
            format!("Failed to write tests/falsification.rs: {e}"),
        ))
    })?;

    Ok(InstallerProject {
        name: project_name.to_string(),
        path: name.to_path_buf(),
        description: description.map(|s| s.to_string()),
    })
}

/// Validate an installer specification without executing it.
///
/// Checks:
/// - TOML syntax is valid
/// - All required fields are present
/// - Step dependencies form a valid DAG (no cycles)
/// - All referenced artifacts exist
/// - All preconditions/postconditions are well-formed
pub fn validate_installer(path: &Path) -> Result<ValidationResult> {
    // Find installer.toml
    let installer_toml = if path.is_dir() {
        path.join("installer.toml")
    } else {
        path.to_path_buf()
    };

    if !installer_toml.exists() {
        return Err(Error::Validation(format!(
            "installer.toml not found at {}",
            installer_toml.display()
        )));
    }

    // Parse the TOML
    let content = std::fs::read_to_string(&installer_toml).map_err(|e| {
        Error::Io(std::io::Error::new(
            e.kind(),
            format!("Failed to read installer.toml: {e}"),
        ))
    })?;

    let spec = InstallerSpec::parse(&content)?;

    // Validate the spec
    let plan = InstallerPlan::from_spec(spec)?;

    Ok(ValidationResult {
        valid: true,
        steps: plan.steps().len(),
        artifacts: plan.artifacts().len(),
        warnings: Vec::new(),
        errors: Vec::new(),
    })
}

/// Result of initializing an installer project
#[derive(Debug, Clone)]
pub struct InstallerProject {
    /// Project name
    pub name: String,
    /// Path to the project directory
    pub path: std::path::PathBuf,
    /// Optional description
    pub description: Option<String>,
}

/// Result of validating an installer
#[derive(Debug, Clone)]
pub struct ValidationResult {
    /// Whether the installer is valid
    pub valid: bool,
    /// Number of steps
    pub steps: usize,
    /// Number of artifacts
    pub artifacts: usize,
    /// Warnings found during validation
    pub warnings: Vec<String>,
    /// Errors found during validation
    pub errors: Vec<String>,
}

/// Generate initial installer.toml content
fn generate_installer_toml(name: &str, description: Option<&str>) -> String {
    let desc = description.unwrap_or("A TDD-first installer generated by bashrs");
    format!(
        r#"# Installer specification for {name}
# Generated by bashrs installer init

[installer]
name = "{name}"
version = "1.0.0"
description = "{desc}"
author = ""

[installer.requirements]
# Supported operating systems
os = ["ubuntu >= 20.04", "debian >= 11"]
# Required architectures
arch = ["x86_64", "aarch64"]
# Required privileges ("root" or "user")
privileges = "user"
# Network access required
network = false

[installer.environment]
# Environment variables with defaults
# EXAMPLE_VAR = {{ default = "value", validate = "string" }}

[installer.security]
# Trust model: "keyring" (explicit) or "tofu" (Trust On First Use)
trust_model = "tofu"
# Require signatures for all external artifacts
require_signatures = false

# =============================================================================
# Artifacts: Externally-sourced files with verification
# =============================================================================

# [[artifact]]
# id = "example-artifact"
# url = "https://example.com/file.tar.gz"
# sha256 = "..."
# signature = "https://example.com/file.tar.gz.sig"
# signed_by = "example-key"

# =============================================================================
# Steps: Each step is atomic, idempotent, and testable
# =============================================================================

[[step]]
id = "hello-world"
name = "Hello World Example Step"
action = "script"

[step.script]
interpreter = "sh"
content = '''
echo "Hello from {name} installer!"
'''

[step.preconditions]
# Preconditions that must be true before this step runs

[step.postconditions]
# Postconditions that must be true after this step completes

[step.checkpoint]
enabled = true
"#
    )
}

/// Generate tests/mod.rs content
fn generate_test_mod(name: &str) -> String {
    format!(
        r"//! Test module for {name} installer
//!
//! These tests follow EXTREME TDD principles:
//! - RED: Write failing test first
//! - GREEN: Implement to make test pass
//! - REFACTOR: Clean up while keeping tests green

mod falsification;

// Re-export all tests
pub use falsification::*;
"
    )
}

/// Generate tests/falsification.rs content with Popper-style tests
fn generate_falsification_tests(name: &str) -> String {
    format!(
        r#"//! Falsification tests for {name} installer
//!
//! Karl Popper's Falsificationism: A claim is only scientific if it can be proven false.
//! Each test here is designed to DISPROVE a specific claim about the installer.
//!
//! Implement these tests as you add execution capabilities to your installer.

/// FALSIFIABLE: "Every step is idempotent"
/// DISPROOF: Run step twice, system state differs
#[test]
fn falsify_step_idempotency() {{
    // Placeholder: implement when step execution is added
    // Example test structure:
    // let step = load_step("hello-world");
    // let state1 = execute_and_capture_state(&step);
    // let state2 = execute_and_capture_state(&step);
    // assert_eq!(state1, state2, "FALSIFIED: Step is not idempotent");
    assert!(true, "Test placeholder - implement with step execution");
}}

/// FALSIFIABLE: "Dry-run accurately predicts changes"
/// DISPROOF: Dry-run prediction differs from actual execution
#[test]
fn falsify_dry_run_accuracy() {{
    // Placeholder: implement when dry-run diff is added
    // Example test structure:
    // let predicted = execute_dry_run(&installer);
    // let actual = execute_and_capture_diff(&installer);
    // assert_eq!(predicted, actual, "FALSIFIED: Dry-run inaccurate");
    assert!(true, "Test placeholder - implement with dry-run diff");
}}

/// FALSIFIABLE: "Rollback restores original state"
/// DISPROOF: State after rollback differs from state before execution
#[test]
fn falsify_rollback_completeness() {{
    // Placeholder: implement when rollback is added
    // Example test structure:
    // let state_before = capture_state();
    // execute_step(&step);
    // rollback_step(&step);
    // let state_after = capture_state();
    // assert_eq!(state_before, state_after, "FALSIFIED: Rollback incomplete");
    assert!(true, "Test placeholder - implement with rollback");
}}
"#
    )
}
