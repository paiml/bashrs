#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]
#![allow(non_snake_case)]
//! Provable Contract Tests: linter-security-rules-v1.yaml
//!
//! Each test pair attempts to FALSIFY a security rule contract.
//! SOUND tests verify detection (must find the vulnerability).
//! PRECISE tests verify no false positives (must NOT flag safe code).
//!
//! Reference: GH-183 (KZ-11: Missing provable contracts)

/// Helper: check if lint result contains a specific rule code
fn has_diagnostic(source: &str, code: &str) -> bool {
    let result = bashrs::linter::lint_shell(source);
    result.diagnostics.iter().any(|d| d.code == code)
}

// ============================================================================
// F-SEC001: eval injection
// ============================================================================

#[test]
fn falsify_SEC001_SOUND_eval_with_variable() {
    assert!(
        has_diagnostic(r#"eval "$user_input""#, "SEC001"),
        "F-SEC001-SOUND: eval with variable input MUST be flagged"
    );
}

#[test]
fn falsify_SEC001_SOUND_eval_with_backtick() {
    assert!(
        has_diagnostic("eval `cat /etc/passwd`", "SEC001"),
        "F-SEC001-SOUND: eval with backtick MUST be flagged"
    );
}

#[test]
fn falsify_SEC001_PRECISE_embedded_word() {
    // "eval" as substring of another word must NOT be flagged
    assert!(
        !has_diagnostic("medieval=true", "SEC001"),
        "F-SEC001-PRECISE: 'eval' as substring must NOT be flagged"
    );
}

#[test]
fn falsify_SEC001_PRECISE_safe_eval_indirection() {
    // $(eval "printf '%s' ...") is safe POSIX array indirection — not injection
    assert!(
        !has_diagnostic(r#"val="$(eval "printf '%s' \"$arr_0\"")""#, "SEC001"),
        "F-SEC001-PRECISE: safe eval indirection pattern must NOT be flagged"
    );
}

// ============================================================================
// F-SEC002: unquoted variable in dangerous command
// ============================================================================

#[test]
fn falsify_SEC002_SOUND_curl_unquoted() {
    assert!(
        has_diagnostic("curl $URL", "SEC002"),
        "F-SEC002-SOUND: curl with unquoted variable MUST be flagged"
    );
}

#[test]
fn falsify_SEC002_SOUND_wget_unquoted() {
    assert!(
        has_diagnostic("wget $REMOTE_FILE", "SEC002"),
        "F-SEC002-SOUND: wget with unquoted variable MUST be flagged"
    );
}

#[test]
fn falsify_SEC002_PRECISE_curl_quoted() {
    assert!(
        !has_diagnostic(r#"curl "$URL""#, "SEC002"),
        "F-SEC002-PRECISE: curl with quoted variable must NOT be flagged"
    );
}

// ============================================================================
// F-SEC003: find -exec sh -c with {}
// ============================================================================

#[test]
fn falsify_SEC003_SOUND_find_exec_sh_c_braces() {
    assert!(
        has_diagnostic("find /tmp -exec sh -c 'echo {}' \\;", "SEC003"),
        "F-SEC003-SOUND: find -exec sh -c with {{}} MUST be flagged"
    );
}

#[test]
fn falsify_SEC003_PRECISE_find_exec_direct() {
    assert!(
        !has_diagnostic("find /tmp -exec rm {} \\;", "SEC003"),
        "F-SEC003-PRECISE: find -exec without sh -c must NOT be flagged"
    );
}

// ============================================================================
// F-SEC004: TLS verification disabled
// ============================================================================

#[test]
fn falsify_SEC004_SOUND_curl_insecure() {
    assert!(
        has_diagnostic("curl --insecure https://example.com", "SEC004"),
        "F-SEC004-SOUND: curl --insecure MUST be flagged"
    );
}

#[test]
fn falsify_SEC004_SOUND_wget_no_check() {
    assert!(
        has_diagnostic("wget --no-check-certificate https://example.com", "SEC004"),
        "F-SEC004-SOUND: wget --no-check-certificate MUST be flagged"
    );
}

#[test]
fn falsify_SEC004_PRECISE_curl_normal() {
    assert!(
        !has_diagnostic("curl https://example.com", "SEC004"),
        "F-SEC004-PRECISE: curl with TLS enabled must NOT be flagged"
    );
}

// ============================================================================
// F-SEC005: hardcoded secrets
// ============================================================================

#[test]
fn falsify_SEC005_SOUND_api_key() {
    assert!(
        has_diagnostic(
            r#"API_KEY="sk-1234567890abcdef1234567890abcdef""#,
            "SEC005"
        ),
        "F-SEC005-SOUND: hardcoded API key MUST be flagged"
    );
}

#[test]
fn falsify_SEC005_PRECISE_normal_var() {
    assert!(
        !has_diagnostic(r#"LOG_LEVEL="debug""#, "SEC005"),
        "F-SEC005-PRECISE: normal variable assignment must NOT be flagged"
    );
}

// ============================================================================
// F-SEC006: unsafe temporary files
// ============================================================================

#[test]
fn falsify_SEC006_SOUND_predictable_tmp() {
    assert!(
        has_diagnostic(r#"TMPFILE="/tmp/myapp_output""#, "SEC006"),
        "F-SEC006-SOUND: predictable temp file MUST be flagged"
    );
}

#[test]
fn falsify_SEC006_PRECISE_mktemp() {
    assert!(
        !has_diagnostic("tmpfile=$(mktemp)", "SEC006"),
        "F-SEC006-PRECISE: mktemp usage must NOT be flagged"
    );
}

// ============================================================================
// F-SEC007: sudo without validation
// ============================================================================

#[test]
fn falsify_SEC007_SOUND_sudo_rm_rf_variable() {
    assert!(
        has_diagnostic("sudo rm -rf $DIR", "SEC007"),
        "F-SEC007-SOUND: sudo rm -rf with unquoted variable MUST be flagged"
    );
}

#[test]
fn falsify_SEC007_SOUND_sudo_chmod_777_variable() {
    assert!(
        has_diagnostic("sudo chmod 777 $FILE", "SEC007"),
        "F-SEC007-SOUND: sudo chmod 777 with unquoted variable MUST be flagged"
    );
}

#[test]
fn falsify_SEC007_PRECISE_sudo_literal() {
    assert!(
        !has_diagnostic("sudo apt-get update", "SEC007"),
        "F-SEC007-PRECISE: sudo with literal command must NOT be flagged"
    );
}

// ============================================================================
// F-SEC008: curl | sh pattern
// ============================================================================

#[test]
fn falsify_SEC008_SOUND_curl_pipe_bash() {
    assert!(
        has_diagnostic("curl https://example.com/install.sh | bash", "SEC008"),
        "F-SEC008-SOUND: curl piped to bash MUST be flagged"
    );
}

#[test]
fn falsify_SEC008_SOUND_wget_pipe_sh() {
    assert!(
        has_diagnostic("wget -qO- https://example.com | sh", "SEC008"),
        "F-SEC008-SOUND: wget piped to sh MUST be flagged"
    );
}

#[test]
fn falsify_SEC008_PRECISE_curl_to_file() {
    assert!(
        !has_diagnostic("curl -o install.sh https://example.com/install.sh", "SEC008"),
        "F-SEC008-PRECISE: curl to file must NOT be flagged"
    );
}
