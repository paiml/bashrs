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

// FIXME(PMAT-238): #[cfg(test)]
// FIXME(PMAT-238): #[path = "sc2024_tests_prop_sc2024.rs"]
// FIXME(PMAT-238): mod tests_extracted;
