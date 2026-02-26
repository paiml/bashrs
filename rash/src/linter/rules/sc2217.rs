// SC2217: Use [ p ] || [ q ] instead of [ p -o q ]
use crate::linter::{Diagnostic, LintResult, Severity, Span};
use regex::Regex;

static DEPRECATED_O_OPERATOR: std::sync::LazyLock<Regex> = std::sync::LazyLock::new(|| {
    // Match [ ... -o ... ] in single bracket tests
    Regex::new(r"\[\s+[^\]]*\s+-o\s+[^\]]*\]").unwrap()
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
        // Skip set -o / shopt -o (shell options)
        if line.contains("set -o") || line.contains("shopt -o") {
            continue;
        }

        if DEPRECATED_O_OPERATOR.is_match(line) {
            let diagnostic = Diagnostic::new(
                "SC2217",
                Severity::Warning,
                "Use [ p ] || [ q ] instead of deprecated [ p -o q ] for better portability"
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
    fn test_sc2217_deprecated_o() {
        let code = r#"[ "$x" -eq 1 -o "$y" -eq 2 ]"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }
    #[test]
    fn test_sc2217_separate_ok() {
        let code = r#"[ "$x" -eq 1 ] || [ "$y" -eq 2 ]"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }
    #[test]
    fn test_sc2217_double_bracket_ok() {
        let code = r#"[[ "$x" -eq 1 -o "$y" -eq 2 ]]"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0); // Not single bracket
    }
    #[test]
    fn test_sc2217_set_option_ok() {
        let code = r#"set -o errexit"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0); // Shell option
    }
    #[test]
    fn test_sc2217_shopt_ok() {
        let code = r#"shopt -o nullglob"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0); // Shell option
    }
    #[test]
    fn test_sc2217_string_comparison() {
        let code = r#"[ "$a" = "x" -o "$b" = "y" ]"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }
    #[test]
    fn test_sc2217_nested_conditions() {
        let code = r#"if [ "$x" -gt 0 -o "$y" -lt 10 ]; then"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }
    #[test]
    fn test_sc2217_comment_skipped() {
        let code = r#"# [ "$a" -eq 1 -o "$b" -eq 2 ]"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }
    #[test]
    fn test_sc2217_file_tests() {
        let code = r#"[ -f "$file" -o -d "$dir" ]"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }
    #[test]
    fn test_sc2217_no_o_ok() {
        let code = r#"[ "$x" -eq 1 ]"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }
}
