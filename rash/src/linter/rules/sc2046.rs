//! SC2046: Quote command substitutions to prevent word splitting
//!
//! Detects unquoted command substitutions like $(cmd) or `cmd` that could
//! cause word splitting on the output.
//!
//! References:
//! - https://www.shellcheck.net/wiki/SC2046

use crate::linter::{Diagnostic, Fix, LintResult, Severity, Span};
use regex::Regex;

/// Check for unquoted command substitutions (SC2046)
pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();

    // Pattern for command substitution: $(...)
    let cmd_sub_pattern = Regex::new(r#"(?m)(?P<pre>[^"']|^)\$\((?P<cmd>[^)]+)\)"#).unwrap();

    // Pattern for backtick command substitution: `...`
    let backtick_pattern = Regex::new(r#"(?m)(?P<pre>[^"']|^)`(?P<cmd>[^`]+)`"#).unwrap();

    for (line_num, line) in source.lines().enumerate() {
        let line_num = line_num + 1;

        // Skip comments
        if line.trim_start().starts_with('#') {
            continue;
        }

        // Check $(...) substitutions
        for cap in cmd_sub_pattern.captures_iter(line) {
            let full_match = cap.get(0).unwrap();
            let col = full_match.start() + 1;
            let end_col = full_match.end();

            // Check if already quoted
            if col > 1 && line.chars().nth(col - 2) == Some('"') {
                continue;
            }

            let span = Span::new(line_num, col, line_num, end_col);
            let cmd_text = format!("$({})", cap.name("cmd").unwrap().as_str());
            let fix = Fix::new(format!("\"{}\"", cmd_text));

            let diag = Diagnostic::new(
                "SC2046",
                Severity::Warning,
                format!("Quote this to prevent word splitting: {}", cmd_text),
                span,
            )
            .with_fix(fix);

            result.add(diag);
        }

        // Check backtick substitutions
        for cap in backtick_pattern.captures_iter(line) {
            let full_match = cap.get(0).unwrap();
            let col = full_match.start() + 1;
            let end_col = full_match.end();

            let span = Span::new(line_num, col, line_num, end_col);
            let cmd = cap.name("cmd").unwrap().as_str();
            let fix = Fix::new(format!("\"$({})\"", cmd));

            let diag = Diagnostic::new(
                "SC2046",
                Severity::Warning,
                format!("Quote this and use $(...) instead of backticks"),
                span,
            )
            .with_fix(fix);

            result.add(diag);
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sc2046_basic_detection() {
        let bash_code = "files=$(find . -name '*.txt')";
        let result = check(bash_code);

        assert_eq!(result.diagnostics.len(), 1);
        assert_eq!(result.diagnostics[0].code, "SC2046");
        assert!(result.diagnostics[0].message.contains("Quote this"));
    }

    #[test]
    fn test_sc2046_autofix() {
        let bash_code = "files=$(ls)";
        let result = check(bash_code);

        assert!(result.diagnostics[0].fix.is_some());
        assert_eq!(result.diagnostics[0].fix.as_ref().unwrap().replacement, "\"$(ls)\"");
    }

    #[test]
    fn test_sc2046_backtick_detection() {
        let bash_code = "files=`ls *.txt`";
        let result = check(bash_code);

        assert_eq!(result.diagnostics.len(), 1);
        assert_eq!(result.diagnostics[0].code, "SC2046");
        assert!(result.diagnostics[0].message.contains("backticks"));
    }

    #[test]
    fn test_sc2046_backtick_autofix() {
        let bash_code = "files=`ls`";
        let result = check(bash_code);

        assert!(result.diagnostics[0].fix.is_some());
        assert_eq!(result.diagnostics[0].fix.as_ref().unwrap().replacement, "\"$(ls)\"");
    }

    #[test]
    fn test_sc2046_skip_quoted() {
        let bash_code = r#"files="$(find . -name '*.txt')""#;
        let result = check(bash_code);

        // Should NOT trigger - already quoted
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2046_multiple_substitutions() {
        let bash_code = r#"
result=$(echo $(cat file.txt))
"#;
        let result = check(bash_code);

        // Should detect nested unquoted substitutions
        assert!(result.diagnostics.len() >= 1);
    }

    #[test]
    fn test_sc2046_severity() {
        let bash_code = "files=$(ls)";
        let result = check(bash_code);

        assert_eq!(result.diagnostics[0].severity, Severity::Warning);
    }
}
