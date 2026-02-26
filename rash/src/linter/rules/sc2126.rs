// SC2126: Consider using grep -c instead of grep | wc
//
// Using grep | wc -l is inefficient. Use grep -c which counts matches directly.
// This is faster and more idiomatic.
//
// Examples:
// Bad:
//   grep pattern file | wc -l       // Inefficient
//   ps aux | grep process | wc -l   // Two pipes
//
// Good:
//   grep -c pattern file            // Direct count
//   pgrep -c process                // Even better for processes
//
// Impact: Performance, code clarity

use crate::linter::{Diagnostic, LintResult, Severity, Span};
use regex::Regex;

static GREP_WC: std::sync::LazyLock<Regex> = std::sync::LazyLock::new(|| {
    // Match: grep ... | wc -l
    Regex::new(r"\bgrep\b[^|]*\|\s*wc\s+-l\b").unwrap()
});

pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();

    for (line_num, line) in source.lines().enumerate() {
        let line_num = line_num + 1;

        if line.trim_start().starts_with('#') {
            continue;
        }

        for mat in GREP_WC.find_iter(line) {
            let start_col = mat.start() + 1;
            let end_col = mat.end() + 1;

            let diagnostic = Diagnostic::new(
                "SC2126",
                Severity::Info,
                "Consider using grep -c instead of grep | wc -l".to_string(),
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
    fn test_sc2126_grep_wc() {
        let code = "grep pattern file | wc -l";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2126_grep_c_ok() {
        let code = "grep -c pattern file";
        let result = check(code);
        // grep -c is correct
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2126_comment_ok() {
        let code = "# grep pattern file | wc -l";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2126_with_flags() {
        let code = "grep -i pattern file | wc -l";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2126_ps_grep() {
        let code = "ps aux | grep process | wc -l";
        let result = check(code);
        // Should detect the grep | wc portion
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2126_multiline() {
        let code = r#"
count=$(grep error log.txt | wc -l)
echo "Errors: $count"
"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2126_spaces() {
        let code = "grep pattern file  |  wc  -l";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2126_in_subshell() {
        let code = "count=$(grep pattern file | wc -l)";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2126_wc_without_l_ok() {
        let code = "grep pattern file | wc";
        let result = check(code);
        // wc without -l might be intentional (counts words/chars too)
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2126_egrep() {
        let code = "egrep pattern file | wc -l";
        let result = check(code);
        // egrep is also grep
        assert_eq!(result.diagnostics.len(), 0); // Our regex only matches \bgrep\b
    }
}
