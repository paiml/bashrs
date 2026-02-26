// SC2158: [ false ] is true. Remove the brackets or use `! false`
//
// The `false` command returns exit status 1, but when used inside test brackets
// like `[ false ]`, it's testing if the STRING "false" is non-empty, which is always true.
//
// Examples:
// Bad:
//   if [ false ]; then  # Always true! Tests if string "false" is non-empty
//   if [ true ]; then   # Also always true
//
// Good:
//   if false; then      # Correctly tests exit status
//   if ! false; then    # Correctly inverts exit status
//   if true; then       # Correctly tests exit status

use crate::linter::{Diagnostic, LintResult, Severity, Span};
use regex::Regex;

static LITERAL_BOOL_IN_BRACKETS: std::sync::LazyLock<Regex> = std::sync::LazyLock::new(|| {
    // Match: [ false ] or [ true ] (literal commands treated as strings)
    // Only match single brackets, not [[
    Regex::new(r"(?:^|[^\[])\[\s+(true|false)\s+\]").unwrap()
});

pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();

    for (line_num, line) in source.lines().enumerate() {
        let line_num = line_num + 1;

        if line.trim_start().starts_with('#') {
            continue;
        }

        for cap in LITERAL_BOOL_IN_BRACKETS.captures_iter(line) {
            if let Some(command) = cap.get(1) {
                let start_col = cap.get(0).unwrap().start() + 1;
                let end_col = cap.get(0).unwrap().end() + 1;

                let cmd_text = command.as_str();
                let expected = if cmd_text == "false" {
                    "always true"
                } else {
                    "always false"
                };

                let diagnostic = Diagnostic::new(
                    "SC2158",
                    Severity::Warning,
                    format!(
                        "[ {} ] is {}. Remove the brackets or use the command directly",
                        cmd_text, expected
                    ),
                    Span::new(line_num, start_col, line_num, end_col),
                );

                result.add(diagnostic);
            }
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sc2158_false_in_brackets() {
        let code = r#"if [ false ]; then"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
        assert_eq!(result.diagnostics[0].code, "SC2158");
        assert_eq!(result.diagnostics[0].severity, Severity::Warning);
        assert!(result.diagnostics[0].message.contains("false"));
        assert!(result.diagnostics[0].message.contains("always true"));
    }

    #[test]
    fn test_sc2158_true_in_brackets() {
        let code = r#"if [ true ]; then"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
        assert!(result.diagnostics[0].message.contains("true"));
    }

    #[test]
    fn test_sc2158_false_without_brackets_ok() {
        let code = r#"if false; then"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2158_true_without_brackets_ok() {
        let code = r#"if true; then"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2158_negated_false_ok() {
        let code = r#"if ! false; then"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2158_double_brackets() {
        let code = r#"if [[ false ]]; then"#;
        let result = check(code);
        // Our regex only catches single brackets
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2158_variable_named_false_ok() {
        let code = r#"if [ "$false" ]; then"#;
        let result = check(code);
        // Variable reference, not literal
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2158_command_in_brackets_ok() {
        let code = r#"if [ -f false ]; then"#;
        let result = check(code);
        // Testing file named "false", not literal command
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2158_multiple_on_line() {
        let code = r#"if [ false ] || [ true ]; then"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 2);
    }

    #[test]
    fn test_sc2158_false_with_extra_spaces() {
        let code = r#"if [  false  ]; then"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }
}
