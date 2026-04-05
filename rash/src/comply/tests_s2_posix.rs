#[test]
fn test_posix_ampersand_redirect_detected() {
    let content = "#!/bin/sh\ncommand &>/dev/null\n";
    let artifact = Artifact::new(
        PathBuf::from("test.sh"),
        Scope::Project,
        ArtifactKind::ShellScript,
    );
    let result = check_rule(RuleId::Posix, content, &artifact);
    assert!(!result.passed, "&> redirect should be detected as bashism");
    assert!(result
        .violations
        .iter()
        .any(|v| v.message.contains("&> redirect")));
}

#[test]
fn test_posix_fd_redirect_no_false_positive() {
    // >&2 is POSIX file descriptor redirect
    let content = "#!/bin/sh\necho error >&2\n";
    let artifact = Artifact::new(
        PathBuf::from("test.sh"),
        Scope::Project,
        ArtifactKind::ShellScript,
    );
    let result = check_rule(RuleId::Posix, content, &artifact);
    assert!(
        result.passed,
        ">&2 should not be flagged: {:?}",
        result.violations
    );
}

#[test]
fn test_posix_redirect_to_file_no_false_positive() {
    // >file 2>&1 is POSIX
    let content = "#!/bin/sh\ncommand >output.log 2>&1\n";
    let artifact = Artifact::new(
        PathBuf::from("test.sh"),
        Scope::Project,
        ArtifactKind::ShellScript,
    );
    let result = check_rule(RuleId::Posix, content, &artifact);
    assert!(
        result.passed,
        ">file 2>&1 should not be flagged: {:?}",
        result.violations
    );
}

#[test]
fn test_posix_multiple_bashisms_counted() {
    // Script with multiple bashisms should report all of them
    let content =
        "#!/bin/bash\nset -euo pipefail\nfunction greet {\n  echo ${var,,}\n}\n(( i++ ))\n";
    let artifact = Artifact::new(
        PathBuf::from("test.sh"),
        Scope::Project,
        ArtifactKind::ShellScript,
    );
    let result = check_rule(RuleId::Posix, content, &artifact);
    assert!(!result.passed);
    // Should have: shebang + pipefail + function + case_mod + (( ))
    assert!(
        result.violations.len() >= 5,
        "Expected at least 5 violations, got {}: {:?}",
        result.violations.len(),
        result.violations
    );
}

// ─── COMPLY-006 ShellCheck Pattern Expansion ───

#[test]
fn test_sc2164_bare_cd_detected() {
    let content = "#!/bin/sh\ncd /some/dir\n";
    let artifact = Artifact::new(
        PathBuf::from("test.sh"),
        Scope::Project,
        ArtifactKind::ShellScript,
    );
    let result = check_rule(RuleId::ShellCheck, content, &artifact);
    assert!(!result.passed, "bare cd should be flagged");
    assert!(result
        .violations
        .iter()
        .any(|v| v.message.contains("SC2164")));
}

#[test]
fn test_sc2164_cd_or_exit_no_false_positive() {
    let content = "#!/bin/sh\ncd /some/dir || exit 1\n";
    let artifact = Artifact::new(
        PathBuf::from("test.sh"),
        Scope::Project,
        ArtifactKind::ShellScript,
    );
    let result = check_rule(RuleId::ShellCheck, content, &artifact);
    assert!(
        result.passed,
        "cd || exit should not be flagged: {:?}",
        result.violations
    );
}

#[test]
fn test_sc2164_cd_or_return_no_false_positive() {
    let content = "#!/bin/sh\ncd /some/dir || return 1\n";
    let artifact = Artifact::new(
        PathBuf::from("test.sh"),
        Scope::Project,
        ArtifactKind::ShellScript,
    );
    let result = check_rule(RuleId::ShellCheck, content, &artifact);
    assert!(
        result.passed,
        "cd || return should not be flagged: {:?}",
        result.violations
    );
}

#[test]
fn test_sc2164_cd_home_no_false_positive() {
    // Just "cd" (go home) is always safe
    let content = "#!/bin/sh\ncd\n";
    let artifact = Artifact::new(
        PathBuf::from("test.sh"),
        Scope::Project,
        ArtifactKind::ShellScript,
    );
    let result = check_rule(RuleId::ShellCheck, content, &artifact);
    assert!(
        result.passed,
        "bare cd (home) should not be flagged: {:?}",
        result.violations
    );
}

#[test]
fn test_sc2162_read_without_r_detected() {
    let content = "#!/bin/sh\nread line\n";
    let artifact = Artifact::new(
        PathBuf::from("test.sh"),
        Scope::Project,
        ArtifactKind::ShellScript,
    );
    let result = check_rule(RuleId::ShellCheck, content, &artifact);
    assert!(!result.passed, "read without -r should be flagged");
    assert!(result
        .violations
        .iter()
        .any(|v| v.message.contains("SC2162")));
}

#[test]
fn test_sc2162_read_with_r_no_false_positive() {
    let content = "#!/bin/sh\nread -r line\n";
    let artifact = Artifact::new(
        PathBuf::from("test.sh"),
        Scope::Project,
        ArtifactKind::ShellScript,
    );
    let result = check_rule(RuleId::ShellCheck, content, &artifact);
    // Filter to only SC2162 violations
    let sc2162: Vec<_> = result
        .violations
        .iter()
        .filter(|v| v.message.contains("SC2162"))
        .collect();
    assert!(
        sc2162.is_empty(),
        "read -r should not trigger SC2162: {:?}",
        sc2162
    );
}

#[test]
fn test_sc2162_pipe_read_without_r_detected() {
    let content = "#!/bin/sh\necho hello | read line\n";
    let artifact = Artifact::new(
        PathBuf::from("test.sh"),
        Scope::Project,
        ArtifactKind::ShellScript,
    );
    let result = check_rule(RuleId::ShellCheck, content, &artifact);
    assert!(result
        .violations
        .iter()
        .any(|v| v.message.contains("SC2162")));
}

#[test]
fn test_sc2181_dollar_question_detected() {
    let content = "#!/bin/sh\ncommand\nif [ $? -eq 0 ]; then echo ok; fi\n";
    let artifact = Artifact::new(
        PathBuf::from("test.sh"),
        Scope::Project,
        ArtifactKind::ShellScript,
    );
    let result = check_rule(RuleId::ShellCheck, content, &artifact);
    assert!(!result.passed, "$? check should be flagged");
    assert!(result
        .violations
        .iter()
        .any(|v| v.message.contains("SC2181")));
}

#[test]
fn test_sc2181_direct_command_no_false_positive() {
    let content = "#!/bin/sh\nif command; then echo ok; fi\n";
    let artifact = Artifact::new(
        PathBuf::from("test.sh"),
        Scope::Project,
        ArtifactKind::ShellScript,
    );
    let result = check_rule(RuleId::ShellCheck, content, &artifact);
    let sc2181: Vec<_> = result
        .violations
        .iter()
        .filter(|v| v.message.contains("SC2181"))
        .collect();
    assert!(
        sc2181.is_empty(),
        "direct if command should not trigger SC2181: {:?}",
        sc2181
    );
}

#[test]
fn test_sc2012_ls_iteration_detected() {
    let content = "#!/bin/sh\nfor f in $(ls *.txt); do echo $f; done\n";
    let artifact = Artifact::new(
        PathBuf::from("test.sh"),
        Scope::Project,
        ArtifactKind::ShellScript,
    );
    let result = check_rule(RuleId::ShellCheck, content, &artifact);
    assert!(result
        .violations
        .iter()
        .any(|v| v.message.contains("SC2012")));
}

#[test]
fn test_sc2012_backtick_ls_detected() {
    let content = "#!/bin/sh\nfor f in `ls *.txt`; do echo $f; done\n";
    let artifact = Artifact::new(
        PathBuf::from("test.sh"),
        Scope::Project,
        ArtifactKind::ShellScript,
    );
    let result = check_rule(RuleId::ShellCheck, content, &artifact);
    assert!(result
        .violations
        .iter()
        .any(|v| v.message.contains("SC2012")));
}

#[test]
fn test_sc2012_glob_no_false_positive() {
    let content = "#!/bin/sh\nfor f in *.txt; do echo \"$f\"; done\n";
    let artifact = Artifact::new(
        PathBuf::from("test.sh"),
        Scope::Project,
        ArtifactKind::ShellScript,
    );
    let result = check_rule(RuleId::ShellCheck, content, &artifact);
    let sc2012: Vec<_> = result
        .violations
        .iter()
        .filter(|v| v.message.contains("SC2012"))
        .collect();
    assert!(
        sc2012.is_empty(),
        "glob should not trigger SC2012: {:?}",
        sc2012
    );
}

#[test]
fn test_sc2035_bare_glob_detected() {
    let content = "#!/bin/sh\nfor f in *; do echo \"$f\"; done\n";
    let artifact = Artifact::new(
        PathBuf::from("test.sh"),
        Scope::Project,
        ArtifactKind::ShellScript,
    );
    let result = check_rule(RuleId::ShellCheck, content, &artifact);
    assert!(result
        .violations
        .iter()
        .any(|v| v.message.contains("SC2035")));
}

#[test]
fn test_sc2035_dot_slash_glob_no_false_positive() {
    let content = "#!/bin/sh\nfor f in ./*; do echo \"$f\"; done\n";
    let artifact = Artifact::new(
        PathBuf::from("test.sh"),
        Scope::Project,
        ArtifactKind::ShellScript,
    );
    let result = check_rule(RuleId::ShellCheck, content, &artifact);
    let sc2035: Vec<_> = result
        .violations
        .iter()
        .filter(|v| v.message.contains("SC2035"))
        .collect();
    assert!(
        sc2035.is_empty(),
        "./* should not trigger SC2035: {:?}",
        sc2035
    );
}

#[test]
fn test_sc2035_qualified_glob_no_false_positive() {
    // *.txt is already qualified (not bare *)
    let content = "#!/bin/sh\nfor f in *.txt; do echo \"$f\"; done\n";
    let artifact = Artifact::new(
        PathBuf::from("test.sh"),
        Scope::Project,
        ArtifactKind::ShellScript,
    );
    let result = check_rule(RuleId::ShellCheck, content, &artifact);
    let sc2035: Vec<_> = result
        .violations
        .iter()
        .filter(|v| v.message.contains("SC2035"))
        .collect();
    assert!(
        sc2035.is_empty(),
        "*.txt should not trigger SC2035: {:?}",
        sc2035
    );
}

#[test]
fn test_shellcheck_multiple_violations() {
    // Script with multiple issues
    let content =
        "#!/bin/sh\ncd /tmp\nresult=`whoami`\nread name\nif [ $? -eq 0 ]; then echo ok; fi\n";
    let artifact = Artifact::new(
        PathBuf::from("test.sh"),
        Scope::Project,
        ArtifactKind::ShellScript,
    );
    let result = check_rule(RuleId::ShellCheck, content, &artifact);
    assert!(!result.passed);
    // Should have: SC2164 (cd) + SC2006 (backtick) + SC2162 (read) + SC2181 ($?)
    assert!(
        result.violations.len() >= 4,
        "Expected at least 4 violations, got {}: {:?}",
        result.violations.len(),
        result.violations
    );
}

// ─── COMPLY-008 Dockerfile Pattern Expansion ───

#[test]
fn test_docker_untagged_from_detected() {
    let content = "FROM ubuntu\nRUN echo hello\nUSER app\n";
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
            .any(|v| v.message.contains("DOCKER001")),
        "Untagged FROM should be detected: {:?}",
        result.violations
    );
}

#[test]
fn test_docker_latest_tag_detected() {
    let content = "FROM ubuntu:latest\nRUN echo hello\nUSER app\n";
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
            .any(|v| v.message.contains("DOCKER001")),
        "FROM :latest should be detected: {:?}",
        result.violations
    );
}

#[test]
fn test_docker_pinned_tag_no_false_positive() {
    let content = "FROM ubuntu:22.04\nRUN echo hello\nUSER app\n";
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
        "Pinned FROM should not trigger DOCKER001: {:?}",
        d001
    );
}

#[test]
fn test_docker_digest_pin_no_false_positive() {
    let content = "FROM ubuntu@sha256:abc123\nRUN echo hello\nUSER app\n";
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
        "Digest-pinned FROM should not trigger DOCKER001: {:?}",
        d001
    );
}

