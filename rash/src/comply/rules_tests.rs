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
    let result = check_rule(RuleId::Determinism, "dd if=/dev/random of=key bs=32 count=1", &art);
    assert!(!result.passed);
    assert!(result.violations[0].message.contains("/dev/urandom or /dev/random"));
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
fn test_idempotency_mkdir_no_p() {
    let art = shell_artifact();
    let result = check_rule(RuleId::Idempotency, "mkdir /opt/app", &art);
    assert!(!result.passed);
    assert!(result.violations[0].message.contains("mkdir without -p"));
}

#[test]
fn test_idempotency_mkdir_p_ok() {
    let art = shell_artifact();
    let result = check_rule(RuleId::Idempotency, "mkdir -p /opt/app", &art);
    assert!(result.passed);
}

#[test]
fn test_idempotency_rm_no_f() {
    let art = shell_artifact();
    let result = check_rule(RuleId::Idempotency, "rm /tmp/file.txt", &art);
    assert!(!result.passed);
    assert!(result.violations[0].message.contains("rm without -f"));
}

#[test]
fn test_idempotency_rm_f_ok() {
    let art = shell_artifact();
    let result = check_rule(RuleId::Idempotency, "rm -f /tmp/file.txt", &art);
    assert!(result.passed);
}

#[test]
fn test_idempotency_rm_rf_ok() {
    let art = shell_artifact();
    let result = check_rule(RuleId::Idempotency, "rm -rf /tmp/dir", &art);
    assert!(result.passed);
}

#[test]
fn test_idempotency_ln_s_no_f() {
    let art = shell_artifact();
    let result = check_rule(RuleId::Idempotency, "ln -s /src /dst", &art);
    assert!(!result.passed);
    assert!(result.violations[0].message.contains("ln -s without -f"));
}

#[test]
fn test_idempotency_ln_sf_ok() {
    let art = shell_artifact();
    let result = check_rule(RuleId::Idempotency, "ln -sf /src /dst", &art);
    assert!(result.passed);
}

#[test]
fn test_idempotency_useradd_no_guard() {
    let art = shell_artifact();
    let result = check_rule(RuleId::Idempotency, "useradd appuser", &art);
    assert!(!result.passed);
    assert!(result.violations[0].message.contains("useradd/groupadd"));
}

#[test]
fn test_idempotency_useradd_guarded() {
    let art = shell_artifact();
    let result = check_rule(RuleId::Idempotency, "useradd appuser || true", &art);
    assert!(result.passed);
}

#[test]
fn test_idempotency_groupadd_no_guard() {
    let art = shell_artifact();
    let result = check_rule(RuleId::Idempotency, "groupadd devs", &art);
    assert!(!result.passed);
    assert!(result.violations[0].message.contains("useradd/groupadd"));
}

#[test]
fn test_idempotency_git_clone_unguarded() {
    let art = shell_artifact();
    let result = check_rule(
        RuleId::Idempotency,
        "git clone https://github.com/user/repo.git",
        &art,
    );
    assert!(!result.passed);
    assert!(result.violations[0].message.contains("git clone"));
}

#[test]
fn test_idempotency_git_clone_guarded() {
    let art = shell_artifact();
    let result = check_rule(
        RuleId::Idempotency,
        "[ -d repo ] || git clone https://github.com/user/repo.git",
        &art,
    );
    // The line starts with [ -d, not git clone, so the is_unguarded_git_clone check does not match
    assert!(result.passed);
}

#[test]
fn test_idempotency_createdb_unguarded() {
    let art = shell_artifact();
    let result = check_rule(RuleId::Idempotency, "createdb myapp", &art);
    assert!(!result.passed);
    assert!(result.violations[0].message.contains("createdb"));
}

#[test]
fn test_idempotency_createdb_guarded() {
    let art = shell_artifact();
    let result = check_rule(RuleId::Idempotency, "createdb myapp || true", &art);
    assert!(result.passed);
}

#[test]
fn test_idempotency_append_config() {
    let art = shell_artifact();
    let result = check_rule(
        RuleId::Idempotency,
        "echo 'export PATH=/usr/local/bin:$PATH' >> .bashrc",
        &art,
    );
    assert!(!result.passed);
    assert!(result.violations[0].message.contains(">> append"));
}

#[test]
fn test_idempotency_append_config_guarded() {
    let art = shell_artifact();
    let result = check_rule(
        RuleId::Idempotency,
        "grep -q 'export PATH' .bashrc || echo 'export PATH=/usr/local/bin:$PATH' >> .bashrc",
        &art,
    );
    assert!(result.passed);
}

#[test]
fn test_idempotency_non_shell_artifact() {
    let art = makefile_artifact();
    let result = check_rule(RuleId::Idempotency, "mkdir /opt/app", &art);
    // Makefile is not ShellScript or ShellConfig, so idempotency checks are skipped
    assert!(result.passed);
    assert!(result.violations.is_empty());
}

// =============================================================================
// Security (COMPLY-004)
// =============================================================================

#[test]
fn test_security_eval_injection() {
    let art = shell_artifact();
    let result = check_rule(RuleId::Security, "eval \"$USER_INPUT\"", &art);
    assert!(!result.passed);
    assert!(result.violations[0].message.contains("SEC001"));
}

#[test]
fn test_security_eval_safe() {
    let art = shell_artifact();
    // eval without $ or backtick is not flagged
    let result = check_rule(RuleId::Security, "eval set -- foo bar", &art);
    assert!(result.passed);
}

#[test]
fn test_security_curl_pipe_bash() {
    let art = shell_artifact();
    let result = check_rule(
        RuleId::Security,
        "curl -fsSL https://example.com/install.sh | bash",
        &art,
    );
    assert!(!result.passed);
    assert!(result.violations[0].message.contains("SEC002"));
}

#[test]
fn test_security_wget_pipe_sh() {
    let art = shell_artifact();
    let result = check_rule(
        RuleId::Security,
        "wget -q https://example.com/setup.sh | sh",
        &art,
    );
    assert!(!result.passed);
    assert!(result.violations[0].message.contains("SEC002"));
}

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
    let result = check_rule(RuleId::ShellCheck, "if [ $? -eq 0 ]; then echo ok; fi", &art);
    assert!(!result.passed);
    assert!(result.violations[0].message.contains("SC2181"));
}

#[test]
fn test_shellcheck_ls_iteration() {
    let art = shell_artifact();
    let result = check_rule(RuleId::ShellCheck, "for f in $(ls *.txt); do echo $f; done", &art);
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
    assert!(result.violations.iter().any(|v| v.message.contains("MAKE001")));
}

#[test]
fn test_make_bare_make() {
    let art = makefile_artifact();
    let content = "all:\n\tmake clean";
    let result = check_rule(RuleId::MakefileSafety, content, &art);
    assert!(!result.passed);
    assert!(result.violations.iter().any(|v| v.message.contains("MAKE002")));
}

#[test]
fn test_make_dollar_make_ok() {
    let art = makefile_artifact();
    let content = "all:\n\t$(MAKE) clean";
    let result = check_rule(RuleId::MakefileSafety, content, &art);
    // No MAKE002 violation ($(MAKE) is correct)
    assert!(
        !result.violations.iter().any(|v| v.message.contains("MAKE002")),
        "$(MAKE) should not trigger MAKE002"
    );
}

#[test]
fn test_make_rm_rf_variable() {
    let art = makefile_artifact();
    let content = "clean:\n\trm -rf $$dir";
    let result = check_rule(RuleId::MakefileSafety, content, &art);
    assert!(!result.passed);
    assert!(result.violations.iter().any(|v| v.message.contains("MAKE003")));
}

#[test]
fn test_make_missing_phony() {
    let art = makefile_artifact();
    // "clean:" target without .PHONY declaration
    let content = "clean:\n\trm -f *.o";
    let result = check_rule(RuleId::MakefileSafety, content, &art);
    assert!(!result.passed);
    assert!(result.violations.iter().any(|v| v.message.contains("MAKE004")));
}

#[test]
fn test_make_phony_declared_ok() {
    let art = makefile_artifact();
    let content = ".PHONY: clean\nclean:\n\trm -f *.o";
    let result = check_rule(RuleId::MakefileSafety, content, &art);
    // No MAKE004 violation for clean
    assert!(
        !result.violations.iter().any(|v| v.message.contains("MAKE004")),
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
    assert!(result.violations.iter().any(|v| v.message.contains("DOCKER008")));
}

#[test]
fn test_docker_add_url_ok() {
    let art = dockerfile_artifact();
    let content = "FROM ubuntu:22.04\nADD https://example.com/file.tar.gz /opt\nUSER appuser";
    let result = check_rule(RuleId::DockerfileBest, content, &art);
    // ADD with URL is acceptable
    assert!(
        !result.violations.iter().any(|v| v.message.contains("DOCKER008")),
        "ADD with https URL should not trigger DOCKER008"
    );
}

#[test]
fn test_docker_unpinned_from() {
    let art = dockerfile_artifact();
    let content = "FROM ubuntu\nRUN echo hello\nUSER appuser";
    let result = check_rule(RuleId::DockerfileBest, content, &art);
    assert!(!result.passed);
    assert!(result.violations.iter().any(|v| v.message.contains("DOCKER001")));
}

#[test]
fn test_docker_latest_from() {
    let art = dockerfile_artifact();
    let content = "FROM ubuntu:latest\nRUN echo hello\nUSER appuser";
    let result = check_rule(RuleId::DockerfileBest, content, &art);
    assert!(!result.passed);
    assert!(result.violations.iter().any(|v| v.message.contains("DOCKER001")));
}

#[test]
fn test_docker_pinned_from_ok() {
    let art = dockerfile_artifact();
    let content = "FROM ubuntu:22.04\nRUN echo hello\nUSER appuser";
    let result = check_rule(RuleId::DockerfileBest, content, &art);
    assert!(
        !result.violations.iter().any(|v| v.message.contains("DOCKER001")),
        "Pinned FROM should not trigger DOCKER001"
    );
}

#[test]
fn test_docker_digest_ok() {
    let art = dockerfile_artifact();
    let content =
        "FROM ubuntu@sha256:abcdef1234567890\nRUN echo hello\nUSER appuser";
    let result = check_rule(RuleId::DockerfileBest, content, &art);
    assert!(
        !result.violations.iter().any(|v| v.message.contains("DOCKER001")),
        "FROM with digest should not trigger DOCKER001"
    );
}

#[test]
fn test_docker_scratch_ok() {
    let art = dockerfile_artifact();
    let content = "FROM scratch\nCOPY app /app\nUSER appuser";
    let result = check_rule(RuleId::DockerfileBest, content, &art);
    assert!(
        !result.violations.iter().any(|v| v.message.contains("DOCKER001")),
        "FROM scratch should not trigger DOCKER001"
    );
}

#[test]
fn test_docker_apt_no_clean() {
    let art = dockerfile_artifact();
    let content = "FROM ubuntu:22.04\nRUN apt-get install -y curl\nUSER appuser";
    let result = check_rule(RuleId::DockerfileBest, content, &art);
    assert!(!result.passed);
    assert!(result.violations.iter().any(|v| v.message.contains("DOCKER003")));
}

#[test]
fn test_docker_apt_with_clean_ok() {
    let art = dockerfile_artifact();
    let content = "FROM ubuntu:22.04\nRUN apt-get install -y curl && rm -rf /var/lib/apt/lists/*\nUSER appuser";
    let result = check_rule(RuleId::DockerfileBest, content, &art);
    assert!(
        !result.violations.iter().any(|v| v.message.contains("DOCKER003")),
        "apt-get with cleanup should not trigger DOCKER003"
    );
}

#[test]
fn test_docker_bare_expose() {
    let art = dockerfile_artifact();
    let content = "FROM ubuntu:22.04\nEXPOSE\nUSER appuser";
    let result = check_rule(RuleId::DockerfileBest, content, &art);
    assert!(!result.passed);
    assert!(result.violations.iter().any(|v| v.message.contains("DOCKER004")));
}

#[test]
fn test_docker_missing_user() {
    let art = dockerfile_artifact();
    let content = "FROM ubuntu:22.04\nRUN echo hello";
    let result = check_rule(RuleId::DockerfileBest, content, &art);
    assert!(!result.passed);
    assert!(result.violations.iter().any(|v| v.message.contains("DOCKER010")));
}

#[test]
fn test_docker_has_user_ok() {
    let art = dockerfile_artifact();
    let content = "FROM ubuntu:22.04\nRUN echo hello\nUSER appuser";
    let result = check_rule(RuleId::DockerfileBest, content, &art);
    assert!(
        !result.violations.iter().any(|v| v.message.contains("DOCKER010")),
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
