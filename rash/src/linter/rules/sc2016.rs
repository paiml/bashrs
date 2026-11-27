// SC2016: Expressions don't expand in single quotes, use double quotes for that
//
// Single quotes preserve literal strings - variables and command substitutions
// don't expand inside them. Use double quotes if you want expansion.
//
// Examples:
// Bad:
//   echo 'Hello $USER'              // Prints literal "$USER"
//   msg='Value: $value'             // $value doesn't expand
//   cmd='$(date)'                   // Command doesn't run
//
// Good:
//   echo "Hello $USER"              // Variable expands
//   msg="Value: $value"             // Variable expands
//   cmd="$(date)"                   // Command runs
//   literal='$50 per item'          // OK - intentional literal
//
// Note: This rule detects likely mistakes where users expect expansion
// but use single quotes. Intentional literals with $ are acceptable.
//
// bashrs#335: quotes are paired inside each word a quote-aware lexer delimited
// (`quoted_segments`), never across the raw line. The regex this replaced
// reported ` $x ` in `f 'a' $x 'b'` as single-quoted text.

use crate::linter::quoted_segments::single_quoted_segments;
use crate::linter::{Diagnostic, LintResult, Severity, Span};
use regex::Regex;

#[allow(clippy::unwrap_used)] // Compile-time regex, panic on invalid pattern is acceptable
static EXPANSION_IN_CONTENT: std::sync::LazyLock<Regex> = std::sync::LazyLock::new(|| {
    // Match: $var or ${var} or $(cmd), in the text between single quotes
    Regex::new(r"\$[a-zA-Z_][a-zA-Z0-9_]*|\$\{[^}]+\}|\$\([^)]+\)").unwrap()
});

/// F025: Check if the variable in single quotes is wrapped in double quotes
/// Pattern: '"$var"' or '${var}' inside quotes means it's likely documentation/example
fn is_documentation_pattern(matched: &str) -> bool {
    // Check if the variable is wrapped in double quotes (e.g., '"$var"')
    // This is a common pattern for showing shell syntax as literal text
    if matched.contains("\"$") || matched.contains("\"${") || matched.contains("\"$(") {
        return true;
    }

    // Check for JSON-like patterns (e.g., '{"key": "$val"}')
    if matched.contains('{') && matched.contains(':') && matched.contains("\"$") {
        return true;
    }

    false
}

/// Commands that run their arguments as a script (a remote shell, `sh -c`,
/// `eval`, a privilege or container wrapper). ShellCheck 0.8.0 does not report
/// SC2016 for single-quoted text that is an argument of one of these, because
/// the `$` is meant for the inner shell. `trap` is included (bashrs#668): its
/// action runs later, when the trap fires. Matched exactly, on the command's own
/// first word: `nohup ssh`, `env ssh`, `su -c` and `dash -c` are still reported.
const SCRIPT_RUNNERS: &[&str] = &[
    "ssh", "sh", "bash", "ksh", "zsh", "eval", "sudo", "docker", "podman", "trap",
];

/// Is this segment an argument of a [`SCRIPT_RUNNERS`] command?
fn is_script_argument(head: Option<&str>) -> bool {
    head.is_some_and(|h| SCRIPT_RUNNERS.contains(&h))
}

/// Is this single-quoted text likely an expansion its author expected to run?
fn expects_expansion(content: &str) -> bool {
    if !EXPANSION_IN_CONTENT.is_match(content) {
        return false;
    }
    // The filters below were written against the quoted match, so they are
    // handed the quoted text they were written for.
    let matched = format!("'{content}'");

    // Skip some common false positives
    // Skip if it's clearly a price/money (like '$50')
    if matched.contains("$0") || matched.contains("$1") && matched.len() < 10 {
        return false;
    }

    // F025: Skip if variable is in double quotes (documentation pattern)
    // e.g., 'Value: "$var"' or 'Use "$(cmd)"' are intentional literals
    !is_documentation_pattern(&matched)
}

pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();

    for (line_num, line) in source.lines().enumerate() {
        let line_num = line_num + 1;

        if line.trim_start().starts_with('#') {
            continue;
        }

        // Look for single-quoted strings with $ expressions
        for seg in single_quoted_segments(line)
            .into_iter()
            .filter(|seg| !is_script_argument(seg.head.as_deref()))
            .filter(|seg| expects_expansion(&seg.content))
        {
            result.add(Diagnostic::new(
                "SC2016",
                Severity::Info,
                "Expressions don't expand in single quotes, use double quotes for that".to_string(),
                Span::new(line_num, seg.start_col, line_num, seg.end_col),
            ));
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sc2016_var_in_single_quotes() {
        let code = r#"echo 'Hello $USER'"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
        assert_eq!(result.diagnostics[0].code, "SC2016");
        assert_eq!(result.diagnostics[0].severity, Severity::Info);
        assert!(result.diagnostics[0].message.contains("single quotes"));
    }

    #[test]
    fn test_sc2016_var_with_braces() {
        let code = r#"msg='Value: ${value}'"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2016_command_substitution() {
        let code = r#"cmd='$(date)'"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2016_multiple_vars() {
        let code = r#"text='$name is $age years old'"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2016_double_quotes_ok() {
        let code = r#"echo "Hello $USER""#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2016_literal_dollar_ok() {
        let code = r#"price='$50 per item'"#;
        let result = check(code);
        // Price pattern, likely intentional
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2016_no_dollar_ok() {
        let code = r#"msg='Hello World'"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2016_escaped_dollar_ok() {
        let code = r#"echo "\$USER""#;
        let result = check(code);
        // Not in single quotes
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2016_multiple_issues() {
        let code = r#"
msg1='Hello $USER'
msg2='Today is $(date)'
"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 2);
    }

    #[test]
    fn test_sc2016_in_assignment() {
        let code = r#"VAR='Current path: $PWD'"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    // ===== F025: Literal quotes in single-quoted strings =====
    // When single-quoted string contains quoted variables like "$var",
    // it's likely documentation or intentional literal output

    #[test]
    fn test_FP_025_literal_quoted_var_not_flagged() {
        // User wants to output literal '"$var"' - documentation pattern
        let code = r#"echo 'Value: "$var"'"#;
        let result = check(code);
        assert_eq!(
            result.diagnostics.len(),
            0,
            "SC2016 must NOT flag quotes inside literal (documentation pattern)"
        );
    }

    #[test]
    fn test_FP_025_syntax_example_not_flagged() {
        // Common pattern: showing shell syntax as literal text
        let code = r#"echo 'The syntax is: "$variable"'"#;
        let result = check(code);
        assert_eq!(
            result.diagnostics.len(),
            0,
            "SC2016 must NOT flag syntax examples in single quotes"
        );
    }

    #[test]
    fn test_FP_025_json_pattern_not_flagged() {
        // JSON-like patterns with quoted variables
        let code = r#"echo '{"name": "$name"}'"#;
        let result = check(code);
        assert_eq!(
            result.diagnostics.len(),
            0,
            "SC2016 must NOT flag JSON-like patterns"
        );
    }

    #[test]
    fn test_FP_025_bare_var_still_flagged() {
        // Bare variable without quotes should still be flagged (likely mistake)
        let code = r#"echo 'Value: $var'"#;
        let result = check(code);
        assert_eq!(
            result.diagnostics.len(),
            1,
            "SC2016 SHOULD flag bare variable in single quotes"
        );
    }

    #[test]
    fn test_FP_025_command_subst_in_quotes_not_flagged() {
        // Documentation showing command substitution syntax
        let code = r#"echo 'Use "$(command)" for substitution'"#;
        let result = check(code);
        assert_eq!(
            result.diagnostics.len(),
            0,
            "SC2016 must NOT flag quoted command substitution in docs"
        );
    }

    #[test]
    fn test_sc2016_335_no_match_across_string_boundaries() {
        // bashrs#335: `'[^']*\$…[^']*'` against the raw line pairs the closing
        // quote of one string with the opening quote of the next. `$x` here is
        // unquoted, not single-quoted.
        let code = r#"f 'a' $x 'b'"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0, "{:?}", result.diagnostics);
    }

    // ===== bashrs#526: remote-command contexts (ShellCheck 0.8.0 parity) =====
    // ShellCheck is silent when the single-quoted text is an argument of a
    // command that runs it as a script (ssh, sh, bash, ksh, zsh, eval, sudo,
    // docker, podman). The match is on the command's own first word, exactly.

    fn flagged(code: &str) -> usize {
        check(code).diagnostics.len()
    }

    #[test]
    fn test_sc2016_526_ssh_remote_command_not_flagged() {
        assert_eq!(flagged("ssh h 'echo $HOME'"), 0);
    }

    #[test]
    fn test_sc2016_526_shell_dash_c_not_flagged() {
        assert_eq!(flagged("sh -c 'echo $HOME'"), 0);
        assert_eq!(flagged("bash -c 'echo $HOME'"), 0);
        assert_eq!(flagged("ksh -c 'echo $HOME'"), 0);
        assert_eq!(flagged("zsh -c 'echo $HOME'"), 0);
    }

    #[test]
    fn test_sc2016_526_other_script_runners_not_flagged() {
        assert_eq!(flagged("eval 'echo $HOME'"), 0);
        assert_eq!(flagged("sudo sh -c 'echo $HOME'"), 0);
        assert_eq!(flagged("docker exec c sh -c 'echo $HOME'"), 0);
        assert_eq!(flagged("podman exec c sh -c 'echo $HOME'"), 0);
    }

    #[test]
    fn test_sc2016_526_indented_and_tab_separated() {
        assert_eq!(flagged("    ssh h 'echo $HOME'"), 0);
        assert_eq!(flagged("\tssh h 'echo $HOME'"), 0);
        assert_eq!(flagged("ssh\th\t'echo $HOME'"), 0);
        assert_eq!(flagged("ssh   h   'echo $HOME'"), 0);
    }

    #[test]
    fn test_sc2016_526_options_assignments_and_path_prefix() {
        assert_eq!(flagged("ssh -p 22 -o X=y h 'echo $HOME'"), 0);
        assert_eq!(flagged("FOO=1 ssh h 'echo $HOME'"), 0);
        assert_eq!(flagged("/usr/bin/ssh h 'echo $HOME'"), 0);
        assert_eq!(flagged("'ssh' h 'echo $HOME'"), 0);
        assert_eq!(flagged("if ssh h 'echo $HOME'; then :; fi"), 0);
        assert_eq!(flagged("! ssh h 'echo $HOME'"), 0);
    }

    #[test]
    fn test_sc2016_526_every_argument_and_mixed_quoting() {
        assert_eq!(flagged("ssh h 'echo $A' 'echo $B'"), 0);
        assert_eq!(flagged(r#"ssh h 'a'"b"'$X'"#), 0);
        assert_eq!(flagged("ssh h 'a ${X}' 'b $(date)'"), 0);
    }

    #[test]
    fn test_sc2016_526_inside_command_substitution() {
        assert_eq!(flagged("x=$(ssh h 'echo $HOME')"), 0);
        assert_eq!(flagged(r#"echo "$(ssh h 'a $X')""#), 0);
    }

    #[test]
    fn test_sc2016_526_next_command_still_flagged() {
        assert_eq!(flagged("ssh h 'echo $HOME'; echo 'x $Y'"), 1);
        assert_eq!(flagged("ssh h 'echo $HOME' && echo 'x $Y'"), 1);
        assert_eq!(flagged("ssh h 'echo $HOME' || echo 'x $Y'"), 1);
        assert_eq!(flagged("ssh h 'echo $HOME' | echo 'x $Y'"), 1);
        assert_eq!(flagged("ssh h 'echo $HOME' & echo 'x $Y'"), 1);
        assert_eq!(flagged("sudo sh -c 'a $X'; echo 'b $Y'"), 1);
    }

    #[test]
    fn test_sc2016_526_commands_shellcheck_still_flags() {
        // ShellCheck 0.8.0 reports each of these.
        assert_eq!(flagged("su -c 'echo $HOME' u"), 1);
        assert_eq!(flagged("xargs sh -c 'echo $HOME'"), 1);
        assert_eq!(flagged("nohup ssh h 'echo $HOME'"), 1);
        assert_eq!(flagged("env ssh h 'echo $HOME'"), 1);
        assert_eq!(flagged("timeout 5 ssh h 'echo $HOME'"), 1);
        assert_eq!(flagged("dash -c 'echo $HOME'"), 1);
        assert_eq!(flagged("sshpass ssh h 'a $X'"), 1);
        assert_eq!(flagged("scp 'a $X' h:"), 1);
    }

    #[test]
    fn test_sc2016_526_name_must_match_exactly() {
        assert_eq!(flagged("SSH h 'a $X'"), 1);
        assert_eq!(flagged("sshx h 'a $X'"), 1);
        assert_eq!(flagged("xssh h 'a $X'"), 1);
    }

    #[test]
    fn test_sc2016_526_word_mentioning_ssh_is_not_ssh() {
        assert_eq!(flagged("echo 'ssh h $X'"), 1);
        assert_eq!(flagged("foo 'a $X' | ssh h"), 1);
        assert_eq!(flagged("cmd 'echo $HOME' ssh"), 1);
        assert_eq!(flagged("# ssh h 'a $X'"), 0);
    }

    #[test]
    fn test_sc2016_526_nested_command_owns_its_quotes() {
        // The inner `echo` owns the quote even though `ssh` encloses it.
        assert_eq!(flagged(r#"ssh h "$(echo 'a $X')""#), 1);
        assert_eq!(flagged("ssh h $(echo 'a $X')"), 1);
    }

    #[test]
    fn test_sc2016_526_non_literal_command_word_still_flagged() {
        assert_eq!(flagged("$cmd h 'a $X'"), 1);
        assert_eq!(flagged("$(which ssh) h 'a $X'"), 1);
    }

    #[test]
    fn test_sc2016_526_shell_script_operand_with_unquoted_tail_not_flagged() {
        // The synthetic inner command of `sh -c 'echo $X'$Y` must not take
        // ownership of the quote from `sh`.
        assert_eq!(flagged("sh -c 'echo $X'$Y"), 0);
    }

    // ===== bashrs#668: trap action =====
    // The trap action is expanded when the trap fires, not when it is set, so a
    // single-quoted `$` in it is intentional. ShellCheck 0.8.0 is silent on it.

    #[test]
    fn test_sc2016_668_trap_action_not_flagged() {
        assert_eq!(flagged("x=1\ntrap 'echo $x' INT TERM"), 0);
        assert_eq!(flagged("trap 'echo $x' EXIT"), 0);
        assert_eq!(flagged("    trap 'echo $x' INT"), 0);
    }

    #[test]
    fn test_sc2016_668_plain_single_quoted_dollar_still_flagged() {
        assert_eq!(flagged("echo '$HOME'"), 1);
    }
}
