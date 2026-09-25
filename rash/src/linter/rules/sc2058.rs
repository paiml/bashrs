// SC2058: Unknown unary operator in test expression
//
// Test commands support specific unary operators for file tests and string tests.
// Using an invalid unary operator causes syntax errors or unexpected behavior.
//
// Examples:
// Bad:
//   [ -q file ]              // -q is not a valid test operator
//   [ -m file ]              // -m is not a valid test operator
//   test -j file             // -j is not a valid test operator
//
// Good:
//   [ -f file ]              // File exists and is a regular file
//   [ -d dir ]               // Directory exists
//   [ -z "$var" ]            // String is empty
//   [ -n "$var" ]            // String is non-empty
//   [ -e file ]              // File exists
//   test -r file             // File is readable
//
// Valid unary operators:
//   File: -e, -f, -d, -r, -w, -x, -s, -h, -L, -p, -b, -c, -t, -S, -g, -u, -k, -O, -G, -N, -a
//   String: -z, -n
//
// GH-371: `test` and `[` are builtins only in COMMAND position, so both are
// located through `shell_words::simple_commands`. The old line regexes
// `\btest\s+-X` and `\[\s+-X` also matched `cargo test -q` and
// `"$wrapper" test -q`, where `test` is an argument.

use crate::linter::shell_words::{simple_commands, ShellWord, SimpleCommand, WordRole};
use crate::linter::{Diagnostic, LintResult, Severity, Span};

/// Valid unary test operators in POSIX and bash test expressions.
const VALID_UNARY_OPS: &[&str] = &[
    "e", "f", "d", "r", "w", "x", "s", "z", "n", "h", "L", "p", "b", "c", "t", "S", "g", "u", "k",
    "O", "G", "N", "a",
];

fn is_valid_unary_op(op: &str) -> bool {
    VALID_UNARY_OPS.contains(&op)
}

/// Does `w` open a test expression: `test` or `[` as the command name, or the
/// reserved word `[[`?
fn opens_test(cmd: &SimpleCommand, w: &ShellWord) -> bool {
    match w.role {
        WordRole::CommandName => matches!(cmd.name.as_deref(), Some("test" | "[")),
        WordRole::Reserved => w.expansions.is_empty() && w.literal == "[[",
        _ => false,
    }
}

/// The unknown unary operator in `op`, if `op` is a fully literal `-X` word.
fn unknown_operator(op: &ShellWord) -> Option<&str> {
    let letters = op.literal.strip_prefix('-')?;
    let literal_word = op.expansions.is_empty() && op.substitutions.is_empty();
    let is_op = !letters.is_empty() && letters.bytes().all(|b| b.is_ascii_alphabetic());
    (literal_word && is_op && !is_valid_unary_op(letters)).then_some(letters)
}

/// `(start_col, end_col, operator)` for every test expression on `line` whose first
/// operand is an unknown unary operator followed by another word.
fn unknown_unary_tests(line: &str) -> Vec<(usize, usize, String)> {
    let mut out = Vec::new();
    for cmd in simple_commands(line) {
        for (i, w) in cmd.words.iter().enumerate() {
            if !opens_test(&cmd, w) || cmd.words.get(i + 2).is_none() {
                continue;
            }
            let Some(op) = cmd.words.get(i + 1) else {
                continue;
            };
            if let Some(letters) = unknown_operator(op) {
                out.push((w.col, op.col + op.raw.len(), letters.to_string()));
            }
        }
    }
    out
}

pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();

    for (line_num, line) in source.lines().enumerate() {
        let line_num = line_num + 1;

        if line.trim_start().starts_with('#') {
            continue;
        }

        for (start_col, end_col, operator) in unknown_unary_tests(line) {
            result.add(Diagnostic::new(
                "SC2058",
                Severity::Error,
                format!(
                    "Unknown unary operator '-{}' in test expression. Use a valid operator like -f, -d, -e, -z, -n, etc.",
                    operator
                ),
                Span::new(line_num, start_col, line_num, end_col),
            ));
        }
    }

    result
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]
    use super::*;

    #[test]
    fn test_sc2058_unknown_operator_q() {
        let code = "[ -q file ]";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
        assert_eq!(result.diagnostics[0].code, "SC2058");
        assert!(result.diagnostics[0].message.contains("-q"));
    }

    #[test]
    fn test_sc2058_unknown_operator_m() {
        let code = "[ -m file ]";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
        assert!(result.diagnostics[0].message.contains("-m"));
    }

    #[test]
    fn test_sc2058_test_builtin_unknown() {
        let code = "test -q file";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
        assert_eq!(result.diagnostics[0].code, "SC2058");
        assert!(result.diagnostics[0].message.contains("-q"));
    }

    #[test]
    fn test_sc2058_valid_f() {
        let code = "[ -f file ]";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2058_valid_d() {
        let code = "[ -d dir ]";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2058_valid_z() {
        let code = r#"[ -z "$var" ]"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2058_valid_n() {
        let code = r#"[ -n "$var" ]"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2058_valid_e() {
        let code = "[ -e /tmp/file ]";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2058_valid_r_w_x() {
        let code = "[ -r file ] && [ -w file ] && [ -x file ]";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2058_comment_ignored() {
        let code = "# [ -q file ]";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2058_test_builtin_valid() {
        let code = "test -f file";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2058_multiple_unknown() {
        let code = "[ -q file ] && [ -m dir ]";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 2);
    }

    #[test]
    fn test_sc2058_valid_capital_l() {
        let code = "[ -L /path/to/link ]";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2058_valid_capital_s() {
        let code = "[ -S /path/to/socket ]";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }
}

/// GH-371: `test` and `[` are builtins only in COMMAND position. In
/// `cargo test -q` and `"$wrapper" test -q` the word `test` is an argument.
#[cfg(test)]
mod gh371_tests {
    use super::*;

    fn count(code: &str) -> usize {
        check(code).diagnostics.len()
    }

    #[test]
    fn test_gh371_wrapper_test_q_with_env_prefix_and_redirects() {
        let code =
            r#"if MEMCAP_PROBE_MIB=1024 "$wrapper" test -q >"$work/l1" 2>&1; then echo ok; fi"#;
        assert_eq!(count(code), 0);
    }

    #[test]
    fn test_gh371_quoted_variable_command_test_q() {
        assert_eq!(count(r#""$wrapper" test -q --lib"#), 0);
    }

    #[test]
    fn test_gh371_cargo_test_q() {
        assert_eq!(count("cargo test -q --lib"), 0);
    }

    #[test]
    fn test_gh371_env_prefix_variable_command_test_q() {
        assert_eq!(count(r#"FOO=1 "$w" test -q --lib"#), 0);
    }

    #[test]
    fn test_gh371_echo_bracket_is_an_argument() {
        assert_eq!(count("echo [ -q x ]"), 0);
    }

    // Negative controls: the builtin in command position still fires.
    #[test]
    fn test_gh371_control_if_bracket_still_fires() {
        assert_eq!(count("if [ -q x ]; then :; fi"), 1);
    }

    #[test]
    fn test_gh371_control_if_test_still_fires() {
        assert_eq!(count("if test -q x; then :; fi"), 1);
    }

    #[test]
    fn test_gh371_control_after_and_still_fires() {
        assert_eq!(count("true && test -q x"), 1);
    }

    #[test]
    fn test_gh371_control_env_prefix_test_still_fires() {
        assert_eq!(count("FOO=1 test -q x"), 1);
    }

    #[test]
    fn test_gh371_control_through_wrapper_still_fires() {
        assert_eq!(count("sudo test -q x"), 1);
    }

    #[test]
    fn test_gh371_control_double_bracket_still_fires() {
        assert_eq!(count("[[ -q x ]]"), 1);
    }

    #[test]
    fn test_gh371_control_inside_command_substitution_still_fires() {
        assert_eq!(count(r#"out="$(test -q x)""#), 1);
    }
}
