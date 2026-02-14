#![allow(clippy::unwrap_used)]

use super::config::*;
use super::discovery::*;
use super::rules::*;
use super::runner;
use super::scoring::*;
use std::path::PathBuf;

// ─── F-001: Empty project produces score 0, no crash ───
#[test]
fn test_f001_empty_project_no_crash() {
    let config = ComplyConfig::new_default("7.1.0");
    let score = runner::run_check(std::path::Path::new("/nonexistent"), None, &config);
    assert_eq!(score.total_artifacts, 0);
    assert_eq!(score.grade, Grade::APlus); // vacuously true
}

// ─── F-002: Project with no shell files is vacuously compliant ───
#[test]
fn test_f002_no_shell_files_vacuously_compliant() {
    let scores: Vec<ArtifactScore> = vec![];
    let project = compute_project_score(scores);
    assert_eq!(project.score, 100.0);
    assert_eq!(project.grade, Grade::APlus);
}

// ─── F-003: $RANDOM detected falsifies COMPLY-002 ───
#[test]
fn test_f003_random_falsifies_determinism() {
    let content = "#!/bin/sh\nSESSION=$RANDOM\necho $SESSION\n";
    let artifact = Artifact::new(PathBuf::from("test.sh"), Scope::Project, ArtifactKind::ShellScript);
    let result = check_rule(RuleId::Determinism, content, &artifact);
    assert!(!result.passed, "COMPLY-002 should be falsified by $RANDOM");
    assert!(!result.violations.is_empty());
}

// ─── F-004: mkdir without -p falsifies COMPLY-003 ───
#[test]
fn test_f004_mkdir_no_p_falsifies_idempotency() {
    let content = "#!/bin/sh\nmkdir /foo/bar\n";
    let artifact = Artifact::new(PathBuf::from("test.sh"), Scope::Project, ArtifactKind::ShellScript);
    let result = check_rule(RuleId::Idempotency, content, &artifact);
    assert!(!result.passed, "COMPLY-003 should be falsified by mkdir without -p");
}

// ─── F-004b: mkdir -p is compliant ───
#[test]
fn test_f004b_mkdir_p_is_compliant() {
    let content = "#!/bin/sh\nmkdir -p /foo/bar\n";
    let artifact = Artifact::new(PathBuf::from("test.sh"), Scope::Project, ArtifactKind::ShellScript);
    let result = check_rule(RuleId::Idempotency, content, &artifact);
    assert!(result.passed, "mkdir -p should be compliant");
}

// ─── F-005: eval with variable falsifies COMPLY-004 ───
#[test]
fn test_f005_eval_falsifies_security() {
    let content = "#!/bin/sh\neval \"$USER_INPUT\"\n";
    let artifact = Artifact::new(PathBuf::from("test.sh"), Scope::Project, ArtifactKind::ShellScript);
    let result = check_rule(RuleId::Security, content, &artifact);
    assert!(!result.passed, "COMPLY-004 should be falsified by eval with variable");
}

// ─── F-005b: curl | bash falsifies COMPLY-004 ───
#[test]
fn test_f005b_curl_pipe_bash_falsifies_security() {
    let content = "#!/bin/sh\ncurl https://example.com/install.sh | bash\n";
    let artifact = Artifact::new(PathBuf::from("test.sh"), Scope::Project, ArtifactKind::ShellScript);
    let result = check_rule(RuleId::Security, content, &artifact);
    assert!(!result.passed, "curl | bash should be falsified");
}

// ─── F-006: Unquoted variable falsifies COMPLY-005 ───
#[test]
fn test_f006_unquoted_var_falsifies_quoting() {
    let content = "#!/bin/sh\necho $MYVAR\n";
    let artifact = Artifact::new(PathBuf::from("test.sh"), Scope::Project, ArtifactKind::ShellScript);
    let result = check_rule(RuleId::Quoting, content, &artifact);
    assert!(!result.passed, "COMPLY-005 should be falsified by unquoted $MYVAR");
}

// ─── F-006b: Quoted variable is compliant ───
#[test]
fn test_f006b_quoted_var_is_compliant() {
    let content = "#!/bin/sh\necho \"$MYVAR\"\n";
    let artifact = Artifact::new(PathBuf::from("test.sh"), Scope::Project, ArtifactKind::ShellScript);
    let result = check_rule(RuleId::Quoting, content, &artifact);
    assert!(result.passed, "Quoted variable should be compliant");
}

// ─── COMPLY-001: POSIX detection ───
#[test]
fn test_comply_001_bash_shebang_non_posix() {
    let content = "#!/bin/bash\necho hello\n";
    let artifact = Artifact::new(PathBuf::from("test.sh"), Scope::Project, ArtifactKind::ShellScript);
    let result = check_rule(RuleId::Posix, content, &artifact);
    assert!(!result.passed, "#!/bin/bash should be non-POSIX");
}

#[test]
fn test_comply_001_sh_shebang_is_posix() {
    let content = "#!/bin/sh\necho hello\n";
    let artifact = Artifact::new(PathBuf::from("test.sh"), Scope::Project, ArtifactKind::ShellScript);
    let result = check_rule(RuleId::Posix, content, &artifact);
    assert!(result.passed, "#!/bin/sh should be POSIX compliant");
}

#[test]
fn test_comply_001_double_brackets_non_posix() {
    let content = "#!/bin/sh\nif [[ -f /etc/passwd ]]; then echo yes; fi\n";
    let artifact = Artifact::new(PathBuf::from("test.sh"), Scope::Project, ArtifactKind::ShellScript);
    let result = check_rule(RuleId::Posix, content, &artifact);
    assert!(!result.passed, "[[ ]] should be non-POSIX");
}

// ─── COMPLY-007: Makefile safety ───
#[test]
fn test_comply_007_eval_in_makefile_recipe() {
    let content = "all:\n\teval \"$(GENERATED)\"\n";
    let artifact = Artifact::new(PathBuf::from("Makefile"), Scope::Project, ArtifactKind::Makefile);
    let result = check_rule(RuleId::MakefileSafety, content, &artifact);
    assert!(!result.passed, "eval in Makefile recipe should fail");
}

// ─── COMPLY-008: Dockerfile best practices ───
#[test]
fn test_comply_008_dockerfile_missing_user() {
    let content = "FROM ubuntu:22.04\nRUN apt-get update\n";
    let artifact = Artifact::new(PathBuf::from("Dockerfile"), Scope::Project, ArtifactKind::Dockerfile);
    let result = check_rule(RuleId::DockerfileBest, content, &artifact);
    assert!(!result.passed, "Dockerfile without USER should fail");
}

#[test]
fn test_comply_008_dockerfile_add_instead_of_copy() {
    let content = "FROM ubuntu:22.04\nADD ./app /app\nUSER nobody\n";
    let artifact = Artifact::new(PathBuf::from("Dockerfile"), Scope::Project, ArtifactKind::Dockerfile);
    let result = check_rule(RuleId::DockerfileBest, content, &artifact);
    assert!(!result.passed, "ADD for local files should fail");
}

// ─── COMPLY-009: Config hygiene ───
#[test]
fn test_comply_009_too_many_path_exports() {
    let content = "export PATH=/a:$PATH\nexport PATH=/b:$PATH\nexport PATH=/c:$PATH\nexport PATH=/d:$PATH\n";
    let artifact = Artifact::new(PathBuf::from(".zshrc"), Scope::User, ArtifactKind::ShellConfig);
    let result = check_rule(RuleId::ConfigHygiene, content, &artifact);
    assert!(!result.passed, "4+ PATH modifications should flag config hygiene");
}

// ─── Scoring ───
#[test]
fn test_scoring_perfect_score() {
    let results = vec![
        RuleResult { rule: RuleId::Posix, passed: true, violations: vec![] },
        RuleResult { rule: RuleId::Determinism, passed: true, violations: vec![] },
        RuleResult { rule: RuleId::Security, passed: true, violations: vec![] },
    ];
    let score = compute_artifact_score("test.sh", &results);
    assert_eq!(score.score, 100.0);
    assert_eq!(score.grade, Grade::APlus);
}

#[test]
fn test_scoring_gateway_barrier() {
    // Only security passes out of posix(20) + det(15) + idem(15) + sec(20) + quoting(10) + sc(10) = 90
    // sec(20)/90 = 22% → below 60% gateway → score capped at 22% * 0.4 = 8.9%
    let results = vec![
        RuleResult { rule: RuleId::Posix, passed: false, violations: vec![Violation { rule: RuleId::Posix, line: Some(1), message: "test".into() }] },
        RuleResult { rule: RuleId::Determinism, passed: false, violations: vec![Violation { rule: RuleId::Determinism, line: Some(1), message: "test".into() }] },
        RuleResult { rule: RuleId::Idempotency, passed: false, violations: vec![Violation { rule: RuleId::Idempotency, line: Some(1), message: "test".into() }] },
        RuleResult { rule: RuleId::Security, passed: true, violations: vec![] },
        RuleResult { rule: RuleId::Quoting, passed: false, violations: vec![Violation { rule: RuleId::Quoting, line: Some(1), message: "test".into() }] },
        RuleResult { rule: RuleId::ShellCheck, passed: false, violations: vec![Violation { rule: RuleId::ShellCheck, line: Some(1), message: "test".into() }] },
    ];
    let score = compute_artifact_score("bad.sh", &results);
    assert!(score.score < 60.0, "Score below gateway should be capped");
    assert_eq!(score.grade, Grade::F);
}

#[test]
fn test_scoring_project_aggregate() {
    let scores = vec![
        ArtifactScore {
            artifact_name: "a.sh".into(),
            score: 100.0,
            grade: Grade::APlus,
            rules_tested: 6,
            rules_passed: 6,
            violations: 0,
            results: vec![],
        },
        ArtifactScore {
            artifact_name: "b.sh".into(),
            score: 80.0,
            grade: Grade::A,
            rules_tested: 6,
            rules_passed: 5,
            violations: 1,
            results: vec![],
        },
    ];
    let project = compute_project_score(scores);
    assert_eq!(project.total_artifacts, 2);
    assert_eq!(project.compliant_artifacts, 1);
    assert_eq!(project.score, 90.0);
    assert_eq!(project.grade, Grade::A); // 90.0 is grade A (A+ requires >= 95.0)
}

// ─── Discovery ───
#[test]
fn test_classify_shell_script() {
    assert_eq!(classify(std::path::Path::new("test.sh")), Some(ArtifactKind::ShellScript));
}

#[test]
fn test_classify_makefile() {
    assert_eq!(classify(std::path::Path::new("Makefile")), Some(ArtifactKind::Makefile));
}

#[test]
fn test_classify_dockerfile() {
    assert_eq!(classify(std::path::Path::new("Dockerfile")), Some(ArtifactKind::Dockerfile));
}

#[test]
fn test_classify_mk_extension() {
    assert_eq!(classify(std::path::Path::new("rules.mk")), Some(ArtifactKind::Makefile));
}

// ─── Rule applicability ───
#[test]
fn test_shell_script_has_all_core_rules() {
    let rules = RuleId::applicable_rules(ArtifactKind::ShellScript);
    assert!(rules.contains(&RuleId::Posix));
    assert!(rules.contains(&RuleId::Determinism));
    assert!(rules.contains(&RuleId::Idempotency));
    assert!(rules.contains(&RuleId::Security));
    assert!(rules.contains(&RuleId::Quoting));
    assert!(rules.contains(&RuleId::ShellCheck));
}

#[test]
fn test_makefile_has_makefile_safety() {
    let rules = RuleId::applicable_rules(ArtifactKind::Makefile);
    assert!(rules.contains(&RuleId::MakefileSafety));
    assert!(!rules.contains(&RuleId::Posix));
}

#[test]
fn test_dockerfile_has_dockerfile_best() {
    let rules = RuleId::applicable_rules(ArtifactKind::Dockerfile);
    assert!(rules.contains(&RuleId::DockerfileBest));
    assert!(rules.contains(&RuleId::Security));
}

#[test]
fn test_config_has_hygiene() {
    let rules = RuleId::applicable_rules(ArtifactKind::ShellConfig);
    assert!(rules.contains(&RuleId::ConfigHygiene));
    assert!(rules.contains(&RuleId::Security));
}

// ─── Config ───
#[test]
fn test_config_default_creation() {
    let config = ComplyConfig::new_default("7.1.0");
    assert_eq!(config.comply.version, "1.0.0");
    assert_eq!(config.comply.bashrs_version, "7.1.0");
    assert!(config.scopes.project);
    assert!(!config.scopes.user);
    assert!(config.rules.posix);
    assert!(config.rules.security);
}

// ─── Format output ───
#[test]
fn test_format_human_no_crash() {
    let score = compute_project_score(vec![]);
    let output = runner::format_human(&score);
    assert!(output.contains("COMPLIANCE CHECK"));
    assert!(output.contains("Layer 1"));
}

#[test]
fn test_format_json_valid() {
    let score = compute_project_score(vec![]);
    let output = runner::format_json(&score);
    assert!(output.contains("bashrs-comply-check-v1"));
    assert!(output.contains("\"total_artifacts\":0"));
}

// ─── Grade display ───
#[test]
fn test_grade_display() {
    assert_eq!(format!("{}", Grade::APlus), "A+");
    assert_eq!(format!("{}", Grade::A), "A");
    assert_eq!(format!("{}", Grade::B), "B");
    assert_eq!(format!("{}", Grade::C), "C");
    assert_eq!(format!("{}", Grade::F), "F");
}

#[test]
fn test_grade_from_score_boundaries() {
    assert_eq!(Grade::from_score(100.0), Grade::APlus);
    assert_eq!(Grade::from_score(95.0), Grade::APlus);
    assert_eq!(Grade::from_score(94.9), Grade::A);
    assert_eq!(Grade::from_score(85.0), Grade::A);
    assert_eq!(Grade::from_score(84.9), Grade::B);
    assert_eq!(Grade::from_score(70.0), Grade::B);
    assert_eq!(Grade::from_score(69.9), Grade::C);
    assert_eq!(Grade::from_score(50.0), Grade::C);
    assert_eq!(Grade::from_score(49.9), Grade::F);
    assert_eq!(Grade::from_score(0.0), Grade::F);
}

// ─── Violation display ───
#[test]
fn test_violation_display_with_line() {
    let v = Violation { rule: RuleId::Determinism, line: Some(14), message: "$RANDOM found".into() };
    let s = format!("{v}");
    assert!(s.contains("COMPLY-002"));
    assert!(s.contains("line 14"));
    assert!(s.contains("$RANDOM found"));
}

#[test]
fn test_violation_display_without_line() {
    let v = Violation { rule: RuleId::DockerfileBest, line: None, message: "Missing USER".into() };
    let s = format!("{v}");
    assert!(s.contains("COMPLY-008"));
    assert!(s.contains("Missing USER"));
}

// ─── Comments are skipped ───
#[test]
fn test_comments_not_flagged() {
    let content = "#!/bin/sh\n# $RANDOM is used for demo\necho hello\n";
    let artifact = Artifact::new(PathBuf::from("test.sh"), Scope::Project, ArtifactKind::ShellScript);
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
    let artifact = Artifact::new(PathBuf::from("test.sh"), Scope::Project, ArtifactKind::ShellScript);
    let result = check_rule(RuleId::ShellCheck, content, &artifact);
    assert!(!result.passed, "Backticks should be flagged as SC2006");
}

// ─── Clean script passes all rules ───
#[test]
fn test_clean_script_passes_all() {
    let content = "#!/bin/sh\necho \"hello world\"\nmkdir -p /tmp/test\n";
    let artifact = Artifact::new(PathBuf::from("clean.sh"), Scope::Project, ArtifactKind::ShellScript);

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
    let artifact = Artifact::new(PathBuf::from("Makefile"), Scope::Project, ArtifactKind::Makefile);
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
    let artifact = Artifact::new(PathBuf::from("Makefile"), Scope::Project, ArtifactKind::Makefile);
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
    let artifact = Artifact::new(PathBuf::from("Makefile"), Scope::Project, ArtifactKind::Makefile);
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
    let artifact = Artifact::new(PathBuf::from("script.sh"), Scope::Project, ArtifactKind::ShellScript);
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
    let artifact = Artifact::new(PathBuf::from("Makefile"), Scope::Project, ArtifactKind::Makefile);
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
    let artifact = Artifact::new(PathBuf::from("validate.sh"), Scope::Project, ArtifactKind::ShellScript);
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
    let artifact = Artifact::new(PathBuf::from("process.sh"), Scope::Project, ArtifactKind::ShellScript);
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
    let artifact = Artifact::new(PathBuf::from("deploy.sh"), Scope::Project, ArtifactKind::ShellScript);
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
    let artifact = Artifact::new(PathBuf::from("test.sh"), Scope::Project, ArtifactKind::ShellScript);
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
    let artifact = Artifact::new(PathBuf::from("test.sh"), Scope::Project, ArtifactKind::ShellScript);
    let result = check_rule(RuleId::Security, content, &artifact);
    assert!(
        !result.passed,
        "eval after ; should still be flagged"
    );
}

#[test]
fn test_gh135_eval_after_and_still_flagged() {
    // eval after && is still a command
    let content = "#!/bin/sh\ntest -f config && eval \"$CONFIG\"\n";
    let artifact = Artifact::new(PathBuf::from("test.sh"), Scope::Project, ArtifactKind::ShellScript);
    let result = check_rule(RuleId::Security, content, &artifact);
    assert!(
        !result.passed,
        "eval after && should still be flagged"
    );
}

#[test]
fn test_gh135_workflow_yq_eval_not_flagged() {
    // yq eval in GitHub workflow YAML should not be flagged
    let content = "    - name: Validate\n      run: |\n        for f in playbooks/**/*.yaml; do\n          yq eval '.' \"$f\" > /dev/null\n        done\n";
    let artifact = Artifact::new(PathBuf::from(".github/workflows/ci.yml"), Scope::Project, ArtifactKind::Workflow);
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
    assert!(
        !result.passed,
        "eval after || should still be flagged"
    );
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
fn test_config_load_returns_none_on_corrupt() {
    let dir = tempfile::tempdir().unwrap();
    let config_dir = dir.path().join(".bashrs");
    std::fs::create_dir_all(&config_dir).unwrap();
    std::fs::write(config_dir.join("comply.toml"), "this is not valid toml {{{{").unwrap();
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
    assert!(score.score < 100.0, "Script with $RANDOM should score below 100");
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
    let posix_violations: usize = score.artifact_scores.iter()
        .flat_map(|a| a.results.iter())
        .filter(|r| r.rule == RuleId::Posix)
        .map(|r| r.violations.len())
        .sum();
    assert_eq!(posix_violations, 0, "Disabled POSIX rule should not produce violations");
}

#[test]
fn test_runner_user_scope_disabled_returns_empty() {
    let dir = tempfile::tempdir().unwrap();
    let mut config = ComplyConfig::new_default("7.1.0");
    config.scopes.user = false;
    let score = runner::run_check(dir.path(), Some(Scope::User), &config);
    assert_eq!(score.total_artifacts, 0, "Disabled user scope should return no artifacts");
}

#[test]
fn test_runner_system_scope_disabled_returns_empty() {
    let dir = tempfile::tempdir().unwrap();
    let mut config = ComplyConfig::new_default("7.1.0");
    config.scopes.system = false;
    let score = runner::run_check(dir.path(), Some(Scope::System), &config);
    assert_eq!(score.total_artifacts, 0, "Disabled system scope should return no artifacts");
}

#[test]
fn test_runner_multiple_artifacts() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("a.sh"), "#!/bin/sh\necho a\n").unwrap();
    std::fs::write(dir.path().join("b.sh"), "#!/bin/sh\necho b\n").unwrap();
    std::fs::write(dir.path().join("Makefile"), "all:\n\techo done\n").unwrap();
    let config = ComplyConfig::new_default("7.1.0");
    let score = runner::run_check(dir.path(), Some(Scope::Project), &config);
    assert!(score.total_artifacts >= 3, "Should find at least 3 artifacts, found {}", score.total_artifacts);
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
    std::fs::write(dir.path().join("bad.sh"), "#!/bin/bash\neval \"$USER_INPUT\"\nmkdir /foo\n").unwrap();
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
    assert!(artifacts[0].path.to_string_lossy().contains("scripts/build.sh"));
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
    let count = artifacts.iter().filter(|a| a.path.to_string_lossy().contains("install.sh")).count();
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
    assert_eq!(classify(std::path::Path::new(".bashrc")), Some(ArtifactKind::ShellConfig));
    assert_eq!(classify(std::path::Path::new(".zshrc")), Some(ArtifactKind::ShellConfig));
    assert_eq!(classify(std::path::Path::new(".profile")), Some(ArtifactKind::ShellConfig));
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
    let artifact = Artifact::new(PathBuf::from("scripts/build.sh"), Scope::Project, ArtifactKind::ShellScript);
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
        RuleResult { rule: RuleId::Security, passed: false, violations: vec![
            Violation { rule: RuleId::Security, line: Some(1), message: "test".into() }
        ]},
        RuleResult { rule: RuleId::Determinism, passed: true, violations: vec![] },
    ];
    let score = compute_artifact_score("test.sh", &results);
    assert!(score.score < 60.0, "Below gateway should be capped");
}

#[test]
fn test_scoring_exactly_at_gateway_60() {
    // Need exactly 60%: e.g. Posix(20) + Det(15) + Sec(20) = 55 total weight
    // If Posix(20) + Det(15) pass = 35 passed, 35/55 = 63.6% → above gateway
    let results = vec![
        RuleResult { rule: RuleId::Posix, passed: true, violations: vec![] },
        RuleResult { rule: RuleId::Determinism, passed: true, violations: vec![] },
        RuleResult { rule: RuleId::Security, passed: false, violations: vec![
            Violation { rule: RuleId::Security, line: Some(1), message: "test".into() }
        ]},
    ];
    let score = compute_artifact_score("test.sh", &results);
    assert!(score.score >= 60.0, "63.6% should be above gateway");
    assert_eq!(score.grade, Grade::C); // 63.6% is grade C (50-70)
}

#[test]
fn test_project_score_average_calculation() {
    let scores = vec![
        ArtifactScore {
            artifact_name: "a.sh".into(), score: 100.0, grade: Grade::APlus,
            rules_tested: 3, rules_passed: 3, violations: 0, results: vec![],
        },
        ArtifactScore {
            artifact_name: "b.sh".into(), score: 60.0, grade: Grade::C,
            rules_tested: 3, rules_passed: 2, violations: 1, results: vec![],
        },
        ArtifactScore {
            artifact_name: "c.sh".into(), score: 80.0, grade: Grade::B,
            rules_tested: 3, rules_passed: 2, violations: 1, results: vec![],
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

#[test]
fn test_idempotency_makefile_not_checked() {
    // Idempotency rules only apply to ShellScript and ShellConfig, not Makefile
    let content = "all:\n\tmkdir /tmp/build\n";
    let artifact = Artifact::new(PathBuf::from("Makefile"), Scope::Project, ArtifactKind::Makefile);
    let result = check_rule(RuleId::Idempotency, content, &artifact);
    assert!(result.passed, "Makefile should skip idempotency check");
}

#[test]
fn test_idempotency_rm_rf_is_fine() {
    let content = "#!/bin/sh\nrm -rf /tmp/build\n";
    let artifact = Artifact::new(PathBuf::from("test.sh"), Scope::Project, ArtifactKind::ShellScript);
    let result = check_rule(RuleId::Idempotency, content, &artifact);
    assert!(result.passed, "rm -rf should be considered idempotent");
}

#[test]
fn test_idempotency_ln_sf_is_fine() {
    let content = "#!/bin/sh\nln -sf /src /dst\n";
    let artifact = Artifact::new(PathBuf::from("test.sh"), Scope::Project, ArtifactKind::ShellScript);
    let result = check_rule(RuleId::Idempotency, content, &artifact);
    assert!(result.passed, "ln -sf should be considered idempotent");
}

#[test]
fn test_idempotency_ln_s_without_f_fails() {
    let content = "#!/bin/sh\nln -s /src /dst\n";
    let artifact = Artifact::new(PathBuf::from("test.sh"), Scope::Project, ArtifactKind::ShellScript);
    let result = check_rule(RuleId::Idempotency, content, &artifact);
    assert!(!result.passed, "ln -s without -f should fail idempotency");
}

#[test]
fn test_determinism_date_patterns() {
    let content = "#!/bin/sh\nTIMESTAMP=$(date +%s)\n";
    let artifact = Artifact::new(PathBuf::from("test.sh"), Scope::Project, ArtifactKind::ShellScript);
    let result = check_rule(RuleId::Determinism, content, &artifact);
    assert!(!result.passed, "date +%s should be flagged as non-deterministic");
}

#[test]
fn test_determinism_date_nano() {
    let content = "#!/bin/sh\nNANO=$(date +%N)\n";
    let artifact = Artifact::new(PathBuf::from("test.sh"), Scope::Project, ArtifactKind::ShellScript);
    let result = check_rule(RuleId::Determinism, content, &artifact);
    assert!(!result.passed, "date +%N should be flagged as non-deterministic");
}

#[test]
fn test_security_wget_pipe_sh() {
    let content = "#!/bin/sh\nwget -q https://example.com/setup.sh | sh\n";
    let artifact = Artifact::new(PathBuf::from("test.sh"), Scope::Project, ArtifactKind::ShellScript);
    let result = check_rule(RuleId::Security, content, &artifact);
    assert!(!result.passed, "wget | sh should be flagged as SEC002");
}

#[test]
fn test_shellcheck_dangerous_rm_rf() {
    let content = "#!/bin/sh\nrm -rf /$DIR\n";
    let artifact = Artifact::new(PathBuf::from("test.sh"), Scope::Project, ArtifactKind::ShellScript);
    let result = check_rule(RuleId::ShellCheck, content, &artifact);
    assert!(!result.passed, "rm -rf with variable path should be flagged as SC2115");
}

#[test]
fn test_dockerfile_add_http_is_ok() {
    let content = "FROM ubuntu:22.04\nADD https://example.com/file.tar.gz /app/\nUSER nobody\n";
    let artifact = Artifact::new(PathBuf::from("Dockerfile"), Scope::Project, ArtifactKind::Dockerfile);
    let result = check_rule(RuleId::DockerfileBest, content, &artifact);
    assert!(result.passed, "ADD with HTTP URL should be allowed");
}

#[test]
fn test_pzsh_budget_always_passes() {
    let content = "anything here";
    let artifact = Artifact::new(PathBuf::from("test.sh"), Scope::Project, ArtifactKind::ShellScript);
    let result = check_rule(RuleId::PzshBudget, content, &artifact);
    assert!(result.passed, "PzshBudget should always pass (handled externally)");
}

#[test]
fn test_rule_id_codes_complete() {
    // Verify all 10 rules have unique codes
    let rules = vec![
        RuleId::Posix, RuleId::Determinism, RuleId::Idempotency,
        RuleId::Security, RuleId::Quoting, RuleId::ShellCheck,
        RuleId::MakefileSafety, RuleId::DockerfileBest,
        RuleId::ConfigHygiene, RuleId::PzshBudget,
    ];
    let codes: Vec<&str> = rules.iter().map(|r| r.code()).collect();
    assert_eq!(codes.len(), 10);
    // Verify sequential COMPLY-001 through COMPLY-010
    for (i, code) in codes.iter().enumerate() {
        assert_eq!(*code, format!("COMPLY-{:03}", i + 1));
    }
}

#[test]
fn test_rule_weights_sum_to_110() {
    // Total weight pool: 20+15+15+20+10+10+5+5+5+5 = 110
    let rules = vec![
        RuleId::Posix, RuleId::Determinism, RuleId::Idempotency,
        RuleId::Security, RuleId::Quoting, RuleId::ShellCheck,
        RuleId::MakefileSafety, RuleId::DockerfileBest,
        RuleId::ConfigHygiene, RuleId::PzshBudget,
    ];
    let total: u32 = rules.iter().map(|r| r.weight()).sum();
    assert_eq!(total, 110, "Total weight pool should be 110");
}

#[test]
fn test_devcontainer_has_no_applicable_rules() {
    let rules = RuleId::applicable_rules(ArtifactKind::DevContainer);
    assert!(rules.is_empty(), "DevContainer should have no applicable rules");
}

#[test]
fn test_workflow_only_has_security() {
    let rules = RuleId::applicable_rules(ArtifactKind::Workflow);
    assert_eq!(rules.len(), 1);
    assert_eq!(rules[0], RuleId::Security);
}

// ═══════════════════════════════════════════════════════════════
// COMPLY-005 quote tracker: escaped quotes and subshell handling
// ═══════════════════════════════════════════════════════════════

// ═══════════════════════════════════════════════════════════════
// SEC004: TLS verification disabled
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_sec004_wget_no_check_certificate() {
    let content = "#!/bin/sh\nwget --no-check-certificate https://example.com/file\n";
    let artifact = Artifact::new(PathBuf::from("test.sh"), Scope::Project, ArtifactKind::ShellScript);
    let result = check_rule(RuleId::Security, content, &artifact);
    assert!(!result.passed, "SEC004: --no-check-certificate should be flagged");
    assert!(result.violations.iter().any(|v| v.message.contains("SEC004")));
}

#[test]
fn test_sec004_curl_insecure() {
    let content = "#!/bin/sh\ncurl --insecure https://api.example.com/data\n";
    let artifact = Artifact::new(PathBuf::from("test.sh"), Scope::Project, ArtifactKind::ShellScript);
    let result = check_rule(RuleId::Security, content, &artifact);
    assert!(!result.passed, "SEC004: --insecure should be flagged");
}

#[test]
fn test_sec004_curl_k_flag() {
    let content = "#!/bin/sh\ncurl -k https://api.example.com/data\n";
    let artifact = Artifact::new(PathBuf::from("test.sh"), Scope::Project, ArtifactKind::ShellScript);
    let result = check_rule(RuleId::Security, content, &artifact);
    assert!(!result.passed, "SEC004: curl -k should be flagged");
}

#[test]
fn test_sec004_curl_without_k_is_ok() {
    let content = "#!/bin/sh\ncurl https://api.example.com/data\n";
    let artifact = Artifact::new(PathBuf::from("test.sh"), Scope::Project, ArtifactKind::ShellScript);
    let result = check_rule(RuleId::Security, content, &artifact);
    assert!(result.passed, "curl without TLS flags should pass");
}

// ═══════════════════════════════════════════════════════════════
// SEC005: Hardcoded secrets
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_sec005_hardcoded_api_key() {
    let content = "#!/bin/sh\nAPI_KEY=\"sk-1234567890abcdef\"\n";
    let artifact = Artifact::new(PathBuf::from("test.sh"), Scope::Project, ArtifactKind::ShellScript);
    let result = check_rule(RuleId::Security, content, &artifact);
    assert!(!result.passed, "SEC005: hardcoded API_KEY should be flagged");
    assert!(result.violations.iter().any(|v| v.message.contains("SEC005")));
}

#[test]
fn test_sec005_hardcoded_password() {
    let content = "#!/bin/sh\nPASSWORD=\"MyS3cret!\"\n";
    let artifact = Artifact::new(PathBuf::from("test.sh"), Scope::Project, ArtifactKind::ShellScript);
    let result = check_rule(RuleId::Security, content, &artifact);
    assert!(!result.passed, "SEC005: hardcoded PASSWORD should be flagged");
}

#[test]
fn test_sec005_github_token_prefix() {
    let content = "#!/bin/sh\nTOKEN=\"ghp_xxxxxxxxxxxxxxxxxxxx\"\n";
    let artifact = Artifact::new(PathBuf::from("test.sh"), Scope::Project, ArtifactKind::ShellScript);
    let result = check_rule(RuleId::Security, content, &artifact);
    assert!(!result.passed, "SEC005: ghp_ token prefix should be flagged");
}

#[test]
fn test_sec005_variable_expansion_not_flagged() {
    let content = "#!/bin/sh\nAPI_KEY=\"$MY_API_KEY\"\n";
    let artifact = Artifact::new(PathBuf::from("test.sh"), Scope::Project, ArtifactKind::ShellScript);
    let result = check_rule(RuleId::Security, content, &artifact);
    // Should not flag variable expansion as hardcoded secret
    let sec005_violations: Vec<_> = result.violations.iter()
        .filter(|v| v.message.contains("SEC005"))
        .collect();
    assert!(sec005_violations.is_empty(), "Variable expansion should not trigger SEC005: {:?}", sec005_violations);
}

#[test]
fn test_sec005_empty_value_not_flagged() {
    let content = "#!/bin/sh\nAPI_KEY=\"\"\n";
    let artifact = Artifact::new(PathBuf::from("test.sh"), Scope::Project, ArtifactKind::ShellScript);
    let result = check_rule(RuleId::Security, content, &artifact);
    let sec005_violations: Vec<_> = result.violations.iter()
        .filter(|v| v.message.contains("SEC005"))
        .collect();
    assert!(sec005_violations.is_empty(), "Empty value should not trigger SEC005");
}

// ═══════════════════════════════════════════════════════════════
// SEC006: Unsafe temporary files
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_sec006_unsafe_tmp_path() {
    let content = "#!/bin/sh\nTMPFILE=\"/tmp/myapp.tmp\"\n";
    let artifact = Artifact::new(PathBuf::from("test.sh"), Scope::Project, ArtifactKind::ShellScript);
    let result = check_rule(RuleId::Security, content, &artifact);
    assert!(!result.passed, "SEC006: /tmp/ literal path should be flagged");
    assert!(result.violations.iter().any(|v| v.message.contains("SEC006")));
}

#[test]
fn test_sec006_mktemp_is_ok() {
    let content = "#!/bin/sh\nTMPFILE=\"$(mktemp)\"\n";
    let artifact = Artifact::new(PathBuf::from("test.sh"), Scope::Project, ArtifactKind::ShellScript);
    let result = check_rule(RuleId::Security, content, &artifact);
    assert!(result.passed, "mktemp usage should not be flagged");
}

// ═══════════════════════════════════════════════════════════════
// SEC007: sudo with dangerous command
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_sec007_sudo_rm_rf_unquoted() {
    let content = "#!/bin/sh\nsudo rm -rf $DIR\n";
    let artifact = Artifact::new(PathBuf::from("test.sh"), Scope::Project, ArtifactKind::ShellScript);
    let result = check_rule(RuleId::Security, content, &artifact);
    assert!(!result.passed, "SEC007: sudo rm -rf with unquoted var should be flagged");
    assert!(result.violations.iter().any(|v| v.message.contains("SEC007")));
}

#[test]
fn test_sec007_sudo_chmod_777() {
    let content = "#!/bin/sh\nsudo chmod 777 $FILE\n";
    let artifact = Artifact::new(PathBuf::from("test.sh"), Scope::Project, ArtifactKind::ShellScript);
    let result = check_rule(RuleId::Security, content, &artifact);
    assert!(!result.passed, "SEC007: sudo chmod 777 with unquoted var should be flagged");
}

#[test]
fn test_sec007_sudo_rm_rf_quoted_is_ok() {
    let content = "#!/bin/sh\nsudo rm -rf \"$DIR\"\n";
    let artifact = Artifact::new(PathBuf::from("test.sh"), Scope::Project, ArtifactKind::ShellScript);
    let result = check_rule(RuleId::Security, content, &artifact);
    let sec007_violations: Vec<_> = result.violations.iter()
        .filter(|v| v.message.contains("SEC007"))
        .collect();
    assert!(sec007_violations.is_empty(), "Quoted variable with sudo should not trigger SEC007");
}

#[test]
fn test_sec007_sudo_safe_command_not_flagged() {
    let content = "#!/bin/sh\nsudo apt-get update\n";
    let artifact = Artifact::new(PathBuf::from("test.sh"), Scope::Project, ArtifactKind::ShellScript);
    let result = check_rule(RuleId::Security, content, &artifact);
    let sec007_violations: Vec<_> = result.violations.iter()
        .filter(|v| v.message.contains("SEC007"))
        .collect();
    assert!(sec007_violations.is_empty(), "sudo with safe command should not trigger SEC007");
}

#[test]
fn test_quoting_escaped_quotes_no_false_positive() {
    // echo "echo \"Line $i: Hello\"" — $i is inside double quotes (escaped inner quotes)
    let content = "#!/bin/sh\necho \"echo \\\"Line $i: Hello\\\"\"\n";
    let artifact = Artifact::new(PathBuf::from("test.sh"), Scope::Project, ArtifactKind::ShellScript);
    let result = check_rule(RuleId::Quoting, content, &artifact);
    assert!(result.passed, "Escaped quotes should not cause false positive: {:?}", result.violations);
}

#[test]
fn test_quoting_subshell_no_false_positive() {
    // SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
    let content = "#!/bin/sh\nSCRIPT_DIR=\"$(cd \"$(dirname \"${BASH_SOURCE[0]}\")\" && pwd)\"\n";
    let artifact = Artifact::new(PathBuf::from("test.sh"), Scope::Project, ArtifactKind::ShellScript);
    let result = check_rule(RuleId::Quoting, content, &artifact);
    assert!(result.passed, "Subshell with nested quotes should not flag: {:?}", result.violations);
}

#[test]
fn test_quoting_simple_subshell_not_flagged() {
    // OUTPUT=$(date +%Y-%m-%d) — inside $() is a separate context
    let content = "#!/bin/sh\nOUTPUT=\"$(date +%Y-%m-%d)\"\n";
    let artifact = Artifact::new(PathBuf::from("test.sh"), Scope::Project, ArtifactKind::ShellScript);
    let result = check_rule(RuleId::Quoting, content, &artifact);
    assert!(result.passed, "Variable in subshell should not be flagged");
}

#[test]
fn test_quoting_unquoted_still_detected() {
    // Plain unquoted $VAR should still be detected
    let content = "#!/bin/sh\necho $UNQUOTED\n";
    let artifact = Artifact::new(PathBuf::from("test.sh"), Scope::Project, ArtifactKind::ShellScript);
    let result = check_rule(RuleId::Quoting, content, &artifact);
    assert!(!result.passed, "Unquoted $UNQUOTED should still be detected");
}

#[test]
fn test_quoting_backslash_dollar_not_flagged() {
    // \$VAR is literal, not an expansion
    let content = "#!/bin/sh\necho \\$NOTAVAR\n";
    let artifact = Artifact::new(PathBuf::from("test.sh"), Scope::Project, ArtifactKind::ShellScript);
    let result = check_rule(RuleId::Quoting, content, &artifact);
    assert!(result.passed, "Escaped \\$VAR should not be flagged: {:?}", result.violations);
}

// ─── COMPLY-001 Bashism Detection Expansion ───

#[test]
fn test_posix_function_keyword_detected() {
    let content = "#!/bin/sh\nfunction greet {\n  echo hello\n}\n";
    let artifact = Artifact::new(PathBuf::from("test.sh"), Scope::Project, ArtifactKind::ShellScript);
    let result = check_rule(RuleId::Posix, content, &artifact);
    assert!(!result.passed, "function keyword should be detected as bashism");
    assert!(result.violations.iter().any(|v| v.message.contains("function keyword")));
}

#[test]
fn test_posix_function_keyword_with_parens_detected() {
    let content = "#!/bin/sh\nfunction greet() {\n  echo hello\n}\n";
    let artifact = Artifact::new(PathBuf::from("test.sh"), Scope::Project, ArtifactKind::ShellScript);
    let result = check_rule(RuleId::Posix, content, &artifact);
    assert!(!result.passed, "function greet() should be detected as bashism");
}

#[test]
fn test_posix_name_parens_no_false_positive() {
    // POSIX-valid function definition: name() { ... }
    let content = "#!/bin/sh\ngreet() {\n  echo hello\n}\n";
    let artifact = Artifact::new(PathBuf::from("test.sh"), Scope::Project, ArtifactKind::ShellScript);
    let result = check_rule(RuleId::Posix, content, &artifact);
    assert!(result.passed, "POSIX name() should not be flagged: {:?}", result.violations);
}

#[test]
fn test_posix_standalone_arithmetic_detected() {
    let content = "#!/bin/sh\n(( i++ ))\n";
    let artifact = Artifact::new(PathBuf::from("test.sh"), Scope::Project, ArtifactKind::ShellScript);
    let result = check_rule(RuleId::Posix, content, &artifact);
    assert!(!result.passed, "(( )) should be detected as bashism");
    assert!(result.violations.iter().any(|v| v.message.contains("(( ))")));
}

#[test]
fn test_posix_dollar_arithmetic_no_false_positive() {
    // $(( )) is POSIX arithmetic expansion
    let content = "#!/bin/sh\nresult=$(( 1 + 2 ))\n";
    let artifact = Artifact::new(PathBuf::from("test.sh"), Scope::Project, ArtifactKind::ShellScript);
    let result = check_rule(RuleId::Posix, content, &artifact);
    assert!(result.passed, "$(( )) should not be flagged: {:?}", result.violations);
}

#[test]
fn test_posix_arithmetic_after_semicolon() {
    let content = "#!/bin/sh\necho start; (( count++ ))\n";
    let artifact = Artifact::new(PathBuf::from("test.sh"), Scope::Project, ArtifactKind::ShellScript);
    let result = check_rule(RuleId::Posix, content, &artifact);
    assert!(!result.passed, "(( )) after semicolon should be detected");
}

#[test]
fn test_posix_herestring_detected() {
    let content = "#!/bin/sh\nread x <<< \"hello\"\n";
    let artifact = Artifact::new(PathBuf::from("test.sh"), Scope::Project, ArtifactKind::ShellScript);
    let result = check_rule(RuleId::Posix, content, &artifact);
    assert!(!result.passed, "<<< here-string should be detected as bashism");
    assert!(result.violations.iter().any(|v| v.message.contains("here-string")));
}

#[test]
fn test_posix_heredoc_no_false_positive() {
    // << heredoc is POSIX
    let content = "#!/bin/sh\ncat << EOF\nhello\nEOF\n";
    let artifact = Artifact::new(PathBuf::from("test.sh"), Scope::Project, ArtifactKind::ShellScript);
    let result = check_rule(RuleId::Posix, content, &artifact);
    assert!(result.passed, "<< heredoc should not be flagged: {:?}", result.violations);
}

#[test]
fn test_posix_select_statement_detected() {
    let content = "#!/bin/sh\nselect opt in a b c; do echo $opt; done\n";
    let artifact = Artifact::new(PathBuf::from("test.sh"), Scope::Project, ArtifactKind::ShellScript);
    let result = check_rule(RuleId::Posix, content, &artifact);
    assert!(!result.passed, "select statement should be detected as bashism");
    assert!(result.violations.iter().any(|v| v.message.contains("select")));
}

#[test]
fn test_posix_pattern_substitution_detected() {
    let content = "#!/bin/sh\necho ${var//old/new}\n";
    let artifact = Artifact::new(PathBuf::from("test.sh"), Scope::Project, ArtifactKind::ShellScript);
    let result = check_rule(RuleId::Posix, content, &artifact);
    assert!(!result.passed, "pattern substitution should be detected");
    assert!(result.violations.iter().any(|v| v.message.contains("pattern substitution")));
}

#[test]
fn test_posix_single_pattern_substitution_detected() {
    let content = "#!/bin/sh\necho ${var/old/new}\n";
    let artifact = Artifact::new(PathBuf::from("test.sh"), Scope::Project, ArtifactKind::ShellScript);
    let result = check_rule(RuleId::Posix, content, &artifact);
    assert!(!result.passed, "single pattern substitution should be detected");
}

#[test]
fn test_posix_default_expansion_no_false_positive() {
    // ${var:-default} is POSIX
    let content = "#!/bin/sh\necho ${var:-default}\n";
    let artifact = Artifact::new(PathBuf::from("test.sh"), Scope::Project, ArtifactKind::ShellScript);
    let result = check_rule(RuleId::Posix, content, &artifact);
    assert!(result.passed, "POSIX default expansion should not be flagged: {:?}", result.violations);
}

#[test]
fn test_posix_default_with_path_no_false_positive() {
    // ${TMPDIR:-/tmp} is POSIX default expansion containing a path — NOT pattern substitution
    let content = "#!/bin/sh\ntrap 'rm -rf \"${TMPDIR:-/tmp}/rash\"' EXIT\n";
    let artifact = Artifact::new(PathBuf::from("test.sh"), Scope::Project, ArtifactKind::ShellScript);
    let result = check_rule(RuleId::Posix, content, &artifact);
    assert!(result.passed, "POSIX default with path value should not be flagged: {:?}", result.violations);
}

#[test]
fn test_posix_prefix_removal_no_false_positive() {
    // ${var#*/} is POSIX prefix removal — NOT pattern substitution
    let content = "#!/bin/sh\necho ${path#*/}\n";
    let artifact = Artifact::new(PathBuf::from("test.sh"), Scope::Project, ArtifactKind::ShellScript);
    let result = check_rule(RuleId::Posix, content, &artifact);
    assert!(result.passed, "POSIX prefix removal should not be flagged: {:?}", result.violations);
}

#[test]
fn test_posix_suffix_removal_no_false_positive() {
    // ${var%/*} is POSIX suffix removal — NOT pattern substitution
    let content = "#!/bin/sh\necho ${path%/*}\n";
    let artifact = Artifact::new(PathBuf::from("test.sh"), Scope::Project, ArtifactKind::ShellScript);
    let result = check_rule(RuleId::Posix, content, &artifact);
    assert!(result.passed, "POSIX suffix removal should not be flagged: {:?}", result.violations);
}

#[test]
fn test_posix_error_expansion_no_false_positive() {
    // ${var:?error} is POSIX
    let content = "#!/bin/sh\necho ${var:?error}\n";
    let artifact = Artifact::new(PathBuf::from("test.sh"), Scope::Project, ArtifactKind::ShellScript);
    let result = check_rule(RuleId::Posix, content, &artifact);
    assert!(result.passed, "POSIX error expansion should not be flagged: {:?}", result.violations);
}

#[test]
fn test_posix_case_modification_lower_detected() {
    let content = "#!/bin/sh\necho ${var,,}\n";
    let artifact = Artifact::new(PathBuf::from("test.sh"), Scope::Project, ArtifactKind::ShellScript);
    let result = check_rule(RuleId::Posix, content, &artifact);
    assert!(!result.passed, "lowercase case modification should be detected");
    assert!(result.violations.iter().any(|v| v.message.contains("case modification")));
}

#[test]
fn test_posix_case_modification_upper_detected() {
    let content = "#!/bin/sh\necho ${var^^}\n";
    let artifact = Artifact::new(PathBuf::from("test.sh"), Scope::Project, ArtifactKind::ShellScript);
    let result = check_rule(RuleId::Posix, content, &artifact);
    assert!(!result.passed, "uppercase case modification should be detected");
}

#[test]
fn test_posix_pipefail_detected() {
    let content = "#!/bin/sh\nset -o pipefail\n";
    let artifact = Artifact::new(PathBuf::from("test.sh"), Scope::Project, ArtifactKind::ShellScript);
    let result = check_rule(RuleId::Posix, content, &artifact);
    assert!(!result.passed, "set -o pipefail should be detected as bashism");
    assert!(result.violations.iter().any(|v| v.message.contains("pipefail")));
}

#[test]
fn test_posix_euo_pipefail_detected() {
    let content = "#!/bin/sh\nset -euo pipefail\n";
    let artifact = Artifact::new(PathBuf::from("test.sh"), Scope::Project, ArtifactKind::ShellScript);
    let result = check_rule(RuleId::Posix, content, &artifact);
    assert!(!result.passed, "set -euo pipefail should be detected as bashism");
}

#[test]
fn test_posix_set_e_no_false_positive() {
    // set -e is POSIX
    let content = "#!/bin/sh\nset -e\n";
    let artifact = Artifact::new(PathBuf::from("test.sh"), Scope::Project, ArtifactKind::ShellScript);
    let result = check_rule(RuleId::Posix, content, &artifact);
    assert!(result.passed, "set -e should not be flagged: {:?}", result.violations);
}

#[test]
fn test_posix_ampersand_redirect_detected() {
    let content = "#!/bin/sh\ncommand &>/dev/null\n";
    let artifact = Artifact::new(PathBuf::from("test.sh"), Scope::Project, ArtifactKind::ShellScript);
    let result = check_rule(RuleId::Posix, content, &artifact);
    assert!(!result.passed, "&> redirect should be detected as bashism");
    assert!(result.violations.iter().any(|v| v.message.contains("&> redirect")));
}

#[test]
fn test_posix_fd_redirect_no_false_positive() {
    // >&2 is POSIX file descriptor redirect
    let content = "#!/bin/sh\necho error >&2\n";
    let artifact = Artifact::new(PathBuf::from("test.sh"), Scope::Project, ArtifactKind::ShellScript);
    let result = check_rule(RuleId::Posix, content, &artifact);
    assert!(result.passed, ">&2 should not be flagged: {:?}", result.violations);
}

#[test]
fn test_posix_redirect_to_file_no_false_positive() {
    // >file 2>&1 is POSIX
    let content = "#!/bin/sh\ncommand >output.log 2>&1\n";
    let artifact = Artifact::new(PathBuf::from("test.sh"), Scope::Project, ArtifactKind::ShellScript);
    let result = check_rule(RuleId::Posix, content, &artifact);
    assert!(result.passed, ">file 2>&1 should not be flagged: {:?}", result.violations);
}

#[test]
fn test_posix_multiple_bashisms_counted() {
    // Script with multiple bashisms should report all of them
    let content = "#!/bin/bash\nset -euo pipefail\nfunction greet {\n  echo ${var,,}\n}\n(( i++ ))\n";
    let artifact = Artifact::new(PathBuf::from("test.sh"), Scope::Project, ArtifactKind::ShellScript);
    let result = check_rule(RuleId::Posix, content, &artifact);
    assert!(!result.passed);
    // Should have: shebang + pipefail + function + case_mod + (( ))
    assert!(result.violations.len() >= 5,
        "Expected at least 5 violations, got {}: {:?}", result.violations.len(), result.violations);
}

// ─── COMPLY-006 ShellCheck Pattern Expansion ───

#[test]
fn test_sc2164_bare_cd_detected() {
    let content = "#!/bin/sh\ncd /some/dir\n";
    let artifact = Artifact::new(PathBuf::from("test.sh"), Scope::Project, ArtifactKind::ShellScript);
    let result = check_rule(RuleId::ShellCheck, content, &artifact);
    assert!(!result.passed, "bare cd should be flagged");
    assert!(result.violations.iter().any(|v| v.message.contains("SC2164")));
}

#[test]
fn test_sc2164_cd_or_exit_no_false_positive() {
    let content = "#!/bin/sh\ncd /some/dir || exit 1\n";
    let artifact = Artifact::new(PathBuf::from("test.sh"), Scope::Project, ArtifactKind::ShellScript);
    let result = check_rule(RuleId::ShellCheck, content, &artifact);
    assert!(result.passed, "cd || exit should not be flagged: {:?}", result.violations);
}

#[test]
fn test_sc2164_cd_or_return_no_false_positive() {
    let content = "#!/bin/sh\ncd /some/dir || return 1\n";
    let artifact = Artifact::new(PathBuf::from("test.sh"), Scope::Project, ArtifactKind::ShellScript);
    let result = check_rule(RuleId::ShellCheck, content, &artifact);
    assert!(result.passed, "cd || return should not be flagged: {:?}", result.violations);
}

#[test]
fn test_sc2164_cd_home_no_false_positive() {
    // Just "cd" (go home) is always safe
    let content = "#!/bin/sh\ncd\n";
    let artifact = Artifact::new(PathBuf::from("test.sh"), Scope::Project, ArtifactKind::ShellScript);
    let result = check_rule(RuleId::ShellCheck, content, &artifact);
    assert!(result.passed, "bare cd (home) should not be flagged: {:?}", result.violations);
}

#[test]
fn test_sc2162_read_without_r_detected() {
    let content = "#!/bin/sh\nread line\n";
    let artifact = Artifact::new(PathBuf::from("test.sh"), Scope::Project, ArtifactKind::ShellScript);
    let result = check_rule(RuleId::ShellCheck, content, &artifact);
    assert!(!result.passed, "read without -r should be flagged");
    assert!(result.violations.iter().any(|v| v.message.contains("SC2162")));
}

#[test]
fn test_sc2162_read_with_r_no_false_positive() {
    let content = "#!/bin/sh\nread -r line\n";
    let artifact = Artifact::new(PathBuf::from("test.sh"), Scope::Project, ArtifactKind::ShellScript);
    let result = check_rule(RuleId::ShellCheck, content, &artifact);
    // Filter to only SC2162 violations
    let sc2162: Vec<_> = result.violations.iter().filter(|v| v.message.contains("SC2162")).collect();
    assert!(sc2162.is_empty(), "read -r should not trigger SC2162: {:?}", sc2162);
}

#[test]
fn test_sc2162_pipe_read_without_r_detected() {
    let content = "#!/bin/sh\necho hello | read line\n";
    let artifact = Artifact::new(PathBuf::from("test.sh"), Scope::Project, ArtifactKind::ShellScript);
    let result = check_rule(RuleId::ShellCheck, content, &artifact);
    assert!(result.violations.iter().any(|v| v.message.contains("SC2162")));
}

#[test]
fn test_sc2181_dollar_question_detected() {
    let content = "#!/bin/sh\ncommand\nif [ $? -eq 0 ]; then echo ok; fi\n";
    let artifact = Artifact::new(PathBuf::from("test.sh"), Scope::Project, ArtifactKind::ShellScript);
    let result = check_rule(RuleId::ShellCheck, content, &artifact);
    assert!(!result.passed, "$? check should be flagged");
    assert!(result.violations.iter().any(|v| v.message.contains("SC2181")));
}

#[test]
fn test_sc2181_direct_command_no_false_positive() {
    let content = "#!/bin/sh\nif command; then echo ok; fi\n";
    let artifact = Artifact::new(PathBuf::from("test.sh"), Scope::Project, ArtifactKind::ShellScript);
    let result = check_rule(RuleId::ShellCheck, content, &artifact);
    let sc2181: Vec<_> = result.violations.iter().filter(|v| v.message.contains("SC2181")).collect();
    assert!(sc2181.is_empty(), "direct if command should not trigger SC2181: {:?}", sc2181);
}

#[test]
fn test_sc2012_ls_iteration_detected() {
    let content = "#!/bin/sh\nfor f in $(ls *.txt); do echo $f; done\n";
    let artifact = Artifact::new(PathBuf::from("test.sh"), Scope::Project, ArtifactKind::ShellScript);
    let result = check_rule(RuleId::ShellCheck, content, &artifact);
    assert!(result.violations.iter().any(|v| v.message.contains("SC2012")));
}

#[test]
fn test_sc2012_backtick_ls_detected() {
    let content = "#!/bin/sh\nfor f in `ls *.txt`; do echo $f; done\n";
    let artifact = Artifact::new(PathBuf::from("test.sh"), Scope::Project, ArtifactKind::ShellScript);
    let result = check_rule(RuleId::ShellCheck, content, &artifact);
    assert!(result.violations.iter().any(|v| v.message.contains("SC2012")));
}

#[test]
fn test_sc2012_glob_no_false_positive() {
    let content = "#!/bin/sh\nfor f in *.txt; do echo \"$f\"; done\n";
    let artifact = Artifact::new(PathBuf::from("test.sh"), Scope::Project, ArtifactKind::ShellScript);
    let result = check_rule(RuleId::ShellCheck, content, &artifact);
    let sc2012: Vec<_> = result.violations.iter().filter(|v| v.message.contains("SC2012")).collect();
    assert!(sc2012.is_empty(), "glob should not trigger SC2012: {:?}", sc2012);
}

#[test]
fn test_sc2035_bare_glob_detected() {
    let content = "#!/bin/sh\nfor f in *; do echo \"$f\"; done\n";
    let artifact = Artifact::new(PathBuf::from("test.sh"), Scope::Project, ArtifactKind::ShellScript);
    let result = check_rule(RuleId::ShellCheck, content, &artifact);
    assert!(result.violations.iter().any(|v| v.message.contains("SC2035")));
}

#[test]
fn test_sc2035_dot_slash_glob_no_false_positive() {
    let content = "#!/bin/sh\nfor f in ./*; do echo \"$f\"; done\n";
    let artifact = Artifact::new(PathBuf::from("test.sh"), Scope::Project, ArtifactKind::ShellScript);
    let result = check_rule(RuleId::ShellCheck, content, &artifact);
    let sc2035: Vec<_> = result.violations.iter().filter(|v| v.message.contains("SC2035")).collect();
    assert!(sc2035.is_empty(), "./* should not trigger SC2035: {:?}", sc2035);
}

#[test]
fn test_sc2035_qualified_glob_no_false_positive() {
    // *.txt is already qualified (not bare *)
    let content = "#!/bin/sh\nfor f in *.txt; do echo \"$f\"; done\n";
    let artifact = Artifact::new(PathBuf::from("test.sh"), Scope::Project, ArtifactKind::ShellScript);
    let result = check_rule(RuleId::ShellCheck, content, &artifact);
    let sc2035: Vec<_> = result.violations.iter().filter(|v| v.message.contains("SC2035")).collect();
    assert!(sc2035.is_empty(), "*.txt should not trigger SC2035: {:?}", sc2035);
}

#[test]
fn test_shellcheck_multiple_violations() {
    // Script with multiple issues
    let content = "#!/bin/sh\ncd /tmp\nresult=`whoami`\nread name\nif [ $? -eq 0 ]; then echo ok; fi\n";
    let artifact = Artifact::new(PathBuf::from("test.sh"), Scope::Project, ArtifactKind::ShellScript);
    let result = check_rule(RuleId::ShellCheck, content, &artifact);
    assert!(!result.passed);
    // Should have: SC2164 (cd) + SC2006 (backtick) + SC2162 (read) + SC2181 ($?)
    assert!(result.violations.len() >= 4,
        "Expected at least 4 violations, got {}: {:?}", result.violations.len(), result.violations);
}

// ─── COMPLY-008 Dockerfile Pattern Expansion ───

#[test]
fn test_docker_untagged_from_detected() {
    let content = "FROM ubuntu\nRUN echo hello\nUSER app\n";
    let artifact = Artifact::new(PathBuf::from("Dockerfile"), Scope::Project, ArtifactKind::Dockerfile);
    let result = check_rule(RuleId::DockerfileBest, content, &artifact);
    assert!(result.violations.iter().any(|v| v.message.contains("DOCKER001")),
        "Untagged FROM should be detected: {:?}", result.violations);
}

#[test]
fn test_docker_latest_tag_detected() {
    let content = "FROM ubuntu:latest\nRUN echo hello\nUSER app\n";
    let artifact = Artifact::new(PathBuf::from("Dockerfile"), Scope::Project, ArtifactKind::Dockerfile);
    let result = check_rule(RuleId::DockerfileBest, content, &artifact);
    assert!(result.violations.iter().any(|v| v.message.contains("DOCKER001")),
        "FROM :latest should be detected: {:?}", result.violations);
}

#[test]
fn test_docker_pinned_tag_no_false_positive() {
    let content = "FROM ubuntu:22.04\nRUN echo hello\nUSER app\n";
    let artifact = Artifact::new(PathBuf::from("Dockerfile"), Scope::Project, ArtifactKind::Dockerfile);
    let result = check_rule(RuleId::DockerfileBest, content, &artifact);
    let d001: Vec<_> = result.violations.iter().filter(|v| v.message.contains("DOCKER001")).collect();
    assert!(d001.is_empty(), "Pinned FROM should not trigger DOCKER001: {:?}", d001);
}

#[test]
fn test_docker_digest_pin_no_false_positive() {
    let content = "FROM ubuntu@sha256:abc123\nRUN echo hello\nUSER app\n";
    let artifact = Artifact::new(PathBuf::from("Dockerfile"), Scope::Project, ArtifactKind::Dockerfile);
    let result = check_rule(RuleId::DockerfileBest, content, &artifact);
    let d001: Vec<_> = result.violations.iter().filter(|v| v.message.contains("DOCKER001")).collect();
    assert!(d001.is_empty(), "Digest-pinned FROM should not trigger DOCKER001: {:?}", d001);
}

#[test]
fn test_docker_scratch_no_false_positive() {
    let content = "FROM scratch\nCOPY binary /app\nUSER app\n";
    let artifact = Artifact::new(PathBuf::from("Dockerfile"), Scope::Project, ArtifactKind::Dockerfile);
    let result = check_rule(RuleId::DockerfileBest, content, &artifact);
    let d001: Vec<_> = result.violations.iter().filter(|v| v.message.contains("DOCKER001")).collect();
    assert!(d001.is_empty(), "FROM scratch should not trigger DOCKER001: {:?}", d001);
}

#[test]
fn test_docker_arg_from_no_false_positive() {
    let content = "ARG BASE=ubuntu:22.04\nFROM $BASE\nRUN echo hello\nUSER app\n";
    let artifact = Artifact::new(PathBuf::from("Dockerfile"), Scope::Project, ArtifactKind::Dockerfile);
    let result = check_rule(RuleId::DockerfileBest, content, &artifact);
    let d001: Vec<_> = result.violations.iter().filter(|v| v.message.contains("DOCKER001")).collect();
    assert!(d001.is_empty(), "FROM $ARG should not trigger DOCKER001: {:?}", d001);
}

#[test]
fn test_docker_apt_without_clean_detected() {
    let content = "FROM ubuntu:22.04\nRUN apt-get update && apt-get install -y curl\nUSER app\n";
    let artifact = Artifact::new(PathBuf::from("Dockerfile"), Scope::Project, ArtifactKind::Dockerfile);
    let result = check_rule(RuleId::DockerfileBest, content, &artifact);
    assert!(result.violations.iter().any(|v| v.message.contains("DOCKER003")),
        "apt-get install without cleanup should be detected: {:?}", result.violations);
}

#[test]
fn test_docker_apt_with_clean_no_false_positive() {
    let content = "FROM ubuntu:22.04\nRUN apt-get update && apt-get install -y curl && rm -rf /var/lib/apt/lists/*\nUSER app\n";
    let artifact = Artifact::new(PathBuf::from("Dockerfile"), Scope::Project, ArtifactKind::Dockerfile);
    let result = check_rule(RuleId::DockerfileBest, content, &artifact);
    let d003: Vec<_> = result.violations.iter().filter(|v| v.message.contains("DOCKER003")).collect();
    assert!(d003.is_empty(), "apt-get with cleanup should not trigger DOCKER003: {:?}", d003);
}

#[test]
fn test_docker_apt_autoremove_no_false_positive() {
    let content = "FROM ubuntu:22.04\nRUN apt-get update && apt-get install -y curl && apt-get autoremove\nUSER app\n";
    let artifact = Artifact::new(PathBuf::from("Dockerfile"), Scope::Project, ArtifactKind::Dockerfile);
    let result = check_rule(RuleId::DockerfileBest, content, &artifact);
    let d003: Vec<_> = result.violations.iter().filter(|v| v.message.contains("DOCKER003")).collect();
    assert!(d003.is_empty(), "apt-get autoremove should not trigger DOCKER003: {:?}", d003);
}

#[test]
fn test_docker_multistage_from_as_no_false_positive() {
    // Multi-stage: FROM image:tag AS builder
    let content = "FROM rust:1.75 AS builder\nRUN cargo build\nFROM debian:bookworm-slim\nCOPY --from=builder /app /app\nUSER app\n";
    let artifact = Artifact::new(PathBuf::from("Dockerfile"), Scope::Project, ArtifactKind::Dockerfile);
    let result = check_rule(RuleId::DockerfileBest, content, &artifact);
    let d001: Vec<_> = result.violations.iter().filter(|v| v.message.contains("DOCKER001")).collect();
    assert!(d001.is_empty(), "Pinned multi-stage FROM should not trigger DOCKER001: {:?}", d001);
}

#[test]
fn test_docker_multiple_violations() {
    let content = "FROM ubuntu\nADD . /app\nRUN apt-get install -y curl\n";
    let artifact = Artifact::new(PathBuf::from("Dockerfile"), Scope::Project, ArtifactKind::Dockerfile);
    let result = check_rule(RuleId::DockerfileBest, content, &artifact);
    // DOCKER001 (untagged) + DOCKER008 (ADD) + DOCKER003 (apt) + DOCKER010 (no USER)
    assert!(result.violations.len() >= 4,
        "Expected at least 4 violations, got {}: {:?}", result.violations.len(), result.violations);
}

// ─── COMPLY-007 Makefile Safety Expansion ───

#[test]
fn test_make_eval_in_recipe_detected() {
    let content = ".PHONY: all\nall:\n\teval \"$(SOME_CMD)\"\n";
    let artifact = Artifact::new(PathBuf::from("Makefile"), Scope::Project, ArtifactKind::Makefile);
    let result = check_rule(RuleId::MakefileSafety, content, &artifact);
    assert!(result.violations.iter().any(|v| v.message.contains("MAKE001")),
        "eval in recipe should be detected: {:?}", result.violations);
}

#[test]
fn test_make_recursive_bare_detected() {
    let content = ".PHONY: all\nall:\n\tmake clean\n";
    let artifact = Artifact::new(PathBuf::from("Makefile"), Scope::Project, ArtifactKind::Makefile);
    let result = check_rule(RuleId::MakefileSafety, content, &artifact);
    assert!(result.violations.iter().any(|v| v.message.contains("MAKE002")),
        "bare make should be detected: {:?}", result.violations);
}

#[test]
fn test_make_recursive_dollar_make_no_false_positive() {
    let content = ".PHONY: all\nall:\n\t$(MAKE) clean\n";
    let artifact = Artifact::new(PathBuf::from("Makefile"), Scope::Project, ArtifactKind::Makefile);
    let result = check_rule(RuleId::MakefileSafety, content, &artifact);
    let m002: Vec<_> = result.violations.iter().filter(|v| v.message.contains("MAKE002")).collect();
    assert!(m002.is_empty(), "$(MAKE) should not trigger MAKE002: {:?}", m002);
}

#[test]
fn test_make_recursive_chained_detected() {
    let content = ".PHONY: all\nall:\n\techo starting && make clean\n";
    let artifact = Artifact::new(PathBuf::from("Makefile"), Scope::Project, ArtifactKind::Makefile);
    let result = check_rule(RuleId::MakefileSafety, content, &artifact);
    assert!(result.violations.iter().any(|v| v.message.contains("MAKE002")));
}

#[test]
fn test_make_dangerous_rm_detected() {
    let content = ".PHONY: clean\nclean:\n\trm -rf $(BUILD_DIR)\n";
    let artifact = Artifact::new(PathBuf::from("Makefile"), Scope::Project, ArtifactKind::Makefile);
    let result = check_rule(RuleId::MakefileSafety, content, &artifact);
    assert!(result.violations.iter().any(|v| v.message.contains("MAKE003")),
        "rm -rf with variable should be detected: {:?}", result.violations);
}

#[test]
fn test_make_safe_rm_literal_no_false_positive() {
    // rm -rf on a literal path (no variable) is fine
    let content = ".PHONY: clean\nclean:\n\trm -rf /tmp/build\n";
    let artifact = Artifact::new(PathBuf::from("Makefile"), Scope::Project, ArtifactKind::Makefile);
    let result = check_rule(RuleId::MakefileSafety, content, &artifact);
    let m003: Vec<_> = result.violations.iter().filter(|v| v.message.contains("MAKE003")).collect();
    assert!(m003.is_empty(), "rm -rf on literal path should not trigger MAKE003: {:?}", m003);
}

#[test]
fn test_make_missing_phony_detected() {
    // Common targets without .PHONY declaration
    let content = "all:\n\techo building\nclean:\n\trm -f output\ntest:\n\tcargo test\n";
    let artifact = Artifact::new(PathBuf::from("Makefile"), Scope::Project, ArtifactKind::Makefile);
    let result = check_rule(RuleId::MakefileSafety, content, &artifact);
    assert!(result.violations.iter().any(|v| v.message.contains("MAKE004")),
        "Missing .PHONY should be detected: {:?}", result.violations);
    // Should flag all three: all, clean, test
    let m004: Vec<_> = result.violations.iter().filter(|v| v.message.contains("MAKE004")).collect();
    assert!(m004.len() >= 3, "Expected at least 3 missing .PHONY, got {}: {:?}", m004.len(), m004);
}

#[test]
fn test_make_with_phony_no_false_positive() {
    let content = ".PHONY: all clean test\nall:\n\techo building\nclean:\n\trm -f output\ntest:\n\tcargo test\n";
    let artifact = Artifact::new(PathBuf::from("Makefile"), Scope::Project, ArtifactKind::Makefile);
    let result = check_rule(RuleId::MakefileSafety, content, &artifact);
    let m004: Vec<_> = result.violations.iter().filter(|v| v.message.contains("MAKE004")).collect();
    assert!(m004.is_empty(), "Declared .PHONY should not trigger MAKE004: {:?}", m004);
}

#[test]
fn test_make_non_standard_target_no_false_positive() {
    // Custom targets not in COMMON_PHONY_TARGETS should not be flagged
    let content = "my-custom-target:\n\techo custom\n";
    let artifact = Artifact::new(PathBuf::from("Makefile"), Scope::Project, ArtifactKind::Makefile);
    let result = check_rule(RuleId::MakefileSafety, content, &artifact);
    let m004: Vec<_> = result.violations.iter().filter(|v| v.message.contains("MAKE004")).collect();
    assert!(m004.is_empty(), "Custom target should not trigger MAKE004: {:?}", m004);
}

#[test]
fn test_make_multiple_violations() {
    let content = "all:\n\teval \"$CMD\"\n\tmake clean\n\trm -rf $(DIR)\n";
    let artifact = Artifact::new(PathBuf::from("Makefile"), Scope::Project, ArtifactKind::Makefile);
    let result = check_rule(RuleId::MakefileSafety, content, &artifact);
    // MAKE001 (eval) + MAKE002 (bare make) + MAKE003 (rm -rf) + MAKE004 (no .PHONY all)
    assert!(result.violations.len() >= 4,
        "Expected at least 4 violations, got {}: {:?}", result.violations.len(), result.violations);
}

// ─── Runner output format tests ───

#[test]
fn test_format_human_failures_only_excludes_compliant() {
    use super::runner;
    let scores = vec![
        super::scoring::compute_artifact_score("clean.sh", &[]),
        super::scoring::compute_artifact_score("bad.sh", &[RuleResult {
            rule: RuleId::Determinism,
            passed: false,
            violations: vec![Violation {
                rule: RuleId::Determinism,
                line: Some(1),
                message: "test violation".to_string(),
            }],
        }]),
    ];
    let project = super::scoring::compute_project_score(scores);
    let output = runner::format_human_failures_only(&project);
    assert!(output.contains("bad.sh"), "Should show non-compliant artifact");
    assert!(!output.contains("clean.sh"), "Should NOT show compliant artifact");
    assert!(output.contains("Failures Only"), "Should have failures-only header");
}

#[test]
fn test_format_human_failures_only_all_compliant() {
    use super::runner;
    let scores = vec![
        super::scoring::compute_artifact_score("clean.sh", &[]),
    ];
    let project = super::scoring::compute_project_score(scores);
    let output = runner::format_human_failures_only(&project);
    assert!(output.contains("No violations found"), "Should show no-violations message");
}
