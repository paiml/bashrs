// SC2097: This assignment is only seen by the forked process
//
// When an assignment appears directly before a command (not on a separate line),
// it only affects the environment of that command, not the current shell.
// This is often misunderstood.
//
// Examples:
// Bad (if you wanted to set the variable for later use):
//   PATH=/usr/local/bin:$PATH command
//   # PATH is only changed for 'command', not after
//
//   DEBUG=1 ./script.sh
//   # DEBUG is only set in script.sh's environment
//
// Good (set for current shell):
//   PATH=/usr/local/bin:$PATH
//   command
//
//   export DEBUG=1
//   ./script.sh
//
// Good (intentional one-time environment):
//   LC_ALL=C sort file.txt  # Only affects sort

use crate::linter::{Diagnostic, LintResult, Severity, Span};
use regex::Regex;

static ASSIGNMENT_BEFORE_COMMAND: std::sync::LazyLock<Regex> = std::sync::LazyLock::new(|| {
    // Match: VAR=value ... command (but not just VAR=value)
    // Allows multiple assignments before a command
    Regex::new(
        r"^\s*([A-Z_][A-Z0-9_]*=[^\s]+)(?:\s+[A-Z_][A-Z0-9_]*=[^\s]+)*\s+([a-z./][^\s;|&<>]+)",
    )
    .unwrap()
});

pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();

    for (line_num, line) in source.lines().enumerate() {
        let line_num = line_num + 1;

        if line.trim_start().starts_with('#') {
            continue;
        }

        // Skip export statements (these are intentional)
        if line.trim_start().starts_with("export ") {
            continue;
        }

        // Skip common intentional one-time env vars
        if line.contains("LC_ALL=")
            || line.contains("LANG=")
            || line.contains("TZ=")
            || line.contains("HOME=")
        {
            continue;
        }

        if let Some(cap) = ASSIGNMENT_BEFORE_COMMAND.captures(line) {
            let assignment = cap.get(1).unwrap().as_str();
            let command = cap.get(2).unwrap().as_str();
            let start_col = cap.get(0).unwrap().start() + 1;
            let end_col = cap.get(0).unwrap().end() + 1;

            // Extract variable name
            let var_name = assignment.split('=').next().unwrap();

            let diagnostic = Diagnostic::new(
                "SC2097",
                Severity::Info,
                format!(
                    "This assignment is only seen by '{}', not the current shell. Move to separate line or use 'export' if you want it to persist",
                    command
                ),
                Span::new(line_num, start_col, line_num, end_col),
            );

            result.add(diagnostic);
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sc2097_path_before_command() {
        let code = r#"PATH=/usr/local/bin:$PATH command"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
        assert_eq!(result.diagnostics[0].code, "SC2097");
        assert_eq!(result.diagnostics[0].severity, Severity::Info);
        assert!(result.diagnostics[0].message.contains("command"));
    }

    #[test]
    fn test_sc2097_debug_before_script() {
        let code = r#"DEBUG=1 ./script.sh"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2097_separate_lines_ok() {
        let code = r#"
PATH=/usr/local/bin:$PATH
command
"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2097_export_ok() {
        let code = r#"export DEBUG=1"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2097_lc_all_ok() {
        let code = r#"LC_ALL=C sort file.txt"#;
        let result = check(code);
        // LC_ALL is commonly used this way
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2097_lang_ok() {
        let code = r#"LANG=en_US.UTF-8 date"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2097_multiple_assignments() {
        let code = r#"FOO=bar BAZ=qux ./script.sh"#;
        let result = check(code);
        // Only detects first assignment
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2097_assignment_only_ok() {
        let code = r#"PATH=/usr/local/bin:$PATH"#;
        let result = check(code);
        // No command after, just assignment
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2097_lowercase_var_ok() {
        let code = r#"foo=bar command"#;
        let result = check(code);
        // Lowercase vars are typically local/intentional
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2097_tz_ok() {
        let code = r#"TZ=UTC date"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }
}
