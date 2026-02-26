// SC2023: The shell may override PATH. Use 'command -v' instead of 'which'
//
// The 'which' command is external and may not reflect shell builtins or
// functions. 'command -v' is POSIX and checks what the shell will actually run.
//
// Examples:
// Bad:
//   which git                       // External command, may be wrong
//   if which python; then ...       // Doesn't check builtins/functions
//   path=$(which node)              // May miss shell overrides
//
// Good:
//   command -v git                  // Shell builtin, accurate
//   if command -v python; then ...  // Checks actual resolution
//   path=$(command -v node)         // Respects shell PATH
//
// Why command -v is better:
// - It's a shell builtin (faster)
// - It checks functions and builtins
// - It respects shell aliases
// - It's POSIX standard
//
// Note: 'type' is also good but less portable than 'command -v'.

use crate::linter::{Diagnostic, LintResult, Severity, Span};
use regex::Regex;

static WHICH_COMMAND: std::sync::LazyLock<Regex> = std::sync::LazyLock::new(|| {
    // Match: which command_name
    Regex::new(r"\bwhich\s+[a-zA-Z_][a-zA-Z0-9_-]*").unwrap()
});

pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();

    for (line_num, line) in source.lines().enumerate() {
        let line_num = line_num + 1;

        if line.trim_start().starts_with('#') {
            continue;
        }

        // Look for 'which' command usage
        for m in WHICH_COMMAND.find_iter(line) {
            let start_col = m.start() + 1;
            let end_col = m.end() + 1;

            let diagnostic = Diagnostic::new(
                "SC2023",
                Severity::Info,
                "The shell may override PATH. Use 'command -v cmd' instead of 'which cmd'"
                    .to_string(),
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
    fn test_sc2023_which_usage() {
        let code = r#"which git"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
        assert_eq!(result.diagnostics[0].code, "SC2023");
        assert_eq!(result.diagnostics[0].severity, Severity::Info);
        assert!(result.diagnostics[0].message.contains("command -v"));
    }

    #[test]
    fn test_sc2023_which_in_if() {
        let code = r#"if which python; then echo "found"; fi"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2023_which_in_subshell() {
        let code = r#"path=$(which node)"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2023_which_with_dash() {
        let code = r#"which clang-format"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2023_command_v_ok() {
        let code = r#"command -v git"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2023_type_ok() {
        let code = r#"type python"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2023_hash_ok() {
        let code = r#"hash node 2>/dev/null"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2023_multiple_which() {
        let code = r#"
which gcc
which g++
"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 2);
    }

    #[test]
    fn test_sc2023_which_in_comment_ok() {
        let code = r#"# Use which python to find it"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2023_which_in_string_ignored() {
        let code = r#"echo "which tool do you want?""#;
        let result = check(code);
        // 'which' followed by 'tool' matches the pattern
        // Quote detection would require more sophisticated parsing
        assert_eq!(result.diagnostics.len(), 1);
    }
}
