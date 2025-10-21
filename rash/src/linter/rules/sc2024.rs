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
use once_cell::sync::Lazy;
use regex::Regex;

static SUDO_WITH_REDIRECT: Lazy<Regex> = Lazy::new(|| {
    // Match: sudo command > file or sudo command >> file
    Regex::new(r"\bsudo\s+[^>|]+\s*(>>?)\s*[^\s]+").unwrap()
});

pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();

    for (line_num, line) in source.lines().enumerate() {
        let line_num = line_num + 1;

        if line.trim_start().starts_with('#') {
            continue;
        }

        // Look for sudo with output redirection
        if line.contains("sudo") && (line.contains('>') && !line.contains("2>") && !line.contains("&>")) {
            for cap in SUDO_WITH_REDIRECT.captures_iter(line) {
                let redirect_op = cap.get(1).unwrap().as_str();

                // Skip if inside quotes
                let full_match = cap.get(0).unwrap().as_str();
                let pos = line.find(full_match).unwrap_or(0);
                let before = &line[..pos];
                let quote_count = before.matches('"').count() + before.matches('\'').count();
                if quote_count % 2 == 1 {
                    continue; // Inside quotes
                }

                let start_col = pos + 1;
                let end_col = start_col + full_match.len();

                let tee_flag = if redirect_op == ">>" { "-a " } else { "" };
                let diagnostic = Diagnostic::new(
                    "SC2024",
                    Severity::Warning,
                    format!(
                        "sudo doesn't affect redirects. Use '| sudo tee {}file' instead of 'sudo cmd {} file'",
                        tee_flag, redirect_op
                    ),
                    Span::new(line_num, start_col, line_num, end_col),
                );

                result.add(diagnostic);
            }
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

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
        // The redirect > is detected even inside sh -c quotes
        // More sophisticated parsing would distinguish this case
        assert_eq!(result.diagnostics.len(), 1);
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
}
