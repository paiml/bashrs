//! PERF003: Useless echo piped to command
//!
//! **Rule**: Detect `echo $VAR | cmd` where a here-string would be more efficient
//!
//! **Why this matters**:
//! `echo $VAR | cmd` spawns an extra process (echo) plus a pipe. Using a
//! here-string (`cmd <<< "$VAR"`) or here-doc avoids the extra process.
//!
//! **Auto-fix**: Safe - suggest here-string alternative
//!
//! ## Examples
//!
//! Bad (extra echo process + pipe):
//! ```bash
//! echo "$VAR" | grep pattern
//! ```
//!
//! Good (here-string, no extra process):
//! ```bash
//! grep pattern <<< "$VAR"
//! ```

use crate::linter::{Diagnostic, Fix, LintResult, Severity, Span};
use regex::Regex;

/// Check for useless echo piped to a command
pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();

    // Match: echo $VAR | cmd  or  echo "$VAR" | cmd  or  echo "text" | cmd
    let pattern = Regex::new(r#"\becho\s+(["']?[\$\w][^\|]*?["']?)\s*\|\s*(\w+)"#).unwrap();

    for (line_num, line) in source.lines().enumerate() {
        let trimmed = line.trim_start();

        // Skip comments
        if trimmed.starts_with('#') {
            continue;
        }

        for cap in pattern.captures_iter(line) {
            let full_match = cap.get(0).unwrap();
            let var_part = cap.get(1).unwrap().as_str().trim();
            let command = cap.get(2).unwrap().as_str();

            let start_col = full_match.start() + 1;
            let end_col = full_match.end() + 1;

            let fix_text = format!("{} <<< {}", command, var_part);

            let diagnostic = Diagnostic::new(
                "PERF003",
                Severity::Info,
                format!("Useless echo piped to {}. Consider `{}`", command, fix_text),
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
    fn test_perf003_detects_echo_pipe_grep() {
        let script = r#"echo "$VAR" | grep pattern"#;
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 1);
        assert_eq!(result.diagnostics[0].code, "PERF003");
        assert_eq!(result.diagnostics[0].severity, Severity::Info);
    }

    #[test]
    fn test_perf003_provides_fix() {
        let script = r#"echo "$VAR" | grep pattern"#;
        let result = check(script);
        assert!(result.diagnostics[0].fix.is_some());
        let fix = result.diagnostics[0].fix.as_ref().unwrap();
        assert!(fix.replacement.contains("<<<"));
    }

    #[test]
    fn test_perf003_no_false_positive_comment() {
        let script = r#"# echo "$VAR" | grep pattern"#;
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_perf003_detects_unquoted_var() {
        let script = "echo $VAR | wc -l";
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_perf003_no_false_positive_pipe_to_file() {
        let script = "echo hello > file.txt";
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 0);
    }
}
