//! DET003: Unordered wildcard usage
//!
//! **Rule**: Detect wildcards without sorting for deterministic results
//!
//! **Why this matters**:
//! File glob results vary by filesystem and can change between runs,
//! breaking determinism.
//!
//! **Auto-fix**: Wrap command substitution with sort (only for $(ls ...) patterns)
//!
//! ## Examples
//!
//! ❌ **BAD** (non-deterministic):
//! ```bash
//! FILES=$(ls *.txt)
//! for f in *.c; do echo $f; done
//! ```
//!
//! ✅ **GOOD** (deterministic):
//! ```bash
//! FILES=$(ls *.txt | sort)
//! for f in $(printf '%s\n' *.c | sort); do echo "$f"; done
//! ```
//!
//! **Note**: For `for f in *.c`, no auto-fix is provided since the correct
//! transformation is complex. Users should manually review.

use crate::linter::{Diagnostic, Fix, LintResult, Severity, Span};

/// Check a `$(ls ...)` pattern for unordered wildcards and emit diagnostic with auto-fix
fn check_ls_wildcard(line: &str, line_num: usize, ls_start: usize, result: &mut LintResult) {
    let after_ls = &line[ls_start..];
    if let Some(close_paren) = find_matching_paren(after_ls) {
        let cmd_sub = &after_ls[..=close_paren];
        if cmd_sub.contains('*') {
            let span = Span::new(
                line_num + 1,
                ls_start + 1,
                line_num + 1,
                ls_start + close_paren + 2,
            );
            let inner = &cmd_sub[2..cmd_sub.len() - 1];
            let fixed = format!("$({} | sort)", inner);
            let diag = Diagnostic::new(
                "DET003",
                Severity::Warning,
                "Unordered wildcard in command substitution - results may vary",
                span,
            )
            .with_fix(Fix::new(fixed));
            result.add(diag);
        }
    }
}

/// Check a `for ... in *` pattern for unordered wildcards (no auto-fix)
fn check_for_loop_wildcard(line: &str, line_num: usize, result: &mut LintResult) {
    if line.contains("for ") && line.contains(" in ") {
        if let Some(col) = line.find('*') {
            let span = Span::new(line_num + 1, col + 1, line_num + 1, col + 2);
            let diag = Diagnostic::new(
                "DET003",
                Severity::Info,
                "Unordered wildcard in for-loop - consider sorting for determinism",
                span,
            );
            result.add(diag);
        }
    }
}

/// Check for unordered wildcard usage
pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();

    for (line_num, line) in source.lines().enumerate() {
        if line.contains('*') && !line.contains("| sort") {
            if let Some(ls_start) = line.find("$(ls ") {
                check_ls_wildcard(line, line_num, ls_start, &mut result);
            } else {
                check_for_loop_wildcard(line, line_num, &mut result);
            }
        }
    }

    result
}

/// Find the matching closing parenthesis for a command substitution
fn find_matching_paren(s: &str) -> Option<usize> {
    let mut depth = 0;
    for (i, c) in s.chars().enumerate() {
        match c {
            '(' => depth += 1,
            ')' => {
                depth -= 1;
                if depth == 0 {
                    return Some(i);
                }
            }
            _ => {}
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_DET003_detects_ls_wildcard() {
        let script = "FILES=$(ls *.txt)";
        let result = check(script);

        assert_eq!(result.diagnostics.len(), 1);
        let diag = &result.diagnostics[0];
        assert_eq!(diag.code, "DET003");
        assert_eq!(diag.severity, Severity::Warning);
    }

    #[test]
    fn test_DET003_detects_for_loop_wildcard() {
        let script = "for f in *.c; do echo $f; done";
        let result = check(script);

        assert_eq!(result.diagnostics.len(), 1);
        // For-loop wildcards get Info severity (no auto-fix)
        assert_eq!(result.diagnostics[0].severity, Severity::Info);
    }

    #[test]
    fn test_DET003_no_warning_with_sort() {
        let script = "FILES=$(ls *.txt | sort)";
        let result = check(script);

        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_DET003_provides_correct_fix_for_ls() {
        let script = "FILES=$(ls *.txt)";
        let result = check(script);

        assert!(result.diagnostics[0].fix.is_some());
        let fix = result.diagnostics[0].fix.as_ref().unwrap();
        // Fix should wrap the entire command substitution correctly
        assert_eq!(fix.replacement, "$(ls *.txt | sort)");
    }

    #[test]
    fn test_DET003_no_fix_for_for_loop() {
        // For-loop wildcards are too complex to auto-fix safely
        let script = "for f in *.c; do echo $f; done";
        let result = check(script);

        assert_eq!(result.diagnostics.len(), 1);
        // Should NOT have a fix (user must manually review)
        assert!(result.diagnostics[0].fix.is_none());
    }

    #[test]
    fn test_DET003_fix_span_covers_full_command_sub() {
        let script = "FILES=$(ls *.txt)";
        let result = check(script);

        let diag = &result.diagnostics[0];
        // Span should cover $(ls *.txt) which is columns 7-17 (1-indexed)
        assert_eq!(diag.span.start_col, 7);
        assert_eq!(diag.span.end_col, 18); // Exclusive end
    }

    #[test]
    fn test_DET003_nested_parens() {
        // Test with nested parentheses inside command substitution
        let script = "FILES=$(ls $(echo *.txt))";
        let result = check(script);

        // Should still detect the pattern
        assert!(!result.diagnostics.is_empty());
    }

    #[test]
    fn test_find_matching_paren() {
        assert_eq!(find_matching_paren("()"), Some(1));
        assert_eq!(find_matching_paren("(abc)"), Some(4));
        assert_eq!(find_matching_paren("((nested))"), Some(9));
        assert_eq!(find_matching_paren("(a(b)c)"), Some(6));
        assert_eq!(find_matching_paren("(unclosed"), None);
    }
}
