// SC2271: Prefer printf over echo for non-trivial formatting
use crate::linter::{Diagnostic, LintResult, Severity, Span};
use once_cell::sync::Lazy;
use regex::Regex;

static ECHO_WITH_ESCAPES: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r#"echo\s+(-[en]+\s+)?["'].*\\[ntr]"#).unwrap()
});

pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();
    for (line_num, line) in source.lines().enumerate() {
        let line_num = line_num + 1;
        if line.trim_start().starts_with('#') {
            continue;
        }

        if ECHO_WITH_ESCAPES.is_match(line) {
            let diagnostic = Diagnostic::new(
                "SC2271",
                Severity::Info,
                "Prefer printf over echo for escape sequences (more portable)".to_string(),
                Span::new(line_num, 1, line_num, line.len() + 1),
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
    fn test_sc2271_echo_newline() {
        let code = r#"echo "line1\nline2""#;
        assert_eq!(check(code).diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2271_printf_ok() {
        let code = r#"printf "line1\nline2\n""#;
        assert_eq!(check(code).diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2271_echo_tab() {
        let code = r#"echo "col1\tcol2""#;
        assert_eq!(check(code).diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2271_simple_echo_ok() {
        let code = r#"echo "hello""#;
        assert_eq!(check(code).diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2271_comment() {
        let code = r#"# echo "test\n""#;
        assert_eq!(check(code).diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2271_empty() {
        assert_eq!(check("").diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2271_normal() {
        assert_eq!(check("echo test").diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2271_echo_e_flag() {
        let code = r#"echo -e "test\nline""#;
        assert_eq!(check(code).diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2271_echo_return() {
        let code = r#"echo 'text\r'"#;
        assert_eq!(check(code).diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2271_echo_variable_ok() {
        let code = r#"echo "$var""#;
        assert_eq!(check(code).diagnostics.len(), 0);
    }
}
