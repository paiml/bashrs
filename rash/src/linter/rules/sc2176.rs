// SC2176: 'time' is undefined for pipelines. Time individual commands.
//
// The 'time' keyword behavior with pipelines varies across shells.
//
// Examples:
// Bad:
//   time cmd1 | cmd2             // Undefined which is timed
//
// Good:
//   time { cmd1 | cmd2; }        // Time the whole pipeline
//   time cmd1                     // Time single command
//
// Impact: Unclear what is being measured

use crate::linter::{Diagnostic, LintResult, Severity, Span};
use regex::Regex;

static TIME_WITH_PIPE: std::sync::LazyLock<Regex> = std::sync::LazyLock::new(|| {
    // Match: time cmd1 | cmd2
    // Exclude: time { cmd1 | cmd2 } (has braces)
    // Exclude: time ( cmd1 | cmd2 ) (has subshell)
    Regex::new(r"\btime\s+[^;{(]+\|").unwrap()
});

pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();

    for (line_num, line) in source.lines().enumerate() {
        let line_num = line_num + 1;

        if line.trim_start().starts_with('#') {
            continue;
        }

        for mat in TIME_WITH_PIPE.find_iter(line) {
            let start_col = mat.start() + 1;
            let end_col = mat.end() + 1;

            let diagnostic = Diagnostic::new(
                "SC2176",
                Severity::Warning,
                "'time' with pipelines is undefined. Use 'time { cmd1 | cmd2; }'".to_string(),
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
    fn test_sc2176_time_pipe() {
        let code = "time cmd1 | cmd2";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2176_time_single_ok() {
        let code = "time cmd1";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2176_time_group_ok() {
        let code = "time { cmd1 | cmd2; }";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2176_comment_ok() {
        let code = "# time cmd1 | cmd2";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2176_multiple() {
        let code = "time cmd1 | cmd2\ntime cmd3 | cmd4";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 2);
    }

    #[test]
    fn test_sc2176_time_subshell_ok() {
        let code = "time ( cmd1 | cmd2 )";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2176_time_complex_pipe() {
        let code = "time grep foo bar | sort | uniq";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2176_no_time_ok() {
        let code = "cmd1 | cmd2";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2176_time_and_ok() {
        let code = "time cmd1 && cmd2";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2176_time_semicolon_ok() {
        let code = "time cmd1; cmd2 | cmd3";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }
}
