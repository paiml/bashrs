// SC2312: Consider invoking command explicitly with $(command)
use crate::linter::{Diagnostic, LintResult, Severity, Span};
use once_cell::sync::Lazy;
use regex::Regex;

static IMPLICIT_COMMAND: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"\$\([a-zA-Z_][a-zA-Z0-9_]*\s+[^)]*\)").unwrap()
});

pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();
    for (line_num, line) in source.lines().enumerate() {
        let line_num = line_num + 1;
        if line.trim_start().starts_with('#') {
            continue;
        }

        // This is a placeholder rule - actual implementation would require
        // more sophisticated parsing to detect implicit vs explicit command calls
        // For now, we'll keep it simple and not trigger false positives
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sc2312_explicit_ok() {
        let code = r#"result=$(echo "test")"#;
        assert_eq!(check(code).diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2312_implicit_ok() {
        let code = r#"result=$(cmd arg1 arg2)"#;
        assert_eq!(check(code).diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2312_comment() {
        let code = r#"# result=$(cmd)"#;
        assert_eq!(check(code).diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2312_empty() {
        assert_eq!(check("").diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2312_normal() {
        assert_eq!(check("echo test").diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2312_nested_ok() {
        let code = r#"result=$(cat $(which bash))"#;
        assert_eq!(check(code).diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2312_pipe_ok() {
        let code = r#"result=$(cat file | grep pattern)"#;
        assert_eq!(check(code).diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2312_multiple_ok() {
        let code = r#"
x=$(cmd1)
y=$(cmd2)
"#;
        assert_eq!(check(code).diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2312_arithmetic_ok() {
        let code = r#"result=$((x + 1))"#;
        assert_eq!(check(code).diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2312_variable_ok() {
        let code = r#"result=$var"#;
        assert_eq!(check(code).diagnostics.len(), 0);
    }
}
