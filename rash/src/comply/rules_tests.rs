#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]

use super::config::Scope;
use super::discovery::{Artifact, ArtifactKind};
use super::rules::*;
use std::path::PathBuf;

fn shell_artifact() -> Artifact {
    Artifact::new(
        PathBuf::from("test.sh"),
        Scope::Project,
        ArtifactKind::ShellScript,
    )
}

fn makefile_artifact() -> Artifact {
    Artifact::new(
        PathBuf::from("Makefile"),
        Scope::Project,
        ArtifactKind::Makefile,
    )
}

fn dockerfile_artifact() -> Artifact {
    Artifact::new(
        PathBuf::from("Dockerfile"),
        Scope::Project,
        ArtifactKind::Dockerfile,
    )
}

fn config_artifact() -> Artifact {
    Artifact::new(
        PathBuf::from(".bashrc"),
        Scope::Project,
        ArtifactKind::ShellConfig,
    )
}

fn workflow_artifact() -> Artifact {
    Artifact::new(
        PathBuf::from(".github/workflows/ci.yml"),
        Scope::Project,
        ArtifactKind::Workflow,
    )
}

// =============================================================================
// RuleId methods
// =============================================================================

#[test]
fn test_rule_id_code() {
    assert_eq!(RuleId::Posix.code(), "COMPLY-001");
    assert_eq!(RuleId::Determinism.code(), "COMPLY-002");
    assert_eq!(RuleId::Idempotency.code(), "COMPLY-003");
    assert_eq!(RuleId::Security.code(), "COMPLY-004");
    assert_eq!(RuleId::Quoting.code(), "COMPLY-005");
    assert_eq!(RuleId::ShellCheck.code(), "COMPLY-006");
    assert_eq!(RuleId::MakefileSafety.code(), "COMPLY-007");
    assert_eq!(RuleId::DockerfileBest.code(), "COMPLY-008");
    assert_eq!(RuleId::ConfigHygiene.code(), "COMPLY-009");
    assert_eq!(RuleId::PzshBudget.code(), "COMPLY-010");
}

#[test]
fn test_rule_id_name() {
    assert_eq!(RuleId::Posix.name(), "POSIX Compliance");
    assert_eq!(RuleId::Determinism.name(), "Determinism");
    assert_eq!(RuleId::Idempotency.name(), "Idempotency");
    assert_eq!(RuleId::Security.name(), "Security");
    assert_eq!(RuleId::Quoting.name(), "Variable Quoting");
    assert_eq!(RuleId::ShellCheck.name(), "ShellCheck Clean");
    assert_eq!(RuleId::MakefileSafety.name(), "Makefile Safety");
    assert_eq!(RuleId::DockerfileBest.name(), "Dockerfile Best Practices");
    assert_eq!(RuleId::ConfigHygiene.name(), "Config Hygiene");
    assert_eq!(RuleId::PzshBudget.name(), "pzsh Startup Budget");
}

#[test]
fn test_rule_id_description() {
    for rule in RuleId::all() {
        assert!(
            !rule.description().is_empty(),
            "{:?} has empty description",
            rule
        );
    }
}

#[test]
fn test_rule_id_applies_to() {
    // First 9 rules (Posix through ConfigHygiene) should have non-empty applies_to
    let first_nine = &RuleId::all()[..9];
    for rule in first_nine {
        assert!(
            !rule.applies_to().is_empty(),
            "{:?} should have non-empty applies_to",
            rule
        );
    }
    // DevContainer (PzshBudget is the 10th, not DevContainer — PzshBudget applies to config)
    // Actually PzshBudget applies to ["config"], so it IS non-empty.
    // Let's just verify all 10 directly:
    assert!(!RuleId::Posix.applies_to().is_empty());
    assert!(!RuleId::Determinism.applies_to().is_empty());
    assert!(!RuleId::Idempotency.applies_to().is_empty());
    assert!(!RuleId::Security.applies_to().is_empty());
    assert!(!RuleId::Quoting.applies_to().is_empty());
    assert!(!RuleId::ShellCheck.applies_to().is_empty());
    assert!(!RuleId::MakefileSafety.applies_to().is_empty());
    assert!(!RuleId::DockerfileBest.applies_to().is_empty());
    assert!(!RuleId::ConfigHygiene.applies_to().is_empty());
    assert!(!RuleId::PzshBudget.applies_to().is_empty());
}

#[test]
fn test_rule_id_all() {
    assert_eq!(RuleId::all().len(), 10);
}

#[test]
fn test_rule_id_weight() {
    let total: u32 = RuleId::all().iter().map(|r| r.weight()).sum();
    assert_eq!(total, 110);
}

#[test]
fn test_applicable_rules_shell() {
    let rules = RuleId::applicable_rules(ArtifactKind::ShellScript);
    assert_eq!(rules.len(), 6);
    assert!(rules.contains(&RuleId::Posix));
    assert!(rules.contains(&RuleId::Determinism));
    assert!(rules.contains(&RuleId::Idempotency));
    assert!(rules.contains(&RuleId::Security));
    assert!(rules.contains(&RuleId::Quoting));
    assert!(rules.contains(&RuleId::ShellCheck));
}

#[test]
fn test_applicable_rules_makefile() {
    let rules = RuleId::applicable_rules(ArtifactKind::Makefile);
    assert_eq!(rules.len(), 4);
    assert!(rules.contains(&RuleId::Determinism));
    assert!(rules.contains(&RuleId::Idempotency));
    assert!(rules.contains(&RuleId::Security));
    assert!(rules.contains(&RuleId::MakefileSafety));
}

#[test]
fn test_applicable_rules_dockerfile() {
    let rules = RuleId::applicable_rules(ArtifactKind::Dockerfile);
    assert_eq!(rules.len(), 2);
    assert!(rules.contains(&RuleId::Security));
    assert!(rules.contains(&RuleId::DockerfileBest));
}

#[test]
fn test_applicable_rules_config() {
    let rules = RuleId::applicable_rules(ArtifactKind::ShellConfig);
    assert_eq!(rules.len(), 3);
    assert!(rules.contains(&RuleId::Security));
    assert!(rules.contains(&RuleId::Quoting));
    assert!(rules.contains(&RuleId::ConfigHygiene));
}

#[test]
fn test_applicable_rules_workflow() {
    let rules = RuleId::applicable_rules(ArtifactKind::Workflow);
    assert_eq!(rules.len(), 1);
    assert!(rules.contains(&RuleId::Security));
}

#[test]
fn test_applicable_rules_devcontainer() {
    let rules = RuleId::applicable_rules(ArtifactKind::DevContainer);
    assert_eq!(rules.len(), 0);
}

// =============================================================================
// Violation Display
// =============================================================================

#[test]
fn test_violation_display_with_line() {
    let v = Violation {
        rule: RuleId::Security,
        line: Some(42),
        message: "SEC001: eval with variable input (injection risk)".to_string(),
    };
    let display = format!("{v}");
    assert_eq!(
        display,
        "COMPLY-004: line 42: SEC001: eval with variable input (injection risk)"
    );
}

#[test]
fn test_violation_display_without_line() {
    let v = Violation {
        rule: RuleId::DockerfileBest,
        line: None,
        message: "DOCKER010: Missing USER directive (runs as root)".to_string(),
    };
    let display = format!("{v}");
    assert_eq!(
        display,
        "COMPLY-008: DOCKER010: Missing USER directive (runs as root)"
    );
}

// =============================================================================
// Determinism (COMPLY-002)
// =============================================================================

#[test]
fn test_determinism_random() {
    let art = shell_artifact();
    let result = check_rule(RuleId::Determinism, "echo $RANDOM", &art);
    assert!(!result.passed);
    assert_eq!(result.violations.len(), 1);
    assert!(result.violations[0].message.contains("$RANDOM"));
}

#[test]
fn test_determinism_srandom() {
    let art = shell_artifact();
    let result = check_rule(RuleId::Determinism, "echo $SRANDOM", &art);
    assert!(!result.passed);
    assert!(result.violations[0].message.contains("$SRANDOM"));
}

#[test]
fn test_determinism_bashpid() {
    let art = shell_artifact();
    let result = check_rule(RuleId::Determinism, "echo $BASHPID", &art);
    assert!(!result.passed);
    assert!(result.violations[0].message.contains("$BASHPID"));
}

#[test]
fn test_determinism_pid() {
    let art = shell_artifact();
    let result = check_rule(RuleId::Determinism, "echo $$", &art);
    assert!(!result.passed);
    assert!(result.violations[0].message.contains("$$"));
}

#[test]
fn test_determinism_pid_makefile_ok() {
    let art = makefile_artifact();
    // In Makefiles, $$ is make's escape for a literal $, not bash's $$ PID
    let result = check_rule(RuleId::Determinism, "echo $$var", &art);
    assert!(result.passed);
    assert!(result.violations.is_empty());
}

#[test]
fn test_determinism_date_epoch() {
    let art = shell_artifact();
    let result = check_rule(RuleId::Determinism, "STAMP=$(date +%s)", &art);
    assert!(!result.passed);
    assert!(result.violations[0].message.contains("timestamp"));
}

#[test]
fn test_determinism_date_nano() {
    let art = shell_artifact();
    let result = check_rule(RuleId::Determinism, "NS=$(date +%N)", &art);
    assert!(!result.passed);
    assert!(result.violations[0].message.contains("timestamp"));
}

#[test]
fn test_determinism_urandom() {
    let art = shell_artifact();
    let result = check_rule(RuleId::Determinism, "head -c 16 /dev/urandom", &art);
    assert!(!result.passed);
    assert!(result.violations[0].message.contains("/dev/urandom"));
}

#[test]
fn test_determinism_random_device() {
    let art = shell_artifact();
    let result = check_rule(
        RuleId::Determinism,
        "dd if=/dev/random of=key bs=32 count=1",
        &art,
    );
    assert!(!result.passed);
    assert!(result.violations[0]
        .message
        .contains("/dev/urandom or /dev/random"));
}

#[test]
fn test_determinism_mktemp() {
    let art = shell_artifact();
    let result = check_rule(RuleId::Determinism, "mktemp /tmp/foo.XXXXX", &art);
    assert!(!result.passed);
    assert!(result.violations[0].message.contains("mktemp"));
}

#[test]
fn test_determinism_mktemp_subshell() {
    let art = shell_artifact();
    let result = check_rule(RuleId::Determinism, "TMPFILE=$(mktemp)", &art);
    assert!(!result.passed);
    assert!(result.violations[0].message.contains("mktemp"));
}

#[test]
fn test_determinism_shuf() {
    let art = shell_artifact();
    let result = check_rule(RuleId::Determinism, "shuf -n 1 words.txt", &art);
    assert!(!result.passed);
    assert!(result.violations[0].message.contains("shuf"));
}

#[test]
fn test_determinism_shuf_pipe() {
    let art = shell_artifact();
    let result = check_rule(RuleId::Determinism, "cat list.txt | shuf", &art);
    assert!(!result.passed);
    assert!(result.violations[0].message.contains("shuf"));
}

#[test]
fn test_determinism_clean_script() {
    let art = shell_artifact();
    let content = "#!/bin/sh\necho hello\nset -e\ncp a b\n";
    let result = check_rule(RuleId::Determinism, content, &art);
    assert!(result.passed);
    assert!(result.violations.is_empty());
}

#[test]
fn test_determinism_comments_skipped() {
    let art = shell_artifact();
    // $RANDOM inside a comment should NOT trigger a violation
    let content = "#!/bin/sh\n# echo $RANDOM\necho hello\n";
    let result = check_rule(RuleId::Determinism, content, &art);
    assert!(result.passed);
    assert!(result.violations.is_empty());
}

// =============================================================================
// Idempotency (COMPLY-003)
// =============================================================================

#[test]

include!("rules_tests_incl2.rs");
