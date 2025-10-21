// SC2210: Don't use arithmetic shortcuts like x=++y
use crate::linter::{Diagnostic, LintResult, Severity, Span};
use once_cell::sync::Lazy;
use regex::Regex;

static ARITHMETIC_SHORTCUT: Lazy<Regex> = Lazy::new(|| {
    // Match x=++y or x=--y (C-style prefix operators in assignment)
    Regex::new(r"\w+\s*=\s*(\+\+|--)\w+").unwrap()
});

pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();
    for (line_num, line) in source.lines().enumerate() {
        let line_num = line_num + 1;
        if line.trim_start().starts_with('#') {
            continue;
        }

        // Skip if in arithmetic context
        if line.contains("$((") || line.contains("((") {
            continue;
        }

        if ARITHMETIC_SHORTCUT.is_match(line) {
            let diagnostic = Diagnostic::new(
                "SC2210",
                Severity::Error,
                "Prefix operators (++/--) only work in arithmetic context. Use x=$((y + 1))"
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
    fn test_sc2210_prefix_increment() {
        let code = r#"x=++y"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }
    #[test]
    fn test_sc2210_prefix_decrement() {
        let code = r#"count=--total"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }
    #[test]
    fn test_sc2210_arithmetic_context_ok() {
        let code = r#"x=$((++y))"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }
    #[test]
    fn test_sc2210_double_paren_ok() {
        let code = r#"((x=++y))"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }
    #[test]
    fn test_sc2210_normal_assignment_ok() {
        let code = r#"x=$y"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }
    #[test]
    fn test_sc2210_addition_ok() {
        let code = r#"x=$((y + 1))"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }
    #[test]
    fn test_sc2210_spaces() {
        let code = r#"val = ++count"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }
    #[test]
    fn test_sc2210_comment_skipped() {
        let code = r#"# x=++y"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }
    #[test]
    fn test_sc2210_string_literal_ok() {
        let code = r#"text="++value""#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0); // String, not arithmetic
    }
    #[test]
    fn test_sc2210_underscore_var() {
        let code = r#"_new=++_old"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }
}
