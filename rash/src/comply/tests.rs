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
