//! SC2164: Use cd ... || exit in case cd fails
//!
//! # Examples
//!
//! Bad:
//! ```bash
//! cd /some/directory
//! ./script.sh  # Runs in wrong directory if cd fails!
//! ```
//!
//! Good:
//! ```bash
//! cd /some/directory || exit
//! ./script.sh
//! ```
//!
//! # Rationale
//!
//! If cd fails, subsequent commands run in the wrong directory:
//! - Can cause data loss
//! - Can execute wrong scripts
//! - Hard to debug failures
//!
//! Always check cd exit status or use `set -e`.
//!
//! # Auto-fix
//!
//! Add || exit to cd command

use crate::linter::{Diagnostic, Fix, LintResult, Severity, Span};
use regex::Regex;

/// Check for cd without error handling
pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();

    // Pattern: cd path (simple detection - enhancement needed)
    // TODO: Improve negative lookahead for better detection
    let pattern = Regex::new(r"\bcd\s+([^\s;&|]+)").unwrap();

    for (line_num, line) in source.lines().enumerate() {
        let line_num = line_num + 1;

        if line.trim_start().starts_with('#') {
            continue;
        }

        // Skip if line already has error handling
        if line.contains("|| exit") || line.contains("|| return") || line.contains("&& ") {
            continue;
        }

        for cap in pattern.captures_iter(line) {
            let full_match = cap.get(0).unwrap();
            let directory = cap.get(1).unwrap().as_str();

            let start_col = full_match.start() + 1;
            let end_col = full_match.end() + 1;

            let fix_text = format!("cd {} || exit", directory);

            let diagnostic = Diagnostic::new(
                "SC2164",
                Severity::Warning,
                "Use 'cd ... || exit' in case cd fails",
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
    fn test_sc2164_basic_detection() {
        let script = "cd /tmp";
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 1);
        assert_eq!(result.diagnostics[0].code, "SC2164");
    }

    #[test]
    fn test_sc2164_autofix() {
        let script = "cd /tmp";
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 1);
        assert!(result.diagnostics[0].fix.is_some());
        assert_eq!(
            result.diagnostics[0].fix.as_ref().unwrap().replacement,
            "cd /tmp || exit"
        );
    }

    #[test]
    fn test_sc2164_with_variable() {
        let script = "cd \"$HOME/projects\"";
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2164_relative_path() {
        let script = "cd ../..";
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2164_false_positive_with_exit() {
        let script = "cd /tmp || exit";
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2164_false_positive_with_return() {
        let script = "cd /tmp || return";
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2164_false_positive_in_comment() {
        let script = "# cd /tmp";
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2164_in_script() {
        let script = "cd /var/log\ngrep ERROR syslog";
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2164_with_tilde() {
        let script = "cd ~/documents";
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2164_false_positive_with_and() {
        let script = "cd /tmp && ls";
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 0);
    }
}
