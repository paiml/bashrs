// SC2173: SIGKILL/SIGSTOP can't be trapped.
//
// SIGKILL (9) and SIGSTOP (17/19) cannot be caught or ignored.
//
// Examples:
// Bad:
//   trap "cleanup" SIGKILL       // Won't work
//   trap "handler" SIGSTOP       // Can't be trapped
//   trap "cleanup" 9             // SIGKILL by number
//
// Good:
//   trap "cleanup" SIGTERM       // Can be trapped
//   trap "cleanup" SIGINT        // Can be trapped
//   trap "cleanup" EXIT          // Pseudo-signal, works
//
// Impact: Trap won't work, false sense of control

use crate::linter::{Diagnostic, LintResult, Severity, Span};
use regex::Regex;

static TRAP_SIGKILL_SIGSTOP: std::sync::LazyLock<Regex> = std::sync::LazyLock::new(|| {
    // Match: trap "handler" SIGKILL/SIGSTOP/9/17/19
    // Also match: trap 'handler' SIGKILL
    Regex::new(r#"\btrap\s+["']?[^"'\s]+["']?\s+(SIGKILL|SIGSTOP|9|17|19)\b"#).unwrap()
});

pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();

    for (line_num, line) in source.lines().enumerate() {
        let line_num = line_num + 1;

        if line.trim_start().starts_with('#') {
            continue;
        }

        for mat in TRAP_SIGKILL_SIGSTOP.find_iter(line) {
            let start_col = mat.start() + 1;
            let end_col = mat.end() + 1;

            let diagnostic = Diagnostic::new(
                "SC2173",
                Severity::Error,
                "SIGKILL/SIGSTOP can't be trapped".to_string(),
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
    fn test_sc2173_trap_sigkill() {
        let code = r#"trap "cleanup" SIGKILL"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2173_trap_sigstop() {
        let code = r#"trap "cleanup" SIGSTOP"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2173_trap_9() {
        let code = r#"trap "cleanup" 9"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2173_trap_sigterm_ok() {
        let code = r#"trap "cleanup" SIGTERM"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2173_trap_sigint_ok() {
        let code = r#"trap "cleanup" SIGINT"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2173_trap_exit_ok() {
        let code = r#"trap "cleanup" EXIT"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2173_comment_ok() {
        let code = r#"# trap "cleanup" SIGKILL"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2173_multiple() {
        let code = "trap 'cleanup' SIGKILL\ntrap 'handler' SIGSTOP";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 2);
    }

    #[test]
    fn test_sc2173_trap_17() {
        let code = r#"trap "cleanup" 17"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2173_trap_2_ok() {
        let code = r#"trap "cleanup" 2"#;
        let result = check(code);
        // SIGINT (2) can be trapped
        assert_eq!(result.diagnostics.len(), 0);
    }
}
