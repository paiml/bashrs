#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]

use super::config::Scope;
use super::discovery::{Artifact, ArtifactKind};
use super::rules::*;
use std::path::PathBuf;

#[test]
fn test_docker_pinned_from_ok() {
    let art = dockerfile_artifact();
    let content = "FROM ubuntu:22.04\nRUN echo hello\nUSER appuser";
    let result = check_rule(RuleId::DockerfileBest, content, &art);
    assert!(
        !result
            .violations
            .iter()
            .any(|v| v.message.contains("DOCKER001")),
        "Pinned FROM should not trigger DOCKER001"
    );
}

#[test]
fn test_docker_digest_ok() {
    let art = dockerfile_artifact();
    let content = "FROM ubuntu@sha256:abcdef1234567890\nRUN echo hello\nUSER appuser";
    let result = check_rule(RuleId::DockerfileBest, content, &art);
    assert!(
        !result
            .violations
            .iter()
            .any(|v| v.message.contains("DOCKER001")),
        "FROM with digest should not trigger DOCKER001"
    );
}

#[test]
fn test_docker_scratch_ok() {
    let art = dockerfile_artifact();
    let content = "FROM scratch\nCOPY app /app\nUSER appuser";
    let result = check_rule(RuleId::DockerfileBest, content, &art);
    assert!(
        !result
            .violations
            .iter()
            .any(|v| v.message.contains("DOCKER001")),
        "FROM scratch should not trigger DOCKER001"
    );
}

#[test]
fn test_docker_apt_no_clean() {
    let art = dockerfile_artifact();
    let content = "FROM ubuntu:22.04\nRUN apt-get install -y curl\nUSER appuser";
    let result = check_rule(RuleId::DockerfileBest, content, &art);
    assert!(!result.passed);
    assert!(result
        .violations
        .iter()
        .any(|v| v.message.contains("DOCKER003")));
}

#[test]
fn test_docker_apt_with_clean_ok() {
    let art = dockerfile_artifact();
    let content = "FROM ubuntu:22.04\nRUN apt-get install -y curl && rm -rf /var/lib/apt/lists/*\nUSER appuser";
    let result = check_rule(RuleId::DockerfileBest, content, &art);
    assert!(
        !result
            .violations
            .iter()
            .any(|v| v.message.contains("DOCKER003")),
        "apt-get with cleanup should not trigger DOCKER003"
    );
}

#[test]
fn test_docker_bare_expose() {
    let art = dockerfile_artifact();
    let content = "FROM ubuntu:22.04\nEXPOSE\nUSER appuser";
    let result = check_rule(RuleId::DockerfileBest, content, &art);
    assert!(!result.passed);
    assert!(result
        .violations
        .iter()
        .any(|v| v.message.contains("DOCKER004")));
}

#[test]
fn test_docker_missing_user() {
    let art = dockerfile_artifact();
    let content = "FROM ubuntu:22.04\nRUN echo hello";
    let result = check_rule(RuleId::DockerfileBest, content, &art);
    assert!(!result.passed);
    assert!(result
        .violations
        .iter()
        .any(|v| v.message.contains("DOCKER010")));
}

#[test]
fn test_docker_has_user_ok() {
    let art = dockerfile_artifact();
    let content = "FROM ubuntu:22.04\nRUN echo hello\nUSER appuser";
    let result = check_rule(RuleId::DockerfileBest, content, &art);
    assert!(
        !result
            .violations
            .iter()
            .any(|v| v.message.contains("DOCKER010")),
        "USER directive present should not trigger DOCKER010"
    );
}

// =============================================================================
// Config Hygiene (COMPLY-009)
// =============================================================================

#[test]
fn test_config_many_path_exports() {
    let art = config_artifact();
    let content = "\
export PATH=/usr/local/bin:$PATH
export PATH=/opt/bin:$PATH
export PATH=$HOME/.local/bin:$PATH
export PATH=$HOME/go/bin:$PATH
";
    let result = check_rule(RuleId::ConfigHygiene, content, &art);
    assert!(!result.passed);
    assert!(result.violations[0].message.contains("PATH modified"));
}

#[test]
fn test_config_few_path_ok() {
    let art = config_artifact();
    let content = "\
export PATH=/usr/local/bin:$PATH
export PATH=$HOME/.local/bin:$PATH
export PATH=$HOME/go/bin:$PATH
";
    let result = check_rule(RuleId::ConfigHygiene, content, &art);
    assert!(result.passed);
}

// =============================================================================
// PzshBudget (COMPLY-010)
// =============================================================================

#[test]
fn test_pzsh_budget_always_passes() {
    let art = config_artifact();
    let result = check_rule(RuleId::PzshBudget, "anything here", &art);
    assert!(result.passed);
    assert!(result.violations.is_empty());
}
