//! SEC008: Using `curl | sh` Pattern
//!
//! **Rule**: Detect piping curl/wget output directly to shell execution
//!
//! **Why this matters**:
//! Piping untrusted URLs directly to shell execution is EXTREMELY DANGEROUS.
//! Attackers can serve malicious code, and MITM attacks can inject commands.
//! This is one of the most dangerous patterns in shell scripting.
//!
//! **Auto-fix**: Manual review required (not auto-fixable)
//!
//! ## Examples
//!
//! ❌ **EXTREMELY DANGEROUS**:
//! ```bash
//! curl https://install.example.com/script.sh | sh
//! wget -qO- https://get.example.com | bash
//! curl -sSL https://install.docker.com | sudo sh
//! ```
//!
//! ✅ **DOWNLOAD AND INSPECT FIRST**:
//! ```bash
//! curl -o install.sh https://install.example.com/script.sh
//! # Review install.sh before running
//! chmod +x install.sh
//! ./install.sh
//! ```

use crate::linter::{Diagnostic, LintResult, Severity, Span};

/// Check for curl/wget piped to shell
pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();

    for (line_num, line) in source.lines().enumerate() {
        // Look for curl | sh/bash patterns
        if (line.contains("curl") || line.contains("wget")) && line.contains('|') {
            // Check if piping to shell (possibly with sudo in between)
            let piped_to_shell = line.contains("| sh") || line.contains("| bash") ||
                                 line.contains("|sh") || line.contains("|bash") ||
                                 line.contains("| sudo sh") || line.contains("| sudo bash");

            if piped_to_shell {
                // Find the pipe position
                if let Some(pipe_col) = line.find('|') {
                    let span = Span::new(
                        line_num + 1,
                        pipe_col + 1,
                        line_num + 1,
                        line.len().min(pipe_col + 10),
                    );

                    let diag = Diagnostic::new(
                        "SEC008",
                        Severity::Error,
                        "CRITICAL: Piping curl/wget to shell - download and inspect first",
                        span,
                    );
                    // NO AUTO-FIX: requires manual review and complete workflow change

                    result.add(diag);
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
    fn test_SEC008_detects_curl_pipe_sh() {
        let script = "curl https://install.example.com/script.sh | sh";
        let result = check(script);

        assert_eq!(result.diagnostics.len(), 1);
        let diag = &result.diagnostics[0];
        assert_eq!(diag.code, "SEC008");
        assert_eq!(diag.severity, Severity::Error);
        assert!(diag.message.contains("CRITICAL"));
    }

    #[test]
    fn test_SEC008_detects_wget_pipe_bash() {
        let script = "wget -qO- https://get.example.com | bash";
        let result = check(script);

        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_SEC008_detects_curl_sudo_sh() {
        let script = "curl -sSL https://install.docker.com | sudo sh";
        let result = check(script);

        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_SEC008_no_warning_download_only() {
        let script = "curl -o install.sh https://install.example.com/script.sh";
        let result = check(script);

        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_SEC008_no_warning_pipe_to_file() {
        let script = "curl https://example.com | grep something";
        let result = check(script);

        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_SEC008_no_auto_fix() {
        let script = "wget -qO- https://script.com | sh";
        let result = check(script);

        assert_eq!(result.diagnostics.len(), 1);
        let diag = &result.diagnostics[0];
        assert!(diag.fix.is_none(), "SEC008 should not provide auto-fix");
    }
}
