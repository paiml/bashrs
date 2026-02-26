// SC2024: sudo doesn't affect redirects. Use '| sudo tee' instead
//
// Redirections (>, >>) are processed by the current shell before sudo runs.
// The file is opened with your permissions, not root's.
//
// Examples:
// Bad:
//   sudo echo "text" > /root/file   // Redirect runs as you, not root
//   sudo cat data >> /etc/hosts     // Append runs as you, fails
//   sudo cmd > /var/log/app.log     // Log write uses your permissions
//
// Good:
//   echo "text" | sudo tee /root/file >/dev/null    // tee runs as root
//   cat data | sudo tee -a /etc/hosts >/dev/null    // -a for append
//   sudo sh -c 'cmd > /var/log/app.log'             // sh -c works too
//
// Note: The shell processes redirects before executing sudo, so the file
// operations happen with your UID, not root's.

use crate::linter::{Diagnostic, LintResult, Severity, Span};
use regex::Regex;

#[allow(clippy::expect_used)] // Compile-time regex, panic on invalid pattern is acceptable
static SUDO_WITH_REDIRECT: std::sync::LazyLock<Regex> = std::sync::LazyLock::new(|| {
    // Match: sudo command > file or sudo command >> file
    Regex::new(r"\bsudo\s+[^>|]+\s*(>>?)\s*[^\s]+").expect("valid regex for sudo redirect")
});

/// Issue #101: Detect sudo sh -c or sudo bash -c patterns
/// These patterns wrap the redirect inside the shell, so sudo DOES affect it
#[allow(clippy::expect_used)] // Compile-time regex, panic on invalid pattern is acceptable
static SUDO_SHELL_WRAPPER: std::sync::LazyLock<Regex> = std::sync::LazyLock::new(|| {
    // Match: sudo [flags] sh/bash/dash/ash/zsh -c 'content' or "content"
    // Uses ['"] to match either single or double quote
    Regex::new(r#"\bsudo\s+(?:-\S+\s+)*(?:sh|bash|dash|ash|zsh)\s+-c\s+['"]"#)
        .expect("valid regex for sudo shell wrapper")
});

/// Issue #101: Check if this is a sudo sh -c or sudo bash -c pattern
fn is_sudo_shell_wrapper(line: &str) -> bool {
    SUDO_SHELL_WRAPPER.is_match(line)
}

/// Issue #100: Check if this is a piped sudo tee pattern
/// cmd | sudo tee [file] is the correct way to write to root-owned files
fn is_piped_sudo_tee(line: &str) -> bool {
    // Look for pipe followed by sudo tee
    line.contains("| sudo tee") || line.contains("|sudo tee")
}

/// F004: Check if redirect target is a user-writable location
/// User-writable locations like /tmp, /var/tmp, /dev/null don't need sudo for writing
fn is_redirect_to_user_writable(line: &str) -> bool {
    // Common user-writable paths where sudo redirect warning is not useful
    const USER_WRITABLE_PATHS: &[&str] = &[
        "/tmp/",
        "/tmp",
        "/var/tmp/",
        "/var/tmp",
        "/dev/null",
        "/dev/zero",
        "/dev/stdout",
        "/dev/stderr",
    ];

    // Extract the redirect target from the line
    // Look for > or >> followed by the path
    if let Some(redirect_pos) = line.find('>') {
        let after_redirect = &line[redirect_pos..];
        // Skip the > or >> and any whitespace
        let target = after_redirect.trim_start_matches('>').trim_start();

        // Check if target starts with any user-writable path
        for path in USER_WRITABLE_PATHS {
            if target.starts_with(path) {
                return true;
            }
        }
    }

    false
}

/// Check if a line is a comment
fn is_comment_line(line: &str) -> bool {
    line.trim_start().starts_with('#')
}

/// Check if line has sudo with output redirect
fn has_sudo_with_redirect(line: &str) -> bool {
    line.contains("sudo") && (line.contains('>') && !line.contains("2>") && !line.contains("&>"))
}

/// Check if match is inside quotes
fn is_inside_quotes(line: &str, match_pos: usize) -> bool {
    let before = &line[..match_pos];
    let quote_count = before.matches('"').count() + before.matches('\'').count();
    quote_count % 2 == 1
}

/// Get tee flag for redirect operator
fn get_tee_flag(redirect_op: &str) -> &str {
    if redirect_op == ">>" {
        "-a "
    } else {
        ""
    }
}

/// Create diagnostic for sudo with redirect
fn create_sudo_redirect_diagnostic(
    line_num: usize,
    start_col: usize,
    end_col: usize,
    redirect_op: &str,
) -> Diagnostic {
    let tee_flag = get_tee_flag(redirect_op);
    Diagnostic::new(
        "SC2024",
        Severity::Warning,
        format!(
            "sudo doesn't affect redirects. Use '| sudo tee {}file' instead of 'sudo cmd {} file'",
            tee_flag, redirect_op
        ),
        Span::new(line_num, start_col, line_num, end_col),
    )
}

#[allow(clippy::expect_used)] // Capture groups 0 and 1 always exist when regex matches
pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();

    for (line_num, line) in source.lines().enumerate() {
        let line_num = line_num + 1;

        if is_comment_line(line) {
            continue;
        }

        // Issue #101: Skip if sudo sh -c or sudo bash -c pattern
        // The redirect is inside the shell wrapper, so sudo DOES affect it
        if is_sudo_shell_wrapper(line) {
            continue;
        }

        // Issue #100: Skip if piped sudo tee pattern
        // cmd | sudo tee file is the correct way to write to root-owned files
        if is_piped_sudo_tee(line) {
            continue;
        }

        // F004: Skip if redirect target is user-writable (e.g., /tmp, /dev/null)
        // These locations don't need sudo for writing, so the warning is not useful
        if is_redirect_to_user_writable(line) {
            continue;
        }

        // Look for sudo with output redirection
        if has_sudo_with_redirect(line) {
            for cap in SUDO_WITH_REDIRECT.captures_iter(line) {
                let redirect_op = cap
                    .get(1)
                    .expect("capture group 1 exists for redirect operator")
                    .as_str();

                // Skip if inside quotes
                let full_match = cap
                    .get(0)
                    .expect("capture group 0 exists for full match")
                    .as_str();
                let pos = line.find(full_match).unwrap_or(0);

                if is_inside_quotes(line, pos) {
                    continue;
                }

                let start_col = pos + 1;
                let end_col = start_col + full_match.len();

                let diagnostic =
                    create_sudo_redirect_diagnostic(line_num, start_col, end_col, redirect_op);

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
    fn prop_sc2024_comments_never_diagnosed() {
        // Property: Comment lines should never produce diagnostics
        let test_cases = vec![
            "# sudo echo \"text\" > /root/file",
            "  # sudo cmd >> /var/log/app.log",
            "\t# sudo cat data >> /etc/hosts",
        ];

        for code in test_cases {
            let result = check(code);
            assert_eq!(result.diagnostics.len(), 0);
        }
    }

    #[test]
    fn prop_sc2024_sudo_with_tee_never_diagnosed() {
        // Property: sudo with tee (correct usage) never diagnosed
        let test_cases = vec![
            "echo \"text\" | sudo tee /root/file",
            "cat data | sudo tee -a /etc/hosts",
        ];

        for code in test_cases {
            let result = check(code);
            assert_eq!(result.diagnostics.len(), 0);
        }
    }

    #[test]
    fn prop_sc2024_sudo_tee_with_devnull_redirect() {
        // Property: sudo tee with >/dev/null should NOT be flagged
        // Issue #100 FIX: Piped sudo tee is correct usage
        let code = "cmd | sudo tee /var/log/app.log >/dev/null";
        let result = check(code);
        // Issue #100: No longer produces false positive
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn prop_sc2024_sudo_with_sh_c_never_diagnosed() {
        // Property: sudo sh -c (correct usage) never diagnosed
        // Issue #101 FIX: Redirect inside sh -c is handled by sudo
        let code = "sudo sh -c 'cmd > /var/log/app.log'";
        let result = check(code);
        // Issue #101: No longer produces false positive
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn prop_sc2024_stderr_redirect_never_diagnosed() {
        // Property: stderr redirects (2>, &>) never diagnosed
        let test_cases = vec![
            "sudo cmd 2> /var/log/error.log",
            "sudo command 2>> /var/log/error.log",
            "sudo app &> /dev/null",
        ];

        for code in test_cases {
            let result = check(code);
            assert_eq!(result.diagnostics.len(), 0);
        }
    }

    #[test]
    fn prop_sc2024_no_sudo_never_diagnosed() {
        // Property: Redirects without sudo never diagnosed
        let test_cases = vec![
            "echo \"text\" > file.txt",
            "cat data >> output.log",
            "command > /tmp/file.txt",
        ];

        for code in test_cases {
            let result = check(code);
            assert_eq!(result.diagnostics.len(), 0);
        }
    }

    #[test]
    fn prop_sc2024_sudo_with_stdout_redirect_always_diagnosed() {
        // Property: sudo with stdout redirect always diagnosed
        let test_cases = vec![
            "sudo echo \"text\" > /root/file",
            "sudo cat data >> /etc/hosts",
            "sudo cmd > /var/log/app.log",
        ];

        for code in test_cases {
            let result = check(code);
            assert_eq!(result.diagnostics.len(), 1, "Should diagnose: {}", code);
            assert!(result.diagnostics[0].message.contains("tee"));
        }
    }

    #[test]
    fn prop_sc2024_multiple_violations_all_diagnosed() {
        // Property: Multiple sudo redirects should all be diagnosed
        let code = "sudo echo \"a\" > /root/a\nsudo echo \"b\" > /root/b";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 2);
    }

    #[test]
    fn prop_sc2024_diagnostic_code_always_sc2024() {
        // Property: All diagnostics must have code "SC2024"
        let code = "sudo echo \"a\" > /root/a\nsudo echo \"b\" > /root/b";
        let result = check(code);

        for diagnostic in &result.diagnostics {
            assert_eq!(&diagnostic.code, "SC2024");
        }
    }

    #[test]
    fn prop_sc2024_diagnostic_severity_always_warning() {
        // Property: All diagnostics must be Warning severity
        let code = "sudo echo \"text\" > /root/file";
        let result = check(code);

        for diagnostic in &result.diagnostics {
            assert_eq!(diagnostic.severity, Severity::Warning);
        }
    }

    #[test]
    fn prop_sc2024_empty_source_no_diagnostics() {
        // Property: Empty source should produce no diagnostics
        let result = check("");
        assert_eq!(result.diagnostics.len(), 0);
    }

    // ===== Original Unit Tests =====

    #[test]
    fn test_sc2024_sudo_redirect() {
        let code = r#"sudo echo "text" > /root/file"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
        assert_eq!(result.diagnostics[0].code, "SC2024");
        assert_eq!(result.diagnostics[0].severity, Severity::Warning);
        assert!(result.diagnostics[0].message.contains("tee"));
    }

    #[test]
    fn test_sc2024_sudo_append() {
        let code = r#"sudo cat data >> /etc/hosts"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
        assert!(result.diagnostics[0].message.contains("-a"));
    }

    #[test]
    fn test_sc2024_sudo_log() {
        let code = r#"sudo cmd > /var/log/app.log"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2024_sudo_tee_ok() {
        let code = r#"echo "text" | sudo tee /root/file"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2024_sudo_sh_c_ok() {
        let code = r#"sudo sh -c 'cmd > /var/log/app.log'"#;
        let result = check(code);
        // Issue #101 FIX: sudo sh -c wraps the redirect, so sudo DOES affect it
        // This is now correctly recognized as valid usage
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2024_stderr_redirect_ok() {
        let code = r#"sudo cmd 2> /var/log/error.log"#;
        let result = check(code);
        // stderr redirect (2>) is different, not caught by this rule
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2024_pipe_ok() {
        let code = r#"sudo cmd | grep pattern"#;
        let result = check(code);
        // Pipe is not redirect
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2024_no_sudo_ok() {
        let code = r#"echo "text" > file"#;
        let result = check(code);
        // No sudo, no problem
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2024_multiple_issues() {
        let code = r#"
sudo echo "a" > /root/a
sudo echo "b" > /root/b
"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 2);
    }

    #[test]
    fn test_sc2024_sudo_with_input_redirect_ok() {
        let code = r#"sudo cmd < /etc/config"#;
        let result = check(code);
        // Input redirect is OK
        assert_eq!(result.diagnostics.len(), 0);
    }

    // ===== Issue #101: sudo sh -c Tests =====
    // Redirect inside sh -c is correct - sudo DOES affect it

    #[test]
    fn test_FP_101_sudo_sh_c_redirect_not_flagged() {
        let code = r#"sudo sh -c 'echo 10 > /proc/sys/vm/swappiness'"#;
        let result = check(code);
        assert_eq!(
            result.diagnostics.len(),
            0,
            "SC2024 must NOT flag sudo sh -c 'cmd > file' - redirect is inside sh -c"
        );
    }

    #[test]
    fn test_FP_101_sudo_bash_c_redirect_not_flagged() {
        let code = r#"sudo bash -c 'echo test > /etc/file'"#;
        let result = check(code);
        assert_eq!(
            result.diagnostics.len(),
            0,
            "SC2024 must NOT flag sudo bash -c pattern"
        );
    }

    #[test]
    fn test_FP_101_sudo_sh_c_append_not_flagged() {
        let code = r#"sudo sh -c 'echo line >> /etc/file'"#;
        let result = check(code);
        assert_eq!(
            result.diagnostics.len(),
            0,
            "SC2024 must NOT flag sudo sh -c with append redirect"
        );
    }

    #[test]
    fn test_FP_101_sudo_sh_c_double_quoted_not_flagged() {
        let code = r#"sudo sh -c "echo test > /etc/file""#;
        let result = check(code);
        assert_eq!(
            result.diagnostics.len(),
            0,
            "SC2024 must NOT flag sudo sh -c with double quotes"
        );
    }

    #[test]
    fn test_FP_101_sudo_dash_c_not_flagged() {
        let code = r#"sudo dash -c 'echo test > /etc/file'"#;
        let result = check(code);
        assert_eq!(
            result.diagnostics.len(),
            0,
            "SC2024 must NOT flag sudo dash -c pattern"
        );
    }

    #[test]
    fn test_FP_101_direct_sudo_redirect_still_flagged() {
        let code = r#"sudo echo test > /etc/file"#;
        let result = check(code);
        assert_eq!(
            result.diagnostics.len(),
            1,
            "Direct sudo redirect should still be flagged"
        );
    }

    // ===== Issue #100: sudo tee Tests =====
    // cmd | sudo tee is the correct pattern

    #[test]
    fn test_FP_100_sudo_tee_devnull_not_flagged() {
        let code = r#"echo test | sudo tee /etc/file >/dev/null"#;
        let result = check(code);
        assert_eq!(
            result.diagnostics.len(),
            0,
            "SC2024 must NOT flag 'cmd | sudo tee file >/dev/null'"
        );
    }

    #[test]
    fn test_FP_100_sudo_tee_append_devnull_not_flagged() {
        let code = r#"printf '%s\n' "$VAR" | sudo tee -a /etc/fstab >/dev/null"#;
        let result = check(code);
        assert_eq!(
            result.diagnostics.len(),
            0,
            "SC2024 must NOT flag sudo tee -a pattern"
        );
    }

    #[test]
    fn test_FP_100_sudo_tee_no_devnull_not_flagged() {
        let code = r#"echo test | sudo tee /etc/file"#;
        let result = check(code);
        assert_eq!(
            result.diagnostics.len(),
            0,
            "SC2024 must NOT flag sudo tee without >/dev/null"
        );
    }

    #[test]
    fn test_FP_100_printf_sudo_tee_not_flagged() {
        let code = r#"printf '%s\n' "vm.swappiness=10" | sudo tee -a /etc/sysctl.conf >/dev/null"#;
        let result = check(code);
        assert_eq!(
            result.diagnostics.len(),
            0,
            "SC2024 must NOT flag printf | sudo tee pattern"
        );
    }

    // ===== F004: sudo -u with user-writable target =====
    // When redirecting to a user-writable location like /tmp, the warning is less relevant

    #[test]
    fn test_FP_004_sudo_u_tmp_not_flagged() {
        // sudo -u user redirecting to /tmp - user already has write access
        let code = r#"sudo -u user cmd > /tmp/output.txt"#;
        let result = check(code);
        assert_eq!(
            result.diagnostics.len(),
            0,
            "SC2024 must NOT flag sudo redirect to /tmp (user-writable)"
        );
    }

    #[test]
    fn test_FP_004_sudo_redirect_to_tmp_not_flagged() {
        // Any sudo redirect to /tmp should not be flagged
        let code = r#"sudo echo test > /tmp/file"#;
        let result = check(code);
        assert_eq!(
            result.diagnostics.len(),
            0,
            "SC2024 must NOT flag sudo redirect to /tmp"
        );
    }

    #[test]
    fn test_FP_004_sudo_redirect_to_var_tmp_not_flagged() {
        // sudo redirect to /var/tmp should not be flagged
        let code = r#"sudo cmd > /var/tmp/output"#;
        let result = check(code);
        assert_eq!(
            result.diagnostics.len(),
            0,
            "SC2024 must NOT flag sudo redirect to /var/tmp"
        );
    }

    #[test]
    fn test_FP_004_sudo_redirect_to_devnull_not_flagged() {
        // sudo redirect to /dev/null should not be flagged
        let code = r#"sudo cmd > /dev/null"#;
        let result = check(code);
        assert_eq!(
            result.diagnostics.len(),
            0,
            "SC2024 must NOT flag sudo redirect to /dev/null"
        );
    }

    #[test]
    fn test_FP_004_sudo_redirect_to_root_still_flagged() {
        // sudo redirect to system directories should still be flagged
        let code = r#"sudo echo test > /root/file"#;
        let result = check(code);
        assert_eq!(
            result.diagnostics.len(),
            1,
            "SC2024 SHOULD flag sudo redirect to /root"
        );
    }
}
