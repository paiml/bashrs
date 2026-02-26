//! SC2008: Echo doesn't read from stdin
//!
//! This rule detects when echo is used in a pipeline to read from stdin,
//! which doesn't work because echo ignores stdin and only outputs its arguments.
//!
//! # Examples
//!
//! Bad:
//! ```bash
//! command | echo
//! cat file.txt | echo
//! grep pattern file | echo "result"
//! ```
//!
//! Good:
//! ```bash
//! command | cat
//! command | xargs echo
//! echo "direct output"
//! ```
//!
//! # Rationale
//!
//! The `echo` command does not read from standard input - it only outputs
//! its command-line arguments. Using `echo` at the end of a pipeline will
//! discard all input from the pipe and produce no output (or only output
//! its own arguments).
//!
//! # Fix
//!
//! - Use `cat` to output stdin: `command | cat`
//! - Use `xargs echo` to convert stdin lines to arguments: `command | xargs echo`
//! - If echo has arguments, those will be output but stdin is still ignored
//!
//! # Compatibility
//!
//! Universal - This applies to all shells (POSIX, bash, zsh, etc.)

use crate::linter::diagnostic::FixSafetyLevel;
use crate::linter::{Diagnostic, Fix, LintResult, Severity, Span};
use regex::Regex;

pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();

    // Pattern: detect | echo (optionally with arguments)
    // This matches pipes ending with echo, which doesn't read stdin
    let pattern = Regex::new(r"\|\s*echo(\s|$)").unwrap();

    for (line_num, line) in source.lines().enumerate() {
        let trimmed = line.trim();

        // Skip comments
        if trimmed.starts_with('#') {
            continue;
        }

        // Check for | echo pattern
        if let Some(mat) = pattern.find(line) {
            let start = mat.start();
            let end = mat.end();

            result.diagnostics.push(Diagnostic {
                code: "SC2008".to_string(),
                severity: Severity::Warning,
                message: "echo doesn't read from stdin. Use 'cat' to output stdin, or 'xargs echo' to convert stdin to arguments.".to_string(),
                span: Span {
                    start_line: line_num + 1,  // 1-indexed
                    end_line: line_num + 1,    // 1-indexed
                    start_col: start + 1,      // 1-indexed
                    end_col: end + 1,          // 1-indexed
                },
                fix: Some(Fix {
                    replacement: "| cat".to_string(),
                    safety_level: FixSafetyLevel::Safe,
                    assumptions: vec![],
                    suggested_alternatives: vec!["| xargs echo".to_string()],
                }),
            });
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sc2008_basic_pipe_to_echo() {
        let source = "command | echo";
        let result = check(source);
        assert_eq!(result.diagnostics.len(), 1);
        assert_eq!(result.diagnostics[0].code, "SC2008");
        assert!(result.diagnostics[0].message.contains("echo"));
        assert!(result.diagnostics[0].message.contains("stdin"));
    }

    #[test]
    fn test_sc2008_pipe_with_echo_arguments() {
        let source = r#"grep pattern file | echo "result""#;
        let result = check(source);
        assert_eq!(result.diagnostics.len(), 1);
        assert_eq!(result.diagnostics[0].code, "SC2008");
    }

    #[test]
    fn test_sc2008_multiple_pipes_ending_in_echo() {
        let source = "cat file.txt | grep foo | echo";
        let result = check(source);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2008_echo_without_pipe() {
        // Should NOT trigger - echo used normally
        let source = r#"echo "Hello World""#;
        let result = check(source);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2008_pipe_to_cat() {
        // Should NOT trigger - cat reads stdin correctly
        let source = "command | cat";
        let result = check(source);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2008_pipe_to_xargs_echo() {
        // Should NOT trigger - xargs echo is correct pattern
        let source = "command | xargs echo";
        let result = check(source);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2008_multiline_pipeline() {
        let source = r#"cat file.txt \
  | grep pattern \
  | echo"#;
        let result = check(source);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2008_echo_in_comment() {
        // Should NOT trigger - it's in a comment
        let source = "# This is wrong: command | echo";
        let result = check(source);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2008_auto_fix_suggests_cat() {
        let source = "command | echo";
        let result = check(source);
        assert_eq!(result.diagnostics.len(), 1);

        let fix = &result.diagnostics[0].fix;
        assert!(fix.is_some());
        let fix = fix.as_ref().unwrap();
        assert!(fix.replacement.contains("cat") || fix.replacement.contains("xargs echo"));
    }

    #[test]
    fn test_sc2008_severity_warning() {
        let source = "command | echo";
        let result = check(source);
        assert_eq!(result.diagnostics.len(), 1);
        assert_eq!(result.diagnostics[0].severity, Severity::Warning);
    }
}
