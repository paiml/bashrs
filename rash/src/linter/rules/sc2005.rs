// SC2005: Useless echo before command substitution
//
// When you use echo before command substitution, it's redundant. The command
// substitution already returns a string, so echo adds no value and makes code less efficient.
//
// Examples:
// Bad:
//   result=$(echo $value)          // Useless echo
//   output=$(echo "$var")           // Just use the variable
//   file=$(echo /path/to/file)      // Useless echo
//
// Good:
//   result=$value                   // Direct assignment
//   output=$var                     // Direct assignment
//   file=/path/to/file              // Direct assignment
//   result=$(calculate)             // Meaningful command
//
// Note: echo is only useful when you need its features like newline handling,
// but for simple variable assignment it's redundant.

use crate::linter::{Diagnostic, LintResult, Severity, Span};
use regex::Regex;

static USELESS_ECHO: std::sync::LazyLock<Regex> = std::sync::LazyLock::new(|| {
    // Match: $(echo $var) or $(echo "$var") or $(echo something)
    // Capture the content after echo
    Regex::new(r"\$\(\s*echo\s+([^)]+)\)").unwrap()
});

pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();

    for (line_num, line) in source.lines().enumerate() {
        let line_num = line_num + 1;

        if line.trim_start().starts_with('#') {
            continue;
        }

        // Look for $(echo ...) patterns
        for cap in USELESS_ECHO.captures_iter(line) {
            let full_match = cap.get(0).unwrap().as_str();
            let echo_content = cap.get(1).unwrap().as_str().trim();

            // Skip if echo has flags (-n, -e, etc.)
            if echo_content.starts_with('-') {
                continue;
            }

            // Skip if multiple arguments (might be intentional formatting)
            let arg_count = echo_content.split_whitespace().count();
            if arg_count > 1 && !echo_content.starts_with('$') && !echo_content.starts_with('"') {
                continue;
            }

            let start_col = line.find(full_match).unwrap_or(0) + 1;
            let end_col = start_col + full_match.len();

            let diagnostic = Diagnostic::new(
                "SC2005",
                Severity::Info,
                format!(
                    "Useless 'echo' in command substitution. Use '{}' directly instead of '$(echo {})'",
                    echo_content, echo_content
                ),
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
    fn test_sc2005_useless_echo_var() {
        let code = r#"result=$(echo $value)"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
        assert_eq!(result.diagnostics[0].code, "SC2005");
        assert_eq!(result.diagnostics[0].severity, Severity::Info);
        assert!(result.diagnostics[0].message.contains("Useless"));
    }

    #[test]
    fn test_sc2005_useless_echo_quoted() {
        let code = r#"output=$(echo "$var")"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2005_useless_echo_literal() {
        let code = r#"file=$(echo /path/to/file)"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2005_useless_echo_simple_string() {
        let code = r#"msg=$(echo "hello")"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2005_echo_with_flag_ok() {
        let code = r#"result=$(echo -n $value)"#;
        let result = check(code);
        // -n flag is meaningful, not useless
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2005_echo_with_e_flag_ok() {
        let code = r#"result=$(echo -e "hello\nworld")"#;
        let result = check(code);
        // -e flag is meaningful
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2005_real_command_ok() {
        let code = r#"result=$(calculate $value)"#;
        let result = check(code);
        // Not echo, so no warning
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2005_echo_multiple_words() {
        let code = r#"msg=$(echo hello world)"#;
        let result = check(code);
        // Multiple literal words might be intentional spacing
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2005_multiple_useless_echos() {
        let code = r#"
a=$(echo $x)
b=$(echo $y)
"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 2);
    }

    #[test]
    fn test_sc2005_echo_in_backticks() {
        let code = r#"result=`echo $value`"#;
        let result = check(code);
        // Pattern doesn't match backticks (different style)
        assert_eq!(result.diagnostics.len(), 0);
    }
}
