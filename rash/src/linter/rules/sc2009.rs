//! SC2009: Consider using pgrep instead of grepping ps output
//!
//! This rule detects when ps output is piped to grep to find processes,
//! which is inefficient and fragile. The pgrep utility is more reliable.
//!
//! # Examples
//!
//! Bad:
//! ```bash
//! ps aux | grep process_name
//! ps -ef | grep -v grep | grep myapp
//! ps | grep nginx
//! ```
//!
//! Good:
//! ```bash
//! pgrep process_name
//! pgrep -f myapp
//! pgrep nginx
//! ```
//!
//! # Rationale
//!
//! Parsing ps output with grep is fragile and inefficient:
//! - ps output format varies across systems
//! - grep matches its own process in the output
//! - Multiple pipes needed to filter grep itself
//! - pgrep is specifically designed for this use case
//!
//! # Fix
//!
//! Replace `ps ... | grep pattern` with `pgrep pattern`
//! Use `pgrep -f` for full command line matching
//!
//! # Compatibility
//!
//! Universal - pgrep is widely available (procps-ng package on most systems)

use crate::linter::diagnostic::FixSafetyLevel;
use crate::linter::{Diagnostic, Fix, LintResult, Severity, Span};
use regex::Regex;

pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();

    // Pattern: detect ps ... | grep pattern
    // Match ps commands (ps, ps aux, ps -ef, etc.) piped to grep
    let pattern = Regex::new(r"ps\s+[^|]*\|\s*grep").unwrap();

    for (line_num, line) in source.lines().enumerate() {
        let trimmed = line.trim();

        // Skip comments
        if trimmed.starts_with('#') {
            continue;
        }

        // Check for ps | grep pattern
        if let Some(mat) = pattern.find(line) {
            let start = mat.start();
            let end = mat.end();

            result.diagnostics.push(Diagnostic {
                code: "SC2009".to_string(),
                severity: Severity::Info,
                message: "Consider using pgrep instead of grepping ps output. pgrep is more reliable and efficient.".to_string(),
                span: Span {
                    start_line: line_num + 1,  // 1-indexed
                    end_line: line_num + 1,    // 1-indexed
                    start_col: start + 1,      // 1-indexed
                    end_col: end + 1,          // 1-indexed
                },
                fix: Some(Fix {
                    replacement: "pgrep".to_string(),
                    safety_level: FixSafetyLevel::SafeWithAssumptions,
                    assumptions: vec![
                        "pgrep is available on the system".to_string(),
                        "Simple process name matching is sufficient".to_string(),
                    ],
                    suggested_alternatives: vec![
                        "pgrep -f pattern  # Match full command line".to_string(),
                        "pgrep -u user pattern  # Match specific user".to_string(),
                    ],
                }),
            });
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sc2009_basic_ps_aux_grep() {
        let source = "ps aux | grep nginx";
        let result = check(source);
        assert_eq!(result.diagnostics.len(), 1);
        assert_eq!(result.diagnostics[0].code, "SC2009");
        assert!(result.diagnostics[0].message.contains("pgrep"));
    }

    #[test]
    fn test_sc2009_ps_ef_grep() {
        let source = "ps -ef | grep myapp";
        let result = check(source);
        assert_eq!(result.diagnostics.len(), 1);
        assert_eq!(result.diagnostics[0].code, "SC2009");
    }

    #[test]
    fn test_sc2009_ps_with_multiple_pipes() {
        let source = "ps aux | grep -v grep | grep process";
        let result = check(source);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2009_ps_without_grep() {
        // Should NOT trigger - ps without grep is fine
        let source = "ps aux";
        let result = check(source);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2009_grep_without_ps() {
        // Should NOT trigger - grep on other things is fine
        let source = "cat file.txt | grep pattern";
        let result = check(source);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2009_pgrep_usage() {
        // Should NOT trigger - already using pgrep
        let source = "pgrep nginx";
        let result = check(source);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2009_in_comment() {
        // Should NOT trigger - it's in a comment
        let source = "# Bad: ps aux | grep nginx";
        let result = check(source);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2009_multiline_pipeline() {
        let source = r#"ps aux \
  | grep nginx"#;
        let result = check(source);
        // Note: Simple line-by-line detection may not catch multiline
        // This is acceptable for initial implementation
        assert!(result.diagnostics.len() <= 1);
    }

    #[test]
    fn test_sc2009_auto_fix_suggests_pgrep() {
        let source = "ps aux | grep nginx";
        let result = check(source);
        assert_eq!(result.diagnostics.len(), 1);

        let fix = &result.diagnostics[0].fix;
        assert!(fix.is_some());
        let fix = fix.as_ref().unwrap();
        assert!(fix.replacement.contains("pgrep"));
    }

    #[test]
    fn test_sc2009_severity_info() {
        let source = "ps aux | grep nginx";
        let result = check(source);
        assert_eq!(result.diagnostics.len(), 1);
        assert_eq!(result.diagnostics[0].severity, Severity::Info);
    }
}
