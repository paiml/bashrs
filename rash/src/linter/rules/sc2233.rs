// SC2233: Remove spaces around operators in arithmetic context
use crate::linter::{Diagnostic, LintResult, Severity, Span};
use regex::Regex;

static SPACED_OPERATOR: std::sync::LazyLock<Regex> = std::sync::LazyLock::new(|| {
    // Match: $((expr + expr)) with spaces around operator
    Regex::new(r"\$\(\([^)]+\s[+\-*/]\s[^)]+\)\)").unwrap()
});

pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();
    for (line_num, line) in source.lines().enumerate() {
        let line_num = line_num + 1;
        if line.trim_start().starts_with('#') {
            continue;
        }

        if SPACED_OPERATOR.is_match(line) {
            let diagnostic = Diagnostic::new(
                "SC2233",
                Severity::Info,
                "Spaces around operators are fine in arithmetic but unusual. Consider removing for consistency"
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
    fn test_sc2233_spaced_operator() {
        let code = r#"result=$((a + b))"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }
    #[test]
    fn test_sc2233_no_space_ok() {
        let code = r#"result=$((a+b))"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }
    #[test]
    fn test_sc2233_subtraction() {
        let code = r#"val=$((x - y))"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }
    #[test]
    fn test_sc2233_multiplication() {
        let code = r#"prod=$((m * n))"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }
    #[test]
    fn test_sc2233_division() {
        let code = r#"quot=$((p / q))"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }
    #[test]
    fn test_sc2233_comment_skipped() {
        let code = "# $((a + b))";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }
    #[test]
    fn test_sc2233_double_paren() {
        let code = r#"(( count = count + 1 ))"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0); // Not command substitution
    }
    #[test]
    fn test_sc2233_no_code() {
        let code = "";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }
    #[test]
    fn test_sc2233_normal_command() {
        let code = "echo test";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }
    #[test]
    fn test_sc2233_complex_expr() {
        let code = r#"res=$((a + b * c))"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }
}
