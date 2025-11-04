//! SEC004: wget/curl Without TLS Verification
//!
//! **Rule**: Detect TLS verification being disabled in wget/curl commands
//!
//! **Why this matters**:
//! Disabling TLS verification opens man-in-the-middle attack vectors.
//! Attackers can intercept and modify HTTPS traffic.
//!
//! **Auto-fix**: Potentially unsafe (requires user decision)
//!
//! ## Examples
//!
//! ❌ **INSECURE**:
//! ```bash
//! wget --no-check-certificate https://example.com/file
//! curl -k https://api.example.com/data
//! curl --insecure https://downloads.example.com/app.tar.gz
//! ```
//!
//! ✅ **SECURE**:
//! ```bash
//! wget https://example.com/file  # Remove --no-check-certificate
//! curl https://api.example.com/data  # Remove -k flag
//! curl https://downloads.example.com/app.tar.gz  # Remove --insecure
//! ```

use crate::linter::{Diagnostic, Fix, LintResult, Severity, Span};

/// Check for disabled TLS verification in wget/curl
pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();

    for (line_num, line) in source.lines().enumerate() {
        // Check for wget --no-check-certificate
        if line.contains("wget") && line.contains("--no-check-certificate") {
            if let Some(col) = line.find("--no-check-certificate") {
                let span = Span::new(
                    line_num + 1,
                    col + 1,
                    line_num + 1,
                    col + 23, // "--no-check-certificate" is 22 chars
                );

                let diag = Diagnostic::new(
                    "SEC004",
                    Severity::Warning,
                    "TLS verification disabled in wget - MITM attack risk",
                    span,
                )
                .with_fix(Fix::new("# Remove --no-check-certificate"));

                result.add(diag);
            }
        }

        // Check for curl -k or --insecure
        if line.contains("curl") {
            if let Some(col) = line.find(" -k") {
                let span = Span::new(
                    line_num + 1,
                    col + 2, // Space before -k
                    line_num + 1,
                    col + 4, // -k is 2 chars
                );

                let diag = Diagnostic::new(
                    "SEC004",
                    Severity::Warning,
                    "TLS verification disabled in curl (-k) - MITM attack risk",
                    span,
                )
                .with_fix(Fix::new("# Remove -k"));

                result.add(diag);
            } else if let Some(col) = line.find("--insecure") {
                let span = Span::new(
                    line_num + 1,
                    col + 1,
                    line_num + 1,
                    col + 11, // "--insecure" is 10 chars
                );

                let diag = Diagnostic::new(
                    "SEC004",
                    Severity::Warning,
                    "TLS verification disabled in curl (--insecure) - MITM attack risk",
                    span,
                )
                .with_fix(Fix::new("# Remove --insecure"));

                result.add(diag);
            }
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_SEC004_detects_wget_no_check_certificate() {
        let script = "wget --no-check-certificate https://example.com/file";
        let result = check(script);

        assert_eq!(result.diagnostics.len(), 1);
        let diag = &result.diagnostics[0];
        assert_eq!(diag.code, "SEC004");
        assert_eq!(diag.severity, Severity::Warning);
        assert!(diag.message.contains("TLS"));
    }

    #[test]
    fn test_SEC004_detects_curl_k_flag() {
        let script = "curl -k https://api.example.com/data";
        let result = check(script);

        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_SEC004_detects_curl_insecure() {
        let script = "curl --insecure https://downloads.example.com/app.tar.gz";
        let result = check(script);

        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_SEC004_no_warning_secure_wget() {
        let script = "wget https://example.com/file";
        let result = check(script);

        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_SEC004_no_warning_secure_curl() {
        let script = "curl https://api.example.com/data";
        let result = check(script);

        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_SEC004_provides_fix() {
        let script = "wget --no-check-certificate https://example.com";
        let result = check(script);

        assert!(result.diagnostics[0].fix.is_some());
    }

    // ===== Mutation Coverage Tests - Following SEC001 pattern (100% kill rate) =====

    #[test]
    fn test_mutation_sec004_wget_start_col_exact() {
        // MUTATION: Line 38:21 - replace + with * in line_num + 1
        // MUTATION: Line 39:21 - replace + with * in col + 1
        let bash_code = "wget --no-check-certificate https://example.com";
        let result = check(bash_code);
        assert_eq!(result.diagnostics.len(), 1);
        let span = result.diagnostics[0].span;
        // "--no-check-certificate" starts at column 6 (after "wget ")
        assert_eq!(
            span.start_col, 6,
            "Start column must use col + 1, not col * 1"
        );
    }

    #[test]
    fn test_mutation_sec004_wget_end_col_exact() {
        // MUTATION: Line 41:21 - replace + with * in col + 23
        // MUTATION: Line 41:21 - replace + with - in col + 23
        let bash_code = "wget --no-check-certificate https://example.com";
        let result = check(bash_code);
        assert_eq!(result.diagnostics.len(), 1);
        let span = result.diagnostics[0].span;
        // "--no-check-certificate" is 22 chars, ends at col + 23
        assert_eq!(
            span.end_col, 28,
            "End column must be col + 23, not col * 23 or col - 23"
        );
    }

    #[test]
    fn test_mutation_sec004_curl_k_start_col_exact() {
        // MUTATION: Line 60:21 - replace + with * in line_num + 1
        // MUTATION: Line 61:21 - replace + with * in col + 2
        let bash_code = "curl -k https://api.example.com/data";
        let result = check(bash_code);
        assert_eq!(result.diagnostics.len(), 1);
        let span = result.diagnostics[0].span;
        // " -k" starts at column 6 (space before -k at col 5, -k at col 6)
        assert_eq!(
            span.start_col, 6,
            "Start column must use col + 2, not col * 2"
        );
    }

    #[test]
    fn test_mutation_sec004_curl_k_end_col_exact() {
        // MUTATION: Line 63:21 - replace + with * in col + 4
        // MUTATION: Line 63:21 - replace + with - in col + 4
        let bash_code = "curl -k https://api.example.com/data";
        let result = check(bash_code);
        assert_eq!(result.diagnostics.len(), 1);
        let span = result.diagnostics[0].span;
        // " -k" is 3 chars (space + -k), ends at col + 4
        assert_eq!(
            span.end_col, 8,
            "End column must be col + 4, not col * 4 or col - 4"
        );
    }

    #[test]
    fn test_mutation_sec004_curl_insecure_start_col_exact() {
        // MUTATION: Line 77:21 - replace + with * in line_num + 1
        // MUTATION: Line 78:21 - replace + with * in col + 1
        let bash_code = "curl --insecure https://downloads.example.com/app.tar.gz";
        let result = check(bash_code);
        assert_eq!(result.diagnostics.len(), 1);
        let span = result.diagnostics[0].span;
        // "--insecure" starts at column 6 (after "curl ")
        assert_eq!(
            span.start_col, 6,
            "Start column must use col + 1, not col * 1"
        );
    }

    #[test]
    fn test_mutation_sec004_curl_insecure_end_col_exact() {
        // MUTATION: Line 80:21 - replace + with * in col + 11
        // MUTATION: Line 80:21 - replace + with - in col + 11
        let bash_code = "curl --insecure https://downloads.example.com/app.tar.gz";
        let result = check(bash_code);
        assert_eq!(result.diagnostics.len(), 1);
        let span = result.diagnostics[0].span;
        // "--insecure" is 10 chars, ends at col + 11
        assert_eq!(
            span.end_col, 16,
            "End column must be col + 11, not col * 11 or col - 11"
        );
    }

    #[test]
    fn test_mutation_sec004_line_num_calculation() {
        // MUTATION: Line 38:21 - replace + with * in line_num + 1 (wget)
        // MUTATION: Line 60:21 - replace + with * in line_num + 1 (curl -k)
        // MUTATION: Line 77:21 - replace + with * in line_num + 1 (curl --insecure)
        // Tests line number calculation for multiline input
        let bash_code = "# comment\nwget --no-check-certificate https://example.com";
        let result = check(bash_code);
        assert_eq!(result.diagnostics.len(), 1);
        // With +1: line 2
        // With *1: line 0
        assert_eq!(
            result.diagnostics[0].span.start_line, 2,
            "Line number must use +1, not *1"
        );
    }
}
