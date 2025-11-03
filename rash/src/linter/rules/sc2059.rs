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

    // ===== Mutation Coverage Tests - Iteration 1 =====
    // These 7 tests target the missed mutants from baseline (41.7% kill rate)
    // All 7 missed mutants are arithmetic column calculation mutations
    // Target: 90%+ kill rate (11/12 mutants caught)

    #[test]
    fn test_mutation_sc2059_printf_var_start_col_exact() {
        // MUTATION: Line 53:41 - replace + with * in mat.start() + 1
        // Tests PRINTF_WITH_VAR pattern start column calculation
        let bash_code = "printf $fmt arg"; // $fmt starts at column 8
        let result = check(bash_code);
        assert_eq!(result.diagnostics.len(), 1);
        let span = result.diagnostics[0].span;
        assert_eq!(
            span.start_col, 1,
            "Start column must use +1, not *1 (would be 0 with *)"
        );
    }

    #[test]
    fn test_mutation_sc2059_printf_var_end_col_exact() {
        // MUTATION: Line 54:37 - replace + with * or -
        // Tests PRINTF_WITH_VAR pattern end column calculation
        let bash_code = "printf $fmt"; // $fmt ends at column 12
        let result = check(bash_code);
        assert_eq!(result.diagnostics.len(), 1);
        let span = result.diagnostics[0].span;
        assert_eq!(span.end_col, 12, "End column must use +1, not *1 or -1");
    }

    #[test]
    fn test_mutation_sc2059_printf_expansion_start_col_exact() {
        // MUTATION: Line 71:45 - replace + with * in mat.start() + 1
        // Tests PRINTF_WITH_EXPANSION pattern start column calculation
        let bash_code = r#"printf "hello $name""#; // String starts at column 8
        let result = check(bash_code);
        assert_eq!(result.diagnostics.len(), 1);
        let span = result.diagnostics[0].span;
        assert_eq!(
            span.start_col, 1,
            "Start column calculation must use +1, not *1"
        );
    }

    #[test]
    fn test_mutation_sc2059_printf_expansion_end_col_exact() {
        // MUTATION: Line 72:41 - replace + with * or -
        // Tests PRINTF_WITH_EXPANSION pattern end column calculation
        let bash_code = r#"printf "$var""#; // String ends at column 13
        let result = check(bash_code);
        assert_eq!(result.diagnostics.len(), 1);
        let span = result.diagnostics[0].span;
        assert_eq!(span.end_col, 13, "End column must use +1, not *1 or -1");
    }

    #[test]
    fn test_mutation_sc2059_line_num_calculation() {
        // MUTATION: Line 45:33 - replace + with * in line_num + 1
        // Tests line number calculation (0-indexed → 1-indexed)
        let bash_code = "# comment\nprintf $var"; // printf on line 2
        let result = check(bash_code);
        assert_eq!(result.diagnostics.len(), 1);
        assert_eq!(
            result.diagnostics[0].span.start_line, 2,
            "Line number must use +1 (0-indexed → 1-indexed)"
        );
    }

    #[test]
    fn test_mutation_sc2059_column_positions_with_offset() {
        // Tests column calculations with leading whitespace
        // Verifies column arithmetic works correctly with offsets
        let bash_code = "    printf $fmt"; // $fmt starts at column 12
        let result = check(bash_code);
        assert_eq!(result.diagnostics.len(), 1);
        let span = result.diagnostics[0].span;
        assert_eq!(span.start_col, 5, "Should account for leading spaces");
        assert_eq!(span.end_col, 16, "End column should be start + length");
    }

    #[test]
    fn test_mutation_sc2059_expansion_column_accuracy() {
        // Tests PRINTF_WITH_EXPANSION pattern column accuracy
        // Verifies span covers the entire format string
        let bash_code = r#"printf "test $var""#;
        let result = check(bash_code);
        assert_eq!(result.diagnostics.len(), 1);
        let span = result.diagnostics[0].span;
        // Verify span covers the entire printf command
        assert!(span.end_col > span.start_col, "End must be after start");
        assert_eq!(span.start_col, 1, "Should start at printf");
    }
}
