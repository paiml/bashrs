//! SEC005: Hardcoded Secrets
//!
//! **Rule**: Detect hardcoded secrets, API keys, passwords, and tokens
//!
//! **Why this matters**:
//! Hardcoded secrets in scripts lead to credential leaks when committed to
//! version control. Secrets should be loaded from environment variables or
//! secure secret management systems.
//!
//! **Auto-fix**: Manual review required (not auto-fixable)
//!
//! ## Examples
//!
//! ❌ **HARDCODED SECRET**:
//! ```bash
//! API_KEY="sk-1234567890abcdef"
//! PASSWORD="MyP@ssw0rd"
//! TOKEN="ghp_xxxxxxxxxxxxxxxxxxxx"
//! AWS_SECRET_ACCESS_KEY="AKIAIOSFODNN7EXAMPLE"
//! ```
//!
//! ✅ **USE ENVIRONMENT VARIABLES**:
//! ```bash
//! API_KEY="${API_KEY:-}"
//! PASSWORD="${PASSWORD:-}"
//! TOKEN="${GITHUB_TOKEN:-}"
//! AWS_SECRET_ACCESS_KEY="${AWS_SECRET_ACCESS_KEY:-}"
//! ```

use crate::linter::{Diagnostic, LintResult, Severity, Span};

/// Patterns that indicate hardcoded secrets
const SECRET_PATTERNS: &[(&str, &str)] = &[
    ("API_KEY=", "API key assignment"),
    ("SECRET=", "Secret assignment"),
    ("PASSWORD=", "Password assignment"),
    ("TOKEN=", "Token assignment"),
    ("AWS_SECRET", "AWS secret"),
    ("GITHUB_TOKEN=", "GitHub token"),
    ("PRIVATE_KEY=", "Private key"),
    ("sk-", "OpenAI API key pattern"),
    ("ghp_", "GitHub personal access token"),
    ("gho_", "GitHub OAuth token"),
];

/// Check if a line is a comment
fn is_comment_line(line: &str) -> bool {
    line.trim_start().starts_with('#')
}

/// Extract value after equals sign
fn extract_after_equals(line: &str) -> Option<&str> {
    line.find('=').map(|eq_pos| &line[eq_pos + 1..])
}

/// Check if value is a literal assignment (not $VAR)
fn is_literal_assignment(after_eq: &str) -> bool {
    let trimmed = after_eq.trim_start();
    (trimmed.starts_with('"') && !trimmed.starts_with("\"$")) || trimmed.starts_with('\'')
}

/// Find pattern position in line
fn find_pattern_position(line: &str, pattern: &str) -> Option<usize> {
    line.find(pattern)
}

/// Calculate span for diagnostic
fn calculate_span(line_num: usize, col: usize, line_len: usize, pattern_len: usize) -> Span {
    Span::new(
        line_num + 1,
        col + 1,
        line_num + 1,
        line_len.min(col + pattern_len + 10),
    )
}

/// Create diagnostic for hardcoded secret
fn create_hardcoded_secret_diagnostic(description: &str, span: Span) -> Diagnostic {
    Diagnostic::new(
        "SEC005",
        Severity::Error,
        format!(
            "Hardcoded secret detected: {} - use environment variables",
            description
        ),
        span,
    )
    // NO AUTO-FIX: requires manual review
}

/// Check for hardcoded secrets
pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();

    for (line_num, line) in source.lines().enumerate() {
        if is_comment_line(line) {
            continue;
        }

        // Check each secret pattern
        for (pattern, description) in SECRET_PATTERNS {
            if line.contains(pattern) {
                if let Some(after_eq) = extract_after_equals(line) {
                    if is_literal_assignment(after_eq) {
                        // This looks like a hardcoded secret
                        if let Some(col) = find_pattern_position(line, pattern) {
                            let span = calculate_span(line_num, col, line.len(), pattern.len());
                            let diag = create_hardcoded_secret_diagnostic(description, span);
                            result.add(diag);
                            break; // Only report once per line
                        }
                    }
                }
            }
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    // ===== Manual Property Tests =====

    #[test]
    fn prop_sec005_comments_never_diagnosed() {
        // Property: Comment lines should never produce diagnostics
        let test_cases = vec![
            "# API_KEY=\"sk-1234567890abcdef\"",
            "  # PASSWORD='MyP@ssw0rd'",
            "\t# TOKEN=\"ghp_xxxxxxxxxxxxxxxxxxxx\"",
        ];

        for code in test_cases {
            let result = check(code);
            assert_eq!(result.diagnostics.len(), 0);
        }
    }

    #[test]
    fn prop_sec005_env_vars_never_diagnosed() {
        // Property: Environment variable assignments should never be diagnosed
        let test_cases = vec![
            "API_KEY=\"${API_KEY:-}\"",
            "PASSWORD=\"${PASSWORD:-}\"",
            "TOKEN=\"${GITHUB_TOKEN:-}\"",
            "SECRET=\"${SECRET:-default}\"",
        ];

        for code in test_cases {
            let result = check(code);
            assert_eq!(result.diagnostics.len(), 0);
        }
    }

    #[test]
    fn prop_sec005_variable_expansions_never_diagnosed() {
        // Property: Variable expansions should never be diagnosed
        let test_cases = vec![
            "API_KEY=\"$MY_API_KEY\"",
            "PASSWORD=\"$MY_PASSWORD\"",
            "TOKEN=\"$GITHUB_TOKEN\"",
            "SECRET=\"$MY_SECRET\"",
        ];

        for code in test_cases {
            let result = check(code);
            assert_eq!(result.diagnostics.len(), 0);
        }
    }

    #[test]
    fn prop_sec005_hardcoded_literals_always_diagnosed() {
        // Property: Hardcoded secret literals should always be diagnosed
        let test_cases = vec![
            "API_KEY=\"sk-1234567890abcdef\"",
            "PASSWORD='MyP@ssw0rd'",
            "TOKEN=\"ghp_xxxxxxxxxxxxxxxxxxxx\"",
            "SECRET=\"my-secret-value\"",
            "AWS_SECRET_ACCESS_KEY=\"AKIAIOSFODNN7EXAMPLE\"",
        ];

        for code in test_cases {
            let result = check(code);
            assert_eq!(result.diagnostics.len(), 1, "Should diagnose: {}", code);
            assert!(result.diagnostics[0].message.contains("Hardcoded secret"));
        }
    }

    #[test]
    fn prop_sec005_diagnostic_code_always_sec005() {
        // Property: All diagnostics must have code \"SEC005\"
        let code = "API_KEY=\"sk-123\"\nPASSWORD='pass123'";
        let result = check(code);

        for diagnostic in &result.diagnostics {
            assert_eq!(&diagnostic.code, "SEC005");
        }
    }

    #[test]
    fn prop_sec005_diagnostic_severity_always_error() {
        // Property: All diagnostics must be Error severity
        let code = "SECRET=\"hardcoded-secret\"";
        let result = check(code);

        for diagnostic in &result.diagnostics {
            assert_eq!(diagnostic.severity, Severity::Error);
        }
    }

    #[test]
    fn prop_sec005_no_auto_fix_provided() {
        // Property: SEC005 should never provide auto-fix (security concern)
        let test_cases = vec![
            "API_KEY=\"sk-123\"",
            "PASSWORD='pass'",
            "TOKEN=\"ghp_xxx\"",
            "SECRET=\"secret\"",
        ];

        for code in test_cases {
            let result = check(code);
            if !result.diagnostics.is_empty() {
                for diag in &result.diagnostics {
                    assert!(
                        diag.fix.is_none(),
                        "SEC005 should not provide auto-fix for: {}",
                        code
                    );
                }
            }
        }
    }

    #[test]
    fn prop_sec005_one_diagnostic_per_line() {
        // Property: Only one diagnostic per line (breaks after first match)
        let code = "API_KEY=\"sk-123\" PASSWORD='pass'"; // Multiple secrets in one line
        let result = check(code);
        assert_eq!(
            result.diagnostics.len(),
            1,
            "Should only diagnose once per line"
        );
    }

    #[test]
    fn prop_sec005_multiple_lines_all_diagnosed() {
        // Property: Multiple lines with secrets should all be diagnosed
        let code = "API_KEY=\"sk-123\"\nPASSWORD='pass'\nTOKEN=\"ghp_xxx\"";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 3);
    }

    #[test]
    fn prop_sec005_empty_source_no_diagnostics() {
        // Property: Empty source should produce no diagnostics
        let result = check("");
        assert_eq!(result.diagnostics.len(), 0);
    }

    // ===== Original Unit Tests =====

    #[test]
    fn test_SEC005_detects_hardcoded_api_key() {
        let script = r#"API_KEY="sk-1234567890abcdef""#;
        let result = check(script);

        assert_eq!(result.diagnostics.len(), 1);
        let diag = &result.diagnostics[0];
        assert_eq!(diag.code, "SEC005");
        assert_eq!(diag.severity, Severity::Error);
        assert!(diag.message.contains("Hardcoded"));
    }

    #[test]
    fn test_SEC005_detects_hardcoded_password() {
        let script = "PASSWORD='MyP@ssw0rd'";
        let result = check(script);

        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_SEC005_detects_github_token() {
        let script = r#"TOKEN="ghp_xxxxxxxxxxxxxxxxxxxx""#;
        let result = check(script);

        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_SEC005_no_warning_env_var() {
        let script = r#"API_KEY="${API_KEY:-}""#;
        let result = check(script);

        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_SEC005_no_warning_variable_expansion() {
        let script = "PASSWORD=\"$MY_PASSWORD\"";
        let result = check(script);

        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_SEC005_no_warning_comment() {
        let script = r#"# API_KEY="secret123""#;
        let result = check(script);

        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_SEC005_no_auto_fix() {
        let script = r#"SECRET="my-secret-value""#;
        let result = check(script);

        assert_eq!(result.diagnostics.len(), 1);
        let diag = &result.diagnostics[0];
        assert!(diag.fix.is_none(), "SEC005 should not provide auto-fix");
    }

    // ===== Mutation Coverage Tests - Following SEC001 pattern (100% kill rate) =====

    #[test]
    fn test_mutation_sec005_calculate_span_start_col_exact() {
        // MUTATION: Line 70:9 - replace + with * in line_num + 1
        // MUTATION: Line 71:9 - replace + with * in col + 1
        let bash_code = r#"API_KEY="sk-1234567890abcdef""#;
        let result = check(bash_code);
        assert_eq!(result.diagnostics.len(), 1);
        let span = result.diagnostics[0].span;
        // API_KEY starts at column 1 (0-indexed), span should be col + 1
        assert_eq!(
            span.start_col, 1,
            "Start column must use col + 1, not col * 1"
        );
    }

    #[test]
    fn test_mutation_sec005_calculate_span_line_num_exact() {
        // MUTATION: Line 70:9 - replace + with * in line_num + 1
        // Tests line number calculation for multiline input
        let bash_code = "# comment\nAPI_KEY=\"sk-1234567890abcdef\"";
        let result = check(bash_code);
        assert_eq!(result.diagnostics.len(), 1);
        // With +1: line 2
        // With *1: line 0
        assert_eq!(
            result.diagnostics[0].span.start_line, 2,
            "Line number must use +1, not *1"
        );
    }

    #[test]
    fn test_mutation_sec005_calculate_span_end_col_complex() {
        // MUTATION: Line 73:9 - arithmetic mutations in line_len.min(col + pattern_len + 10)
        // Tests end column calculation with min() function
        let bash_code = r#"API_KEY="sk-123""#;
        let result = check(bash_code);
        assert_eq!(result.diagnostics.len(), 1);
        let span = result.diagnostics[0].span;
        // Verify end column is calculated correctly
        // API_KEY is at column 0, pattern_len is 8 ("API_KEY="), +10 padding
        // Should be min(line_len, col + pattern_len + 10)
        assert!(
            span.end_col > span.start_col,
            "End column must be greater than start column"
        );
        assert!(
            span.end_col <= bash_code.len(),
            "End column must not exceed line length"
        );
    }

    #[test]
    fn test_mutation_sec005_column_with_leading_whitespace() {
        // Tests column calculations with offset
        let bash_code = r#"    SECRET="hardcoded""#;
        let result = check(bash_code);
        assert_eq!(result.diagnostics.len(), 1);
        let span = result.diagnostics[0].span;
        // SECRET starts at column 5 (4 spaces + 1)
        assert_eq!(span.start_col, 5, "Must account for leading whitespace");
    }

    #[test]
    fn test_mutation_sec005_multiple_patterns_first_detected() {
        // Tests that column tracking works correctly when multiple patterns exist
        let bash_code = r#"PASSWORD="pass123""#;
        let result = check(bash_code);
        assert_eq!(result.diagnostics.len(), 1);
        let span = result.diagnostics[0].span;
        // PASSWORD starts at column 1
        assert_eq!(span.start_col, 1, "Should detect first pattern correctly");
    }
}
