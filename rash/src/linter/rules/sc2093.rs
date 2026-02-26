// SC2093: Remove "exec " if script should continue after this command
//
// The exec command replaces the current shell with the command.
// Nothing after exec will run. If this is not intended, remove exec.
//
// Examples:
// Bad:
//   exec command
//   echo "done"                  // Never runs
//
// Good:
//   command                      // Just run the command
//   echo "done"                  // This runs
//
// Or if exec is intentional:
//   exec command                 // Replace shell, script ends
//
// Impact: Code after exec never executes

use crate::linter::{Diagnostic, LintResult, Severity, Span};
use regex::Regex;

static EXEC_COMMAND: std::sync::LazyLock<Regex> = std::sync::LazyLock::new(|| {
    // Match: exec command
    Regex::new(r"^\s*exec\s+\w+").unwrap()
});

pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();
    let lines: Vec<&str> = source.lines().collect();

    for (idx, line) in lines.iter().enumerate() {
        let line_num = idx + 1;

        if line.trim_start().starts_with('#') {
            continue;
        }

        // Check if line has exec
        if let Some(mat) = EXEC_COMMAND.find(line) {
            // Check if there are more non-comment lines after this
            let has_code_after = lines[idx + 1..]
                .iter()
                .any(|l| !l.trim().is_empty() && !l.trim_start().starts_with('#'));

            if has_code_after {
                let start_col = mat.start() + 1;
                let end_col = mat.end() + 1;

                let diagnostic = Diagnostic::new(
                    "SC2093",
                    Severity::Warning,
                    "Remove 'exec' if script should continue after this command".to_string(),
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
    fn test_sc2093_exec_with_code_after() {
        let code = r#"
exec command
echo "done"
"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2093_exec_at_end_ok() {
        let code = r#"
echo "starting"
exec command
"#;
        let result = check(code);
        // exec at end is OK
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2093_exec_with_comments_ok() {
        let code = r#"
exec command
# This is a comment
"#;
        let result = check(code);
        // Only comments after, OK
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2093_no_exec_ok() {
        let code = r#"
command
echo "done"
"#;
        let result = check(code);
        // No exec
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2093_exec_redirect() {
        let code = r#"
exec > logfile
echo "logging"
"#;
        let result = check(code);
        // exec with only redirection doesn't replace shell - code after WILL run
        // This is correct usage, should not be flagged
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2093_multiple() {
        let code = r#"
exec cmd1
echo "1"
exec cmd2
echo "2"
"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 2);
    }

    #[test]
    fn test_sc2093_comment_line() {
        let code = r#"# exec command"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2093_intentional_replacement() {
        let code = "exec python script.py";
        let result = check(code);
        // exec at end, intentional replacement
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2093_with_args() {
        let code = r#"
exec command arg1 arg2
cleanup
"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2093_blank_lines_after() {
        let code = r#"
exec command


"#;
        let result = check(code);
        // Only blank lines after
        assert_eq!(result.diagnostics.len(), 0);
    }
}
