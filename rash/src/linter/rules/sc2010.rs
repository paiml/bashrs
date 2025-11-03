//! SC2010: Don't use ls | grep. Use a glob or find instead
//!
//! Parsing ls output breaks on filenames with spaces, newlines, or special characters.
//!
//! # Examples
//!
//! Bad:
//! ```bash
//! ls | grep pattern
//! ls -la | grep ".txt"
//! ```
//!
//! Good:
//! ```bash
//! for f in *pattern*; do ...; done
//! find . -name "*pattern*"
//! ```
//!
//! # Rationale
//!
//! - ls output is designed for humans, not parsing
//! - Filenames can contain spaces, newlines, special characters
//! - Globs and find are safer and more reliable
//!
//! # Compatibility
//!
//! Universal - Applies to all POSIX shells

use crate::linter::diagnostic::FixSafetyLevel;
use crate::linter::{Diagnostic, Fix, LintResult, Severity, Span};
use regex::Regex;

pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();
    let pattern = Regex::new(r"\bls\b[^|]*\|\s*grep").unwrap();

    for (line_num, line) in source.lines().enumerate() {
        if line.trim().starts_with('#') {
            continue;
        }

        if let Some(mat) = pattern.find(line) {
            result.diagnostics.push(Diagnostic {
                code: "SC2010".to_string(),
                severity: Severity::Warning,
                message: "Don't use ls | grep. Use a glob or find instead, to better handle non-alphanumeric filenames.".to_string(),
                span: Span {
                    start_line: line_num + 1,
                    end_line: line_num + 1,
                    start_col: mat.start() + 1,
                    end_col: mat.end() + 1,
                },
                fix: Some(Fix {
                    replacement: "for f in *pattern*; do ... done".to_string(),
                    safety_level: FixSafetyLevel::Unsafe,
                    assumptions: vec![],
                    suggested_alternatives: vec![
                        "find . -name '*pattern*'".to_string(),
                        "for f in *pattern*; do [ -e \"$f\" ] || continue; ... done".to_string(),
                    ],
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
    fn test_sc2010_basic() {
        let source = "ls | grep pattern";
        let result = check(source);
        assert_eq!(result.diagnostics.len(), 1);
        assert_eq!(result.diagnostics[0].code, "SC2010");
    }

    #[test]
    fn test_sc2010_with_flags() {
        let source = "ls -la | grep .txt";
        let result = check(source);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2010_no_grep() {
        let source = "ls -la";
        let result = check(source);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2010_glob_usage() {
        let source = "for f in *.txt; do echo $f; done";
        let result = check(source);
        assert_eq!(result.diagnostics.len(), 0);
    }
}
