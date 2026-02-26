// SC2157: Argument to -n is always true due to literal string
//
// When using -n or -z with a literal string (not a variable), the result
// is always known at parse time. This is likely a bug.
//
// Examples:
// Bad:
//   if [ -n "literal" ]; then  # Always true
//   if [ -z "text" ]; then     # Always false
//
// Good:
//   if [ -n "$var" ]; then     # Checks if var is non-empty
//   if [ -z "$var" ]; then     # Checks if var is empty

use crate::linter::{Diagnostic, LintResult, Severity, Span};
use regex::Regex;

static N_Z_WITH_LITERAL: std::sync::LazyLock<Regex> = std::sync::LazyLock::new(|| {
    // Match: -n "literal" or -z "literal" (not a variable)
    // Matches: [ -n "text" ] or [ -z "" ] but not [ -n "$var" ]
    Regex::new(r#"\[\s+(-n|-z)\s+"([^$"]*)""#).unwrap()
});

pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();

    for (line_num, line) in source.lines().enumerate() {
        let line_num = line_num + 1;

        if line.trim_start().starts_with('#') {
            continue;
        }

        // Skip double brackets
        if line.contains("[[") {
            continue;
        }

        for cap in N_Z_WITH_LITERAL.captures_iter(line) {
            if let Some(operator) = cap.get(1) {
                if let Some(literal) = cap.get(2) {
                    let start_col = operator.start() + 1;
                    let end_col = operator.end() + 1;

                    let is_n = operator.as_str() == "-n";
                    let lit_text = literal.as_str();
                    let result_str = if is_n { "always true" } else { "always false" };

                    let diagnostic = Diagnostic::new(
                        "SC2157",
                        Severity::Warning,
                        format!(
                            "Argument to {} is {} due to literal string \"{}\"",
                            operator.as_str(),
                            result_str,
                            if lit_text.len() > 20 {
                                format!("{}...", &lit_text[..20])
                            } else {
                                lit_text.to_string()
                            }
                        ),
                        Span::new(line_num, start_col, line_num, end_col),
                    );

                    result.add(diagnostic);
                }
            }
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sc2157_n_with_literal() {
        let code = r#"if [ -n "literal" ]; then"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
        assert_eq!(result.diagnostics[0].code, "SC2157");
        assert_eq!(result.diagnostics[0].severity, Severity::Warning);
        assert!(result.diagnostics[0].message.contains("-n"));
        assert!(result.diagnostics[0].message.contains("always true"));
    }

    #[test]
    fn test_sc2157_z_with_literal() {
        let code = r#"if [ -z "text" ]; then"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
        assert!(result.diagnostics[0].message.contains("-z"));
        assert!(result.diagnostics[0].message.contains("always false"));
    }

    #[test]
    fn test_sc2157_n_with_variable_ok() {
        let code = r#"if [ -n "$var" ]; then"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2157_z_with_variable_ok() {
        let code = r#"if [ -z "$var" ]; then"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2157_double_bracket_ok() {
        let code = r#"if [[ -n "literal" ]]; then"#;
        let result = check(code);
        // Double brackets have different semantics
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2157_empty_string() {
        let code = r#"if [ -n "" ]; then"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2157_no_quotes() {
        let code = r#"if [ -n $var ]; then"#;
        let result = check(code);
        // No quotes means variable expansion
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2157_single_quotes() {
        let code = r#"if [ -n 'literal' ]; then"#;
        let result = check(code);
        // Our regex only catches double quotes, single quotes OK for now
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2157_multiple_on_line() {
        let code = r#"if [ -n "a" ] && [ -z "b" ]; then"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 2);
    }

    #[test]
    fn test_sc2157_long_literal() {
        let code = r#"if [ -n "this is a very long literal string that will be truncated" ]; then"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
        assert!(result.diagnostics[0].message.contains("..."));
    }
}
