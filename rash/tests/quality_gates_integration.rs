#![allow(clippy::expect_used)]
//! Integration Tests for Quality Gates (ML-001 to ML-003)
//!
//! Tests the quality gate system using mock executors to verify
//! gate logic without running actual external commands.
//!
//! # Test Categories
//!
//! 1. Configuration parsing and defaults
//! 2. Gate execution with mock commands
//! 3. Tier filtering and ordering
//! 4. Result aggregation and reporting

use bashrs::quality::{
    mock_executor::{CommandExecutor, CommandResult, MockCommandExecutor},
    GateConfig, Tier,
};

// ============================================================================
// Configuration Tests
// ============================================================================

#[test]
fn test_gate_config_default_values() {
    let config = GateConfig::default();

    // Verify default metadata
    assert_eq!(config.metadata.version, "1.0.0");
    assert_eq!(config.metadata.tool, "bashrs");

    // Verify default gates
    assert!(config.gates.run_clippy);
    assert!(config.gates.clippy_strict);
    assert!(config.gates.run_tests);
    assert_eq!(config.gates.test_timeout, 300);
    assert!(config.gates.check_coverage);
    assert!((config.gates.min_coverage - 85.0).abs() < f64::EPSILON);
    assert!(config.gates.check_complexity);
    assert_eq!(config.gates.max_complexity, 10);
}

#[test]
fn test_gate_config_from_toml() {
    let toml = r#"
[metadata]
version = "2.0.0"
tool = "custom-tool"

[gates]
run_clippy = false
min_coverage = 95.0
max_complexity = 15

[tiers]
tier1 = ["clippy", "complexity"]
tier2 = ["tests", "coverage"]
tier3 = ["mutation", "security"]
"#;

    let config: GateConfig = toml::from_str(toml).expect("should parse");
    assert_eq!(config.metadata.version, "2.0.0");
    assert_eq!(config.metadata.tool, "custom-tool");
    assert!(!config.gates.run_clippy);
    assert!((config.gates.min_coverage - 95.0).abs() < f64::EPSILON);
    assert_eq!(config.gates.max_complexity, 15);
}

// ============================================================================
// Tier Tests
// ============================================================================

#[test]
fn test_tier_ordering() {
    assert!(Tier::Tier1 < Tier::Tier2);
    assert!(Tier::Tier2 < Tier::Tier3);
    assert!(Tier::Tier1 < Tier::Tier3);
}

#[test]
fn test_tier_from_u8() {
    assert_eq!(Tier::from(1), Tier::Tier1);
    assert_eq!(Tier::from(2), Tier::Tier2);
    assert_eq!(Tier::from(3), Tier::Tier3);
    assert_eq!(Tier::from(99), Tier::Tier3); // Default for invalid
}

#[test]
fn test_tier_display() {
    assert_eq!(format!("{}", Tier::Tier1), "Tier 1 (ON-SAVE)");
    assert_eq!(format!("{}", Tier::Tier2), "Tier 2 (ON-COMMIT)");
    assert_eq!(format!("{}", Tier::Tier3), "Tier 3 (NIGHTLY)");
}

// ============================================================================
// Mock Executor Tests
// ============================================================================

#[test]
fn test_mock_executor_cargo_clippy_success() {
    let mut mock = MockCommandExecutor::new();
    mock.register(
        "cargo",
        &["clippy", "--lib", "-p", "bashrs", "--message-format=json"],
        CommandResult::success(""),
    );

    let result = mock
        .execute(
            "cargo",
            &["clippy", "--lib", "-p", "bashrs", "--message-format=json"],
        )
        .expect("should succeed");

    assert!(result.success);
    assert_eq!(result.exit_code, 0);
}

#[test]
fn test_mock_executor_cargo_clippy_failure() {
    let mut mock = MockCommandExecutor::new();
    mock.register(
        "cargo",
        &["clippy", "--lib", "-p", "bashrs", "--message-format=json"],
        CommandResult::failure(r#"{"level":"error","message":"unused variable"}"#, 1),
    );

    let result = mock
        .execute(
            "cargo",
            &["clippy", "--lib", "-p", "bashrs", "--message-format=json"],
        )
        .expect("should return result");

    assert!(!result.success);
    assert_eq!(result.exit_code, 1);
    assert!(result.stderr.contains("error"));
}

#[test]
fn test_mock_executor_cargo_test_success() {
    let mut mock = MockCommandExecutor::new();
    mock.register(
        "cargo",
        &["test", "--lib"],
        CommandResult::with_output(true, "test result: ok. 100 passed; 0 failed", ""),
    );

    let result = mock
        .execute("cargo", &["test", "--lib"])
        .expect("should succeed");
    assert!(result.success);
    assert!(result.stdout.contains("100 passed"));
}

#[test]
fn test_mock_executor_pmat_complexity() {
    let mut mock = MockCommandExecutor::new();
    mock.register(
        "pmat",
        &["analyze", "complexity", "--path", ".", "--max", "10"],
        CommandResult::success("All functions below complexity threshold"),
    );

    let result = mock
        .execute(
            "pmat",
            &["analyze", "complexity", "--path", ".", "--max", "10"],
        )
        .expect("should succeed");

    assert!(result.success);
    assert!(result.stdout.contains("complexity"));
}

#[test]
fn test_mock_executor_grep_satd() {
    let mut mock = MockCommandExecutor::new();
    mock.register(
        "grep",
        &["-rn", "TODO", "src/"],
        CommandResult::with_output(true, "src/main.rs:42: // TODO: implement feature", ""),
    );

    let result = mock
        .execute("grep", &["-rn", "TODO", "src/"])
        .expect("should succeed");
    assert!(result.success);
    assert!(result.stdout.contains("TODO"));
}

#[test]
fn test_mock_executor_cargo_audit() {
    let mut mock = MockCommandExecutor::new();
    mock.register(
        "cargo",
        &["audit"],
        CommandResult::success("No vulnerabilities found"),
    );

    let result = mock.execute("cargo", &["audit"]).expect("should succeed");
    assert!(result.success);
    assert!(result.stdout.contains("No vulnerabilities"));
}

// ============================================================================
// Gate Configuration TOML Tests
// ============================================================================

#[test]
fn test_satd_config_parsing() {
    let toml = r#"
[gates.satd]
enabled = true
patterns = ["TODO", "FIXME", "HACK", "XXX"]
max_count = 5
"#;

    let config: GateConfig = toml::from_str(toml).expect("should parse");
    assert!(config.gates.satd.enabled);
    assert_eq!(config.gates.satd.patterns.len(), 4);
    assert!(config.gates.satd.patterns.contains(&"TODO".to_string()));
    assert_eq!(config.gates.satd.max_count, 5);
}

#[test]
fn test_mutation_config_parsing() {
    let toml = r#"
[gates.mutation]
enabled = true
min_score = 80.0
tool = "cargo-mutants"
strategy = "incremental"
"#;

    let config: GateConfig = toml::from_str(toml).expect("should parse");
    assert!(config.gates.mutation.enabled);
    assert!((config.gates.mutation.min_score - 80.0).abs() < f64::EPSILON);
    assert_eq!(config.gates.mutation.tool, "cargo-mutants");
    assert_eq!(config.gates.mutation.strategy, "incremental");
}

#[test]
fn test_security_config_parsing() {
    let toml = r#"
[gates.security]
enabled = true
audit_vulnerabilities = "deny"
audit_unmaintained = "warn"
"#;

    let config: GateConfig = toml::from_str(toml).expect("should parse");
    assert!(config.gates.security.enabled);
    assert_eq!(config.gates.security.audit_vulnerabilities, "deny");
    assert_eq!(config.gates.security.audit_unmaintained, "warn");
}

#[test]
fn test_risk_based_config_parsing() {
    let toml = r#"
[risk_based]
very_high_risk_mutation_target = 95.0
high_risk_mutation_target = 90.0
very_high_risk_components = ["security", "core"]
high_risk_components = ["parser", "linter"]
"#;

    let config: GateConfig = toml::from_str(toml).expect("should parse");
    assert!((config.risk_based.very_high_risk_mutation_target - 95.0).abs() < f64::EPSILON);
    assert!((config.risk_based.high_risk_mutation_target - 90.0).abs() < f64::EPSILON);
    assert!(config
        .risk_based
        .very_high_risk_components
        .contains(&"security".to_string()));
}

#[test]
fn test_tiers_config_parsing() {
    let toml = r#"
[tiers]
tier1_gates = ["clippy", "complexity"]
tier2_gates = ["tests", "coverage", "satd"]
tier3_gates = ["mutation", "security"]
"#;

    let config: GateConfig = toml::from_str(toml).expect("should parse");
    assert_eq!(config.tiers.tier1_gates, vec!["clippy", "complexity"]);
    assert_eq!(config.tiers.tier2_gates, vec!["tests", "coverage", "satd"]);
    assert_eq!(config.tiers.tier3_gates, vec!["mutation", "security"]);
}

// ============================================================================
// Complete Configuration Test
// ============================================================================

#[test]
fn test_complete_config() {
    let toml = r#"
[metadata]
version = "1.0.0"
tool = "bashrs"

[gates]
run_clippy = true
clippy_strict = true
run_tests = true
test_timeout = 300
check_coverage = true
min_coverage = 85.0
check_complexity = true
max_complexity = 10

[gates.satd]
enabled = true
patterns = ["TODO", "FIXME"]
max_count = 10

[gates.mutation]
enabled = true
min_score = 85.0
tool = "cargo-mutants"
strategy = "incremental"

[gates.security]
enabled = true
audit_vulnerabilities = "deny"
audit_unmaintained = "warn"

[tiers]
tier1_gates = ["clippy", "complexity"]
tier2_gates = ["tests", "coverage", "satd"]
tier3_gates = ["mutation", "security"]

[risk_based]
very_high_risk_mutation_target = 92.5
high_risk_mutation_target = 87.5
"#;

    let config: GateConfig = toml::from_str(toml).expect("should parse");

    // Verify all sections
    assert_eq!(config.metadata.version, "1.0.0");
    assert!(config.gates.run_clippy);
    assert!(config.gates.satd.enabled);
    assert!(config.gates.mutation.enabled);
    assert!(config.gates.security.enabled);
    assert_eq!(config.tiers.tier1_gates.len(), 2);
}

// ============================================================================
// Mock Command Execution Tracking
// ============================================================================

#[test]
fn test_mock_executor_tracks_executions() {
    let mock = MockCommandExecutor::with_fallback(CommandResult::success("ok"));

    mock.execute("cmd1", &["a"]).expect("should succeed");
    mock.execute("cmd2", &["b", "c"]).expect("should succeed");
    mock.execute("cmd1", &["d"]).expect("should succeed");

    let executions = mock.get_executions();
    assert_eq!(executions.len(), 3);

    assert!(mock.was_executed("cmd1", &["a"]));
    assert!(mock.was_executed("cmd2", &["b", "c"]));
    assert!(mock.was_executed("cmd1", &["d"]));
    assert!(!mock.was_executed("cmd3", &[]));
}

#[test]
fn test_mock_executor_program_wildcard_matching() {
    let mut mock = MockCommandExecutor::new();
    mock.register_program("cargo", CommandResult::success("cargo output"));

    // Any cargo command should match
    let result1 = mock.execute("cargo", &["build"]).expect("should succeed");
    let result2 = mock
        .execute("cargo", &["test", "--lib"])
        .expect("should succeed");
    let result3 = mock.execute("cargo", &["clippy"]).expect("should succeed");

    assert!(result1.success);
    assert!(result2.success);
    assert!(result3.success);
}
