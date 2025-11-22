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
    // Match word containing multiple = signs (including paths)
    // Look for pattern: NAME=value=value (where values can include /, -, .)
    Regex::new(r"\b([a-zA-Z_][a-zA-Z0-9_]*=[a-zA-Z0-9_/.:-]+=[a-zA-Z0-9_/.:-]+)\b").unwrap()
});

/// Check if line is a simple assignment (var=value with single =)
fn is_simple_assignment(line: &str) -> bool {
    let trimmed = line.trim();
    if trimmed.contains('=') && !trimmed.contains(' ') {
        let equals_count = trimmed.matches('=').count();
        if equals_count == 1 {
            return true; // Single = is likely a valid assignment
        }
    }
    false
}

/// Check if match is inside quotes (odd number of quotes before it)
fn is_quoted(line: &str, match_text: &str) -> bool {
    let before_match = &line[..line.find(match_text).unwrap_or(0)];
    let quote_count_double = before_match.matches('"').count();
    let quote_count_single = before_match.matches('\'').count();

    // If odd number of quotes, we're inside a quoted string
    quote_count_double % 2 == 1 || quote_count_single % 2 == 1
}

/// Create diagnostic for unquoted word with multiple = signs
fn create_unquoted_diagnostic(
    match_text: &str,
    line: &str,
    line_num: usize,
) -> Diagnostic {
    let start_col = line.find(match_text).unwrap_or(0) + 1;
    let end_col = start_col + match_text.len();

    Diagnostic::new(
        "SC2026",
        Severity::Warning,
        format!(
            "This word '{}' contains multiple '=' signs. Quote it to prevent word splitting",
            match_text
        ),
        Span::new(line_num, start_col, line_num, end_col),
    )
}

pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();

    for (line_num, line) in source.lines().enumerate() {
        let line_num = line_num + 1;

        if line.trim_start().starts_with('#') {
            continue;
        }

        // Skip lines that are pure assignments (var=value)
        if is_simple_assignment(line) {
            continue;
        }

        // Look for patterns like word=value=other
        for cap in UNQUOTED_EQUALS.captures_iter(line) {
            let full_match = cap.get(0).unwrap().as_str();

            // Skip if it's in quotes
            if is_quoted(line, full_match) {
                continue;
            }

            let diagnostic = create_unquoted_diagnostic(full_match, line, line_num);
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
