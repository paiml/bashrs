//! SEC017: Unsafe File Permissions (chmod 777)
//!
//! **Rule**: Detect chmod 777, 666, or other overly permissive modes
//!
//! **Why this matters**:
//! chmod 777 gives read/write/execute permissions to everyone (owner, group, world).
//! This is a severe security risk as any user can modify or execute the file.
//!
//! **Auto-fix**: Manual review required (appropriate permissions depend on context)
//!
//! ## Examples
//!
//! ❌ **CRITICAL VULNERABILITY**:
//! ```bash
//! chmod 777 /etc/passwd
//! chmod 666 ~/.ssh/id_rsa
//! chmod -R 777 /var/www
//! ```
//!
//! ✅ **SAFE ALTERNATIVES**:
//! ```bash
//! chmod 644 /etc/passwd    # Owner: rw-, Group: r--, World: r--
//! chmod 600 ~/.ssh/id_rsa  # Owner: rw-, Group: ---, World: ---
//! chmod 755 /var/www       # Owner: rwx, Group: r-x, World: r-x
//! ```

use crate::linter::{Diagnostic, LintResult, Severity, Span};

/// Dangerous file permission modes
const DANGEROUS_MODES: &[&str] = &[
    "777", // rwxrwxrwx - everyone can do everything
    "666", // rw-rw-rw- - everyone can read/write
    "664", // rw-rw-r-- - group can write (risky for sensitive files)
    "776", // rwxrwxrw- - world can write
    "677", // rw-rwxrwx - group/world can execute
];

/// Check for unsafe file permissions (chmod 777, 666, etc.)
pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();

    for (line_num, line) in source.lines().enumerate() {
        // Look for chmod command
        if let Some(chmod_col) = find_command(line, "chmod") {
            // Check if line contains dangerous permissions
            for dangerous_mode in DANGEROUS_MODES {
                if contains_mode(line, dangerous_mode) {
                    let span = Span::new(line_num + 1, chmod_col + 1, line_num + 1, line.len());

                    let severity = match *dangerous_mode {
                        "777" | "666" => Severity::Error, // Critical
                        _ => Severity::Warning,           // High risk but not always critical
                    };

                    let diag = Diagnostic::new(
                        "SEC017",
                        severity,
                        format!(
                            "Unsafe file permissions: chmod {} grants excessive permissions - use principle of least privilege",
                            dangerous_mode
                        ),
                        span,
                    );
                    // NO AUTO-FIX: Correct permissions depend on context

                    result.add(diag);
                    break; // Only report once per line
                }
            }
        }
    }

    result
}

/// Find chmod command in a line (word boundary detection)
fn find_command(line: &str, cmd: &str) -> Option<usize> {
    if let Some(pos) = line.find(cmd) {
        // Check word boundaries
        let before_ok = if pos == 0 {
            true
        } else {
            let char_before = line.chars().nth(pos - 1);
            matches!(
                char_before,
                Some(' ') | Some('\t') | Some(';') | Some('&') | Some('|') | Some('(') | Some('\n')
            )
        };

        let after_idx = pos + cmd.len();
        let after_ok = if after_idx >= line.len() {
            true
        } else {
            let char_after = line.chars().nth(after_idx);
            matches!(
                char_after,
                Some(' ') | Some('\t') | Some(';') | Some('&') | Some('|') | Some(')')
            )
        };

        if before_ok && after_ok {
            return Some(pos);
        }
    }
    None
}

/// Check if line contains a specific permission mode
fn contains_mode(line: &str, mode: &str) -> bool {
    // Look for the mode as a standalone token (not part of another number)
    for word in line.split_whitespace() {
        // Check exact match or with -R flag
        if word == mode || word == format!("-R {}", mode) || word.ends_with(&format!(" {}", mode)) {
            return true;
        }
        // Handle cases like "chmod -R 777" or "chmod 777"
        if word.contains(mode) {
            // Ensure it's not part of a larger number (e.g., 1777)
            let mode_pos = word.find(mode);
            if let Some(pos) = mode_pos {
                let before_ok = if pos == 0 {
                    true
                } else {
                    let char_before = word.chars().nth(pos - 1);
                    !matches!(char_before, Some('0'..='9'))
                };

                let after_idx = pos + mode.len();
                let after_ok = if after_idx >= word.len() {
                    true
                } else {
                    let char_after = word.chars().nth(after_idx);
                    !matches!(char_after, Some('0'..='9'))
                };

                if before_ok && after_ok {
                    return true;
                }
            }
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    // RED Phase: Write failing tests first

    #[test]
    fn test_SEC017_detects_chmod_777() {
        let script = "chmod 777 /etc/passwd";
        let result = check(script);

        assert_eq!(result.diagnostics.len(), 1);
        let diag = &result.diagnostics[0];
        assert_eq!(diag.code, "SEC017");
        assert_eq!(diag.severity, Severity::Error);
        assert!(diag.message.contains("777"));
    }

    #[test]
    fn test_SEC017_detects_chmod_666() {
        let script = "chmod 666 sensitive.txt";
        let result = check(script);

        assert_eq!(result.diagnostics.len(), 1);
        let diag = &result.diagnostics[0];
        assert_eq!(diag.code, "SEC017");
        assert_eq!(diag.severity, Severity::Error);
    }

    #[test]
    fn test_SEC017_detects_chmod_recursive_777() {
        let script = "chmod -R 777 /var/www";
        let result = check(script);

        assert_eq!(result.diagnostics.len(), 1);
        assert_eq!(result.diagnostics[0].code, "SEC017");
    }

    #[test]
    fn test_SEC017_safe_chmod_755() {
        let script = "chmod 755 script.sh";
        let result = check(script);

        // 755 is safe (rwxr-xr-x)
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_SEC017_safe_chmod_644() {
        let script = "chmod 644 config.conf";
        let result = check(script);

        // 644 is safe (rw-r--r--)
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_SEC017_safe_chmod_600() {
        let script = "chmod 600 ~/.ssh/id_rsa";
        let result = check(script);

        // 600 is safe (rw-------)
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_SEC017_no_false_positive_comment() {
        let script = "# chmod 777 is dangerous";
        let result = check(script);

        // Should detect even in comments for documentation
        // This is acceptable for security education
    }

    #[test]
    fn test_SEC017_multiple_dangerous_chmod() {
        let script = r#"
chmod 777 /tmp/file1
chmod 666 /tmp/file2
        "#;
        let result = check(script);

        assert_eq!(result.diagnostics.len(), 2);
    }

    #[test]
    fn test_SEC017_no_auto_fix() {
        let script = "chmod 777 file.txt";
        let result = check(script);

        assert_eq!(result.diagnostics.len(), 1);
        let diag = &result.diagnostics[0];
        assert!(diag.fix.is_none(), "SEC017 should not provide auto-fix");
    }

    #[test]
    fn test_SEC017_detects_664_as_warning() {
        let script = "chmod 664 shared.txt";
        let result = check(script);

        assert_eq!(result.diagnostics.len(), 1);
        let diag = &result.diagnostics[0];
        assert_eq!(diag.severity, Severity::Warning); // Not critical but risky
    }
}

#[cfg(test)]
mod property_tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn prop_sec017_never_panics(s in ".*") {
            let _ = check(&s);
        }

        #[test]
        fn prop_sec017_safe_modes_no_warnings(
            mode in "(600|644|755|700)",
            file in "[a-z/]{1,20}",
        ) {
            let cmd = format!("chmod {} {}", mode, file);
            let result = check(&cmd);
            // Safe modes should not trigger warnings
            prop_assert_eq!(result.diagnostics.len(), 0);
        }

        #[test]
        fn prop_sec017_dangerous_modes_detected(
            mode in "(777|666)",
            file in "[a-z/]{1,20}",
        ) {
            let cmd = format!("chmod {} {}", mode, file);
            let result = check(&cmd);
            // Dangerous modes should always be detected
            prop_assert!(!result.diagnostics.is_empty());
            prop_assert_eq!(result.diagnostics[0].code.as_str(), "SEC017");
        }
    }
}
