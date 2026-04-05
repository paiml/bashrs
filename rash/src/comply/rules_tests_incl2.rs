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

include!("rules_tests_incl2_incl2.rs");
