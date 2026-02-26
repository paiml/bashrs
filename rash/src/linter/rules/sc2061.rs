// SC2061: Quote parameters to tr to prevent glob expansion
//
// The tr command takes character sets as arguments. When using bracket expressions
// like [a-z], these should be quoted to prevent the shell from expanding them as globs.
//
// Examples:
// Bad:
//   tr [a-z] [A-Z]              // Shell may expand [a-z] as glob
//   echo "hello" | tr [aeiou] *  // Glob expansion
//   tr [:lower:] [:upper:]       // May expand as glob
//
// Good:
//   tr '[a-z]' '[A-Z]'          // Properly quoted
//   echo "hello" | tr '[aeiou]' '*'  // Properly quoted
//   tr '[:lower:]' '[:upper:]'   // Properly quoted
//
// Note: Quoting prevents the shell from interpreting [] as glob patterns
// and ensures tr receives the literal bracket expressions.

use crate::linter::{Diagnostic, LintResult, Severity, Span};
use regex::Regex;

static TR_UNQUOTED_BRACKETS: std::sync::LazyLock<Regex> = std::sync::LazyLock::new(|| {
    // Match: tr followed by unquoted bracket expressions
    // Look for tr with [anything] patterns like [a-z] or [:lower:], but not in quotes
    // [^\s'"]* excludes spaces and quotes
    Regex::new(r#"\btr\s+([^\s'"]*\[[^\]]+\])"#).unwrap()
});

pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();

    for (line_num, line) in source.lines().enumerate() {
        let line_num = line_num + 1;

        if line.trim_start().starts_with('#') {
            continue;
        }

        // Skip if line doesn't contain tr command
        if !line.contains("tr ") {
            continue;
        }

        // Look for unquoted bracket expressions after tr
        for cap in TR_UNQUOTED_BRACKETS.captures_iter(line) {
            let bracket_expr = cap.get(1).unwrap().as_str();

            // Check if this bracket expression is in quotes by examining context
            let match_start = line.find(bracket_expr).unwrap_or(0);
            let before = &line[..match_start];

            // Count quotes before this match
            let double_quotes = before.matches('"').count();
            let single_quotes = before.matches('\'').count();

            // If odd number of quotes, we're inside a quoted string
            if double_quotes % 2 == 1 || single_quotes % 2 == 1 {
                continue;
            }

            let start_col = match_start + 1;
            let end_col = start_col + bracket_expr.len();

            let diagnostic = Diagnostic::new(
                "SC2061",
                Severity::Warning,
                format!(
                    "Quote the tr parameter '{}' to prevent glob expansion: tr '{}' ...",
                    bracket_expr, bracket_expr
                ),
                Span::new(line_num, start_col, line_num, end_col),
            );

            result.add(diagnostic);
            break; // Only warn once per tr command
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sc2061_unquoted_range() {
        let code = r#"tr [a-z] [A-Z]"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
        assert_eq!(result.diagnostics[0].code, "SC2061");
        assert_eq!(result.diagnostics[0].severity, Severity::Warning);
        assert!(result.diagnostics[0].message.contains("glob expansion"));
    }

    #[test]
    fn test_sc2061_unquoted_posix_class() {
        let code = r#"tr [:lower:] [:upper:]"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2061_unquoted_vowels() {
        let code = r#"echo "hello" | tr [aeiou] [AEIOU]"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2061_quoted_single_ok() {
        let code = r#"tr '[a-z]' '[A-Z]'"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2061_quoted_double_ok() {
        let code = r#"tr "[a-z]" "[A-Z]""#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2061_posix_quoted_ok() {
        let code = r#"tr '[:lower:]' '[:upper:]'"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2061_no_brackets_ok() {
        let code = r#"tr abc ABC"#;
        let result = check(code);
        // No brackets, no issue
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2061_tr_with_delete_ok() {
        let code = r#"tr -d 'a-z'"#;
        let result = check(code);
        // Quoted, no issue
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2061_multiple_tr_commands() {
        let code = r#"
tr [a-z] [A-Z]
tr [:digit:] *
"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 2);
    }

    #[test]
    fn test_sc2061_in_pipeline() {
        let code = r#"cat file | tr [a-z] [A-Z] | sort"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }
}
