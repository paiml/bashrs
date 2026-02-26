// SC1086: Don't use `$` on for loop variable
//
// In a `for` loop declaration, the variable name should not be prefixed with `$`.
// The `$` is only for expansion, not declaration.
//
// Examples:
// Bad:
//   for $i in 1 2 3; do echo $i; done
//   for $file in *.txt; do cat $file; done
//
// Good:
//   for i in 1 2 3; do echo $i; done
//   for file in *.txt; do cat $file; done

use crate::linter::{Diagnostic, LintResult, Severity, Span};
use regex::Regex;

/// Matches `for $var in`
static FOR_DOLLAR_VAR: std::sync::LazyLock<Regex> =
    std::sync::LazyLock::new(|| Regex::new(r"\bfor\s+\$(\w+)\s+in\b").unwrap());

pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();

    for (line_num, line) in source.lines().enumerate() {
        let line_num = line_num + 1;
        let trimmed = line.trim_start();

        if trimmed.starts_with('#') {
            continue;
        }

        for caps in FOR_DOLLAR_VAR.captures_iter(line) {
            let full = caps.get(0).unwrap();
            let var_name = caps.get(1).unwrap().as_str();
            let start_col = full.start() + 1;
            let end_col = full.end() + 1;

            let diagnostic = Diagnostic::new(
                "SC1086",
                Severity::Error,
                format!(
                    "Don't use `$` on the for loop variable. Use `for {}` instead of `for ${}`.",
                    var_name, var_name
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
    fn test_sc1086_dollar_in_for() {
        let code = "for $i in 1 2 3; do echo $i; done";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
        assert_eq!(result.diagnostics[0].code, "SC1086");
        assert_eq!(result.diagnostics[0].severity, Severity::Error);
        assert!(result.diagnostics[0].message.contains("for i"));
    }

    #[test]
    fn test_sc1086_dollar_file_var() {
        let code = "for $file in *.txt; do cat $file; done";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
        assert!(result.diagnostics[0].message.contains("for file"));
    }

    #[test]
    fn test_sc1086_normal_for_ok() {
        let code = "for i in 1 2 3; do echo $i; done";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc1086_c_style_for_ok() {
        let code = "for ((i=0; i<10; i++)); do echo $i; done";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc1086_comment_ok() {
        let code = "# for $i in 1 2 3; do echo $i; done";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }
}
