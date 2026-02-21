//! SEC015: Unsafe curl/wget Usage
//!
//! **Rule**: Detect unsafe patterns in curl/wget commands
//!
//! **Why this matters**:
//! - Piping curl/wget to sh executes arbitrary remote code
//! - Disabling certificate verification (--insecure/-k) enables MITM attacks
//! - Missing integrity checks allow tampered downloads
//!
//! ## Examples
//!
//! Bad:
//! ```bash
//! curl https://example.com/install.sh | sh
//! wget -O- https://example.com/setup | bash
//! curl -k https://api.example.com/data
//! curl --insecure https://example.com/file
//! ```
//!
//! Good:
//! ```bash
//! curl -fsSL https://example.com/install.sh -o install.sh
//! sha256sum -c install.sh.sha256
//! bash install.sh
//! ```

use crate::linter::{Diagnostic, LintResult, Severity, Span};

/// Check for unsafe curl/wget usage
pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();

    for (line_num, line) in source.lines().enumerate() {
        let trimmed = line.trim();

        // Skip comments and empty lines
        if trimmed.starts_with('#') || trimmed.is_empty() {
            continue;
        }

        // Pattern 1: curl/wget piped to shell (critical)
        if is_pipe_to_shell(trimmed) {
            let span = Span::new(line_num + 1, 1, line_num + 1, line.len());
            let diag = Diagnostic::new(
                "SEC015",
                Severity::Error,
                "Piping curl/wget to shell executes arbitrary remote code - download, verify, then execute separately",
                span,
            );
            result.add(diag);
            continue; // Don't double-report
        }

        // Pattern 2: --insecure or -k flag (disables TLS verification)
        if has_insecure_flag(trimmed) {
            let span = Span::new(line_num + 1, 1, line_num + 1, line.len());
            let diag = Diagnostic::new(
                "SEC015",
                Severity::Warning,
                "curl/wget with --insecure/-k disables TLS certificate verification - vulnerable to MITM attacks",
                span,
            );
            result.add(diag);
        }

        // Pattern 3: wget --no-check-certificate
        if trimmed.contains("wget") && trimmed.contains("--no-check-certificate") {
            let span = Span::new(line_num + 1, 1, line_num + 1, line.len());
            let diag = Diagnostic::new(
                "SEC015",
                Severity::Warning,
                "wget --no-check-certificate disables TLS verification - vulnerable to MITM attacks",
                span,
            );
            result.add(diag);
        }
    }

    result
}

/// Check if line pipes curl/wget output to a shell
fn is_pipe_to_shell(line: &str) -> bool {
    let has_download = line.contains("curl ") || line.contains("wget ");

    if !has_download {
        return false;
    }

    // Check for pipe to shell patterns
    let shell_targets = [
        "| sh",
        "| bash",
        "| zsh",
        "| dash",
        "| ksh",
        "|sh",
        "|bash",
        "|zsh",
        "|dash",
        "|ksh",
        "| sudo sh",
        "| sudo bash",
        "|sudo sh",
        "|sudo bash",
    ];

    shell_targets.iter().any(|target| line.contains(target))
}

/// Check if curl/wget uses --insecure or -k flag
fn has_insecure_flag(line: &str) -> bool {
    if !line.contains("curl ") && !line.contains("wget ") {
        return false;
    }

    // Check for -k or --insecure flags
    line.split_whitespace().any(|word| {
        word == "-k" || word == "--insecure"
            // Also catch combined flags like -ksSL
            || (word.starts_with('-') && !word.starts_with("--") && word.contains('k') && line.contains("curl"))
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sec015_detects_curl_pipe_sh() {
        let script = "curl https://example.com/install.sh | sh";
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 1);
        assert_eq!(result.diagnostics[0].code, "SEC015");
        assert_eq!(result.diagnostics[0].severity, Severity::Error);
    }

    #[test]
    fn test_sec015_detects_curl_pipe_bash() {
        let script = "curl -fsSL https://example.com/setup | bash";
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 1);
        assert_eq!(result.diagnostics[0].severity, Severity::Error);
    }

    #[test]
    fn test_sec015_detects_wget_pipe_sh() {
        let script = "wget -O- https://example.com/setup | sh";
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sec015_detects_curl_pipe_sudo_bash() {
        let script = "curl https://example.com/install.sh | sudo bash";
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 1);
        assert_eq!(result.diagnostics[0].severity, Severity::Error);
    }

    #[test]
    fn test_sec015_detects_insecure_flag() {
        let script = "curl --insecure https://api.example.com/data";
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 1);
        assert_eq!(result.diagnostics[0].severity, Severity::Warning);
    }

    #[test]
    fn test_sec015_detects_k_flag() {
        let script = "curl -k https://api.example.com/data";
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sec015_detects_wget_no_check_cert() {
        let script = "wget --no-check-certificate https://example.com/file";
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sec015_safe_curl_to_file() {
        let script = "curl -fsSL https://example.com/file -o output.tar.gz";
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sec015_safe_wget_to_file() {
        let script = "wget https://example.com/file -O output.tar.gz";
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sec015_ignores_comments() {
        let script = "# curl https://example.com | sh";
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sec015_empty() {
        let result = check("");
        assert_eq!(result.diagnostics.len(), 0);
    }
}

#[cfg(test)]
mod property_tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #![proptest_config(proptest::test_runner::Config::with_cases(10))]

        #[test]
        fn prop_sec015_never_panics(s in ".*") {
            let _ = check(&s);
        }

        #[test]
        fn prop_sec015_pipe_to_shell_always_detected(
            url in "https://[a-z]{3,10}\\.com/[a-z]{3,10}",
            shell in "(sh|bash|zsh)",
        ) {
            let script = format!("curl {} | {}", url, shell);
            let result = check(&script);
            prop_assert!(!result.diagnostics.is_empty());
        }
    }
}
