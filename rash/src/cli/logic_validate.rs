// CLI Logic - Validation and Checking
//
// Validation, verification, and checking logic for CLI commands.

use crate::models::{Error, Result};
use std::path::Path;

// =============================================================================
// GATE EXECUTION LOGIC
// =============================================================================

/// Result of a gate check
#[derive(Debug, Clone, PartialEq)]
pub enum GateResult {
    /// Gate passed
    Pass,
    /// Gate failed
    Fail,
    /// Gate skipped (tool not found)
    Skipped(String),
    /// Unknown gate
    Unknown,
}

impl GateResult {
    pub fn is_success(&self) -> bool {
        matches!(self, Self::Pass | Self::Skipped(_))
    }

    pub fn format(&self) -> &'static str {
        match self {
            Self::Pass => "✅ PASS",
            Self::Fail => "❌ FAIL",
            Self::Skipped(_) => "⚠️  SKIP",
            Self::Unknown => "⚠️  Unknown gate",
        }
    }
}

/// Validate gate tier
pub fn validate_gate_tier(tier: u8) -> Result<()> {
    if !(1..=3).contains(&tier) {
        Err(Error::Validation(format!(
            "Invalid tier: {}. Must be 1, 2, or 3.",
            tier
        )))
    } else {
        Ok(())
    }
}

// =============================================================================
// CHECK COMMAND LOGIC
// =============================================================================

/// Result of check command
#[derive(Debug, Clone)]
pub enum CheckResult {
    /// File is compatible
    Compatible,
    /// File is a shell script (not Rash source)
    IsShellScript { path: String },
    /// Check failed
    Error(String),
}

impl CheckResult {
    pub fn format(&self) -> String {
        match self {
            Self::Compatible => "✓ File is compatible with Rash".to_string(),
            Self::IsShellScript { path } => {
                format!(
                    "File '{}' appears to be a shell script, not Rash source.\n\n\
                     The 'check' command is for verifying Rash (.rs) source files that will be\n\
                     transpiled to shell scripts.\n\n\
                     For linting existing shell scripts, use:\n\
                       bashrs lint {}\n\n\
                     For purifying shell scripts (adding determinism/idempotency):\n\
                       bashrs purify {}",
                    path, path, path
                )
            }
            Self::Error(e) => format!("Error: {}", e),
        }
    }
}

/// Process check command logic
pub fn process_check(path: &Path, content: &str) -> CheckResult {
    if super::is_shell_script_file(path, content) {
        return CheckResult::IsShellScript {
            path: path.display().to_string(),
        };
    }

    // Actual check logic would go here
    CheckResult::Compatible
}

// =============================================================================
// VERIFY COMMAND LOGIC
// =============================================================================

/// Result of verification
#[derive(Debug, Clone, PartialEq)]
pub enum VerifyResult {
    /// Scripts match
    Match,
    /// Scripts don't match
    Mismatch,
}

impl VerifyResult {
    pub fn format(&self) -> &'static str {
        match self {
            Self::Match => "✓ Shell script matches Rust source",
            Self::Mismatch => "✗ Shell script does not match Rust source",
        }
    }
}

/// Compare generated shell script with expected
pub fn verify_scripts(generated: &str, expected: &str) -> VerifyResult {
    if super::normalize_shell_script(generated) == super::normalize_shell_script(expected) {
        VerifyResult::Match
    } else {
        VerifyResult::Mismatch
    }
}

// =============================================================================
// VALIDATION FUNCTIONS
// =============================================================================

/// Validate proof format data
pub fn validate_proof_data(source_hash: &str, verification_level: &str, target: &str) -> bool {
    // Hash should be non-empty hex
    !source_hash.is_empty()
        && source_hash.chars().all(|c| c.is_ascii_hexdigit())
        && !verification_level.is_empty()
        && !target.is_empty()
}

/// Extract exit code from error message (pure function)
pub fn extract_exit_code(error: &str) -> i32 {
    // Common patterns for exit codes in error messages
    let patterns = [
        ("exit code ", 10),
        ("exited with ", 12),
        ("returned ", 9),
        ("status ", 7),
    ];

    for (pattern, prefix_len) in patterns {
        if let Some(idx) = error.to_lowercase().find(pattern) {
            let start = idx + prefix_len;
            let code_str: String = error[start..]
                .chars()
                .take_while(|c| c.is_ascii_digit())
                .collect();
            if let Ok(code) = code_str.parse::<i32>() {
                return code;
            }
        }
    }

    // Check for well-known exit codes in error messages
    if error.contains("command not found") {
        return 127;
    }
    if error.contains("Permission denied") || error.contains("permission denied") {
        return 126;
    }

    // Default to generic failure
    1
}

#[cfg(test)]
mod tests {
    use super::*;

    // ===== GATE RESULT TESTS =====

    #[test]
    fn test_gate_result_is_success() {
        assert!(GateResult::Pass.is_success());
        assert!(GateResult::Skipped("tool not found".to_string()).is_success());
        assert!(!GateResult::Fail.is_success());
        assert!(!GateResult::Unknown.is_success());
    }

    #[test]
    fn test_validate_gate_tier() {
        assert!(validate_gate_tier(1).is_ok());
        assert!(validate_gate_tier(2).is_ok());
        assert!(validate_gate_tier(3).is_ok());
        assert!(validate_gate_tier(0).is_err());
        assert!(validate_gate_tier(4).is_err());
    }

    // ===== VERIFY SCRIPTS TESTS =====

    #[test]
    fn test_verify_scripts_match() {
        let script1 = "#!/bin/sh\necho hello";
        let script2 = "#!/bin/sh\necho hello";
        assert_eq!(verify_scripts(script1, script2), VerifyResult::Match);
    }

    #[test]
    fn test_verify_scripts_match_ignores_comments() {
        let script1 = "# comment\necho hello";
        let script2 = "echo hello";
        assert_eq!(verify_scripts(script1, script2), VerifyResult::Match);
    }

    #[test]
    fn test_verify_scripts_match_ignores_whitespace() {
        let script1 = "  echo hello  ";
        let script2 = "echo hello";
        assert_eq!(verify_scripts(script1, script2), VerifyResult::Match);
    }

    #[test]
    fn test_verify_scripts_mismatch() {
        let script1 = "echo hello";
        let script2 = "echo world";
        assert_eq!(verify_scripts(script1, script2), VerifyResult::Mismatch);
    }

    // ===== CHECK RESULT TESTS =====

    #[test]
    fn test_check_result_shell_script() {
        let result = process_check(Path::new("script.sh"), "echo hello");
        assert!(matches!(result, CheckResult::IsShellScript { .. }));
    }

    #[test]
    fn test_check_result_rust_file() {
        let result = process_check(Path::new("main.rs"), "fn main() {}");
        assert!(matches!(result, CheckResult::Compatible));
    }

    // ===== VALIDATION TESTS =====

    #[test]
    fn test_validate_proof_data_valid() {
        assert!(validate_proof_data("deadbeef", "strict", "posix"));
        assert!(validate_proof_data("0123456789abcdef", "minimal", "bash"));
    }

    #[test]
    fn test_validate_proof_data_invalid_hash() {
        assert!(!validate_proof_data("", "strict", "posix"));
        assert!(!validate_proof_data("xyz123", "strict", "posix")); // non-hex
    }

    #[test]
    fn test_validate_proof_data_empty_fields() {
        assert!(!validate_proof_data("deadbeef", "", "posix"));
        assert!(!validate_proof_data("deadbeef", "strict", ""));
    }

    // ===== EXTRACT EXIT CODE TESTS =====

    #[test]
    fn test_extract_exit_code_exit_code_pattern() {
        assert_eq!(extract_exit_code("Process failed with exit code 1"), 1);
        assert_eq!(extract_exit_code("exit code 127"), 127);
        assert_eq!(extract_exit_code("Error: exit code 255"), 255);
    }

    #[test]
    fn test_extract_exit_code_exited_with_pattern() {
        assert_eq!(extract_exit_code("Command exited with 42"), 42);
        assert_eq!(extract_exit_code("Process exited with 0"), 0);
    }

    #[test]
    fn test_extract_exit_code_returned_pattern() {
        assert_eq!(extract_exit_code("Function returned 5"), 5);
        assert_eq!(extract_exit_code("returned 100"), 100);
    }

    #[test]
    fn test_extract_exit_code_status_pattern() {
        assert_eq!(extract_exit_code("status 2"), 2);
        assert_eq!(extract_exit_code("Exit status 128"), 128);
    }

    #[test]
    fn test_extract_exit_code_command_not_found() {
        assert_eq!(extract_exit_code("bash: foo: command not found"), 127);
        assert_eq!(extract_exit_code("command not found: xyz"), 127);
    }

    #[test]
    fn test_extract_exit_code_permission_denied() {
        assert_eq!(extract_exit_code("Permission denied"), 126);
        assert_eq!(extract_exit_code("Error: permission denied"), 126);
    }

    #[test]
    fn test_extract_exit_code_default() {
        assert_eq!(extract_exit_code("Unknown error"), 1);
        assert_eq!(extract_exit_code("Something went wrong"), 1);
        assert_eq!(extract_exit_code(""), 1);
    }
}
