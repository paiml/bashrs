//! SC2001: See if you can use ${variable//search/replace} instead of sed
//!
//! # Examples
//!
//! Bad:
//! ```bash
//! result=$(echo "$var" | sed 's/foo/bar/')
//! ```
//!
//! Good:
//! ```bash
//! result="${var//foo/bar}"
//! ```
//!
//! # Rationale
//!
//! Using parameter expansion is:
//! - Faster (no external process)
//! - More readable
//! - POSIX compliant (for simple substitutions)
//!
//! # Auto-fix
//!
//! Suggest using ${var//search/replace} for simple sed patterns

use crate::linter::{Diagnostic, Fix, LintResult, Severity, Span};
use regex::Regex;

/// Check for sed usage that could be replaced with parameter expansion
pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();

    // Pattern: echo "$var" | sed 's/pattern/replacement/'
    // Use simple word patterns (alphanumeric + underscore only)
    let pattern1 =
        Regex::new(r#"echo\s+"\$(\w+)"\s*\|\s*sed\s+'s/([a-zA-Z0-9_]+)/([a-zA-Z0-9_]+)/'"#)
            .unwrap();

    // Pattern: $(echo "$var" | sed 's/pattern/replacement/')
    let pattern2 =
        Regex::new(r#"\$\(echo\s+"\$(\w+)"\s*\|\s*sed\s+'s/([a-zA-Z0-9_]+)/([a-zA-Z0-9_]+)/'\)"#)
            .unwrap();

    for (line_num, line) in source.lines().enumerate() {
        let line_num = line_num + 1;

        if line.trim_start().starts_with('#') {
            continue;
        }

        // Try pattern2 first (command substitution), then pattern1 (echo pipe)
        // This prevents double-matching
        let mut matched = false;

        for cap in pattern2.captures_iter(line) {
            matched = true;
            let full_match = cap.get(0).unwrap();
            let var_name = cap.get(1).unwrap().as_str();
            let search = cap.get(2).unwrap().as_str();
            let replace = cap.get(3).unwrap().as_str();

            let start_col = full_match.start() + 1;
            let end_col = full_match.end() + 1;

            let fix_text = format!("${{{}//{}/{}}}", var_name, search, replace);

            let diagnostic = Diagnostic::new(
                "SC2001",
                Severity::Info,
                "See if you can use ${variable//search/replace} instead of sed",
                Span::new(line_num, start_col, line_num, end_col),
            )
            .with_fix(Fix::new(fix_text));

            result.add(diagnostic);
        }

        // Only check pattern1 if pattern2 didn't match
        if !matched {
            for cap in pattern1.captures_iter(line) {
                let full_match = cap.get(0).unwrap();
                let var_name = cap.get(1).unwrap().as_str();
                let search = cap.get(2).unwrap().as_str();
                let replace = cap.get(3).unwrap().as_str();

                let start_col = full_match.start() + 1;
                let end_col = full_match.end() + 1;

                let fix_text = format!("${{{}//{}/{}}}", var_name, search, replace);

                let diagnostic = Diagnostic::new(
                    "SC2001",
                    Severity::Info,
                    "See if you can use ${variable//search/replace} instead of sed",
                    Span::new(line_num, start_col, line_num, end_col),
                )
                .with_fix(Fix::new(fix_text));

                result.add(diagnostic);
            }
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sc2001_basic_detection() {
        let script = r#"result=$(echo "$var" | sed 's/foo/bar/')"#;
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 1);
        assert_eq!(result.diagnostics[0].code, "SC2001");
    }

    #[test]
    fn test_sc2001_autofix() {
        let script = r#"result=$(echo "$var" | sed 's/foo/bar/')"#;
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 1);
        assert!(result.diagnostics[0].fix.is_some());
        assert_eq!(
            result.diagnostics[0].fix.as_ref().unwrap().replacement,
            "${var//foo/bar}"
        );
    }

    #[test]
    fn test_sc2001_echo_pipe_sed() {
        let script = r#"echo "$name" | sed 's/old/new/'"#;
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2001_simple_substitution() {
        let script = r#"value=$(echo "$input" | sed 's/a/b/')"#;
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 1);
        assert_eq!(
            result.diagnostics[0].fix.as_ref().unwrap().replacement,
            "${input//a/b}"
        );
    }

    #[test]
    fn test_sc2001_false_positive_direct_expansion() {
        let script = r#"result="${var//foo/bar}""#;
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2001_false_positive_complex_sed() {
        let script = r#"result=$(sed 's/foo/bar/g; s/baz/qux/' file.txt)"#;
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 0); // Too complex for parameter expansion
    }

    #[test]
    fn test_sc2001_false_positive_in_comment() {
        let script = r#"# echo "$var" | sed 's/foo/bar/'"#;
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2001_multiple_occurrences() {
        let script = r#"
a=$(echo "$x" | sed 's/1/2/')
b=$(echo "$y" | sed 's/3/4/')
"#;
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 2);
    }

    #[test]
    fn test_sc2001_different_variable_names() {
        let script = r#"output=$(echo "$input_var" | sed 's/pattern/replacement/')"#;
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2001_underscore_in_pattern() {
        let script = r#"result=$(echo "$var" | sed 's/foo_bar/baz/')"#;
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 1);
        assert_eq!(
            result.diagnostics[0].fix.as_ref().unwrap().replacement,
            "${var//foo_bar/baz}"
        );
    }
}
