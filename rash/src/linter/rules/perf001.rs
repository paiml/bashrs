//! PERF001: Useless use of cat (performance)
//!
//! **Rule**: Detect `cat file | cmd` where cmd can read from file directly
//!
//! **Why this matters**:
//! Piping through `cat` spawns an unnecessary process and adds overhead.
//! Most commands (grep, sed, awk, sort, wc) can read files directly.
//!
//! **Auto-fix**: Safe - suggest removing cat and passing file as argument
//!
//! ## Examples
//!
//! Bad (spawns extra process):
//! ```bash
//! cat file.txt | grep pattern
//! ```
//!
//! Good (direct file argument):
//! ```bash
//! grep pattern file.txt
//! ```

use crate::linter::{Diagnostic, Fix, LintResult, Severity, Span};
use regex::Regex;

/// Check for useless use of cat piped to another command
pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();

    let pattern = Regex::new(r"\bcat\s+([^\s|;&]+)\s*\|\s*(\w+)").unwrap();

    for (line_num, line) in source.lines().enumerate() {
        let trimmed = line.trim_start();

        // Skip comments
        if trimmed.starts_with('#') {
            continue;
        }

        for cap in pattern.captures_iter(line) {
            let full_match = cap.get(0).unwrap();
            let filename = cap.get(1).unwrap().as_str();
            let command = cap.get(2).unwrap().as_str();

            // Skip if cat has flags (e.g., cat -n file | ...) - that's not useless
            if filename.starts_with('-') {
                continue;
            }

            let start_col = full_match.start() + 1;
            let end_col = full_match.end() + 1;

            let fix_text = format!("{} {}", command, filename);

            let diagnostic = Diagnostic::new(
                "PERF001",
                Severity::Info,
                format!(
                    "Useless use of cat. Use `{}` instead of `cat {} | {}`",
                    fix_text, filename, command
                ),
                Span::new(line_num + 1, start_col, line_num + 1, end_col),
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
    fn test_perf001_detects_cat_pipe_grep() {
        let script = "cat file.txt | grep pattern";
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 1);
        assert_eq!(result.diagnostics[0].code, "PERF001");
        assert_eq!(result.diagnostics[0].severity, Severity::Info);
    }

    #[test]
    fn test_perf001_provides_fix() {
        let script = "cat file.txt | grep pattern";
        let result = check(script);
        assert!(result.diagnostics[0].fix.is_some());
        let fix = result.diagnostics[0].fix.as_ref().unwrap();
        assert_eq!(fix.replacement, "grep file.txt");
    }

    #[test]
    fn test_perf001_no_false_positive_direct() {
        let script = "grep pattern file.txt";
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_perf001_no_false_positive_comment() {
        let script = "# cat file.txt | grep pattern";
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_perf001_cat_with_flags_not_flagged() {
        let script = "cat -n file.txt | grep pattern";
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_perf001_multiple_occurrences() {
        let script = "cat a.txt | grep x\ncat b.txt | wc -l";
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 2);
    }
}
