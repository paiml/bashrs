// SC2096: Redirections override previously specified redirections
//
// When multiple redirections of the same type are specified for the same file
// descriptor, only the last one takes effect. Earlier redirections are silently
// ignored, which is often unintentional.
//
// Examples:
// Bad:
//   command > file1.txt > file2.txt
//   # Only file2.txt is written, file1.txt is untouched
//
//   command 2> err1.log 2> err2.log
//   # Only err2.log gets stderr
//
// Good:
//   command > file1.txt
//   # Single output
//
//   command > stdout.txt 2> stderr.txt
//   # Different streams to different files

use crate::linter::{Diagnostic, LintResult, Severity, Span};
use once_cell::sync::Lazy;
use regex::Regex;

static MULTIPLE_STDOUT_REDIRECTS: Lazy<Regex> = Lazy::new(|| {
    // Match: > file1 ... > file2  (without 2> in between)
    Regex::new(r">\s*[^\s;&|]+[^2>]*>\s*[^\s;&|]+").unwrap()
});

static MULTIPLE_STDERR_REDIRECTS: Lazy<Regex> = Lazy::new(|| {
    // Match: 2> file1 ... 2> file2
    Regex::new(r"2>\s*[^\s;&|]+.*2>\s*[^\s;&|]+").unwrap()
});

static MULTIPLE_APPEND_REDIRECTS: Lazy<Regex> = Lazy::new(|| {
    // Match: >> file1 ... >> file2
    Regex::new(r">>\s*[^\s;&|]+.*>>\s*[^\s;&|]+").unwrap()
});

pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();

    for (line_num, line) in source.lines().enumerate() {
        let line_num = line_num + 1;

        if line.trim_start().starts_with('#') {
            continue;
        }

        // Skip heredocs and here strings
        if line.contains("<<") || line.contains("<<<") {
            continue;
        }

        // Check for multiple stdout redirects (skip if it has >>)
        if MULTIPLE_STDOUT_REDIRECTS.is_match(line) && !line.contains(">>") {
            // Make sure it's not mixing stdout and stderr
            let parts: Vec<&str> = line.split('>').collect();
            let mut stdout_count = 0;
            for (i, part) in parts.iter().enumerate() {
                if i > 0 && !parts[i - 1].ends_with('2') && !parts[i - 1].ends_with('&') {
                    stdout_count += 1;
                }
            }

            if stdout_count > 1 {
                let diagnostic = Diagnostic::new(
                    "SC2096",
                    Severity::Warning,
                    "Multiple stdout redirections specified. Only the last one will be used, earlier ones are ignored".to_string(),
                    Span::new(line_num, 1, line_num, line.len()),
                );

                result.add(diagnostic);
            }
        }

        // Check for multiple stderr redirects
        if MULTIPLE_STDERR_REDIRECTS.is_match(line) {
            let diagnostic = Diagnostic::new(
                "SC2096",
                Severity::Warning,
                "Multiple stderr redirections specified. Only the last one will be used, earlier ones are ignored".to_string(),
                Span::new(line_num, 1, line_num, line.len()),
            );

            result.add(diagnostic);
        }

        // Check for multiple append redirects
        if MULTIPLE_APPEND_REDIRECTS.is_match(line) {
            let diagnostic = Diagnostic::new(
                "SC2096",
                Severity::Warning,
                "Multiple append redirections specified. Only the last one will be used, earlier ones are ignored".to_string(),
                Span::new(line_num, 1, line_num, line.len()),
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
    fn test_sc2096_multiple_stdout() {
        let code = r#"command > file1.txt > file2.txt"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
        assert_eq!(result.diagnostics[0].code, "SC2096");
        assert_eq!(result.diagnostics[0].severity, Severity::Warning);
        assert!(result.diagnostics[0].message.contains("stdout"));
    }

    #[test]
    fn test_sc2096_multiple_stderr() {
        let code = r#"command 2> err1.log 2> err2.log"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
        assert!(result.diagnostics[0].message.contains("stderr"));
    }

    #[test]
    fn test_sc2096_multiple_append() {
        let code = r#"echo "a" >> file1.txt >> file2.txt"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
        assert!(result.diagnostics[0].message.contains("append"));
    }

    #[test]
    fn test_sc2096_stdout_and_stderr_ok() {
        let code = r#"command > stdout.txt 2> stderr.txt"#;
        let result = check(code);
        // Different streams, this is OK
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2096_single_redirect_ok() {
        let code = r#"command > output.txt"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2096_pipe_ok() {
        let code = r#"command | grep pattern > output.txt"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2096_both_redirect_ok() {
        let code = r#"command &> all.log"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2096_heredoc_ok() {
        let code = r#"cat <<EOF > output.txt"#;
        let result = check(code);
        // Heredoc is not a duplicate redirect
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2096_three_redirects() {
        let code = r#"echo test > a.txt > b.txt > c.txt"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2096_chained_commands_ok() {
        let code = r#"cmd1 > file1.txt && cmd2 > file2.txt"#;
        let result = check(code);
        // Different commands, not duplicate redirects
        assert_eq!(result.diagnostics.len(), 0);
    }
}
