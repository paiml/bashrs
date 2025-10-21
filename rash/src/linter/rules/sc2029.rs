// SC2029: Note that, unescaped, this expands on the client side
//
// In ssh commands, variables expand on the local (client) side unless quoted.
// This can lead to unexpected behavior when you want remote expansion.
//
// Examples:
// Bad (expands locally):
//   ssh user@host echo $PATH          // $PATH expands before ssh runs
//   ssh server ls $HOME               // $HOME is your local home
//   ssh remote "echo $USER"           // $USER expands locally
//
// Good (expands remotely):
//   ssh user@host 'echo $PATH'        // Single quotes: remote expansion
//   ssh server 'ls $HOME'             // Expands on remote
//   ssh remote 'echo $USER'           // Gets remote user
//   ssh remote "echo \$USER"          // Escaped: remote expansion
//
// Note: Use single quotes or escape $ to ensure variables expand on the
// remote side. Double quotes without escaping expand locally.

use crate::linter::{Diagnostic, LintResult, Severity, Span};
use once_cell::sync::Lazy;
use regex::Regex;

static SSH_WITH_VAR: Lazy<Regex> = Lazy::new(|| {
    // Match: ssh host command $VAR (without single quotes)
    Regex::new(r"ssh\s+[^\s]+\s+[^']*\$[a-zA-Z_][a-zA-Z0-9_]*").unwrap()
});

pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();

    for (line_num, line) in source.lines().enumerate() {
        let line_num = line_num + 1;

        if line.trim_start().starts_with('#') {
            continue;
        }

        // Look for ssh with unquoted variables
        if line.contains("ssh ") && line.contains('$') {
            for m in SSH_WITH_VAR.find_iter(line) {
                let matched = m.as_str();

                // Skip if the variable is in single quotes
                if matched.contains("'$") {
                    continue;
                }

                // Skip if the variable is escaped
                if matched.contains("\\$") {
                    continue;
                }

                // Check if we're in single quotes
                let pos = line.find(matched).unwrap_or(0);
                let before = &line[..pos];
                let single_quote_count = before.matches('\'').count();
                if single_quote_count % 2 == 1 {
                    continue; // Inside single quotes
                }

                let start_col = pos + 1;
                let end_col = start_col + matched.len();

                let diagnostic = Diagnostic::new(
                    "SC2029",
                    Severity::Info,
                    "Note that, unescaped, this expands on the client side. Use single quotes or escape $ for remote expansion".to_string(),
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
    fn test_sc2029_ssh_unquoted_var() {
        let code = r#"ssh user@host echo $PATH"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
        assert_eq!(result.diagnostics[0].code, "SC2029");
        assert_eq!(result.diagnostics[0].severity, Severity::Info);
        assert!(result.diagnostics[0].message.contains("client side"));
    }

    #[test]
    fn test_sc2029_ssh_home() {
        let code = r#"ssh server ls $HOME"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2029_ssh_double_quotes() {
        let code = r#"ssh remote "echo $USER""#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2029_ssh_single_quotes_ok() {
        let code = r#"ssh user@host 'echo $PATH'"#;
        let result = check(code);
        // Single quotes prevent local expansion
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2029_ssh_escaped_ok() {
        let code = r#"ssh remote "echo \$USER""#;
        let result = check(code);
        // Escaped dollar prevents local expansion
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2029_ssh_no_var_ok() {
        let code = r#"ssh host echo hello"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2029_local_command_ok() {
        let code = r#"echo $PATH"#;
        let result = check(code);
        // Not ssh, no issue
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2029_multiple_issues() {
        let code = r#"
ssh host1 echo $VAR1
ssh host2 echo $VAR2
"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 2);
    }

    #[test]
    fn test_sc2029_ssh_with_flags() {
        let code = r#"ssh -t user@host echo $PATH"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2029_scp_ok() {
        let code = r#"scp file user@host:$HOME/"#;
        let result = check(code);
        // scp is different from ssh
        assert_eq!(result.diagnostics.len(), 0);
    }
}
