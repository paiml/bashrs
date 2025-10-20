// SC2026: This word is not properly quoted - contains special characters that may cause unexpected word splitting
//
// When a string contains special characters like =, it needs to be quoted to prevent
// word splitting or unexpected interpretation by the shell.
//
// Examples:
// Bad:
//   name=John=Doe             // Will be interpreted as name=John and Doe as separate command
//   echo name=value           // May split unexpectedly
//   var=PATH=$PATH:/new       // Ambiguous
//
// Good:
//   name="John=Doe"           // Properly quoted
//   echo "name=value"         // Properly quoted
//   var="PATH=$PATH:/new"     // Properly quoted
//
// Note: This rule focuses on catching unquoted strings with = signs that are
// likely to be misinterpreted.

use crate::linter::{Diagnostic, LintResult, Severity, Span};
use once_cell::sync::Lazy;
use regex::Regex;

static UNQUOTED_EQUALS: Lazy<Regex> = Lazy::new(|| {
    // Match word containing = that's not a variable assignment
    // Look for = after the first character (not at start of word)
    Regex::new(r"\b([a-zA-Z_][a-zA-Z0-9_]*=[a-zA-Z0-9_]+=[a-zA-Z0-9_]+)\b").unwrap()
});

pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();

    for (line_num, line) in source.lines().enumerate() {
        let line_num = line_num + 1;

        if line.trim_start().starts_with('#') {
            continue;
        }

        // Skip lines that are pure assignments (var=value)
        let trimmed = line.trim();
        if trimmed.contains('=') && !trimmed.contains(' ') {
            let equals_count = trimmed.matches('=').count();
            if equals_count == 1 {
                continue; // Single = is likely a valid assignment
            }
        }

        // Look for patterns like word=value=other
        for cap in UNQUOTED_EQUALS.captures_iter(line) {
            let full_match = cap.get(0).unwrap().as_str();

            // Skip if it's in quotes
            let before_match = &line[..line.find(full_match).unwrap_or(0)];
            let quote_count_double = before_match.matches('"').count();
            let quote_count_single = before_match.matches('\'').count();

            // If odd number of quotes, we're inside a quoted string
            if quote_count_double % 2 == 1 || quote_count_single % 2 == 1 {
                continue;
            }

            let start_col = line.find(full_match).unwrap_or(0) + 1;
            let end_col = start_col + full_match.len();

            let diagnostic = Diagnostic::new(
                "SC2026",
                Severity::Warning,
                format!(
                    "This word '{}' contains multiple '=' signs. Quote it to prevent word splitting",
                    full_match
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
    fn test_sc2026_unquoted_multiple_equals() {
        let code = r#"echo name=John=Doe"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
        assert_eq!(result.diagnostics[0].code, "SC2026");
        assert_eq!(result.diagnostics[0].severity, Severity::Warning);
        assert!(result.diagnostics[0].message.contains("multiple '='"));
    }

    #[test]
    fn test_sc2026_path_assignment() {
        let code = r#"echo PATH=/usr/bin=/usr/local/bin"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2026_config_value() {
        let code = r#"param=key=value=extra"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2026_quoted_ok() {
        let code = r#"echo "name=John=Doe""#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2026_single_quoted_ok() {
        let code = r#"echo 'PATH=/bin=/usr/bin'"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2026_simple_assignment_ok() {
        let code = r#"name=value"#;
        let result = check(code);
        // Single = is valid assignment
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2026_variable_assignment_ok() {
        let code = r#"PATH=$PATH:/new/path"#;
        let result = check(code);
        // Standard assignment pattern
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2026_export_ok() {
        let code = r#"export PATH=/usr/bin"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2026_multiple_issues() {
        let code = r#"
echo foo=bar=baz
echo key=val=extra
"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 2);
    }

    #[test]
    fn test_sc2026_in_command() {
        let code = r#"grep pattern=match=value file.txt"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }
}
