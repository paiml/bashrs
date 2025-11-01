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

/// Check if a line is a comment
fn is_comment_line(line: &str) -> bool {
    line.trim_start().starts_with('#')
}

/// Check if line has curl or wget
fn has_curl_or_wget(line: &str) -> bool {
    line.contains("curl") || line.contains("wget")
}

/// Check if line pipes to shell
fn is_piped_to_shell(line: &str) -> bool {
    line.contains("| sh")
        || line.contains("| bash")
        || line.contains("|sh")
        || line.contains("|bash")
        || line.contains("| sudo sh")
        || line.contains("| sudo bash")
}

/// Create diagnostic for curl/wget piped to shell
fn create_curl_pipe_diagnostic(line_num: usize, pipe_col: usize, line_len: usize) -> Diagnostic {
    let span = Span::new(
        line_num + 1,
        pipe_col + 1,
        line_num + 1,
        line_len.min(pipe_col + 10),
    );

    Diagnostic::new(
        "SEC008",
        Severity::Error,
        "CRITICAL: Piping curl/wget to shell - download and inspect first",
        span,
    )
    // NO AUTO-FIX: requires manual review and complete workflow change
}

/// Check for curl/wget piped to shell
pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();

    for (line_num, line) in source.lines().enumerate() {
        if is_comment_line(line) {
            continue;
        }

        // Look for curl | sh/bash patterns
        if has_curl_or_wget(line) && line.contains('|') && is_piped_to_shell(line) {
            // Find the pipe position
            if let Some(pipe_col) = line.find('|') {
                let diagnostic = create_curl_pipe_diagnostic(line_num, pipe_col, line.len());
                result.add(diagnostic);
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
    fn prop_sec008_comments_never_diagnosed() {
        // Property: Comment lines should never produce diagnostics
        let test_cases = vec![
            "# curl https://example.com | sh",
            "  # wget -qO- https://example.com | bash",
            "\t# curl -sSL https://example.com | sudo sh",
        ];

        for code in test_cases {
            let result = check(code);
            assert_eq!(result.diagnostics.len(), 0);
        }
    }

    #[test]
    fn prop_sec008_download_only_never_diagnosed() {
        // Property: Download-only commands should never be diagnosed
        let test_cases = vec![
            "curl -o install.sh https://example.com/script.sh",
            "wget -O script.sh https://example.com/script.sh",
            "curl -sSL https://example.com > file.sh",
            "wget -qO file.sh https://example.com",
        ];

        for code in test_cases {
            let result = check(code);
            assert_eq!(result.diagnostics.len(), 0);
        }
    }

    #[test]
    fn prop_sec008_pipe_to_non_shell_never_diagnosed() {
        // Property: Piping to non-shell commands should never be diagnosed
        let test_cases = vec![
            "curl https://example.com | grep something",
            "wget -qO- https://example.com | awk '{print $1}'",
            "curl https://example.com | jq '.field'",
            "wget https://example.com | sed 's/foo/bar/'",
        ];

        for code in test_cases {
            let result = check(code);
            assert_eq!(result.diagnostics.len(), 0);
        }
    }

    #[test]
    fn prop_sec008_no_pipe_never_diagnosed() {
        // Property: curl/wget without pipe should never be diagnosed
        let test_cases = vec![
            "curl https://example.com",
            "wget https://example.com",
            "curl -sSL https://example.com",
            "wget -qO- https://example.com",
        ];

        for code in test_cases {
            let result = check(code);
            assert_eq!(result.diagnostics.len(), 0);
        }
    }

    #[test]
    fn prop_sec008_curl_pipe_shell_always_diagnosed() {
        // Property: curl/wget piped to shell should always be diagnosed
        let test_cases = vec![
            "curl https://example.com | sh",
            "curl https://example.com | bash",
            "wget -qO- https://example.com | sh",
            "wget https://example.com | bash",
            "curl -sSL https://example.com | sudo sh",
            "wget -qO- https://example.com | sudo bash",
        ];

        for code in test_cases {
            let result = check(code);
            assert_eq!(result.diagnostics.len(), 1, "Should diagnose: {}", code);
            assert!(result.diagnostics[0].message.contains("CRITICAL"));
        }
    }

    #[test]
    fn prop_sec008_multiple_violations_all_diagnosed() {
        // Property: Multiple curl|sh patterns should all be diagnosed
        let code = "curl https://a.com | sh\nwget https://b.com | bash";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 2);
    }

    #[test]
    fn prop_sec008_diagnostic_code_always_sec008() {
        // Property: All diagnostics must have code "SEC008"
        let code = "curl https://a.com | sh\nwget https://b.com | bash";
        let result = check(code);

        for diagnostic in &result.diagnostics {
            assert_eq!(&diagnostic.code, "SEC008");
        }
    }

    #[test]
    fn prop_sec008_diagnostic_severity_always_error() {
        // Property: All diagnostics must be Error severity
        let code = "curl https://example.com | sh";
        let result = check(code);

        for diagnostic in &result.diagnostics {
            assert_eq!(diagnostic.severity, Severity::Error);
        }
    }

    #[test]
    fn prop_sec008_no_auto_fix_provided() {
        // Property: SEC008 should never provide auto-fix (security concern)
        let test_cases = vec![
            "curl https://example.com | sh",
            "wget -qO- https://example.com | bash",
            "curl -sSL https://example.com | sudo sh",
        ];

        for code in test_cases {
            let result = check(code);
            if !result.diagnostics.is_empty() {
                for diag in &result.diagnostics {
                    assert!(
                        diag.fix.is_none(),
                        "SEC008 should not provide auto-fix for: {}",
                        code
                    );
                }
            }
        }
    }

    #[test]
    fn prop_sec008_empty_source_no_diagnostics() {
        // Property: Empty source should produce no diagnostics
        let result = check("");
        assert_eq!(result.diagnostics.len(), 0);
    }

    // ===== Original Unit Tests =====

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
