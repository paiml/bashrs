#[test]
fn test_docker_scratch_no_false_positive() {
    let content = "FROM scratch\nCOPY binary /app\nUSER app\n";
    let artifact = Artifact::new(
        PathBuf::from("Dockerfile"),
        Scope::Project,
        ArtifactKind::Dockerfile,
    );
    let result = check_rule(RuleId::DockerfileBest, content, &artifact);
    let d001: Vec<_> = result
        .violations
        .iter()
        .filter(|v| v.message.contains("DOCKER001"))
        .collect();
    assert!(
        d001.is_empty(),
        "FROM scratch should not trigger DOCKER001: {:?}",
        d001
    );
}

#[test]
fn test_docker_arg_from_no_false_positive() {
    let content = "ARG BASE=ubuntu:22.04\nFROM $BASE\nRUN echo hello\nUSER app\n";
    let artifact = Artifact::new(
        PathBuf::from("Dockerfile"),
        Scope::Project,
        ArtifactKind::Dockerfile,
    );
    let result = check_rule(RuleId::DockerfileBest, content, &artifact);
    let d001: Vec<_> = result
        .violations
        .iter()
        .filter(|v| v.message.contains("DOCKER001"))
        .collect();
    assert!(
        d001.is_empty(),
        "FROM $ARG should not trigger DOCKER001: {:?}",
        d001
    );
}

#[test]
fn test_docker_apt_without_clean_detected() {
    let content = "FROM ubuntu:22.04\nRUN apt-get update && apt-get install -y curl\nUSER app\n";
    let artifact = Artifact::new(
        PathBuf::from("Dockerfile"),
        Scope::Project,
        ArtifactKind::Dockerfile,
    );
    let result = check_rule(RuleId::DockerfileBest, content, &artifact);
    assert!(
        result
            .violations
            .iter()
            .any(|v| v.message.contains("DOCKER003")),
        "apt-get install without cleanup should be detected: {:?}",
        result.violations
    );
}

#[test]
fn test_docker_apt_with_clean_no_false_positive() {
    let content = "FROM ubuntu:22.04\nRUN apt-get update && apt-get install -y curl && rm -rf /var/lib/apt/lists/*\nUSER app\n";
    let artifact = Artifact::new(
        PathBuf::from("Dockerfile"),
        Scope::Project,
        ArtifactKind::Dockerfile,
    );
    let result = check_rule(RuleId::DockerfileBest, content, &artifact);
    let d003: Vec<_> = result
        .violations
        .iter()
        .filter(|v| v.message.contains("DOCKER003"))
        .collect();
    assert!(
        d003.is_empty(),
        "apt-get with cleanup should not trigger DOCKER003: {:?}",
        d003
    );
}

#[test]
fn test_docker_apt_autoremove_no_false_positive() {
    let content = "FROM ubuntu:22.04\nRUN apt-get update && apt-get install -y curl && apt-get autoremove\nUSER app\n";
    let artifact = Artifact::new(
        PathBuf::from("Dockerfile"),
        Scope::Project,
        ArtifactKind::Dockerfile,
    );
    let result = check_rule(RuleId::DockerfileBest, content, &artifact);
    let d003: Vec<_> = result
        .violations
        .iter()
        .filter(|v| v.message.contains("DOCKER003"))
        .collect();
    assert!(
        d003.is_empty(),
        "apt-get autoremove should not trigger DOCKER003: {:?}",
        d003
    );
}

#[test]
fn test_docker_multistage_from_as_no_false_positive() {
    // Multi-stage: FROM image:tag AS builder
    let content = "FROM rust:1.75 AS builder\nRUN cargo build\nFROM debian:bookworm-slim\nCOPY --from=builder /app /app\nUSER app\n";
    let artifact = Artifact::new(
        PathBuf::from("Dockerfile"),
        Scope::Project,
        ArtifactKind::Dockerfile,
    );
    let result = check_rule(RuleId::DockerfileBest, content, &artifact);
    let d001: Vec<_> = result
        .violations
        .iter()
        .filter(|v| v.message.contains("DOCKER001"))
        .collect();
    assert!(
        d001.is_empty(),
        "Pinned multi-stage FROM should not trigger DOCKER001: {:?}",
        d001
    );
}

#[test]
fn test_docker_multiple_violations() {
    let content = "FROM ubuntu\nADD . /app\nRUN apt-get install -y curl\n";
    let artifact = Artifact::new(
        PathBuf::from("Dockerfile"),
        Scope::Project,
        ArtifactKind::Dockerfile,
    );
    let result = check_rule(RuleId::DockerfileBest, content, &artifact);
    // DOCKER001 (untagged) + DOCKER008 (ADD) + DOCKER003 (apt) + DOCKER010 (no USER)
    assert!(
        result.violations.len() >= 4,
        "Expected at least 4 violations, got {}: {:?}",
        result.violations.len(),
        result.violations
    );
}

// ─── COMPLY-007 Makefile Safety Expansion ───

#[test]
fn test_make_eval_in_recipe_detected() {
    let content = ".PHONY: all\nall:\n\teval \"$(SOME_CMD)\"\n";
    let artifact = Artifact::new(
        PathBuf::from("Makefile"),
        Scope::Project,
        ArtifactKind::Makefile,
    );
    let result = check_rule(RuleId::MakefileSafety, content, &artifact);
    assert!(
        result
            .violations
            .iter()
            .any(|v| v.message.contains("MAKE001")),
        "eval in recipe should be detected: {:?}",
        result.violations
    );
}

#[test]
fn test_make_recursive_bare_detected() {
    let content = ".PHONY: all\nall:\n\tmake clean\n";
    let artifact = Artifact::new(
        PathBuf::from("Makefile"),
        Scope::Project,
        ArtifactKind::Makefile,
    );
    let result = check_rule(RuleId::MakefileSafety, content, &artifact);
    assert!(
        result
            .violations
            .iter()
            .any(|v| v.message.contains("MAKE002")),
        "bare make should be detected: {:?}",
        result.violations
    );
}

#[test]
fn test_make_recursive_dollar_make_no_false_positive() {
    let content = ".PHONY: all\nall:\n\t$(MAKE) clean\n";
    let artifact = Artifact::new(
        PathBuf::from("Makefile"),
        Scope::Project,
        ArtifactKind::Makefile,
    );
    let result = check_rule(RuleId::MakefileSafety, content, &artifact);
    let m002: Vec<_> = result
        .violations
        .iter()
        .filter(|v| v.message.contains("MAKE002"))
        .collect();
    assert!(
        m002.is_empty(),
        "$(MAKE) should not trigger MAKE002: {:?}",
        m002
    );
}

#[test]
fn test_make_recursive_chained_detected() {
    let content = ".PHONY: all\nall:\n\techo starting && make clean\n";
    let artifact = Artifact::new(
        PathBuf::from("Makefile"),
        Scope::Project,
        ArtifactKind::Makefile,
    );
    let result = check_rule(RuleId::MakefileSafety, content, &artifact);
    assert!(result
        .violations
        .iter()
        .any(|v| v.message.contains("MAKE002")));
}

#[test]
fn test_make_dangerous_rm_detected() {
    let content = ".PHONY: clean\nclean:\n\trm -rf $(BUILD_DIR)\n";
    let artifact = Artifact::new(
        PathBuf::from("Makefile"),
        Scope::Project,
        ArtifactKind::Makefile,
    );
    let result = check_rule(RuleId::MakefileSafety, content, &artifact);
    assert!(
        result
            .violations
            .iter()
            .any(|v| v.message.contains("MAKE003")),
        "rm -rf with variable should be detected: {:?}",
        result.violations
    );
}

#[test]
fn test_make_safe_rm_literal_no_false_positive() {
    // rm -rf on a literal path (no variable) is fine
    let content = ".PHONY: clean\nclean:\n\trm -rf /tmp/build\n";
    let artifact = Artifact::new(
        PathBuf::from("Makefile"),
        Scope::Project,
        ArtifactKind::Makefile,
    );
    let result = check_rule(RuleId::MakefileSafety, content, &artifact);
    let m003: Vec<_> = result
        .violations
        .iter()
        .filter(|v| v.message.contains("MAKE003"))
        .collect();
    assert!(
        m003.is_empty(),
        "rm -rf on literal path should not trigger MAKE003: {:?}",
        m003
    );
}

#[test]
fn test_make_missing_phony_detected() {
    // Common targets without .PHONY declaration
    let content = "all:\n\techo building\nclean:\n\trm -f output\ntest:\n\tcargo test\n";
    let artifact = Artifact::new(
        PathBuf::from("Makefile"),
        Scope::Project,
        ArtifactKind::Makefile,
    );
    let result = check_rule(RuleId::MakefileSafety, content, &artifact);
    assert!(
        result
            .violations
            .iter()
            .any(|v| v.message.contains("MAKE004")),
        "Missing .PHONY should be detected: {:?}",
        result.violations
    );
    // Should flag all three: all, clean, test
    let m004: Vec<_> = result
        .violations
        .iter()
        .filter(|v| v.message.contains("MAKE004"))
        .collect();
    assert!(
        m004.len() >= 3,
        "Expected at least 3 missing .PHONY, got {}: {:?}",
        m004.len(),
        m004
    );
}

#[test]
fn test_make_with_phony_no_false_positive() {
    let content = ".PHONY: all clean test\nall:\n\techo building\nclean:\n\trm -f output\ntest:\n\tcargo test\n";
    let artifact = Artifact::new(
        PathBuf::from("Makefile"),
        Scope::Project,
        ArtifactKind::Makefile,
    );
    let result = check_rule(RuleId::MakefileSafety, content, &artifact);
    let m004: Vec<_> = result
        .violations
        .iter()
        .filter(|v| v.message.contains("MAKE004"))
        .collect();
    assert!(
        m004.is_empty(),
        "Declared .PHONY should not trigger MAKE004: {:?}",
        m004
    );
}

#[test]
fn test_make_non_standard_target_no_false_positive() {
    // Custom targets not in COMMON_PHONY_TARGETS should not be flagged
    let content = "my-custom-target:\n\techo custom\n";
    let artifact = Artifact::new(
        PathBuf::from("Makefile"),
        Scope::Project,
        ArtifactKind::Makefile,
    );
    let result = check_rule(RuleId::MakefileSafety, content, &artifact);
    let m004: Vec<_> = result
        .violations
        .iter()
        .filter(|v| v.message.contains("MAKE004"))
        .collect();
    assert!(
        m004.is_empty(),
        "Custom target should not trigger MAKE004: {:?}",
        m004
    );
}

#[test]
fn test_make_multiple_violations() {
    let content = "all:\n\teval \"$CMD\"\n\tmake clean\n\trm -rf $(DIR)\n";
    let artifact = Artifact::new(
        PathBuf::from("Makefile"),
        Scope::Project,
        ArtifactKind::Makefile,
    );
    let result = check_rule(RuleId::MakefileSafety, content, &artifact);
    // MAKE001 (eval) + MAKE002 (bare make) + MAKE003 (rm -rf) + MAKE004 (no .PHONY all)
    assert!(
        result.violations.len() >= 4,
        "Expected at least 4 violations, got {}: {:?}",
        result.violations.len(),
        result.violations
    );
}

// ─── Runner output format tests ───

#[test]
fn test_format_human_failures_only_excludes_compliant() {
    use super::runner;
    let scores = vec![
        super::scoring::compute_artifact_score("clean.sh", &[]),
        super::scoring::compute_artifact_score(
            "bad.sh",
            &[RuleResult {
                rule: RuleId::Determinism,
                passed: false,
                violations: vec![Violation {
                    rule: RuleId::Determinism,
                    line: Some(1),
                    message: "test violation".to_string(),
                }],
            }],
        ),
    ];
    let project = super::scoring::compute_project_score(scores);
    let output = runner::format_human_failures_only(&project);
    assert!(
        output.contains("bad.sh"),
        "Should show non-compliant artifact"
    );
    assert!(
        !output.contains("clean.sh"),
        "Should NOT show compliant artifact"
    );
    assert!(
        output.contains("Failures Only"),
        "Should have failures-only header"
    );
}

#[test]
fn test_format_human_failures_only_all_compliant() {
    use super::runner;
    let scores = vec![super::scoring::compute_artifact_score("clean.sh", &[])];
    let project = super::scoring::compute_project_score(scores);
    let output = runner::format_human_failures_only(&project);
    assert!(
        output.contains("No violations found"),
        "Should show no-violations message"
    );
}

// ═══════════════════════════════════════════════════════════════
// Inline suppression tests (# comply:disable=COMPLY-001)
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_suppression_extract_single_rule() {
    use super::runner;
    let rules = runner::extract_disable_rules("# comply:disable=COMPLY-001");
    assert_eq!(rules, Some(vec!["COMPLY-001".to_string()]));
}

#[test]
fn test_suppression_extract_multiple_rules() {
    use super::runner;
    let rules = runner::extract_disable_rules("# comply:disable=COMPLY-001,COMPLY-004");
    assert_eq!(
        rules,
        Some(vec!["COMPLY-001".to_string(), "COMPLY-004".to_string()])
    );
}

#[test]
fn test_suppression_extract_no_hash() {
    use super::runner;
    // Without # prefix, should not match
    let rules = runner::extract_disable_rules("comply:disable=COMPLY-001");
    assert_eq!(rules, None);
}

#[test]
fn test_suppression_extract_inline_comment() {
    use super::runner;
    let rules = runner::extract_disable_rules("echo $RANDOM # comply:disable=COMPLY-002");
    assert_eq!(rules, Some(vec!["COMPLY-002".to_string()]));
}

