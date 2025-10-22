// SC2266: Prefer [[ ]] over [ ] for regex/glob matching
use crate::linter::{Diagnostic, LintResult, Severity, Span};
use once_cell::sync::Lazy;
use regex::Regex;

static SINGLE_BRACKET_PATTERN: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?:^|[^\[])\[\s+[^\]]+\s+(=~|==.*[\*\?]|!=.*[\*\?])").unwrap()
});

pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();
    for (line_num, line) in source.lines().enumerate() {
        let line_num = line_num + 1;
        if line.trim_start().starts_with('#') {
            continue;
        }

        if SINGLE_BRACKET_PATTERN.is_match(line) {
            let diagnostic = Diagnostic::new(
                "SC2266",
                Severity::Warning,
                "Prefer [[ ]] over [ ] for regex/glob matching".to_string(),
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
    fn test_sc2266_regex_in_single_bracket() {
        let code = r#"[ "$var" =~ pattern ]"#;
        assert_eq!(check(code).diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2266_glob_in_single_bracket() {
        let code = r#"[ "$file" == *.txt ]"#;
        assert_eq!(check(code).diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2266_double_bracket_ok() {
        let code = r#"[[ "$var" =~ pattern ]]"#;
        assert_eq!(check(code).diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2266_simple_comparison_ok() {
        let code = r#"[ "$x" = "value" ]"#;
        assert_eq!(check(code).diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2266_comment() {
        let code = r#"# [ "$var" =~ pattern ]"#;
        assert_eq!(check(code).diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2266_empty() {
        assert_eq!(check("").diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2266_normal_code() {
        assert_eq!(check("echo test").diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2266_glob_not_equal() {
        let code = r#"[ "$file" != *.log ]"#;
        assert_eq!(check(code).diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2266_numeric_comparison_ok() {
        let code = r#"[ "$count" -gt 5 ]"#;
        assert_eq!(check(code).diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2266_string_equality_ok() {
        let code = r#"[ "$var" = "literal" ]"#;
        assert_eq!(check(code).diagnostics.len(), 0);
    }
}
