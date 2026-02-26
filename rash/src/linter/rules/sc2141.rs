// SC2141: This command redirects stdin, but then ignores it. Use < /dev/null to prevent this.
//
// Some commands don't read stdin but their subcommands might. Redirecting stdin
// to a pipeline that doesn't use it wastes resources and can cause confusion.
//
// Examples:
// Bad:
//   cat file | grep pattern | head -1      // head reads stdin, but cat is redundant
//   echo "data" | find . -name "*.txt"     // find ignores stdin
//   < input.txt sudo command                // sudo may not pass stdin
//
// Good:
//   grep pattern file | head -1            // Direct input to grep
//   find . -name "*.txt"                   // No stdin needed
//   sudo command < input.txt               // Or explicitly redirect
//
// Impact: Performance issue, confusing code

use crate::linter::{Diagnostic, LintResult, Severity, Span};
use regex::Regex;

static IGNORED_STDIN: std::sync::LazyLock<Regex> = std::sync::LazyLock::new(|| {
    // Match: commands that typically ignore stdin when piped to
    Regex::new(r"\|\s*(find|xargs\s+-|sudo|ls|echo|printf)\b").unwrap()
});

pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();

    for (line_num, line) in source.lines().enumerate() {
        let line_num = line_num + 1;

        if line.trim_start().starts_with('#') {
            continue;
        }

        // Check for pipes to commands that ignore stdin
        for mat in IGNORED_STDIN.find_iter(line) {
            let matched = mat.as_str();

            // Skip xargs with proper stdin usage (without -)
            if matched.contains("xargs") && !matched.contains("xargs -") {
                continue;
            }

            let start_col = mat.start() + 1;
            let end_col = mat.end() + 1;

            let command = matched.trim_start_matches('|').trim();

            let diagnostic = Diagnostic::new(
                "SC2141",
                Severity::Info,
                format!(
                    "'{}' doesn't read stdin. Consider restructuring the command",
                    command
                ),
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
    fn test_sc2141_pipe_to_find() {
        let code = "echo data | find . -name '*.txt'";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
        assert!(result.diagnostics[0].message.contains("find"));
    }

    #[test]
    fn test_sc2141_find_without_pipe_ok() {
        let code = "find . -name '*.txt'";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2141_pipe_to_echo() {
        let code = "cat file | echo hello";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2141_valid_pipe_ok() {
        let code = "cat file | grep pattern";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2141_comment_ok() {
        let code = "# echo data | find . -name '*.txt'";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2141_pipe_to_ls() {
        let code = "cat file | ls -la";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2141_pipe_to_printf() {
        let code = "cat file | printf '%s\\n' 'test'";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2141_xargs_ok() {
        let code = "find . -name '*.txt' | xargs grep pattern";
        let result = check(code);
        // xargs reads stdin, so this is OK
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2141_multiple() {
        let code = r#"
echo data | find . -name '*.txt'
cat file | ls -la
"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 2);
    }

    #[test]
    fn test_sc2141_pipe_to_sudo() {
        let code = "echo pass | sudo command";
        let result = check(code);
        // sudo may not pass stdin properly
        assert_eq!(result.diagnostics.len(), 1);
    }
}
