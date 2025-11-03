//! SC2012: Use find instead of ls to better handle non-alphanumeric filenames
//!
//! Using ls in loops or with while read breaks on filenames with newlines or special characters.
//!
//! # Examples
//!
//! Bad:
//! ```bash
//! ls | while read file; do ...; done
//! for file in $(ls); do ...; done
//! ```
//!
//! Good:
//! ```bash
//! find . -maxdepth 1 -print0 | while IFS= read -r -d '' file; do ...; done
//! for file in *; do [ -e "$file" ] || continue; ...; done
//! ```
//!
//! # Compatibility
//!
//! Universal

use crate::linter::diagnostic::FixSafetyLevel;
use crate::linter::{Diagnostic, Fix, LintResult, Severity, Span};
use regex::Regex;

pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();
    let pattern = Regex::new(r"\bls\b[^|]*\|\s*while\s+read").unwrap();

    for (line_num, line) in source.lines().enumerate() {
        if line.trim().starts_with('#') {
            continue;
        }

        if let Some(mat) = pattern.find(line) {
            result.diagnostics.push(Diagnostic {
                code: "SC2012".to_string(),
                severity: Severity::Warning,
                message: "Use find instead of ls to better handle non-alphanumeric filenames."
                    .to_string(),
                span: Span {
                    start_line: line_num + 1,
                    end_line: line_num + 1,
                    start_col: mat.start() + 1,
                    end_col: mat.end() + 1,
                },
                fix: Some(Fix {
                    replacement: "find . -maxdepth 1 -print0 | while IFS= read -r -d '' file"
                        .to_string(),
                    safety_level: FixSafetyLevel::SafeWithAssumptions,
                    assumptions: vec!["find -print0 is available".to_string()],
                    suggested_alternatives: vec![
                        "for file in *; do [ -e \"$file\" ] || continue; ...; done".to_string(),
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
    fn test_sc2012_basic() {
        let source = "ls | while read file; do echo $file; done";
        let result = check(source);
        assert_eq!(result.diagnostics.len(), 1);
        assert_eq!(result.diagnostics[0].code, "SC2012");
    }

    #[test]
    fn test_sc2012_find_usage() {
        let source = "find . -print0 | while IFS= read -r -d '' file; do echo $file; done";
        let result = check(source);
        assert_eq!(result.diagnostics.len(), 0);
    }
}
