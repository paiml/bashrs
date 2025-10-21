// SC2059: Don't use variables in the printf format string. Use printf '..%s..' "$foo"
//
// Using variables in printf format strings can lead to format string injection vulnerabilities.
// If the variable contains format specifiers like %s, %d, or %n, they will be interpreted
// by printf, potentially causing crashes, information leaks, or arbitrary code execution.
//
// Examples:
// Bad:
//   printf "$format" "$value"        // Format string injection
//   printf "Value: $var\n"           // Variable expansion in format
//   printf "$msg"                    // Direct variable as format
//
// Good:
//   printf '%s\n' "$value"           // Literal format string
//   printf 'Value: %s\n' "$var"      // Literal format with %s
//   printf '%s' "$msg"               // Safe variable output
//
// Security Impact:
//   - Format string vulnerabilities (arbitrary memory read/write)
//   - Information disclosure
//   - Denial of service (crashes)
//   - Potential code execution in some implementations
//
// Note: Always use literal format strings with printf. Use %s to safely output variables.

use crate::linter::{Diagnostic, LintResult, Severity, Span};
use once_cell::sync::Lazy;
use regex::Regex;

static PRINTF_WITH_VAR: Lazy<Regex> = Lazy::new(|| {
    // Match: printf "$var" or printf "...$var..."
    Regex::new(r#"printf\s+(['"]?)(\$[a-zA-Z_][a-zA-Z0-9_]*|\$\{[a-zA-Z_][a-zA-Z0-9_]*\})"#)
        .unwrap()
});

static PRINTF_WITH_EXPANSION: Lazy<Regex> = Lazy::new(|| {
    // Match: printf "...$var..." (variable in format string)
    Regex::new(r#"printf\s+"[^"]*\$[a-zA-Z_][a-zA-Z0-9_]*"#).unwrap()
});

pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();

    for (line_num, line) in source.lines().enumerate() {
        let line_num = line_num + 1;

        if line.trim_start().starts_with('#') {
            continue;
        }

        // Check for printf with variable as format string
        if let Some(mat) = PRINTF_WITH_VAR.find(line) {
            let start_col = mat.start() + 1;
            let end_col = mat.end() + 1;

            let diagnostic = Diagnostic::new(
                "SC2059",
                Severity::Error,
                "Don't use variables in the printf format string. Use printf '..%s..' \"$foo\""
                    .to_string(),
                Span::new(line_num, start_col, line_num, end_col),
            );

            result.add(diagnostic);
        }

        // Check for printf with variable expansion in format string
        if let Some(mat) = PRINTF_WITH_EXPANSION.find(line) {
            // Skip if already caught by first pattern
            if !PRINTF_WITH_VAR.is_match(line) {
                let start_col = mat.start() + 1;
                let end_col = mat.end() + 1;

                let diagnostic = Diagnostic::new(
                    "SC2059",
                    Severity::Error,
                    "Don't use variables in the printf format string. Use printf '..%s..' \"$foo\""
                        .to_string(),
                    Span::new(line_num, start_col, line_num, end_col),
                );

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
    fn test_sc2059_variable_as_format() {
        let code = r#"printf "$format" "value""#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
        assert_eq!(result.diagnostics[0].code, "SC2059");
        assert_eq!(result.diagnostics[0].severity, Severity::Error);
    }

    #[test]
    fn test_sc2059_braced_variable() {
        let code = r#"printf "${fmt}" "data""#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2059_variable_expansion_in_format() {
        let code = r#"printf "Value: $var\n""#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2059_direct_variable() {
        let code = r#"printf "$msg""#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2059_literal_format_ok() {
        let code = r#"printf '%s\n' "$value""#;
        let result = check(code);
        // Literal format string is safe
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2059_literal_with_percent_ok() {
        let code = r#"printf 'Value: %s\n' "$var""#;
        let result = check(code);
        // Literal format with %s placeholder is safe
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2059_no_variables_ok() {
        let code = r#"printf 'Hello, World!\n'"#;
        let result = check(code);
        // No variables, safe
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2059_comment_ok() {
        let code = r#"# printf "$format" "value""#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2059_single_quotes_ok() {
        let code = r#"printf 'Format: %s' "$value""#;
        let result = check(code);
        // Single quotes prevent expansion, safe
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2059_multiple_args_with_literal() {
        let code = r#"printf '%s %s\n' "$a" "$b""#;
        let result = check(code);
        // Literal format with multiple %s placeholders
        assert_eq!(result.diagnostics.len(), 0);
    }
}
