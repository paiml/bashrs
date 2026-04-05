fn test_posix_sh_shebang_ok() {
    let art = shell_artifact();
    let result = check_rule(RuleId::Posix, "#!/bin/sh\necho hello\n", &art);
    assert!(result.passed);
}

#[test]
fn test_posix_double_bracket() {
    let art = shell_artifact();
    let result = check_rule(RuleId::Posix, "if [[ -f file ]]; then echo yes; fi", &art);
    assert!(!result.passed);
    assert!(result.violations[0].message.contains("[[ ]]"));
}

#[test]
fn test_posix_function_keyword() {
    let art = shell_artifact();
    let result = check_rule(RuleId::Posix, "function foo { echo bar; }", &art);
    assert!(!result.passed);
    assert!(result.violations[0].message.contains("function keyword"));
}

#[test]
fn test_posix_standalone_arith() {
    let art = shell_artifact();
    let result = check_rule(RuleId::Posix, "(( i++ ))", &art);
    assert!(!result.passed);
    assert!(result.violations[0].message.contains("(( ))"));
}

#[test]
fn test_posix_herestring() {
    let art = shell_artifact();
    let result = check_rule(RuleId::Posix, "cat <<< \"hello\"", &art);
    assert!(!result.passed);
    assert!(result.violations[0].message.contains("<<<"));
}

#[test]
fn test_posix_select() {
    let art = shell_artifact();
    let result = check_rule(RuleId::Posix, "select x in a b c; do echo $x; done", &art);
    assert!(!result.passed);
    assert!(result.violations[0].message.contains("select"));
}

#[test]
fn test_posix_pattern_subst() {
    let art = shell_artifact();
    let result = check_rule(RuleId::Posix, "echo ${var/old/new}", &art);
    assert!(!result.passed);
    assert!(result.violations[0].message.contains("${var/...}"));
}

#[test]
fn test_posix_case_mod() {
    let art = shell_artifact();
    let result = check_rule(RuleId::Posix, "echo ${var,,}", &art);
    assert!(!result.passed);
    assert!(result.violations[0].message.contains("${var,,}"));
}

#[test]
fn test_posix_pipefail() {
    let art = shell_artifact();
    let result = check_rule(RuleId::Posix, "set -o pipefail", &art);
    assert!(!result.passed);
    assert!(result.violations[0].message.contains("pipefail"));
}

#[test]
fn test_posix_bash_redirect() {
    let art = shell_artifact();
    let result = check_rule(RuleId::Posix, "command &> /dev/null", &art);
    assert!(!result.passed);
    assert!(result.violations[0].message.contains("&>"));
}

#[test]
fn test_posix_declare_array() {
    let art = shell_artifact();
    let result = check_rule(RuleId::Posix, "declare -a arr=(1 2 3)", &art);
    assert!(!result.passed);
    assert!(result.violations[0].message.contains("declare -a"));
}

#[test]
fn test_posix_clean() {
    let art = shell_artifact();
    let content = "#!/bin/sh\nset -e\necho hello\nif [ -f file ]; then echo yes; fi\n";
    let result = check_rule(RuleId::Posix, content, &art);
    assert!(result.passed);
    assert!(result.violations.is_empty());
}

// =============================================================================
// ShellCheck (COMPLY-006)
// =============================================================================

#[test]
fn test_shellcheck_backticks() {
    let art = shell_artifact();
    let result = check_rule(RuleId::ShellCheck, "FOO=`date`", &art);
    assert!(!result.passed);
    assert!(result.violations[0].message.contains("SC2006"));
}

#[test]
fn test_shellcheck_rm_rf_root() {
    let art = shell_artifact();
    let result = check_rule(RuleId::ShellCheck, "rm -rf /$VAR", &art);
    assert!(!result.passed);
    assert!(result.violations[0].message.contains("SC2115"));
}

#[test]
fn test_shellcheck_bare_cd() {
    let art = shell_artifact();
    let result = check_rule(RuleId::ShellCheck, "cd /some/dir", &art);
    assert!(!result.passed);
    assert!(result.violations[0].message.contains("SC2164"));
}

#[test]
fn test_shellcheck_cd_with_exit_ok() {
    let art = shell_artifact();
    let result = check_rule(RuleId::ShellCheck, "cd /some/dir || exit 1", &art);
    assert!(result.passed);
}

#[test]
fn test_shellcheck_read_no_r() {
    let art = shell_artifact();
    let result = check_rule(RuleId::ShellCheck, "read var", &art);
    assert!(!result.passed);
    assert!(result.violations[0].message.contains("SC2162"));
}

#[test]
fn test_shellcheck_read_r_ok() {
    let art = shell_artifact();
    let result = check_rule(RuleId::ShellCheck, "read -r var", &art);
    assert!(result.passed);
}

#[test]
fn test_shellcheck_dollar_question() {
    let art = shell_artifact();
    let result = check_rule(
        RuleId::ShellCheck,
        "if [ $? -eq 0 ]; then echo ok; fi",
        &art,
    );
    assert!(!result.passed);
    assert!(result.violations[0].message.contains("SC2181"));
}

#[test]
fn test_shellcheck_ls_iteration() {
    let art = shell_artifact();
    let result = check_rule(
        RuleId::ShellCheck,
        "for f in $(ls *.txt); do echo $f; done",
        &art,
    );
    assert!(!result.passed);
    assert!(result.violations[0].message.contains("SC2012"));
}

#[test]
fn test_shellcheck_bare_glob() {
    let art = shell_artifact();
    let result = check_rule(RuleId::ShellCheck, "for f in *; do echo $f; done", &art);
    assert!(!result.passed);
    assert!(result.violations[0].message.contains("SC2035"));
}

// =============================================================================
// Makefile Safety (COMPLY-007)
// =============================================================================

#[test]
fn test_make_eval_in_recipe() {
    let art = makefile_artifact();
    // Tab-indented line in a makefile is a recipe
    let content = "target:\n\teval $(shell_cmd)";
    let result = check_rule(RuleId::MakefileSafety, content, &art);
    assert!(!result.passed);
    assert!(result
        .violations
        .iter()
        .any(|v| v.message.contains("MAKE001")));
}

#[test]
fn test_make_bare_make() {
    let art = makefile_artifact();
    let content = "all:\n\tmake clean";
    let result = check_rule(RuleId::MakefileSafety, content, &art);
    assert!(!result.passed);
    assert!(result
        .violations
        .iter()
        .any(|v| v.message.contains("MAKE002")));
}

#[test]
fn test_make_dollar_make_ok() {
    let art = makefile_artifact();
    let content = "all:\n\t$(MAKE) clean";
    let result = check_rule(RuleId::MakefileSafety, content, &art);
    // No MAKE002 violation ($(MAKE) is correct)
    assert!(
        !result
            .violations
            .iter()
            .any(|v| v.message.contains("MAKE002")),
        "$(MAKE) should not trigger MAKE002"
    );
}

#[test]
fn test_make_rm_rf_variable() {
    let art = makefile_artifact();
    let content = "clean:\n\trm -rf $$dir";
    let result = check_rule(RuleId::MakefileSafety, content, &art);
    assert!(!result.passed);
    assert!(result
        .violations
        .iter()
        .any(|v| v.message.contains("MAKE003")));
}

#[test]
fn test_make_missing_phony() {
    let art = makefile_artifact();
    // "clean:" target without .PHONY declaration
    let content = "clean:\n\trm -f *.o";
    let result = check_rule(RuleId::MakefileSafety, content, &art);
    assert!(!result.passed);
    assert!(result
        .violations
        .iter()
        .any(|v| v.message.contains("MAKE004")));
}

#[test]
fn test_make_phony_declared_ok() {
    let art = makefile_artifact();
    let content = ".PHONY: clean\nclean:\n\trm -f *.o";
    let result = check_rule(RuleId::MakefileSafety, content, &art);
    // No MAKE004 violation for clean
    assert!(
        !result
            .violations
            .iter()
            .any(|v| v.message.contains("MAKE004")),
        ".PHONY declared targets should not trigger MAKE004"
    );
}

// =============================================================================
// Dockerfile (COMPLY-008)
// =============================================================================

#[test]
fn test_docker_add_local() {
    let art = dockerfile_artifact();
    let content = "FROM ubuntu:22.04\nADD file.tar /opt\nUSER appuser";
    let result = check_rule(RuleId::DockerfileBest, content, &art);
    assert!(!result.passed);
    assert!(result
        .violations
        .iter()
        .any(|v| v.message.contains("DOCKER008")));
}

#[test]
fn test_docker_add_url_ok() {
    let art = dockerfile_artifact();
    let content = "FROM ubuntu:22.04\nADD https://example.com/file.tar.gz /opt\nUSER appuser";
    let result = check_rule(RuleId::DockerfileBest, content, &art);
    // ADD with URL is acceptable
    assert!(
        !result
            .violations
            .iter()
            .any(|v| v.message.contains("DOCKER008")),
        "ADD with https URL should not trigger DOCKER008"
    );
}

#[test]
fn test_docker_unpinned_from() {
    let art = dockerfile_artifact();
    let content = "FROM ubuntu\nRUN echo hello\nUSER appuser";
    let result = check_rule(RuleId::DockerfileBest, content, &art);
    assert!(!result.passed);
    assert!(result
        .violations
        .iter()
        .any(|v| v.message.contains("DOCKER001")));
}

#[test]
fn test_docker_latest_from() {
    let art = dockerfile_artifact();
    let content = "FROM ubuntu:latest\nRUN echo hello\nUSER appuser";
    let result = check_rule(RuleId::DockerfileBest, content, &art);
    assert!(!result.passed);
    assert!(result
        .violations
        .iter()
        .any(|v| v.message.contains("DOCKER001")));
}

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
