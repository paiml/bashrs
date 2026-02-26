// SC2290: Remove $ from index in ${array[$i]}
use crate::linter::{Diagnostic, LintResult, Severity, Span};
use regex::Regex;

static ARRAY_INDEX_DOLLAR: std::sync::LazyLock<Regex> =
    std::sync::LazyLock::new(|| Regex::new(r"\$\{[a-zA-Z_][a-zA-Z0-9_]*\[\$[a-zA-Z_]").unwrap());

pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();
    for (line_num, line) in source.lines().enumerate() {
        let line_num = line_num + 1;
        if line.trim_start().starts_with('#') {
            continue;
        }

        if ARRAY_INDEX_DOLLAR.is_match(line) {
            let diagnostic = Diagnostic::new(
                "SC2290",
                Severity::Info,
                "Remove $ from index variables in array subscripts: ${array[i]} not ${array[$i]}"
                    .to_string(),
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
    fn test_sc2290_dollar_in_index() {
        let code = "echo ${array[$i]}";
        assert_eq!(check(code).diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2290_correct_ok() {
        let code = "echo ${array[i]}";
        assert_eq!(check(code).diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2290_literal_index_ok() {
        let code = "echo ${array[0]}";
        assert_eq!(check(code).diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2290_comment() {
        let code = "# ${array[$i]}";
        assert_eq!(check(code).diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2290_empty() {
        assert_eq!(check("").diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2290_normal() {
        assert_eq!(check("echo test").diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2290_at_ok() {
        let code = "echo ${array[@]}";
        assert_eq!(check(code).diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2290_star_ok() {
        let code = "echo ${array[*]}";
        assert_eq!(check(code).diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2290_arithmetic_index_ok() {
        let code = "echo ${array[i+1]}";
        assert_eq!(check(code).diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2290_nested_index() {
        let code = "echo ${matrix[$row]}";
        assert_eq!(check(code).diagnostics.len(), 1);
    }
}
