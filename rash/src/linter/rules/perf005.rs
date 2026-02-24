//! PERF005: External echo/printf instead of builtin
//!
//! **Rule**: Detect `/bin/echo` or `/usr/bin/echo` instead of builtin `echo`
//!
//! **Why this matters**:
//! Using the full path to echo (e.g., `/bin/echo`) bypasses the shell builtin
//! and forks an external process. The shell builtin is significantly faster
//! and available in all POSIX-compliant shells.
//!
//! **Auto-fix**: Safe - remove the path prefix
//!
//! ## Examples
//!
//! Bad (forks external process):
//! ```bash
//! /bin/echo "hello"
//! /usr/bin/echo "hello"
//! /usr/bin/printf "hello\n"
//! ```
//!
//! Good (uses builtin):
//! ```bash
//! echo "hello"
//! printf "hello\n"
//! ```

use crate::linter::{Diagnostic, Fix, LintResult, Severity, Span};
use regex::Regex;

/// Check for external echo/printf instead of builtin
pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();

    let pattern = Regex::new(r"(/(?:usr/)?bin/(echo|printf))\b").unwrap();

    for (line_num, line) in source.lines().enumerate() {
        let trimmed = line.trim_start();

        // Skip comments
        if trimmed.starts_with('#') {
            continue;
        }

        for cap in pattern.captures_iter(line) {
            let full_match = cap.get(1).unwrap();
            let builtin_name = cap.get(2).unwrap().as_str();

            let start_col = full_match.start() + 1;
            let end_col = full_match.end() + 1;

            let diagnostic = Diagnostic::new(
                "PERF005",
                Severity::Info,
                format!(
                    "Use builtin `{}` instead of `{}` to avoid forking an external process",
                    builtin_name,
                    full_match.as_str()
                ),
                Span::new(line_num + 1, start_col, line_num + 1, end_col),
            )
            .with_fix(Fix::new(builtin_name.to_string()));

            result.add(diagnostic);
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_perf005_detects_bin_echo() {
        let script = r#"/bin/echo "hello""#;
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 1);
        assert_eq!(result.diagnostics[0].code, "PERF005");
        assert_eq!(result.diagnostics[0].severity, Severity::Info);
    }

    #[test]
    fn test_perf005_detects_usr_bin_echo() {
        let script = r#"/usr/bin/echo "hello""#;
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_perf005_detects_usr_bin_printf() {
        let script = r#"/usr/bin/printf "hello\n""#;
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_perf005_provides_fix() {
        let script = r#"/bin/echo "hello""#;
        let result = check(script);
        let fix = result.diagnostics[0].fix.as_ref().unwrap();
        assert_eq!(fix.replacement, "echo");
    }

    #[test]
    fn test_perf005_no_false_positive_builtin() {
        let script = r#"echo "hello""#;
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_perf005_no_false_positive_comment() {
        let script = r#"# /bin/echo "hello""#;
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_perf005_fix_for_printf() {
        let script = r#"/usr/bin/printf "%s\n" hello"#;
        let result = check(script);
        let fix = result.diagnostics[0].fix.as_ref().unwrap();
        assert_eq!(fix.replacement, "printf");
    }
}
