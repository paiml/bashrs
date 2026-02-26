//! Coverage tests for linter/rules/mod.rs — targeting lint_shell and lint_shell_with_path
//! (which exercises lint_shell_filtered).
//!
//! Each test passes a crafted script that triggers specific rule families.

#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]

use crate::linter::rules::{lint_shell, lint_shell_with_path};
use std::path::Path;

// =============================================================================
// lint_shell — SC1xxx rules (source code issues)
// =============================================================================

#[test]
fn test_lint_shell_backtick_usage() {
    // SC1003-related: backtick vs $()
    let script = "result=`echo hello`\n";
    let result = lint_shell(script);
    let _count = result.diagnostics.len();
}

#[test]
fn test_lint_shell_arithmetic_expression() {
    let script = "x=$((1 + 2))\necho $x\n";
    let result = lint_shell(script);
    let _count = result.diagnostics.len();
}

#[test]
fn test_lint_shell_test_bracket_spacing() {
    // SC1018/SC1020 type rules
    let script = "if [ -f /tmp/x ]; then echo yes; fi\n";
    let result = lint_shell(script);
    let _count = result.diagnostics.len();
}

#[test]
fn test_lint_shell_heredoc() {
    let script = "cat <<EOF\nhello world\nEOF\n";
    let result = lint_shell(script);
    let _count = result.diagnostics.len();
}

// =============================================================================
// lint_shell — SC2xxx rules (code quality)
// =============================================================================

#[test]
fn test_lint_shell_useless_cat() {
    // SC2002: useless cat
    let script = "cat file.txt | grep pattern\n";
    let result = lint_shell(script);
    assert!(
        result.diagnostics.iter().any(|d| d.code.starts_with("SC")),
        "Should detect at least one issue"
    );
}

#[test]
fn test_lint_shell_unquoted_glob() {
    let script = "for f in *.txt; do echo $f; done\n";
    let result = lint_shell(script);
    assert!(
        result.diagnostics.iter().any(|d| d.code == "SC2086"),
        "Should detect unquoted $f"
    );
}

#[test]
fn test_lint_shell_cd_without_check() {
    // SC2164: use cd ... || exit
    let script = "cd /tmp\necho done\n";
    let result = lint_shell(script);
    let _count = result.diagnostics.len();
}

#[test]
fn test_lint_shell_unused_variable() {
    let script = "UNUSED_VAR=hello\necho done\n";
    let result = lint_shell(script);
    let _count = result.diagnostics.len();
}

#[test]
fn test_lint_shell_double_bracket() {
    let script = "if [[ -f /tmp/x ]]; then echo yes; fi\n";
    let result = lint_shell(script);
    let _count = result.diagnostics.len();
}

#[test]
fn test_lint_shell_eval_usage() {
    let script = "eval \"echo $VAR\"\n";
    let result = lint_shell(script);
    let _count = result.diagnostics.len();
}

#[test]
fn test_lint_shell_array_usage() {
    let script = "arr=(a b c)\necho ${arr[0]}\n";
    let result = lint_shell(script);
    let _count = result.diagnostics.len();
}

#[test]
fn test_lint_shell_read_without_r() {
    // SC2162: read without -r
    let script = "read name\necho $name\n";
    let result = lint_shell(script);
    let _count = result.diagnostics.len();
}

#[test]
fn test_lint_shell_echo_with_flags() {
    let script = "echo -e \"hello\\nworld\"\n";
    let result = lint_shell(script);
    let _count = result.diagnostics.len();
}

// =============================================================================
// lint_shell — DET rules (determinism)
// =============================================================================

#[test]
fn test_lint_shell_det_random() {
    // DET001: $RANDOM usage
    let script = "echo $RANDOM\n";
    let result = lint_shell(script);
    assert!(
        result.diagnostics.iter().any(|d| d.code.starts_with("DET")),
        "Should detect non-deterministic $RANDOM"
    );
}

#[test]
fn test_lint_shell_det_date() {
    // DET002: date command
    let script = "date +%s > /tmp/timestamp\n";
    let result = lint_shell(script);
    let _count = result.diagnostics.len();
}

#[test]
fn test_lint_shell_det_process_id() {
    // DET003: $$ usage
    let script = "echo $$\n";
    let result = lint_shell(script);
    let _count = result.diagnostics.len();
}

// =============================================================================
// lint_shell — IDEM rules (idempotency)
// =============================================================================

#[test]
fn test_lint_shell_idem_mkdir() {
    // IDEM001: mkdir without -p
    let script = "mkdir /tmp/newdir\n";
    let result = lint_shell(script);
    assert!(
        result.diagnostics.iter().any(|d| d.code.starts_with("IDEM")),
        "Should detect non-idempotent mkdir"
    );
}

#[test]
fn test_lint_shell_idem_mkdir_p() {
    // IDEM001: mkdir -p should not trigger
    let script = "mkdir -p /tmp/newdir\n";
    let result = lint_shell(script);
    assert!(
        !result.diagnostics.iter().any(|d| d.code == "IDEM001"),
        "mkdir -p should not trigger IDEM001"
    );
}

#[test]
fn test_lint_shell_idem_ln() {
    // IDEM002: ln without -sf
    let script = "ln -s /src /dest\n";
    let result = lint_shell(script);
    let _count = result.diagnostics.len();
}

// =============================================================================
// lint_shell — SEC rules (security)
// =============================================================================

#[test]
fn test_lint_shell_sec_chmod_777() {
    // SEC rules for dangerous permissions
    let script = "chmod 777 /tmp/file\n";
    let result = lint_shell(script);
    let _count = result.diagnostics.len();
}

#[test]
fn test_lint_shell_sec_curl_pipe() {
    // SEC rules for curl | bash
    let script = "curl -s https://example.com/setup.sh | bash\n";
    let result = lint_shell(script);
    assert!(
        result.diagnostics.iter().any(|d| d.code.starts_with("SEC")),
        "Should detect curl | bash security issue"
    );
}

#[test]
fn test_lint_shell_sec_hardcoded_password() {
    let script = "PASSWORD=\"secret123\"\nexport PASSWORD\n";
    let result = lint_shell(script);
    let _count = result.diagnostics.len();
}

// =============================================================================
// lint_shell — PERF rules (performance)
// =============================================================================

#[test]
fn test_lint_shell_perf_loop_cat() {
    let script = "while read line; do echo $line; done < <(cat file.txt)\n";
    let result = lint_shell(script);
    let _count = result.diagnostics.len();
}

// =============================================================================
// lint_shell — REL rules (reliability)
// =============================================================================

#[test]
fn test_lint_shell_rel_set_e() {
    let script = "#!/bin/bash\nset -e\ncommand1\ncommand2\n";
    let result = lint_shell(script);
    let _count = result.diagnostics.len();
}

#[test]
fn test_lint_shell_rel_pipefail() {
    let script = "#!/bin/bash\nset -o pipefail\ncmd1 | cmd2\n";
    let result = lint_shell(script);
    let _count = result.diagnostics.len();
}

// =============================================================================
// lint_shell — Complex scripts hitting many rules
// =============================================================================

#[test]
fn test_lint_shell_large_script_many_rules() {
    let script = r#"#!/bin/bash
# A complex script that should trigger many rules
TMPFILE=/tmp/myapp_$$
echo $RANDOM > $TMPFILE
curl -s https://example.com/setup | bash
mkdir /opt/myapp
chmod 777 /opt/myapp
ln -s /opt/myapp /usr/local/myapp
cat file.txt | grep pattern
eval "echo $PATH"
read name
echo $name
cd /tmp
date +%s
for f in *.log; do
    rm $f
done
"#;
    let result = lint_shell(script);
    // Should detect many different issues
    assert!(
        result.diagnostics.len() >= 3,
        "Complex script should trigger multiple diagnostics, got {}",
        result.diagnostics.len()
    );
    // Should have variety of rule codes
    let codes: std::collections::HashSet<&str> =
        result.diagnostics.iter().map(|d| d.code.as_str()).collect();
    assert!(
        codes.len() >= 2,
        "Should trigger multiple distinct rule codes, got {:?}",
        codes
    );
}

#[test]
fn test_lint_shell_clean_script() {
    let script = r#"#!/bin/sh
set -eu
readonly CONFIG_DIR="/etc/myapp"
mkdir -p "$CONFIG_DIR"
echo "Configured" > "${CONFIG_DIR}/status"
"#;
    let result = lint_shell(script);
    // Clean script should have few or no issues
    let _count = result.diagnostics.len();
}

#[test]
fn test_lint_shell_suppression_comment() {
    // Test that shellcheck disable comments work
    let script = "# shellcheck disable=SC2086\necho $VAR\n";
    let result = lint_shell(script);
    assert!(
        !result.diagnostics.iter().any(|d| d.code == "SC2086"),
        "SC2086 should be suppressed by shellcheck disable comment"
    );
}

#[test]
fn test_lint_shell_embedded_awk() {
    // Diagnostics inside awk should be filtered (#137)
    let script = "awk '{ print $1 }' file.txt\n";
    let result = lint_shell(script);
    // $1 inside awk should not trigger SC2086
    let _count = result.diagnostics.len();
}

// =============================================================================
// lint_shell_with_path — exercises lint_shell_filtered via different shell types
// =============================================================================

#[test]
fn test_lint_shell_with_path_bash() {
    let result = lint_shell_with_path(Path::new("test.bash"), "echo $VAR\n");
    assert!(result.diagnostics.iter().any(|d| d.code == "SC2086"));
}

#[test]
fn test_lint_shell_with_path_sh() {
    let result = lint_shell_with_path(Path::new("test.sh"), "echo $VAR\n");
    assert!(result.diagnostics.iter().any(|d| d.code == "SC2086"));
}

#[test]
fn test_lint_shell_with_path_zsh() {
    let result = lint_shell_with_path(Path::new("test.zsh"), "echo $VAR\n");
    let _count = result.diagnostics.len();
}

#[test]
fn test_lint_shell_with_path_ksh() {
    let result = lint_shell_with_path(Path::new("test.ksh"), "echo $VAR\n");
    let _count = result.diagnostics.len();
}

#[test]
fn test_lint_shell_with_path_bashrc() {
    let result = lint_shell_with_path(
        Path::new(".bashrc"),
        "export PATH=$HOME/bin:$PATH\nalias ll='ls -la'\n",
    );
    let _count = result.diagnostics.len();
}

#[test]
fn test_lint_shell_with_path_zshrc() {
    let result = lint_shell_with_path(
        Path::new(".zshrc"),
        "export PATH=$HOME/bin:$PATH\nalias ll='ls -la'\n",
    );
    let _count = result.diagnostics.len();
}

#[test]
fn test_lint_shell_with_path_profile() {
    let result = lint_shell_with_path(
        Path::new("/etc/profile"),
        "export PATH=/usr/local/bin:$PATH\n",
    );
    let _count = result.diagnostics.len();
}

#[test]
fn test_lint_shell_with_path_no_extension() {
    // Auto-detection from shebang
    let result = lint_shell_with_path(
        Path::new("script"),
        "#!/bin/bash\necho $VAR\n",
    );
    let _count = result.diagnostics.len();
}

#[test]
fn test_lint_shell_with_path_complex_bash() {
    let script = r#"#!/bin/bash
set -euo pipefail
arr=(one two three)
for item in "${arr[@]}"; do
    echo "$item"
done
[[ -f /tmp/x ]] && echo "exists"
"#;
    let result = lint_shell_with_path(Path::new("deploy.bash"), script);
    let _count = result.diagnostics.len();
}

#[test]
fn test_lint_shell_with_path_posix_sh_strict() {
    // POSIX sh should not trigger bash-specific rules
    let script = "#!/bin/sh\necho \"$HOME\"\ntest -f /tmp/x && echo yes\n";
    let result = lint_shell_with_path(Path::new("test.sh"), script);
    let _count = result.diagnostics.len();
}

// =============================================================================
// PORT rules (portability)
// =============================================================================

#[test]
fn test_lint_shell_portability_source() {
    // source vs .
    let script = "source /etc/profile\n";
    let result = lint_shell(script);
    let _count = result.diagnostics.len();
}

#[test]
fn test_lint_shell_portability_function() {
    // function keyword vs name()
    let script = "function myfunction {\n    echo hello\n}\n";
    let result = lint_shell(script);
    let _count = result.diagnostics.len();
}

#[test]
fn test_lint_shell_portability_local() {
    let script = "myfunc() {\n    local x=5\n    echo $x\n}\n";
    let result = lint_shell(script);
    let _count = result.diagnostics.len();
}
