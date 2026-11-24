//! SC2181: Check exit code directly with e.g. 'if mycmd;', not indirectly with $?
//!
//! # Examples
//!
//! Bad:
//! ```bash
//! command
//! if [ $? -eq 0 ]; then
//!   echo "Success"
//! fi
//! ```
//!
//! Good:
//! ```bash
//! if command; then
//!   echo "Success"
//! fi
//! ```
//!
//! # Rationale
//!
//! Checking $? is:
//! - Less readable
//! - More error-prone (other commands can change $?)
//! - Not idiomatic shell
//!
//! Directly check command exit status in if/while conditions.
//!
//! # Auto-fix
//!
//! Warning only - refactor to use command directly in condition

use crate::linter::{Diagnostic, LintResult, Severity, Span};
use regex::Regex;

/// Check for indirect $? comparisons
static PATTERN: std::sync::LazyLock<Regex> = std::sync::LazyLock::new(|| {
    Regex::new(r"(?:if|while)\s+\[\s*\$\?\s*(?:-eq|-ne)\s*0\s*\]").unwrap()
});

/// `[[ $? OP 0 ]]` and `(( $? ))`-style conditions (shellcheck 0.8.0 flags
/// these as SC2181; `-eq 1`, `-ge 1` and compound conditions stay silent).
/// Kept separate from `PATTERN` so quoting changes to that regex don't overlap.
static DBL_PATTERN: std::sync::LazyLock<Regex> = std::sync::LazyLock::new(|| {
    Regex::new(
        r#"(?:if|while|until)\s+(?:\[\[\s*"?\$\?"?\s*(?:==|=|!=|-eq|-ne|-gt)\s*"?0"?\s*\]\]|\(\(\s*(?:!\s*)?\$\?\s*(?:(?:==|!=|>)\s*0\s*)?\)\))"#,
    )
    .unwrap()
});

fn diagnostics_for_line(line: &str, line_num: usize, result: &mut LintResult) {
    for pattern in [&*PATTERN, &*DBL_PATTERN] {
        for m in pattern.find_iter(line) {
            result.add(Diagnostic::new(
                "SC2181",
                Severity::Info,
                "Check exit code directly with e.g. 'if mycmd;', not indirectly with $?",
                Span::new(line_num, m.start() + 1, line_num, m.end() + 1),
            ));
        }
    }
}

pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();

    for (idx, line) in source.lines().enumerate() {
        if line.trim_start().starts_with('#') {
            continue;
        }
        diagnostics_for_line(line, idx + 1, &mut result);
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sc2181_basic_detection() {
        let script = "if [ $? -eq 0 ]; then echo ok; fi";
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 1);
        assert_eq!(result.diagnostics[0].code, "SC2181");
    }

    #[test]
    fn test_sc2181_not_equal() {
        let script = "if [ $? -ne 0 ]; then echo fail; fi";
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2181_while_loop() {
        let script = "while [ $? -eq 0 ]; do process; done";
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2181_with_spaces() {
        let script = "if [  $?  -eq  0  ]; then ok; fi";
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2181_false_positive_direct_check() {
        let script = "if command; then echo ok; fi";
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2181_false_positive_in_comment() {
        let script = "# if [ $? -eq 0 ]; then";
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2181_multiline() {
        let script = "command\nif [ $? -eq 0 ]; then\n  echo success\nfi";
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2181_in_function() {
        let script = "check() { if [ $? -eq 0 ]; then return 0; fi; }";
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2181_nested() {
        let script = "if [ $? -ne 0 ]; then\n  if [ $? -eq 0 ]; then ok; fi\nfi";
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 2);
    }

    #[test]
    fn test_sc2181_dbl_bracket_forms() {
        for cond in [
            "[[ $? == 0 ]]",
            "[[ $? = 0 ]]",
            "[[ $? != 0 ]]",
            "[[ $? -eq 0 ]]",
            "[[ $? -ne 0 ]]",
            "[[ $? -gt 0 ]]",
        ] {
            let script = format!("foo\nif {cond}; then echo x; fi");
            let result = check(&script);
            assert_eq!(result.diagnostics.len(), 1, "{cond}");
            assert_eq!(result.diagnostics[0].code, "SC2181");
        }
    }

    #[test]
    fn test_sc2181_dbl_bracket_while_until() {
        assert_eq!(
            check("while [[ $? == 0 ]]; do foo; done").diagnostics.len(),
            1
        );
        assert_eq!(check("until (( $? )); do foo; done").diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2181_arith_forms() {
        for cond in [
            "(( $? ))",
            "(($?))",
            "(( $? == 0 ))",
            "(( $? != 0 ))",
            "(( $? > 0 ))",
            "(( ! $? ))",
        ] {
            let script = format!("foo\nif {cond}; then echo x; fi");
            let result = check(&script);
            assert_eq!(result.diagnostics.len(), 1, "{cond}");
        }
    }

    #[test]
    fn test_sc2181_dbl_false_positive_shellcheck_silent() {
        for cond in [
            "[[ $? -eq 1 ]]",
            "[[ $? == 1 ]]",
            "[[ $? -ge 1 ]]",
            "[[ $? -eq 0 && -f x ]]",
            "[[ -f x && $? -eq 0 ]]",
            "(( $? == 1 ))",
            "[[ $x == 0 ]]",
            "(( x ))",
        ] {
            let script = format!("foo\nif {cond}; then echo x; fi");
            assert_eq!(check(&script).diagnostics.len(), 0, "{cond}");
        }
    }

    #[test]
    fn test_sc2181_false_positive_negation() {
        let script = "if ! command; then echo fail; fi";
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 0);
    }
}
