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

/// Check for hardcoded secrets
pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();

    for (line_num, line) in source.lines().enumerate() {
        // Skip comments
        if line.trim_start().starts_with('#') {
            continue;
        }

        // Check each secret pattern
        for (pattern, description) in SECRET_PATTERNS {
            if line.contains(pattern) {
                // Check if it's an assignment with a literal value (not $VAR)
                // Pattern: VAR="literal" or VAR='literal'
                if let Some(eq_pos) = line.find('=') {
                    let after_eq = &line[eq_pos + 1..].trim_start();

                    // Check if it's a quoted literal (not a variable expansion)
                    if (after_eq.starts_with('"') && !after_eq.starts_with("\"$"))
                        || (after_eq.starts_with('\''))
                    {
                        // This looks like a hardcoded secret
                        if let Some(col) = line.find(pattern) {
                            let span = Span::new(
                                line_num + 1,
                                col + 1,
                                line_num + 1,
                                line.len().min(col + pattern.len() + 10),
                            );

                            let diag = Diagnostic::new(
                                "SEC005",
                                Severity::Error,
                                format!(
                                    "Hardcoded secret detected: {} - use environment variables",
                                    description
                                ),
                                span,
                            );
                            // NO AUTO-FIX: requires manual review

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
}
