// SC2238: Redirecting to/from command name instead of file
use crate::linter::{Diagnostic, LintResult, Severity, Span};
use once_cell::sync::Lazy;
use regex::Regex;

static REDIRECT_TO_COMMAND: Lazy<Regex> = Lazy::new(|| {
    // Match: > command_name or < command_name (no path separator), but not >>
    Regex::new(r"([^>]>|<)\s*[a-z_][a-z0-9_-]*\s*($|;|\||&)").unwrap()
});

pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();
    for (line_num, line) in source.lines().enumerate() {
        let line_num = line_num + 1;
        if line.trim_start().starts_with('#') {
            continue;
        }

        if REDIRECT_TO_COMMAND.is_match(line) {
            // Skip if it looks like a file (has extension or path)
            if !line.contains('.') && !line.contains('/') {
                let diagnostic = Diagnostic::new(
                    "SC2238",
                    Severity::Warning,
                    "Redirecting to/from bare word may be a command name. Use ./ or quotes if it's a file"
                        .to_string(),
                    Span::new(line_num, 1, line_num, line.len() + 1),
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
    fn test_sc2238_redirect_to_command() {
        let code = "echo test > output";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }
    #[test]
    fn test_sc2238_redirect_to_file_ok() {
        let code = "echo test > output.txt";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }
    #[test]
    fn test_sc2238_redirect_with_path_ok() {
        let code = "echo test > ./output";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }
    #[test]
    fn test_sc2238_redirect_from_command() {
        let code = "cat < input";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }
    #[test]
    fn test_sc2238_comment_skipped() {
        let code = "# echo > output";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }
    #[test]
    fn test_sc2238_redirect_with_slash() {
        let code = "echo test > /tmp/output";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }
    #[test]
    fn test_sc2238_stderr_redirect() {
        let code = "cmd 2> errors";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }
    #[test]
    fn test_sc2238_no_code() {
        let code = "";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }
    #[test]
    fn test_sc2238_normal_command() {
        let code = "echo test";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }
    #[test]
    fn test_sc2238_append_redirect() {
        let code = "echo data >> logfile";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0); // >> not matched by pattern
    }
}
