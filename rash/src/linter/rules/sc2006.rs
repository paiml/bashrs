//! SC2006: Use $(...) instead of legacy backticks
//!
//! # Examples
//!
//! Bad:
//! ```bash
//! result=`date`
//! files=`ls *.txt`
//! ```
//!
//! Good:
//! ```bash
//! result=$(date)
//! files=$(ls *.txt)
//! ```
//!
//! # Rationale
//!
//! Backticks are the legacy syntax for command substitution. Modern `$(...)` syntax:
//! - Is easier to nest
//! - Is more readable
//! - Is POSIX compliant
//!
//! # Auto-fix
//!
//! Replace backticks with `$(...)`: `` `cmd` `` → `$(cmd)`

use crate::linter::{Diagnostic, Fix, LintResult, Severity, Span};
use regex::Regex;

/// Check for deprecated backtick command substitution
pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();

    // Pattern: `command`
    let pattern = Regex::new(r"`([^`]+)`").unwrap();

    for (line_num, line) in source.lines().enumerate() {
        let line_num = line_num + 1;

        if line.trim_start().starts_with('#') {
            continue;
        }

        for cap in pattern.captures_iter(line) {
            let full_match = cap.get(0).unwrap();
            let command = cap.get(1).unwrap().as_str();

            let start_col = full_match.start() + 1;
            let end_col = full_match.end() + 1;

            let fix_text = format!("$({})", command);

            let diagnostic = Diagnostic::new(
                "SC2006",
                Severity::Info,
                "Use $(...) instead of deprecated backticks",
                Span::new(line_num, start_col, line_num, end_col),
            )
            .with_fix(Fix::new(fix_text));

            result.add(diagnostic);
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sc2006_basic_detection() {
        let script = r#"result=`date`"#;
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 1);
        assert_eq!(result.diagnostics[0].code, "SC2006");
    }

    #[test]
    fn test_sc2006_autofix() {
        let script = r#"result=`date`"#;
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 1);
        assert!(result.diagnostics[0].fix.is_some());
        assert_eq!(
            result.diagnostics[0].fix.as_ref().unwrap().replacement,
            "$(date)"
        );
    }

    #[test]
    fn test_sc2006_ls_command() {
        let script = r#"files=`ls *.txt`"#;
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2006_command_with_args() {
        let script = r#"output=`grep "pattern" file.txt`"#;
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2006_false_positive_modern_syntax() {
        let script = r#"result=$(date)"#;
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2006_false_positive_in_comment() {
        let script = r#"# This is a comment with `backticks`"#;
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2006_multiple_occurrences() {
        let script = r#"
a=`cmd1`
b=`cmd2`
c=`cmd3`
"#;
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 3);
    }

    #[test]
    fn test_sc2006_in_if_statement() {
        let script = r#"if [ "`whoami`" = "root" ]; then echo "root"; fi"#;
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2006_echo_statement() {
        let script = r#"echo "Current date: `date`""#;
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2006_assignment_in_function() {
        let script = r#"
function get_time() {
    time=`date +%H:%M:%S`
    echo "$time"
}
"#;
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 1);
    }
}
