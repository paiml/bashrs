// SC2146: This action ignores everything before the -o. Use \( \) to group.
//
// Actually, SC2146 is about find command -o operator, not exec.
// Let me correct this to match ShellCheck SC2146.
//
// Examples:
// Bad:
//   find . -name "*.txt" -o -name "*.md" -exec rm {} \;
//   // -exec only applies to -name "*.md", not both
//
// Good:
//   find . \( -name "*.txt" -o -name "*.md" \) -exec rm {} \;
//   // Parentheses group the expressions
//
// Impact: Command only applies to second condition, not first

use crate::linter::{Diagnostic, LintResult, Severity, Span};
use regex::Regex;

static FIND_OR_WITHOUT_PARENS: std::sync::LazyLock<Regex> = std::sync::LazyLock::new(|| {
    // Match: find with -o followed by action (-exec, -delete, -print) without parentheses
    // Pattern: find ... -o ... -exec/-delete/-print
    Regex::new(r"\bfind\b.*\s+-o\s+.*\s+-(?:exec|delete|print)").unwrap()
});

pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();

    for (line_num, line) in source.lines().enumerate() {
        let line_num = line_num + 1;

        if line.trim_start().starts_with('#') {
            continue;
        }

        // Check for find with -o followed by action without parentheses
        for mat in FIND_OR_WITHOUT_PARENS.find_iter(line) {
            // Skip if line contains \( or \) (proper grouping)
            if line.contains(r"\(") || line.contains(r"\)") {
                continue;
            }

            let start_col = mat.start() + 1;
            let end_col = mat.end() + 1;

            let diagnostic = Diagnostic::new(
                "SC2146",
                Severity::Warning,
                "This action only applies to the second condition. Use \\( \\) to group"
                    .to_string(),
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
    fn test_sc2146_find_or_exec_without_parens() {
        let code = r#"find . -name "*.txt" -o -name "*.md" -exec rm {} \;"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
        assert!(result.diagnostics[0].message.contains("group"));
    }

    #[test]
    fn test_sc2146_find_or_delete_without_parens() {
        let code = r#"find . -type f -o -type d -delete"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2146_find_or_with_parens_ok() {
        let code = r#"find . \( -name "*.txt" -o -name "*.md" \) -exec rm {} \;"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2146_find_or_without_action_ok() {
        let code = r#"find . -name "*.txt" -o -name "*.md""#;
        let result = check(code);
        // No action, so no warning
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2146_find_without_or_ok() {
        let code = r#"find . -name "*.txt" -exec rm {} \;"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2146_comment_ok() {
        let code = r#"# find . -name "*.txt" -o -name "*.md" -exec rm {} \;"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2146_find_or_print() {
        let code = r#"find . -type f -o -type l -print"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2146_multiple() {
        let code = r#"
find . -name "*.txt" -o -name "*.md" -delete
find /tmp -type f -o -type d -exec echo {} \;
"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 2);
    }

    #[test]
    fn test_sc2146_find_and_exec_ok() {
        let code = r#"find . -name "*.txt" -a -name "*old*" -exec rm {} \;"#;
        let result = check(code);
        // -a (and) doesn't have the same issue as -o
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2146_nested_find_ok() {
        let code = r#"find . \( -name "*.txt" -o -name "*.md" \) -a -size +1M -delete"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }
}
