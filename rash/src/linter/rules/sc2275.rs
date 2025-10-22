// SC2275: Quote array expansions to prevent word splitting
use crate::linter::{Diagnostic, LintResult, Severity, Span};
use once_cell::sync::Lazy;
use regex::Regex;

static UNQUOTED_ARRAY: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"\$\{[a-zA-Z_][a-zA-Z0-9_]*\[@\]\}").unwrap()
});

pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();
    for (line_num, line) in source.lines().enumerate() {
        let line_num = line_num + 1;
        if line.trim_start().starts_with('#') {
            continue;
        }

        // Check for unquoted array expansions (not preceded or followed by quotes)
        for mat in UNQUOTED_ARRAY.find_iter(line) {
            let start = mat.start();
            let end = mat.end();

            // Check if it's quoted
            let before_quote = start > 0 && line.as_bytes()[start - 1] == b'"';
            let after_quote = end < line.len() && line.as_bytes()[end] == b'"';

            if !before_quote || !after_quote {
                let diagnostic = Diagnostic::new(
                    "SC2275",
                    Severity::Warning,
                    "Quote array expansions to prevent word splitting".to_string(),
                    Span::new(line_num, 1, line_num, line.len() + 1),
                );
                result.add(diagnostic);
            }
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sc2275_unquoted_array() {
        let code = "cmd ${array[@]}";
        assert_eq!(check(code).diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2275_quoted_array_ok() {
        let code = r#"cmd "${array[@]}""#;
        assert_eq!(check(code).diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2275_comment() {
        let code = "# ${array[@]}";
        assert_eq!(check(code).diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2275_empty() {
        assert_eq!(check("").diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2275_normal() {
        assert_eq!(check("echo test").diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2275_array_star() {
        let code = "cmd ${array[*]}";
        assert_eq!(check(code).diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2275_single_element_ok() {
        let code = "echo ${array[0]}";
        assert_eq!(check(code).diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2275_in_assignment_ok() {
        let code = r#"var="${array[@]}""#;
        assert_eq!(check(code).diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2275_in_for_loop() {
        let code = "for item in ${items[@]}; do";
        assert_eq!(check(code).diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2275_multiple_arrays() {
        let code = "cmd ${arr1[@]} ${arr2[@]}";
        assert_eq!(check(code).diagnostics.len(), 2);
    }
}
