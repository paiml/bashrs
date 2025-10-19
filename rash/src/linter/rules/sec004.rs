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
                    col + 23,  // "--no-check-certificate" is 22 chars
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
                    col + 2,  // Space before -k
                    line_num + 1,
                    col + 4,  // -k is 2 chars
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
                    col + 11,  // "--insecure" is 10 chars
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
}
