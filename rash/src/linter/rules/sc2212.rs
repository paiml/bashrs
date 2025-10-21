// SC2212: Use [ p ] && [ q ] instead of [ p -a q ]
use crate::linter::{Diagnostic, LintResult, Severity, Span};
use once_cell::sync::Lazy;
use regex::Regex;

static DEPRECATED_A_OPERATOR: Lazy<Regex> = Lazy::new(|| {
    // Match [ ... -a ... ] in single bracket tests
    Regex::new(r"\[\s+[^\]]*\s+-a\s+[^\]]*\]").unwrap()
});

pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();
    for (line_num, line) in source.lines().enumerate() {
        let line_num = line_num + 1;
        if line.trim_start().starts_with('#') {
            continue;
        }

        // Skip [[ ]] (has different semantics)
        if line.contains("[[") {
            continue;
        }
        // Skip file test -a (different meaning: file exists)
        if line.contains("[ -a ") || line.contains("[-a ") {
            continue;
        }

        if DEPRECATED_A_OPERATOR.is_match(line) {
            let diagnostic = Diagnostic::new(
                "SC2212",
                Severity::Warning,
                "Use [ p ] && [ q ] instead of deprecated [ p -a q ] for better portability"
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
    fn test_sc2212_deprecated_a() {
        let code = r#"[ "$x" -eq 1 -a "$y" -eq 2 ]"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }
    #[test]
    fn test_sc2212_separate_ok() {
        let code = r#"[ "$x" -eq 1 ] && [ "$y" -eq 2 ]"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }
    #[test]
    fn test_sc2212_double_bracket_ok() {
        let code = r#"[[ "$x" -eq 1 -a "$y" -eq 2 ]]"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0); // Not single bracket
    }
    #[test]
    fn test_sc2212_file_test_ok() {
        let code = r#"[ -a /tmp/file ]"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0); // -a as file exists
    }
    #[test]
    fn test_sc2212_string_comparison() {
        let code = r#"[ "$a" = "x" -a "$b" = "y" ]"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }
    #[test]
    fn test_sc2212_nested_conditions() {
        let code = r#"if [ "$x" -gt 0 -a "$y" -lt 10 ]; then"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }
    #[test]
    fn test_sc2212_comment_skipped() {
        let code = r#"# [ "$a" -eq 1 -a "$b" -eq 2 ]"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }
    #[test]
    fn test_sc2212_multiple_a_operators() {
        let code = r#"[ "$a" -eq 1 -a "$b" -eq 2 -a "$c" -eq 3 ]"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }
    #[test]
    fn test_sc2212_mixed_conditions() {
        let code = r#"[ -f "$file" -a -r "$file" ]"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }
    #[test]
    fn test_sc2212_no_a_ok() {
        let code = r#"[ "$x" -eq 1 ]"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }
}
