#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]

use super::config::*;
use super::discovery::*;
use super::rules::*;
use super::runner;
use super::scoring::*;
use std::path::PathBuf;

// ─── F-001: Empty project produces score 0, no crash ───
#[test]
fn test_comments_not_flagged() {
    let content = "#!/bin/sh\n# $RANDOM is used for demo\necho hello\n";
    let artifact = Artifact::new(
        PathBuf::from("test.sh"),
        Scope::Project,
        ArtifactKind::ShellScript,
    );
    let result = check_rule(RuleId::Determinism, content, &artifact);
    assert!(result.passed, "Comments should not be flagged");
}

// ─── Scope display ───
#[test]
fn test_scope_display() {
    assert_eq!(format!("{}", Scope::Project), "project");
    assert_eq!(format!("{}", Scope::User), "user");
    assert_eq!(format!("{}", Scope::System), "system");
}

// ─── ShellCheck patterns ───
#[test]
fn test_shellcheck_backtick_detection() {
    let content = "#!/bin/sh\nresult=`ls -la`\n";
    let artifact = Artifact::new(
        PathBuf::from("test.sh"),
        Scope::Project,
        ArtifactKind::ShellScript,
    );
    let result = check_rule(RuleId::ShellCheck, content, &artifact);
    assert!(!result.passed, "Backticks should be flagged as SC2006");
}

// ─── Clean script passes all rules ───
#[test]
fn test_clean_script_passes_all() {
    let content = "#!/bin/sh\necho \"hello world\"\nmkdir -p /tmp/test\n";
    let artifact = Artifact::new(
        PathBuf::from("clean.sh"),
        Scope::Project,
        ArtifactKind::ShellScript,
    );

    for rule in RuleId::applicable_rules(ArtifactKind::ShellScript) {
        let result = check_rule(rule, content, &artifact);
        assert!(result.passed, "{} should pass on clean script", rule.code());
    }
}

// ─── GH-134: COMPLY-002 false positive on Makefile $$ escape syntax ───

#[test]
fn test_gh134_makefile_dollar_escape_not_flagged() {
    // In Makefiles, $$ is Make's escape for a literal $, not bash's $$ PID
    let content = "coverage-check:\n\t@cargo llvm-cov 2>&1 | awk '{print $$10}'\n";
    let artifact = Artifact::new(
        PathBuf::from("Makefile"),
        Scope::Project,
        ArtifactKind::Makefile,
    );
    let result = check_rule(RuleId::Determinism, content, &artifact);
    assert!(
        result.passed,
        "Makefile $$ should not be flagged as COMPLY-002: {:?}",
        result.violations
    );
}

#[test]
fn test_gh134_makefile_dollar_escape_in_sed() {
    let content = "clean:\n\t@sed 's/$$HOME/\\/root/g' input.txt\n";
    let artifact = Artifact::new(
        PathBuf::from("Makefile"),
        Scope::Project,
        ArtifactKind::Makefile,
    );
    let result = check_rule(RuleId::Determinism, content, &artifact);
    assert!(
        result.passed,
        "Makefile $$ in sed should not be flagged: {:?}",
        result.violations
    );
}

#[test]
fn test_gh134_makefile_dollar_escape_loop_var() {
    // Common pattern: for f in *.txt; do echo $$f; done
    let content = "process:\n\t@for f in *.txt; do echo $$f; done\n";
    let artifact = Artifact::new(
        PathBuf::from("Makefile"),
        Scope::Project,
        ArtifactKind::Makefile,
    );
    let result = check_rule(RuleId::Determinism, content, &artifact);
    assert!(
        result.passed,
        "Makefile $$ loop variable should not be flagged: {:?}",
        result.violations
    );
}

#[test]
fn test_gh134_shell_script_pid_still_flagged() {
    // $$ in shell scripts is still the PID variable and should be flagged
    let content = "#!/bin/sh\ntmpfile=/tmp/foo.$$\n";
    let artifact = Artifact::new(
        PathBuf::from("script.sh"),
        Scope::Project,
        ArtifactKind::ShellScript,
    );
    let result = check_rule(RuleId::Determinism, content, &artifact);
    assert!(
        !result.passed,
        "Shell script $$ should still be flagged as PID usage"
    );
}

#[test]
fn test_gh134_makefile_random_still_flagged() {
    // $RANDOM in Makefiles is still non-deterministic
    let content = "generate:\n\t@echo $RANDOM\n";
    let artifact = Artifact::new(
        PathBuf::from("Makefile"),
        Scope::Project,
        ArtifactKind::Makefile,
    );
    let result = check_rule(RuleId::Determinism, content, &artifact);
    assert!(
        !result.passed,
        "$RANDOM should still be flagged in Makefiles"
    );
}

// ─── GH-135: COMPLY-004 false positive on yq eval subcommand ───

#[test]
fn test_gh135_yq_eval_not_flagged() {
    // yq eval is a subcommand, not bash's eval builtin
    let content = "#!/bin/sh\nyq eval '.' \"$f\" > /dev/null\n";
    let artifact = Artifact::new(
        PathBuf::from("validate.sh"),
        Scope::Project,
        ArtifactKind::ShellScript,
    );
    let result = check_rule(RuleId::Security, content, &artifact);
    assert!(
        result.passed,
        "yq eval should not be flagged as SEC001: {:?}",
        result.violations
    );
}

#[test]
fn test_gh135_jq_eval_not_flagged() {
    let content = "#!/bin/sh\njq eval \"$expr\" input.json\n";
    let artifact = Artifact::new(
        PathBuf::from("process.sh"),
        Scope::Project,
        ArtifactKind::ShellScript,
    );
    let result = check_rule(RuleId::Security, content, &artifact);
    assert!(
        result.passed,
        "jq eval should not be flagged as SEC001: {:?}",
        result.violations
    );
}

#[test]
fn test_gh135_helm_eval_not_flagged() {
    let content = "#!/bin/sh\nhelm eval \"$template\" --values values.yaml\n";
    let artifact = Artifact::new(
        PathBuf::from("deploy.sh"),
        Scope::Project,
        ArtifactKind::ShellScript,
    );
    let result = check_rule(RuleId::Security, content, &artifact);
    assert!(
        result.passed,
        "helm eval should not be flagged as SEC001: {:?}",
        result.violations
    );
}

#[test]
fn test_gh135_bare_eval_still_flagged() {
    // Plain eval with variable input is still dangerous
    let content = "#!/bin/sh\neval \"$USER_INPUT\"\n";
    let artifact = Artifact::new(
        PathBuf::from("test.sh"),
        Scope::Project,
        ArtifactKind::ShellScript,
    );
    let result = check_rule(RuleId::Security, content, &artifact);
    assert!(
        !result.passed,
        "Bare eval with variable should still be flagged"
    );
}

#[test]
fn test_gh135_eval_after_semicolon_still_flagged() {
    // eval after command separator is still a command
    let content = "#!/bin/sh\ncd /tmp; eval \"$CMD\"\n";
    let artifact = Artifact::new(
        PathBuf::from("test.sh"),
        Scope::Project,
        ArtifactKind::ShellScript,
    );
    let result = check_rule(RuleId::Security, content, &artifact);
    assert!(!result.passed, "eval after ; should still be flagged");
}

#[test]
fn test_gh135_eval_after_and_still_flagged() {
    // eval after && is still a command
    let content = "#!/bin/sh\ntest -f config && eval \"$CONFIG\"\n";
    let artifact = Artifact::new(
        PathBuf::from("test.sh"),
        Scope::Project,
        ArtifactKind::ShellScript,
    );
    let result = check_rule(RuleId::Security, content, &artifact);
    assert!(!result.passed, "eval after && should still be flagged");
}

#[test]
fn test_gh135_workflow_yq_eval_not_flagged() {
    // yq eval in GitHub workflow YAML should not be flagged
    let content = "    - name: Validate\n      run: |\n        for f in playbooks/**/*.yaml; do\n          yq eval '.' \"$f\" > /dev/null\n        done\n";
    let artifact = Artifact::new(
        PathBuf::from(".github/workflows/ci.yml"),
        Scope::Project,
        ArtifactKind::Workflow,
    );
    let result = check_rule(RuleId::Security, content, &artifact);
    assert!(
        result.passed,
        "yq eval in workflow should not be flagged: {:?}",
        result.violations
    );
}

// BH-MUT-0015: is_eval_command separator coverage
// Kills mutation of removing "|| " from the separator list

#[test]
fn test_gh135_eval_after_or_still_flagged() {
    // eval after || is still a command
    let content = "#!/bin/sh\ntest -f config || eval \"$FALLBACK\"\n";
    let artifact = Artifact::new(
        PathBuf::from("test.sh"),
        Scope::Project,
        ArtifactKind::ShellScript,
    );
    let result = check_rule(RuleId::Security, content, &artifact);
    assert!(!result.passed, "eval after || should still be flagged");
}

// ═══════════════════════════════════════════════════════════════
// Phase 1 Completion: Config persistence round-trip tests
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_config_save_load_roundtrip() {
    let dir = tempfile::tempdir().unwrap();
    let config = ComplyConfig::new_default("7.1.0");
    config.save(dir.path()).unwrap();

    let loaded = ComplyConfig::load(dir.path()).expect("Failed to load saved config");
    assert_eq!(loaded.comply.version, "1.0.0");
    assert_eq!(loaded.comply.bashrs_version, "7.1.0");
    assert!(loaded.scopes.project);
    assert!(!loaded.scopes.user);
    assert!(!loaded.scopes.system);
    assert!(loaded.rules.posix);
    assert!(loaded.rules.security);
    assert_eq!(loaded.thresholds.min_score, 80);
}

#[test]
fn test_config_save_creates_directory() {
    let dir = tempfile::tempdir().unwrap();
    let bashrs_dir = dir.path().join(".bashrs");
    assert!(!bashrs_dir.exists());

    let config = ComplyConfig::new_default("7.1.0");
    config.save(dir.path()).unwrap();

    assert!(bashrs_dir.exists());
    assert!(bashrs_dir.join("comply.toml").exists());
}

#[test]
fn test_config_exists_false_when_no_file() {
    let dir = tempfile::tempdir().unwrap();
    assert!(!ComplyConfig::exists(dir.path()));
}

#[test]
fn test_config_exists_true_after_save() {
    let dir = tempfile::tempdir().unwrap();
    let config = ComplyConfig::new_default("7.1.0");
    config.save(dir.path()).unwrap();
    assert!(ComplyConfig::exists(dir.path()));
}

#[test]
fn test_config_load_returns_none_when_missing() {
    let dir = tempfile::tempdir().unwrap();
    assert!(ComplyConfig::load(dir.path()).is_none());
}

#[test]

include!("tests_s2_incl2.rs");
