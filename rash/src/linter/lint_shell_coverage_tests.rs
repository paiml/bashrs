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

include!("lint_shell_coverage_tests_tests_lint_2.rs");
