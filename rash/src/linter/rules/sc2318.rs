// SC2318: Deprecated $[ ] syntax - use $(( )) instead
use crate::linter::{Diagnostic, LintResult, Severity, Span};
use once_cell::sync::Lazy;
use regex::Regex;

static DEPRECATED_ARITH: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"\$\[[^\]]+\]").unwrap()
});

pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();
    for (line_num, line) in source.lines().enumerate() {
        let line_num = line_num + 1;
        if line.trim_start().starts_with('#') {
            continue;
        }

        if DEPRECATED_ARITH.is_match(line) {
            let diagnostic = Diagnostic::new(
                "SC2318",
                Severity::Warning,
                "Deprecated $[ ] arithmetic syntax. Use $(( )) instead".to_string(),
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
    fn test_sc2318_deprecated_syntax() {
        let code = "result=$[5 + 3]";
        assert_eq!(check(code).diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2318_modern_ok() {
        let code = "result=$((5 + 3))";
        assert_eq!(check(code).diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2318_array_ok() {
        let code = "echo ${arr[0]}";
        assert_eq!(check(code).diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2318_comment() {
        let code = "# result=$[5 + 3]";
        assert_eq!(check(code).diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2318_empty() {
        assert_eq!(check("").diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2318_normal() {
        assert_eq!(check("echo test").diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2318_with_variable() {
        let code = "x=$[$y + 1]";
        assert_eq!(check(code).diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2318_nested() {
        let code = "result=$[$x * $[$y + 1]]";
        assert_eq!(check(code).diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2318_in_condition() {
        let code = "if [ $x -eq $[5 + 3] ]; then";
        assert_eq!(check(code).diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2318_multiple() {
        let code = r#"
a=$[1 + 2]
b=$[3 + 4]
"#;
        assert_eq!(check(code).diagnostics.len(), 2);
    }
}
