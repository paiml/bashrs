//! SC2013: To read lines rather than words, pipe/redirect to 'while read' loop
//!
//! Using `for line in $(cat file)` splits on whitespace (words), not lines.
//!
//! # Examples
//!
//! Bad:
//! ```bash
//! for line in $(cat file); do echo $line; done
//! ```
//!
//! Good:
//! ```bash
//! while IFS= read -r line; do echo "$line"; done < file
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
    let pattern = Regex::new(r"for\s+\w+\s+in\s+\$\(\s*cat").unwrap();

    for (line_num, line) in source.lines().enumerate() {
        if line.trim().starts_with('#') {
            continue;
        }

        if let Some(mat) = pattern.find(line) {
            result.diagnostics.push(Diagnostic {
                code: "SC2013".to_string(),
                severity: Severity::Warning,
                message: "To read lines rather than words, pipe/redirect to a 'while read' loop.".to_string(),
                span: Span {
                    start_line: line_num + 1,
                    end_line: line_num + 1,
                    start_col: mat.start() + 1,
                    end_col: mat.end() + 1,
                },
                fix: Some(Fix {
                    replacement: "while IFS= read -r line; do ... done < file".to_string(),
                    safety_level: FixSafetyLevel::Safe,
                    assumptions: vec![],
                    suggested_alternatives: vec![
                        "cat file | while IFS= read -r line; do ... done".to_string(),
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
    fn test_sc2013_basic() {
        let source = "for line in $(cat file); do echo $line; done";
        let result = check(source);
        assert_eq!(result.diagnostics.len(), 1);
        assert_eq!(result.diagnostics[0].code, "SC2013");
    }

    #[test]
    fn test_sc2013_while_read() {
        let source = "while IFS= read -r line; do echo $line; done < file";
        let result = check(source);
        assert_eq!(result.diagnostics.len(), 0);
    }
}
