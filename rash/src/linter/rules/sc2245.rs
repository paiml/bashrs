// SC2245: Arithmetic contexts don't require $ prefix
use crate::linter::{Diagnostic, LintResult, Severity, Span};
use once_cell::sync::Lazy;
use regex::Regex;

static DOLLAR_IN_ARITHMETIC: Lazy<Regex> = Lazy::new(|| {
    // Match: $(( $var )) or (( $var ))
    Regex::new(r"\(\(\s*\$\w+").unwrap()
});

pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();
    for (line_num, line) in source.lines().enumerate() {
        let line_num = line_num + 1;
        if line.trim_start().starts_with('#') {
            continue;
        }

        if DOLLAR_IN_ARITHMETIC.is_match(line) {
            let diagnostic = Diagnostic::new(
                "SC2245",
                Severity::Info,
                "$ is unnecessary in arithmetic contexts: use (( var )) instead of (( $var ))"
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
    fn test_sc2245_dollar_in_arithmetic() {
        let code = "(( $count + 1 ))";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }
    #[test]
    fn test_sc2245_no_dollar_ok() {
        let code = "(( count + 1 ))";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }
    #[test]
    fn test_sc2245_dollar_cmdsub() {
        let code = "result=$(( $a + $b ))";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }
    #[test]
    fn test_sc2245_no_dollar_cmdsub_ok() {
        let code = "result=$(( a + b ))";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }
    #[test]
    fn test_sc2245_comment_skipped() {
        let code = "# (( $var ))";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }
    #[test]
    fn test_sc2245_increment() {
        let code = "(( $i++ ))";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }
    #[test]
    fn test_sc2245_assignment() {
        let code = "(( $x = 5 ))";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }
    #[test]
    fn test_sc2245_no_code() {
        let code = "";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }
    #[test]
    fn test_sc2245_normal_command() {
        let code = "echo test";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }
    #[test]
    fn test_sc2245_complex_expr() {
        let code = "(( $a * $b + $c ))";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1); // Detects first $
    }
}
