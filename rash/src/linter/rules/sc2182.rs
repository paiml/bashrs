// SC2182: This printf format string has no variables. Use 'echo' instead.
//
// printf with constant string is unnecessarily complex. Use echo.
//
// Examples:
// Bad:
//   printf "hello\n"             // No formatting needed
//   printf "constant string"     // Use echo instead
//
// Good:
//   echo "hello"                 // Simpler
//   printf "%s\n" "$var"         // Format needed
//   printf "Value: %d\n" 42      // Format needed
//
// Impact: Unnecessary complexity

use crate::linter::{Diagnostic, LintResult, Severity, Span};
use regex::Regex;

static PRINTF_NO_VARS: std::sync::LazyLock<Regex> =
    std::sync::LazyLock::new(|| Regex::new(r#"\bprintf\s+"[^"%]*\\n"#).unwrap());

pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();

    for (line_num, line) in source.lines().enumerate() {
        let line_num = line_num + 1;

        if line.trim_start().starts_with('#') {
            continue;
        }

        for mat in PRINTF_NO_VARS.find_iter(line) {
            let matched = mat.as_str();

            // Skip if it contains format specifiers
            if matched.contains("%s")
                || matched.contains("%d")
                || matched.contains("%f")
                || matched.contains("%x")
            {
                continue;
            }

            let start_col = mat.start() + 1;
            let end_col = mat.end() + 1;

            let diagnostic = Diagnostic::new(
                "SC2182",
                Severity::Info,
                "This printf has no format specifiers. Consider using 'echo' instead".to_string(),
                Span::new(line_num, start_col, line_num, end_col),
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
    fn test_sc2182_printf_no_vars() {
        let code = r#"printf "hello\n""#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2182_printf_with_var_ok() {
        let code = r#"printf "%s\n" "$var""#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2182_printf_format_ok() {
        let code = r#"printf "Value: %d\n" 42"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2182_echo_ok() {
        let code = r#"echo "hello""#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2182_comment_ok() {
        let code = r#"# printf "hello\n""#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2182_multiple() {
        let code = r#"
printf "line1\n"
printf "line2\n"
"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 2);
    }

    #[test]
    fn test_sc2182_printf_escape_ok() {
        let code = r#"printf "\033[1mBold\033[0m\n""#;
        let result = check(code);
        // ANSI codes - printf is appropriate
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2182_printf_hex_ok() {
        let code = r#"printf "%x\n" 255"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2182_printf_float_ok() {
        let code = r#"printf "%.2f\n" 3.14159"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2182_printf_no_newline() {
        let code = r#"printf "hello""#;
        let result = check(code);
        // No \n so pattern won't match
        assert_eq!(result.diagnostics.len(), 0);
    }
}
