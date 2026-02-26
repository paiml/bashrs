// SC2311: Use single quotes for literal strings in assignments
use crate::linter::{Diagnostic, LintResult, Severity, Span};
use regex::Regex;

static LITERAL_DOUBLE_QUOTES: std::sync::LazyLock<Regex> =
    std::sync::LazyLock::new(|| Regex::new(r#"^[a-zA-Z_][a-zA-Z0-9_]*="[^$`\\]*"$"#).unwrap());

pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();
    for (line_num, line) in source.lines().enumerate() {
        let line_num = line_num + 1;
        if line.trim_start().starts_with('#') {
            continue;
        }

        let trimmed = line.trim();
        if LITERAL_DOUBLE_QUOTES.is_match(trimmed) {
            let diagnostic = Diagnostic::new(
                "SC2311",
                Severity::Info,
                "Use single quotes for literal strings that don't contain expansions".to_string(),
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
    fn test_sc2311_literal_in_double_quotes() {
        let code = r#"msg="hello world""#;
        assert_eq!(check(code).diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2311_single_quotes_ok() {
        let code = r#"msg='hello world'"#;
        assert_eq!(check(code).diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2311_with_expansion_ok() {
        let code = r#"msg="hello $USER""#;
        assert_eq!(check(code).diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2311_comment() {
        let code = r#"# msg="hello""#;
        assert_eq!(check(code).diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2311_empty() {
        assert_eq!(check("").diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2311_normal() {
        assert_eq!(check("echo test").diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2311_with_backslash_ok() {
        let code = r#"path="C:\Windows""#;
        assert_eq!(check(code).diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2311_with_backtick_ok() {
        let code = r#"result="`date`""#;
        assert_eq!(check(code).diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2311_literal_number() {
        let code = r#"count="42""#;
        assert_eq!(check(code).diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2311_multiple_lines() {
        let code = r#"
x="literal1"
y="literal2"
"#;
        assert_eq!(check(code).diagnostics.len(), 2);
    }
}
