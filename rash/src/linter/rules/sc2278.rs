// SC2278: Use [[ ]] for glob/regex patterns
use crate::linter::{Diagnostic, LintResult, Severity, Span};
use once_cell::sync::Lazy;
use regex::Regex;

static SINGLE_BRACKET_GLOB: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?:^|[^\[])\[\s+[^\]]+\s+==?\s+[^\]]*[\*\?]").unwrap()
});

pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();
    for (line_num, line) in source.lines().enumerate() {
        let line_num = line_num + 1;
        if line.trim_start().starts_with('#') {
            continue;
        }

        if SINGLE_BRACKET_GLOB.is_match(line) {
            let diagnostic = Diagnostic::new(
                "SC2278",
                Severity::Warning,
                "Use [[ ]] instead of [ ] for glob pattern matching".to_string(),
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
    fn test_sc2278_glob_in_single_bracket() {
        let code = "[ $file = *.txt ]";
        assert_eq!(check(code).diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2278_double_bracket_ok() {
        let code = "[[ $file = *.txt ]]";
        assert_eq!(check(code).diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2278_literal_comparison_ok() {
        let code = r#"[ "$var" = "value" ]"#;
        assert_eq!(check(code).diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2278_comment() {
        let code = "# [ $file = *.txt ]";
        assert_eq!(check(code).diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2278_empty() {
        assert_eq!(check("").diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2278_normal() {
        assert_eq!(check("echo test").diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2278_question_mark() {
        let code = "[ $file == ?.log ]";
        assert_eq!(check(code).diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2278_numeric_ok() {
        let code = "[ $count -eq 5 ]";
        assert_eq!(check(code).diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2278_string_test_ok() {
        let code = "[ -n $var ]";
        assert_eq!(check(code).diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2278_file_test_ok() {
        let code = "[ -f $file ]";
        assert_eq!(check(code).diagnostics.len(), 0);
    }
}
