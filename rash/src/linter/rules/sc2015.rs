// SC2015: Note that A && B || C is not if-then-else. C may run when A is true.
//
// A common mistake is thinking `A && B || C` means "if A then B else C".
// It actually means "(A and B) or C". If B fails, C will run even if A succeeded.
//
// Examples:
// Bad:
//   test && success || failure      // failure runs if success fails
//   [[ -f file ]] && cat file || echo "missing"  // wrong logic
//   command && echo "ok" || echo "fail"  // misleading
//
// Good:
//   if test; then success; else failure; fi  // Clear if-then-else
//   if [[ -f file ]]; then cat file; else echo "missing"; fi
//   if command; then echo "ok"; else echo "fail"; fi
//
// Why it's wrong:
//   true && false || echo "runs"    // Runs even though true succeeded
//   The || sees the false from && and runs
//
// Note: Use proper if-then-else for conditional logic to avoid surprises.

use crate::linter::{Diagnostic, LintResult, Severity, Span};
use regex::Regex;

static AND_OR_PATTERN: std::sync::LazyLock<Regex> = std::sync::LazyLock::new(|| {
    // Match: command && command || command
    // Looking for pattern with both && and ||
    Regex::new(r"[^|&]+&&[^|&]+\|\|[^|&]+").unwrap()
});

/// ShellCheck 0.8.0 does not flag `A && B || C` when C is an error-reporting or
/// terminating command (the usual "A && B || die" idiom).
fn fallback_is_exempt(matched: &str) -> bool {
    let fallback = matched.rsplit("||").next().unwrap_or("").trim_start();
    let cmd = fallback.split_whitespace().next().unwrap_or("");
    matches!(cmd, "echo" | "exit" | "return" | "printf")
}

pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();

    for (line_num, line) in source.lines().enumerate() {
        let line_num = line_num + 1;

        let trimmed = line.trim_start();
        if trimmed.starts_with('#') {
            continue;
        }

        // Skip if line is inside a comment or string
        // Simple heuristic: skip lines that start with if/elif/while (proper control flow)
        if trimmed.starts_with("if ")
            || trimmed.starts_with("elif ")
            || trimmed.starts_with("while ")
        {
            continue;
        }

        // Look for && followed by ||
        if let Some(m) = AND_OR_PATTERN.find(line) {
            let matched = m.as_str();
            if fallback_is_exempt(matched) {
                continue;
            }
            let start_col = m.start() + 1;
            let end_col = m.end() + 1;

            let diagnostic = Diagnostic::new(
                "SC2015",
                Severity::Info,
                "Note that 'A && B || C' is not if-then-else. C may run when A is true. Use 'if A; then B; else C; fi' for proper conditional logic".to_string(),
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
    fn test_sc2015_and_or_pattern() {
        let code = r#"test && success || failure"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
        assert_eq!(result.diagnostics[0].code, "SC2015");
        assert_eq!(result.diagnostics[0].severity, Severity::Info);
        assert!(result.diagnostics[0].message.contains("not if-then-else"));
    }

    #[test]
    fn test_sc2015_file_check() {
        let code = r#"[[ -f file ]] && cat file || handle_missing"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2015_command_check() {
        let code = r#"command && echo "ok" || cleanup"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2015_complex_commands() {
        let code = r#"grep pattern file && process data || handle_error"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2015_if_statement_ok() {
        let code = r#"if test; then success; else failure; fi"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2015_if_with_and_ok() {
        let code = r#"if command && other; then action; fi"#;
        let result = check(code);
        // Inside if statement, && || is fine
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2015_only_and_ok() {
        let code = r#"test && success && another"#;
        let result = check(code);
        // Only &&, no ||
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2015_only_or_ok() {
        let code = r#"test || fallback || default"#;
        let result = check(code);
        // Only ||, no &&
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2015_multiple_issues() {
        let code = r#"
check1 && ok1 || fail1
check2 && ok2 || fail2
"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 2);
    }

    #[test]
    fn test_sc2015_parity_echo_exit_return_printf_fallback_exempt() {
        // shellcheck 0.8.0 does not flag these fallbacks (error-handling idiom)
        for c in [
            "foo && bar || echo b",
            "foo && bar || exit 1",
            "foo && bar || return 1",
            "foo && bar || printf x",
            "[ -f x ] && echo a || echo b >&2",
        ] {
            assert_eq!(check(c).diagnostics.len(), 0, "{c}");
        }
    }

    #[test]
    fn test_sc2015_parity_other_fallback_still_flagged() {
        for c in [
            "foo && bar || true",
            "foo && bar || :",
            "foo && bar || cd d",
            "foo && bar || continue",
        ] {
            assert_eq!(check(c).diagnostics.len(), 1, "{c}");
        }
    }

    #[test]
    fn test_sc2015_in_function() {
        let code = r#"
my_func() {
    validate && process || error
}
"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }
}
