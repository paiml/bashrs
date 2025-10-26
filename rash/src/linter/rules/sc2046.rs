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
            // Find the actual $( position (not including 'pre' capture)
            let cmd_match = cap.name("cmd").unwrap();
            let dollar_paren_pos = line[..cmd_match.start()]
                .rfind("$(")
                .unwrap_or(cmd_match.start());

            let col = dollar_paren_pos + 1; // 1-indexed
            let end_col = cmd_match.end() + 2; // +1 for ) and +1 for 1-indexing

            // Check if already quoted
            if dollar_paren_pos > 0 && line.chars().nth(dollar_paren_pos - 1) == Some('"') {
                continue;
            }

            let span = Span::new(line_num, col, line_num, end_col);
            let cmd_text = format!("$({})", cmd_match.as_str());
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
            // Find the actual backtick position (not including 'pre' capture)
            let cmd_match = cap.name("cmd").unwrap();
            let backtick_pos = line[..cmd_match.start()]
                .rfind('`')
                .unwrap_or(cmd_match.start());

            let col = backtick_pos + 1; // 1-indexed
            let end_col = cmd_match.end() + 2; // +1 for closing ` and +1 for 1-indexing

            let span = Span::new(line_num, col, line_num, end_col);
            let cmd = cmd_match.as_str();
            let fix = Fix::new(format!("\"$({})\"", cmd));

            let diag = Diagnostic::new(
                "SC2046",
                Severity::Warning,
                "Quote this and use $(...) instead of backticks".to_string(),
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
        assert_eq!(
            result.diagnostics[0].fix.as_ref().unwrap().replacement,
            "\"$(ls)\""
        );
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
        assert_eq!(
            result.diagnostics[0].fix.as_ref().unwrap().replacement,
            "\"$(ls)\""
        );
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
        assert!(!result.diagnostics.is_empty());
    }

    #[test]
    fn test_sc2046_severity() {
        let bash_code = "files=$(ls)";
        let result = check(bash_code);

        assert_eq!(result.diagnostics[0].severity, Severity::Warning);
    }
}
