// SC2316: Prefer [[ ]] over [ ] for string comparison
use crate::linter::{Diagnostic, LintResult, Severity, Span};
use once_cell::sync::Lazy;
use regex::Regex;

static SINGLE_BRACKET_STRING: Lazy<Regex> =
    Lazy::new(|| Regex::new(r#"(?:^|[^\[])\[\s+"[^"]*"\s+(?:=|!=)\s+"[^"]*"\s+\]"#).unwrap());

pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();
    for (line_num, line) in source.lines().enumerate() {
        let line_num = line_num + 1;
        if line.trim_start().starts_with('#') {
            continue;
        }

        if SINGLE_BRACKET_STRING.is_match(line) {
            let diagnostic = Diagnostic::new(
                "SC2316",
                Severity::Info,
                "Prefer [[ ]] over [ ] for string comparison (better handling of special characters)".to_string(),
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
    fn test_sc2316_single_bracket_string() {
        let code = r#"[ "abc" = "def" ]"#;
        assert_eq!(check(code).diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2316_double_bracket_ok() {
        let code = r#"[[ "abc" = "def" ]]"#;
        assert_eq!(check(code).diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2316_numeric_ok() {
        let code = "[ 5 -eq 10 ]";
        assert_eq!(check(code).diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2316_comment() {
        let code = r#"# [ "a" = "b" ]"#;
        assert_eq!(check(code).diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2316_empty() {
        assert_eq!(check("").diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2316_normal() {
        assert_eq!(check("echo test").diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2316_not_equal() {
        let code = r#"[ "x" != "y" ]"#;
        assert_eq!(check(code).diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2316_unquoted_ok() {
        let code = "[ $x = $y ]";
        assert_eq!(check(code).diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2316_file_test_ok() {
        let code = "[ -f file ]";
        assert_eq!(check(code).diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2316_multiple() {
        let code = r#"
[ "a" = "b" ]
[ "c" != "d" ]
"#;
        assert_eq!(check(code).diagnostics.len(), 2);
    }
}
