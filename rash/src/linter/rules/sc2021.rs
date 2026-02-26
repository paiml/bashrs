// SC2021: Don't use [] around ranges in tr, it replaces literal square brackets
//
// In tr, square brackets are literal characters, not range operators.
// Use a-z without brackets to specify ranges.
//
// Examples:
// Bad:
//   tr '[a-z]' '[A-Z]'              // Matches literal '[', 'a-z', ']'
//   tr '[0-9]' 'x'                  // Matches '[', '0-9', ']'
//   echo "hello" | tr '[aeiou]' 'X' // Matches '[', 'aeiou', ']'
//
// Good:
//   tr 'a-z' 'A-Z'                  // Range without brackets
//   tr '0-9' 'x'                    // Range without brackets
//   echo "hello" | tr 'aeiou' 'X'   // Set without brackets
//
// Note: Unlike other tools (grep, awk), tr doesn't use brackets for ranges.
// Brackets are treated as literal characters to match.

use crate::linter::{Diagnostic, LintResult, Severity, Span};
use regex::Regex;

static TR_BRACKETED_RANGE: std::sync::LazyLock<Regex> = std::sync::LazyLock::new(|| {
    // Match: tr [flags] '[something]' where something looks like a range or set
    // Avoid matching [[:posix:]] classes
    // Allow optional flags like -d, -s, etc.
    Regex::new(r#"tr\s+(?:-[a-z]+\s+)?['"](\[[^\]:]+\])['"]"#).unwrap()
});

pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();

    for (line_num, line) in source.lines().enumerate() {
        let line_num = line_num + 1;

        if line.trim_start().starts_with('#') {
            continue;
        }

        // Look for tr with [bracketed] patterns
        for cap in TR_BRACKETED_RANGE.captures_iter(line) {
            let bracketed = cap.get(1).unwrap().as_str();

            // Skip POSIX character classes like [[:lower:]]
            if bracketed.starts_with("[[:") {
                continue;
            }

            // It's a literal bracket usage
            let start_col = line.find(bracketed).map_or(1, |p| p + 1);
            let end_col = start_col + bracketed.len();

            let unbracketed = &bracketed[1..bracketed.len() - 1];
            let diagnostic = Diagnostic::new(
                "SC2021",
                Severity::Info,
                format!(
                    "Don't use [] around ranges in tr, it replaces literal square brackets. Use '{}' instead",
                    unbracketed
                ),
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
    fn test_sc2021_bracketed_range() {
        let code = r#"tr '[a-z]' '[A-Z]'"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
        assert_eq!(result.diagnostics[0].code, "SC2021");
        assert_eq!(result.diagnostics[0].severity, Severity::Info);
        assert!(result.diagnostics[0].message.contains("square brackets"));
    }

    #[test]
    fn test_sc2021_bracketed_digits() {
        let code = r#"tr '[0-9]' 'x'"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2021_bracketed_vowels() {
        let code = r#"echo "hello" | tr '[aeiou]' 'X'"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2021_unbracketed_ok() {
        let code = r#"tr 'a-z' 'A-Z'"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2021_posix_class_ok() {
        let code = r#"tr '[[:lower:]]' '[[:upper:]]'"#;
        let result = check(code);
        // POSIX classes need double brackets
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2021_no_brackets_ok() {
        let code = r#"tr 'aeiou' 'AEIOU'"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2021_multiple_issues() {
        let code = r#"
tr '[a-z]' '[A-Z]'
tr '[0-9]' 'x'
"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 2);
    }

    #[test]
    fn test_sc2021_in_pipeline() {
        let code = r#"cat file | tr '[a-z]' '[A-Z]' | sort"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2021_with_delete() {
        let code = r#"tr -d '[0-9]'"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2021_single_bracket_literal() {
        let code = r#"echo "[test]" | tr '[' '('"#;
        let result = check(code);
        // '[' without closing is different pattern
        assert_eq!(result.diagnostics.len(), 0);
    }
}
