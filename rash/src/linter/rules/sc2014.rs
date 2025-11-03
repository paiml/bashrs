//! SC2014: This will expand before brace expansion happens
//!
//! Variables in brace expansions like `{$start..$end}` don't work - the variable
//! expands before brace expansion, resulting in literal braces.
//!
//! # Examples
//!
//! Bad:
//! ```bash
//! for i in {$start..$end}; do echo $i; done
//! echo {$a..$b}
//! ```
//!
//! Good:
//! ```bash
//! for ((i=start; i<=end; i++)); do echo $i; done
//! seq $start $end
//! ```
//!
//! # Compatibility
//!
//! NotSh for (( )) loops, Universal for seq

use crate::linter::diagnostic::FixSafetyLevel;
use crate::linter::{Diagnostic, Fix, LintResult, Severity, Span};
use regex::Regex;

pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();
    // Match {$var..something} or {something..$var}
    let pattern = Regex::new(r"\{\$\w+\.\.[^\}]*\}|\{[^\$]*\.\.\$\w+\}").unwrap();

    for (line_num, line) in source.lines().enumerate() {
        if line.trim().starts_with('#') {
            continue;
        }

        if let Some(mat) = pattern.find(line) {
            result.diagnostics.push(Diagnostic {
                code: "SC2014".to_string(),
                severity: Severity::Warning,
                message: "This will expand before brace expansion happens. Use seq or a for loop instead.".to_string(),
                span: Span {
                    start_line: line_num + 1,
                    end_line: line_num + 1,
                    start_col: mat.start() + 1,
                    end_col: mat.end() + 1,
                },
                fix: Some(Fix {
                    replacement: "seq $start $end".to_string(),
                    safety_level: FixSafetyLevel::SafeWithAssumptions,
                    assumptions: vec!["seq is available".to_string()],
                    suggested_alternatives: vec![
                        "for ((i=start; i<=end; i++)); do ... done  # bash/ksh/zsh only".to_string(),
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
    fn test_sc2014_basic() {
        let source = "for i in {$start..$end}; do echo $i; done";
        let result = check(source);
        assert_eq!(result.diagnostics.len(), 1);
        assert_eq!(result.diagnostics[0].code, "SC2014");
    }

    #[test]
    fn test_sc2014_reverse() {
        let source = "echo {1..$end}";
        let result = check(source);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2014_seq_usage() {
        let source = "for i in $(seq $start $end); do echo $i; done";
        let result = check(source);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2014_literal_braces() {
        let source = "for i in {1..10}; do echo $i; done";
        let result = check(source);
        assert_eq!(result.diagnostics.len(), 0);
    }
}
