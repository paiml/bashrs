// SC2219: Prefer (( expr )) to let expr
use crate::linter::{Diagnostic, LintResult, Severity, Span};
use once_cell::sync::Lazy;
use regex::Regex;

static LET_COMMAND: Lazy<Regex> = Lazy::new(|| {
    // Match 'let' command with expressions
    Regex::new(r"\blet\s+\S+\s*[+\-*/=]").unwrap()
});

pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();
    for (line_num, line) in source.lines().enumerate() {
        let line_num = line_num + 1;
        if line.trim_start().starts_with('#') {
            continue;
        }
        // Skip echo commands (string literals)
        if line.trim_start().starts_with("echo ") {
            continue;
        }

        if LET_COMMAND.is_match(line) {
            let diagnostic = Diagnostic::new(
                "SC2219",
                Severity::Info,
                "Prefer (( expr )) to 'let expr' for arithmetic - more readable and consistent"
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
    fn test_sc2219_let_increment() {
        let code = r#"let count=count+1"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }
    #[test]
    fn test_sc2219_double_paren_ok() {
        let code = r#"(( count = count + 1 ))"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }
    #[test]
    fn test_sc2219_let_assignment() {
        let code = r#"let x=5"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }
    #[test]
    fn test_sc2219_let_subtraction() {
        let code = r#"let y=y-2"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }
    #[test]
    fn test_sc2219_let_multiply() {
        let code = r#"let z=z*3"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }
    #[test]
    fn test_sc2219_let_divide() {
        let code = r#"let a=a/2"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }
    #[test]
    fn test_sc2219_comment_skipped() {
        let code = r#"# let x=x+1"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }
    #[test]
    fn test_sc2219_string_literal() {
        let code = r#"echo "let x=y+1""#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0); // In string, not command
    }
    #[test]
    fn test_sc2219_let_compound() {
        let code = r#"let "x = y + z""#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }
    #[test]
    fn test_sc2219_arithmetic_ok() {
        let code = r#"x=$((x + 1))"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }
}
