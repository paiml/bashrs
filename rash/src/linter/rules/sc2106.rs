// SC2106: Consider using pgrep instead of grepping ps output
//
// Using ps | grep to find processes is fragile and can match grep itself.
// Use pgrep which is designed for this purpose.
//
// Examples:
// Bad:
//   ps aux | grep process_name  // Fragile, matches grep
//   ps -ef | grep "[p]rocess"   // Hacky workaround
//
// Good:
//   pgrep process_name           // Designed for this
//   pgrep -f process_name        // Match full command line
//
// Impact: Fragile process detection, false matches

use crate::linter::{Diagnostic, LintResult, Severity, Span};
use once_cell::sync::Lazy;
use regex::Regex;

static PS_GREP_PATTERN: Lazy<Regex> = Lazy::new(|| {
    // Match: ps ... | grep
    Regex::new(r"\bps\s+[^|]*\|\s*grep\b").unwrap()
});

pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();

    for (line_num, line) in source.lines().enumerate() {
        let line_num = line_num + 1;

        if line.trim_start().starts_with('#') {
            continue;
        }

        for mat in PS_GREP_PATTERN.find_iter(line) {
            let start_col = mat.start() + 1;
            let end_col = mat.end() + 1;

            let diagnostic = Diagnostic::new(
                "SC2106",
                Severity::Info,
                "Consider using pgrep instead of grepping ps output".to_string(),
                Span::new(line_num, start_col, line_num, end_col),
            );

            result.add(diagnostic);
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sc2106_ps_grep() {
        let code = "ps aux | grep process_name";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2106_ps_ef_grep() {
        let code = "ps -ef | grep myapp";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2106_pgrep_ok() {
        let code = "pgrep process_name";
        let result = check(code);
        // pgrep is correct
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2106_pgrep_full_ok() {
        let code = "pgrep -f process_name";
        let result = check(code);
        // pgrep with flags
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2106_comment_ok() {
        let code = "# ps aux | grep process_name";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2106_grep_bracket_trick() {
        let code = "ps aux | grep [p]rocess";
        let result = check(code);
        // Still should use pgrep
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2106_ps_with_flags() {
        let code = "ps -aux | grep nginx";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2106_multiple_pipes() {
        let code = "ps aux | grep nginx | awk '{print $2}'";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2106_grep_not_ps() {
        let code = "ls | grep pattern";
        let result = check(code);
        // Not ps, just normal grep
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2106_pkill_ok() {
        let code = "pkill process_name";
        let result = check(code);
        // pkill is also correct
        assert_eq!(result.diagnostics.len(), 0);
    }
}
