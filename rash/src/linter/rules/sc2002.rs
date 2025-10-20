//! SC2002: Useless use of cat
//!
//! # Examples
//!
//! Bad:
//! ```bash
//! cat file.txt | grep pattern
//! ```
//!
//! Good:
//! ```bash
//! grep pattern file.txt
//! ```
//!
//! # Rationale
//!
//! Using cat to pipe a file to a command is inefficient:
//! - Spawns unnecessary process
//! - Most commands can read files directly
//! - Known as "Useless Use of Cat" (UUOC)
//!
//! # Auto-fix
//!
//! Suggest removing cat and passing file as argument

use crate::linter::{Diagnostic, Fix, LintResult, Severity, Span};
use regex::Regex;

/// Check for useless use of cat
pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();

    // Pattern: cat file | command
    let pattern = Regex::new(r"cat\s+([^\s|]+)\s*\|\s*(\w+)").unwrap();

    for (line_num, line) in source.lines().enumerate() {
        let line_num = line_num + 1;

        if line.trim_start().starts_with('#') {
            continue;
        }

        for cap in pattern.captures_iter(line) {
            let full_match = cap.get(0).unwrap();
            let filename = cap.get(1).unwrap().as_str();
            let command = cap.get(2).unwrap().as_str();

            let start_col = full_match.start() + 1;
            let end_col = full_match.end() + 1;

            let fix_text = format!("{} {}", command, filename);

            let diagnostic = Diagnostic::new(
                "SC2002",
                Severity::Info,
                "Useless use of cat. Consider using command directly on the file.",
                Span::new(line_num, start_col, line_num, end_col),
            )
            .with_fix(Fix::new(fix_text));

            result.add(diagnostic);
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sc2002_basic_detection() {
        let script = "cat file.txt | grep pattern";
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 1);
        assert_eq!(result.diagnostics[0].code, "SC2002");
    }

    #[test]
    fn test_sc2002_autofix() {
        let script = "cat file.txt | grep pattern";
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 1);
        assert!(result.diagnostics[0].fix.is_some());
        assert_eq!(
            result.diagnostics[0].fix.as_ref().unwrap().replacement,
            "grep file.txt"
        );
    }

    #[test]
    fn test_sc2002_with_wc() {
        let script = "cat data.log | wc -l";
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2002_with_sed() {
        let script = "cat input.txt | sed 's/old/new/'";
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2002_false_positive_direct_command() {
        let script = "grep pattern file.txt";
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2002_false_positive_in_comment() {
        let script = "# cat file.txt | grep pattern";
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2002_multiple_occurrences() {
        let script = "cat a.txt | grep x\ncat b.txt | wc -l";
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 2);
    }

    #[test]
    fn test_sc2002_with_path() {
        let script = "cat /var/log/syslog | tail -100";
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2002_with_awk() {
        let script = "cat data.csv | awk '{print $1}'";
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2002_with_sort() {
        let script = "cat list.txt | sort";
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 1);
    }
}
