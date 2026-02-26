// SC2307: Use [[ ]] or quote to prevent word splitting
use crate::linter::{Diagnostic, LintResult, Severity, Span};
use regex::Regex;

static UNQUOTED_VAR_TEST: std::sync::LazyLock<Regex> = std::sync::LazyLock::new(|| {
    Regex::new(r"(?:^|[^\[])\[\s+\$[a-zA-Z_][a-zA-Z0-9_]*\s+(?:-[a-z]+|[!=<>]+)").unwrap()
});

pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();
    for (line_num, line) in source.lines().enumerate() {
        let line_num = line_num + 1;
        if line.trim_start().starts_with('#') {
            continue;
        }

        if UNQUOTED_VAR_TEST.is_match(line) {
            let diagnostic = Diagnostic::new(
                "SC2307",
                Severity::Warning,
                "Use [[ ]] or quote variable to prevent word splitting in tests".to_string(),
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
    fn test_sc2307_unquoted_var() {
        let code = r#"[ $var = value ]"#;
        assert_eq!(check(code).diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2307_quoted_var_ok() {
        let code = r#"[ "$var" = value ]"#;
        assert_eq!(check(code).diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2307_double_bracket_ok() {
        let code = r#"[[ $var = value ]]"#;
        assert_eq!(check(code).diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2307_comment() {
        let code = r#"# [ $var = value ]"#;
        assert_eq!(check(code).diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2307_empty() {
        assert_eq!(check("").diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2307_normal() {
        assert_eq!(check("echo test").diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2307_numeric_test() {
        let code = r#"[ $count -gt 5 ]"#;
        assert_eq!(check(code).diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2307_not_equal() {
        let code = r#"[ $var != value ]"#;
        assert_eq!(check(code).diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2307_braced_var() {
        let code = r#"[ ${var} = value ]"#;
        assert_eq!(check(code).diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2307_multiple_tests() {
        let code = r#"
[ $x -eq 1 ]
[ $y -gt 2 ]
"#;
        assert_eq!(check(code).diagnostics.len(), 2);
    }
}
