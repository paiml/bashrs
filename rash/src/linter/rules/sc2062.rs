// SC2062: Quote the grep pattern so the shell won't interpret it

use crate::linter::{Diagnostic, LintResult, Severity, Span};
use once_cell::sync::Lazy;
use regex::Regex;

static GREP_UNQUOTED: Lazy<Regex> = Lazy::new(|| {
    // Match grep followed by optional flags, then a pattern with glob chars
    // (?:\s+-\S+)* handles flags like -r, -i, --recursive, etc.
    Regex::new(r"\bgrep(?:\s+-\S+)*\s+\S*[\*\?\[]").unwrap()
});

/// Issue #125: Check if grep is inside an SSH remote command string
/// SSH commands like `ssh user@host "grep pattern file"` have their patterns
/// protected from local shell expansion by the outer quotes
fn is_ssh_remote_command(line: &str) -> bool {
    // Pattern: ssh followed by eventually a quoted command containing grep
    // The grep is inside double quotes as an argument to ssh
    let trimmed = line.trim();

    // Look for ssh command with grep inside quoted argument
    if let Some(ssh_pos) = trimmed.find("ssh ") {
        // Check if there's a quoted string after ssh that contains grep
        let after_ssh = &trimmed[ssh_pos..];
        // If grep appears after a quote character following ssh, it's a remote command
        if let Some(quote_pos) = after_ssh.find('"') {
            if let Some(grep_pos) = after_ssh.find("grep") {
                // grep is inside the quoted SSH command string
                if grep_pos > quote_pos {
                    return true;
                }
            }
        }
    }
    false
}

pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();

    for (line_num, line) in source.lines().enumerate() {
        let line_num = line_num + 1;

        if line.trim_start().starts_with('#') {
            continue;
        }

        // Simple check: if line has quoted grep, skip
        if line.contains("grep '") || line.contains("grep \"") {
            continue;
        }

        // Issue #125: Skip SSH remote commands - patterns are evaluated remotely
        if is_ssh_remote_command(line) {
            continue;
        }

        if let Some(mat) = GREP_UNQUOTED.find(line) {
            let start_col = mat.start() + 1;
            let end_col = mat.end() + 1;

            let diagnostic = Diagnostic::new(
                "SC2062",
                Severity::Warning,
                "Quote the grep pattern so the shell won't interpret it".to_string(),
                Span::new(line_num, start_col, line_num, end_col),
            );

            result.add(diagnostic);
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sc2062_unquoted_glob() {
        let code = r#"grep *.txt file"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2062_bracket_expression() {
        let code = r#"grep [0-9]+ data"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2062_question_mark() {
        let code = r#"grep file?.txt data"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2062_quoted_single_ok() {
        let code = r#"grep '*.txt' file"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2062_quoted_double_ok() {
        let code = r#"grep "*.txt" file"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2062_no_special_chars_ok() {
        let code = r#"grep pattern file"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2062_comment_ok() {
        let code = r#"# grep *.txt file"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2062_with_flags() {
        let code = r#"grep -r *.log ."#;
        let result = check(code);
        // Regex now handles optional flags before pattern
        assert_eq!(result.diagnostics.len(), 1); // Has glob, should warn
    }

    #[test]
    fn test_sc2062_pipe() {
        let code = r#"cat file | grep [ERROR]"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2062_variable_ok() {
        let code = r#"grep "$pattern" file"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    // ===== Issue #125: SSH remote command strings =====
    // Patterns inside SSH command strings are evaluated remotely, not locally

    #[test]
    fn test_issue_125_ssh_remote_grep_not_flagged() {
        // SSH command - grep is inside the quoted remote command string
        let code = r#"ssh user@host "grep -E '^(Mem|Swap)' /proc/meminfo""#;
        let result = check(code);
        assert_eq!(
            result.diagnostics.len(),
            0,
            "SC2062 must NOT flag grep inside SSH remote command string (Issue #125)"
        );
    }

    #[test]
    fn test_issue_125_ssh_with_pipe_not_flagged() {
        // SSH command with pipe - still inside the quoted remote command
        let code = r#"ssh user@host "df -h | grep -E '^/dev/(nvme|mmcblk)'""#;
        let result = check(code);
        assert_eq!(
            result.diagnostics.len(),
            0,
            "SC2062 must NOT flag grep with pipe inside SSH remote command (Issue #125)"
        );
    }

    #[test]
    fn test_issue_125_ssh_simple_pattern() {
        // Simple SSH grep command
        let code = r#"ssh server "grep [0-9]+ file""#;
        let result = check(code);
        assert_eq!(
            result.diagnostics.len(),
            0,
            "SC2062 must NOT flag grep pattern inside SSH command"
        );
    }

    #[test]
    fn test_issue_125_local_grep_still_flagged() {
        // Local grep (not inside SSH) should still be flagged
        let code = r#"grep -E '^[0-9]+' file"#;
        let result = check(code);
        // Note: This particular pattern uses [0-9] which is a regex bracket, flagged by GREP_UNQUOTED
        assert_eq!(
            result.diagnostics.len(),
            1,
            "Local grep with glob-like patterns SHOULD still be flagged"
        );
    }
}
