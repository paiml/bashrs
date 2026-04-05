fn test_config_load_returns_none_on_corrupt() {
    let dir = tempfile::tempdir().unwrap();
    let config_dir = dir.path().join(".bashrs");
    std::fs::create_dir_all(&config_dir).unwrap();
    std::fs::write(
        config_dir.join("comply.toml"),
        "this is not valid toml {{{{",
    )
    .unwrap();
    assert!(ComplyConfig::load(dir.path()).is_none());
}

#[test]
fn test_config_path_returns_correct_path() {
    let dir = tempfile::tempdir().unwrap();
    let expected = dir.path().join(".bashrs").join("comply.toml");
    assert_eq!(ComplyConfig::config_path(dir.path()), expected);
}

#[test]
fn test_config_roundtrip_preserves_custom_rules() {
    let dir = tempfile::tempdir().unwrap();
    let mut config = ComplyConfig::new_default("7.2.0");
    config.rules.posix = false;
    config.rules.quoting = false;
    config.rules.pzsh_budget = "disabled".to_string();
    config.thresholds.min_score = 50;
    config.thresholds.shellcheck_severity = "error".to_string();
    config.scopes.user = true;
    config.scopes.system = true;
    config.save(dir.path()).unwrap();

    let loaded = ComplyConfig::load(dir.path()).unwrap();
    assert!(!loaded.rules.posix);
    assert!(!loaded.rules.quoting);
    assert_eq!(loaded.rules.pzsh_budget, "disabled");
    assert_eq!(loaded.thresholds.min_score, 50);
    assert_eq!(loaded.thresholds.shellcheck_severity, "error");
    assert!(loaded.scopes.user);
    assert!(loaded.scopes.system);
}

// ═══════════════════════════════════════════════════════════════
// Phase 1 Completion: Runner coverage tests
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_runner_check_with_real_temp_dir() {
    let dir = tempfile::tempdir().unwrap();
    // Create a clean shell script
    std::fs::write(dir.path().join("clean.sh"), "#!/bin/sh\necho \"hello\"\n").unwrap();
    let config = ComplyConfig::new_default("7.1.0");
    let score = runner::run_check(dir.path(), Some(Scope::Project), &config);
    assert_eq!(score.total_artifacts, 1);
    assert_eq!(score.compliant_artifacts, 1);
    assert_eq!(score.score, 100.0);
}

#[test]
fn test_runner_check_detects_violations_in_temp_dir() {
    let dir = tempfile::tempdir().unwrap();
    // Script with $RANDOM (non-deterministic)
    std::fs::write(dir.path().join("bad.sh"), "#!/bin/sh\necho $RANDOM\n").unwrap();
    let config = ComplyConfig::new_default("7.1.0");
    let score = runner::run_check(dir.path(), Some(Scope::Project), &config);
    assert_eq!(score.total_artifacts, 1);
    assert!(
        score.score < 100.0,
        "Script with $RANDOM should score below 100"
    );
    assert!(score.successful_falsifications > 0);
}

#[test]
fn test_runner_scope_project_only() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("test.sh"), "#!/bin/sh\necho hello\n").unwrap();
    let config = ComplyConfig::new_default("7.1.0");
    let score = runner::run_check(dir.path(), Some(Scope::Project), &config);
    // Should only find project artifacts, not user/system
    assert!(score.total_artifacts >= 1);
}

#[test]
fn test_runner_disabled_rule_not_checked() {
    let dir = tempfile::tempdir().unwrap();
    // Script that would fail POSIX (#!/bin/bash)
    std::fs::write(dir.path().join("test.sh"), "#!/bin/bash\necho hello\n").unwrap();
    let mut config = ComplyConfig::new_default("7.1.0");
    config.rules.posix = false; // Disable POSIX rule
    let score = runner::run_check(dir.path(), Some(Scope::Project), &config);
    // With POSIX disabled, #!/bin/bash should not produce a violation from that rule
    let posix_violations: usize = score
        .artifact_scores
        .iter()
        .flat_map(|a| a.results.iter())
        .filter(|r| r.rule == RuleId::Posix)
        .map(|r| r.violations.len())
        .sum();
    assert_eq!(
        posix_violations, 0,
        "Disabled POSIX rule should not produce violations"
    );
}

#[test]
fn test_runner_user_scope_disabled_returns_empty() {
    let dir = tempfile::tempdir().unwrap();
    let mut config = ComplyConfig::new_default("7.1.0");
    config.scopes.user = false;
    let score = runner::run_check(dir.path(), Some(Scope::User), &config);
    assert_eq!(
        score.total_artifacts, 0,
        "Disabled user scope should return no artifacts"
    );
}

#[test]
fn test_runner_system_scope_disabled_returns_empty() {
    let dir = tempfile::tempdir().unwrap();
    let mut config = ComplyConfig::new_default("7.1.0");
    config.scopes.system = false;
    let score = runner::run_check(dir.path(), Some(Scope::System), &config);
    assert_eq!(
        score.total_artifacts, 0,
        "Disabled system scope should return no artifacts"
    );
}

#[test]
fn test_runner_multiple_artifacts() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("a.sh"), "#!/bin/sh\necho a\n").unwrap();
    std::fs::write(dir.path().join("b.sh"), "#!/bin/sh\necho b\n").unwrap();
    std::fs::write(dir.path().join("Makefile"), "all:\n\techo done\n").unwrap();
    let config = ComplyConfig::new_default("7.1.0");
    let score = runner::run_check(dir.path(), Some(Scope::Project), &config);
    assert!(
        score.total_artifacts >= 3,
        "Should find at least 3 artifacts, found {}",
        score.total_artifacts
    );
}

#[test]
fn test_runner_format_json_has_schema() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("test.sh"), "#!/bin/sh\necho ok\n").unwrap();
    let config = ComplyConfig::new_default("7.1.0");
    let score = runner::run_check(dir.path(), Some(Scope::Project), &config);
    let json = runner::format_json(&score);
    assert!(json.contains("bashrs-comply-check-v1"));
    assert!(json.contains("\"total_artifacts\""));
    assert!(json.contains("\"grade\""));
}

#[test]
fn test_runner_format_human_shows_violations() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(
        dir.path().join("bad.sh"),
        "#!/bin/bash\neval \"$USER_INPUT\"\nmkdir /foo\n",
    )
    .unwrap();
    let config = ComplyConfig::new_default("7.1.0");
    let score = runner::run_check(dir.path(), Some(Scope::Project), &config);
    let human = runner::format_human(&score);
    assert!(human.contains("NON-COMPLIANT"));
    assert!(human.contains("Falsification"));
}

// ═══════════════════════════════════════════════════════════════
// Phase 1 Completion: Discovery coverage tests
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_discover_project_finds_shell_scripts() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("deploy.sh"), "#!/bin/sh\necho deploy\n").unwrap();
    let artifacts = super::discovery::discover(dir.path(), Scope::Project);
    assert_eq!(artifacts.len(), 1);
    assert_eq!(artifacts[0].kind, ArtifactKind::ShellScript);
}

#[test]
fn test_discover_project_finds_makefile() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("Makefile"), "all:\n\techo done\n").unwrap();
    let artifacts = super::discovery::discover(dir.path(), Scope::Project);
    assert_eq!(artifacts.len(), 1);
    assert_eq!(artifacts[0].kind, ArtifactKind::Makefile);
}

#[test]
fn test_discover_project_finds_dockerfile() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("Dockerfile"), "FROM ubuntu:22.04\n").unwrap();
    let artifacts = super::discovery::discover(dir.path(), Scope::Project);
    assert_eq!(artifacts.len(), 1);
    assert_eq!(artifacts[0].kind, ArtifactKind::Dockerfile);
}

#[test]
fn test_discover_project_finds_scripts_dir() {
    let dir = tempfile::tempdir().unwrap();
    let scripts_dir = dir.path().join("scripts");
    std::fs::create_dir_all(&scripts_dir).unwrap();
    std::fs::write(scripts_dir.join("build.sh"), "#!/bin/sh\necho build\n").unwrap();
    let artifacts = super::discovery::discover(dir.path(), Scope::Project);
    assert_eq!(artifacts.len(), 1);
    assert!(artifacts[0]
        .path
        .to_string_lossy()
        .contains("scripts/build.sh"));
}

#[test]
fn test_discover_empty_dir_returns_empty() {
    let dir = tempfile::tempdir().unwrap();
    let artifacts = super::discovery::discover(dir.path(), Scope::Project);
    assert!(artifacts.is_empty());
}

#[test]
fn test_discover_deduplicates_artifacts() {
    let dir = tempfile::tempdir().unwrap();
    // install.sh matches *.sh pattern — should only appear once
    std::fs::write(dir.path().join("install.sh"), "#!/bin/sh\necho install\n").unwrap();
    let artifacts = super::discovery::discover(dir.path(), Scope::Project);
    let count = artifacts
        .iter()
        .filter(|a| a.path.to_string_lossy().contains("install.sh"))
        .count();
    assert_eq!(count, 1, "Artifact should not be duplicated");
}

#[test]
fn test_classify_workflow_yaml() {
    assert_eq!(
        classify(std::path::Path::new(".github/workflows/ci.yml")),
        Some(ArtifactKind::Workflow)
    );
    assert_eq!(
        classify(std::path::Path::new(".github/workflows/deploy.yaml")),
        Some(ArtifactKind::Workflow)
    );
}

#[test]
fn test_classify_docker_compose() {
    assert_eq!(
        classify(std::path::Path::new("docker-compose.yml")),
        Some(ArtifactKind::Workflow)
    );
}

#[test]
fn test_classify_devcontainer() {
    assert_eq!(
        classify(std::path::Path::new(".devcontainer/devcontainer.json")),
        Some(ArtifactKind::DevContainer)
    );
}

#[test]
fn test_classify_gnumakefile() {
    assert_eq!(
        classify(std::path::Path::new("GNUmakefile")),
        Some(ArtifactKind::Makefile)
    );
}

#[test]
fn test_classify_bash_extension() {
    assert_eq!(
        classify(std::path::Path::new("script.bash")),
        Some(ArtifactKind::ShellScript)
    );
}

#[test]
fn test_classify_unknown_returns_none() {
    assert_eq!(classify(std::path::Path::new("readme.txt")), None);
    assert_eq!(classify(std::path::Path::new("main.rs")), None);
    assert_eq!(classify(std::path::Path::new("package.json")), None);
}

#[test]
fn test_classify_shell_configs() {
    assert_eq!(
        classify(std::path::Path::new(".bashrc")),
        Some(ArtifactKind::ShellConfig)
    );
    assert_eq!(
        classify(std::path::Path::new(".zshrc")),
        Some(ArtifactKind::ShellConfig)
    );
    assert_eq!(
        classify(std::path::Path::new(".profile")),
        Some(ArtifactKind::ShellConfig)
    );
}

#[test]
fn test_classify_by_shebang() {
    let dir = tempfile::tempdir().unwrap();
    let script = dir.path().join("myscript");
    std::fs::write(&script, "#!/bin/sh\necho hello\n").unwrap();
    assert_eq!(classify(&script), Some(ArtifactKind::ShellScript));
}

#[test]
fn test_classify_by_shebang_env_bash() {
    let dir = tempfile::tempdir().unwrap();
    let script = dir.path().join("runner");
    std::fs::write(&script, "#!/usr/bin/env bash\necho hello\n").unwrap();
    assert_eq!(classify(&script), Some(ArtifactKind::ShellScript));
}

#[test]
fn test_classify_by_shebang_not_shell() {
    let dir = tempfile::tempdir().unwrap();
    let script = dir.path().join("pyscript");
    std::fs::write(&script, "#!/usr/bin/env python3\nprint('hi')\n").unwrap();
    assert_eq!(classify(&script), None);
}

#[test]
fn test_artifact_display_name_project() {
    let artifact = Artifact::new(
        PathBuf::from("scripts/build.sh"),
        Scope::Project,
        ArtifactKind::ShellScript,
    );
    assert_eq!(artifact.display_name(), "scripts/build.sh");
}

#[test]
fn test_artifact_kind_display() {
    assert_eq!(format!("{}", ArtifactKind::ShellScript), "shell");
    assert_eq!(format!("{}", ArtifactKind::Makefile), "makefile");
    assert_eq!(format!("{}", ArtifactKind::Dockerfile), "dockerfile");
    assert_eq!(format!("{}", ArtifactKind::ShellConfig), "config");
    assert_eq!(format!("{}", ArtifactKind::Workflow), "workflow");
    assert_eq!(format!("{}", ArtifactKind::DevContainer), "devcontainer");
}

// ═══════════════════════════════════════════════════════════════
// Phase 1 Completion: Scoring edge case tests
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_scoring_empty_results_is_perfect() {
    let score = compute_artifact_score("empty.sh", &[]);
    assert_eq!(score.score, 100.0);
    assert_eq!(score.grade, Grade::APlus);
    assert_eq!(score.rules_tested, 0);
    assert_eq!(score.rules_passed, 0);
}

#[test]
fn test_scoring_single_failed_heavyweight_rule() {
    // Security (weight 20) fails, Determinism (weight 15) passes
    // passed_weight = 15, total_weight = 35, score = 15/35 * 100 = 42.8%
    // Below 60% gateway → 42.8% * 0.4 = 17.1%
    let results = vec![
        RuleResult {
            rule: RuleId::Security,
            passed: false,
            violations: vec![Violation {
                rule: RuleId::Security,
                line: Some(1),
                message: "test".into(),
            }],
        },
        RuleResult {
            rule: RuleId::Determinism,
            passed: true,
            violations: vec![],
        },
    ];
    let score = compute_artifact_score("test.sh", &results);
    assert!(score.score < 60.0, "Below gateway should be capped");
}

#[test]
fn test_scoring_exactly_at_gateway_60() {
    // Need exactly 60%: e.g. Posix(20) + Det(15) + Sec(20) = 55 total weight
    // If Posix(20) + Det(15) pass = 35 passed, 35/55 = 63.6% → above gateway
    let results = vec![
        RuleResult {
            rule: RuleId::Posix,
            passed: true,
            violations: vec![],
        },
        RuleResult {
            rule: RuleId::Determinism,
            passed: true,
            violations: vec![],
        },
        RuleResult {
            rule: RuleId::Security,
            passed: false,
            violations: vec![Violation {
                rule: RuleId::Security,
                line: Some(1),
                message: "test".into(),
            }],
        },
    ];
    let score = compute_artifact_score("test.sh", &results);
    assert!(score.score >= 60.0, "63.6% should be above gateway");
    assert_eq!(score.grade, Grade::C); // 63.6% is grade C (50-70)
}

#[test]
fn test_project_score_average_calculation() {
    let scores = vec![
        ArtifactScore {
            artifact_name: "a.sh".into(),
            score: 100.0,
            grade: Grade::APlus,
            rules_tested: 3,
            rules_passed: 3,
            violations: 0,
            results: vec![],
        },
        ArtifactScore {
            artifact_name: "b.sh".into(),
            score: 60.0,
            grade: Grade::C,
            rules_tested: 3,
            rules_passed: 2,
            violations: 1,
            results: vec![],
        },
        ArtifactScore {
            artifact_name: "c.sh".into(),
            score: 80.0,
            grade: Grade::B,
            rules_tested: 3,
            rules_passed: 2,
            violations: 1,
            results: vec![],
        },
    ];
    let project = compute_project_score(scores);
    assert_eq!(project.total_artifacts, 3);
    assert_eq!(project.compliant_artifacts, 1); // only a.sh has 0 violations
    assert_eq!(project.score, 80.0); // (100 + 60 + 80) / 3 = 80
    assert_eq!(project.grade, Grade::B);
}

// ═══════════════════════════════════════════════════════════════
// Phase 1 Completion: Rule edge case tests
// ═══════════════════════════════════════════════════════════════

