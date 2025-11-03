//! SC2011: Use find -print0 | xargs -0 instead of ls | xargs
//!
//! Using ls | xargs breaks on filenames with spaces or special characters.
//!
//! # Examples
//!
//! Bad:
//! ```bash
//! ls | xargs command
//! ls -1 | xargs rm
//! ```
//!
//! Good:
//! ```bash
//! find . -print0 | xargs -0 command
//! find . -maxdepth 1 -print0 | xargs -0 rm
//! ```
//!
//! # Rationale
//!
//! - ls output is space-delimited, breaks on filenames with spaces
//! - find -print0 with xargs -0 uses null delimiters (safe)
//!
//! # Compatibility
//!
//! Universal - find -print0 is standard

use crate::linter::diagnostic::FixSafetyLevel;
use crate::linter::{Diagnostic, Fix, LintResult, Severity, Span};
use regex::Regex;

pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();
    let pattern = Regex::new(r"\bls\b[^|]*\|\s*xargs").unwrap();

    for (line_num, line) in source.lines().enumerate() {
        if line.trim().starts_with('#') {
            continue;
        }

        if let Some(mat) = pattern.find(line) {
            result.diagnostics.push(Diagnostic {
                code: "SC2011".to_string(),
                severity: Severity::Warning,
                message: "Use 'find -print0 | xargs -0' instead of 'ls | xargs' for better handling of filenames with spaces.".to_string(),
                span: Span {
                    start_line: line_num + 1,
                    end_line: line_num + 1,
                    start_col: mat.start() + 1,
                    end_col: mat.end() + 1,
                },
                fix: Some(Fix {
                    replacement: "find . -maxdepth 1 -print0 | xargs -0".to_string(),
                    safety_level: FixSafetyLevel::SafeWithAssumptions,
                    assumptions: vec!["find -print0 and xargs -0 are available".to_string()],
                    suggested_alternatives: vec![
                        "find . -maxdepth 1 -exec command {} +".to_string(),
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
    fn test_sc2011_basic() {
        let source = "ls | xargs rm";
        let result = check(source);
        assert_eq!(result.diagnostics.len(), 1);
        assert_eq!(result.diagnostics[0].code, "SC2011");
    }

    #[test]
    fn test_sc2011_with_flags() {
        let source = "ls -1 | xargs command";
        let result = check(source);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2011_find_usage() {
        let source = "find . -print0 | xargs -0 rm";
        let result = check(source);
        assert_eq!(result.diagnostics.len(), 0);
    }
}
