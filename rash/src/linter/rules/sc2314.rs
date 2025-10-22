// SC2314: Use arithmetic context (( )) for numeric comparison
use crate::linter::{Diagnostic, LintResult, Severity, Span};
use once_cell::sync::Lazy;
use regex::Regex;

static STRING_COMPARISON: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r#"\[\[\s+\d+\s+(?:==|!=)\s+\d+\s+\]\]"#).unwrap()
});

pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();
    for (line_num, line) in source.lines().enumerate() {
        let line_num = line_num + 1;
        if line.trim_start().starts_with('#') {
            continue;
        }

        if STRING_COMPARISON.is_match(line) {
            let diagnostic = Diagnostic::new(
                "SC2314",
                Severity::Info,
                "Consider using (( )) instead of [[ ]] for numeric comparisons".to_string(),
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
    fn test_sc2314_numeric_in_double_bracket() {
        let code = "[[ 5 == 5 ]]";
        assert_eq!(check(code).diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2314_arithmetic_ok() {
        let code = "(( 5 == 5 ))";
        assert_eq!(check(code).diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2314_variable_ok() {
        let code = "[[ $x == 5 ]]";
        assert_eq!(check(code).diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2314_comment() {
        let code = "# [[ 5 == 5 ]]";
        assert_eq!(check(code).diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2314_empty() {
        assert_eq!(check("").diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2314_normal() {
        assert_eq!(check("echo test").diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2314_not_equal() {
        let code = "[[ 10 != 5 ]]";
        assert_eq!(check(code).diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2314_string_ok() {
        let code = r#"[[ "abc" == "def" ]]"#;
        assert_eq!(check(code).diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2314_single_bracket_ok() {
        let code = "[ 5 -eq 5 ]";
        assert_eq!(check(code).diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2314_multiple() {
        let code = r#"
[[ 1 == 1 ]]
[[ 2 != 3 ]]
"#;
        assert_eq!(check(code).diagnostics.len(), 2);
    }
}
