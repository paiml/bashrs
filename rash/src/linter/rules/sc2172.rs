// SC2172: Trapping signals by number is not portable. Use signal names instead.
//
// Signal numbers can vary between different Unix systems. Always use signal names
// (SIGTERM, SIGINT, etc.) instead of numbers for portability.
//
// Examples:
// Bad:
//   trap cleanup 15      # Signal 15 (SIGTERM) - not portable
//   trap '' 2            # Signal 2 (SIGINT) - not portable
//   trap handler 1 2 3   # Multiple numeric signals
//
// Good:
//   trap cleanup SIGTERM  # Portable
//   trap '' SIGINT        # Portable
//   trap handler SIGHUP SIGINT SIGQUIT

use crate::linter::{Diagnostic, LintResult, Severity, Span};
use once_cell::sync::Lazy;
use regex::Regex;

static TRAP_WITH_NUMBER: Lazy<Regex> = Lazy::new(|| {
    // Match: trap <handler> <number> or trap <handler> <number> <number>...
    Regex::new(r#"\btrap\s+(?:'[^']*'|"[^"]*"|[A-Za-z_][A-Za-z0-9_]*)\s+(\d+)"#).unwrap()
});

pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();

    for (line_num, line) in source.lines().enumerate() {
        let line_num = line_num + 1;

        if line.trim_start().starts_with('#') {
            continue;
        }

        for cap in TRAP_WITH_NUMBER.captures_iter(line) {
            if let Some(signal_num) = cap.get(1) {
                let start_col = signal_num.start() + 1;
                let end_col = signal_num.end() + 1;

                let num_text = signal_num.as_str();
                let signal_name = match num_text {
                    "1" => "SIGHUP",
                    "2" => "SIGINT",
                    "3" => "SIGQUIT",
                    "6" => "SIGABRT",
                    "9" => "SIGKILL",
                    "14" => "SIGALRM",
                    "15" => "SIGTERM",
                    _ => "SIG<name>",
                };

                let diagnostic = Diagnostic::new(
                    "SC2172",
                    Severity::Warning,
                    format!(
                        "Trapping signal {} by number is not portable. Use {} instead",
                        num_text, signal_name
                    ),
                    Span::new(line_num, start_col, line_num, end_col),
                );

                result.add(diagnostic);
            }
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sc2172_trap_sigterm_number() {
        let code = r#"trap cleanup 15"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
        assert_eq!(result.diagnostics[0].code, "SC2172");
        assert_eq!(result.diagnostics[0].severity, Severity::Warning);
        assert!(result.diagnostics[0].message.contains("15"));
        assert!(result.diagnostics[0].message.contains("SIGTERM"));
    }

    #[test]
    fn test_sc2172_trap_sigint_number() {
        let code = r#"trap '' 2"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
        assert!(result.diagnostics[0].message.contains("2"));
        assert!(result.diagnostics[0].message.contains("SIGINT"));
    }

    #[test]
    fn test_sc2172_trap_with_name_ok() {
        let code = r#"trap cleanup SIGTERM"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2172_trap_exit_ok() {
        let code = r#"trap cleanup EXIT"#;
        let result = check(code);
        // EXIT is a name, not a number
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2172_multiple_signals_numeric() {
        let code = r#"trap handler 1 2 3"#;
        let result = check(code);
        // Should detect first numeric signal (1)
        assert!(result.diagnostics.len() >= 1);
    }

    #[test]
    fn test_sc2172_quoted_handler() {
        let code = r#"trap "echo done" 15"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2172_single_quoted_handler() {
        let code = r#"trap 'cleanup' 2"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2172_function_handler() {
        let code = r#"trap my_cleanup 9"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
        assert!(result.diagnostics[0].message.contains("SIGKILL"));
    }

    #[test]
    fn test_sc2172_unknown_signal_number() {
        let code = r#"trap handler 42"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
        assert!(result.diagnostics[0].message.contains("42"));
        assert!(result.diagnostics[0].message.contains("SIG<name>"));
    }

    #[test]
    fn test_sc2172_no_trap_command() {
        let code = r#"echo 15"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }
}
