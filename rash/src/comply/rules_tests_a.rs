#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]

use super::config::Scope;
use super::discovery::{Artifact, ArtifactKind};
use super::rules::*;
use std::path::PathBuf;

#[test]
fn test_security_tls_disabled_insecure() {
    let art = shell_artifact();
    let result = check_rule(
        RuleId::Security,
        "curl --insecure https://example.com",
        &art,
    );
    assert!(!result.passed);
    assert!(result.violations[0].message.contains("SEC004"));
}

#[test]
fn test_security_tls_disabled_no_check() {
    let art = shell_artifact();
    let result = check_rule(
        RuleId::Security,
        "wget --no-check-certificate https://example.com",
        &art,
    );
    assert!(!result.passed);
    assert!(result.violations[0].message.contains("SEC004"));
}

#[test]
fn test_security_curl_k() {
    let art = shell_artifact();
    let result = check_rule(RuleId::Security, "curl -k https://example.com", &art);
    assert!(!result.passed);
    assert!(result.violations[0].message.contains("SEC004"));
}

#[test]
fn test_security_hardcoded_api_key() {
    let art = shell_artifact();
    let result = check_rule(RuleId::Security, "API_KEY=\"abc123\"", &art);
    assert!(!result.passed);
    assert!(result.violations[0].message.contains("SEC005"));
}

#[test]
fn test_security_hardcoded_token_prefix() {
    let art = shell_artifact();
    // sk- prefix detected
    let result = check_rule(RuleId::Security, "TOKEN=sk-1234567890abcdef", &art);
    assert!(!result.passed);
    assert!(result.violations[0].message.contains("SEC005"));

    // ghp_ prefix detected
    let result2 = check_rule(RuleId::Security, "GH_TOKEN=ghp_abcdef123456", &art);
    assert!(!result2.passed);
    assert!(result2.violations[0].message.contains("SEC005"));
}

#[test]
fn test_security_secret_variable_ref_ok() {
    let art = shell_artifact();
    // PASSWORD assigned from variable expansion is OK
    let result = check_rule(RuleId::Security, "PASSWORD=\"$VAULT_PASS\"", &art);
    assert!(result.passed);
}

#[test]
fn test_security_unsafe_tmp() {
    let art = shell_artifact();
    let result = check_rule(RuleId::Security, "TMPFILE=\"/tmp/foo.txt\"", &art);
    assert!(!result.passed);
    assert!(result.violations[0].message.contains("SEC006"));
}

#[test]
fn test_security_mktemp_ok() {
    let art = shell_artifact();
    let result = check_rule(RuleId::Security, "TMPFILE=$(mktemp)", &art);
    // No SEC006 (mktemp is safe) but check_determinism would flag mktemp separately
    // For security check only, this should pass
    assert!(result.passed);
}

#[test]
fn test_security_sudo_rm_rf_unquoted() {
    let art = shell_artifact();
    let result = check_rule(RuleId::Security, "sudo rm -rf $DIR", &art);
    assert!(!result.passed);
    assert!(result.violations[0].message.contains("SEC007"));
}

#[test]
fn test_security_sudo_safe() {
    let art = shell_artifact();
    // Quoted variable: sudo rm -rf "$DIR"
    let result = check_rule(RuleId::Security, "sudo rm -rf \"$DIR\"", &art);
    assert!(result.passed);
}

#[test]
fn test_security_comments_skipped() {
    let art = shell_artifact();
    let content = "#!/bin/sh\n# eval \"$DANGER\"\necho safe\n";
    let result = check_rule(RuleId::Security, content, &art);
    assert!(result.passed);
}

// =============================================================================
// Quoting (COMPLY-005)
// =============================================================================

#[test]
fn test_quoting_unquoted_var() {
    let art = shell_artifact();
    let result = check_rule(RuleId::Quoting, "echo $FOO", &art);
    assert!(!result.passed);
    assert!(result.violations[0].message.contains("Unquoted variable"));
}

#[test]
fn test_quoting_quoted_var() {
    let art = shell_artifact();
    let result = check_rule(RuleId::Quoting, "echo \"$FOO\"", &art);
    assert!(result.passed);
}

#[test]
fn test_quoting_single_quoted() {
    let art = shell_artifact();
    let result = check_rule(RuleId::Quoting, "echo '$FOO'", &art);
    assert!(result.passed);
}

#[test]
fn test_quoting_subshell_ok() {
    let art = shell_artifact();
    let result = check_rule(RuleId::Quoting, "echo $(cmd)", &art);
    assert!(result.passed);
}

#[test]
fn test_quoting_backslash_escaped() {
    let art = shell_artifact();
    let result = check_rule(RuleId::Quoting, "echo \\$FOO", &art);
    assert!(result.passed);
}

#[test]
fn test_quoting_comments_skipped() {
    let art = shell_artifact();
    let content = "#!/bin/sh\n# echo $UNQUOTED\necho \"$SAFE\"\n";
    let result = check_rule(RuleId::Quoting, content, &art);
    assert!(result.passed);
}

// =============================================================================
// POSIX (COMPLY-001)
// =============================================================================

#[test]
fn test_posix_bash_shebang() {
    let art = shell_artifact();
    let result = check_rule(RuleId::Posix, "#!/bin/bash\necho hello\n", &art);
    assert!(!result.passed);
    assert!(result.violations[0].message.contains("Non-POSIX shebang"));
}

#[test]
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

