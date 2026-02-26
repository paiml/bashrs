// SC2132: For loop variable redeclared as read-only
//
// If a variable is declared readonly, you can't use it as a for loop variable.
// The for loop will try to reassign it and fail.
//
// Examples:
// Bad:
//   readonly VAR
//   for VAR in 1 2 3; do    // Error: VAR is readonly
//
//   declare -r VAR=5
//   for VAR in a b c; do    // Error: VAR is readonly
//
// Good:
//   readonly CONST
//   for VAR in 1 2 3; do    // Different variable
//
//   for VAR in 1 2 3; do
//       readonly VAR         // Readonly inside loop (unusual but OK)
//   done
//
// Impact: Runtime error - script will fail

use crate::linter::{Diagnostic, LintResult, Severity, Span};
use regex::Regex;
use std::collections::HashSet;

static READONLY_DECL: std::sync::LazyLock<Regex> = std::sync::LazyLock::new(|| {
    // Match: readonly VAR or declare -r VAR
    Regex::new(r"\b(readonly|declare\s+-[a-zA-Z]*r)\s+([a-zA-Z_][a-zA-Z0-9_]*)").unwrap()
});

static FOR_LOOP: std::sync::LazyLock<Regex> = std::sync::LazyLock::new(|| {
    // Match: for VAR in ...
    Regex::new(r"\bfor\s+([a-zA-Z_][a-zA-Z0-9_]*)\s+in\b").unwrap()
});

pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();
    let mut readonly_vars: HashSet<String> = HashSet::new();

    // First pass: collect all readonly variables
    for line in source.lines() {
        if line.trim_start().starts_with('#') {
            continue;
        }

        for cap in READONLY_DECL.captures_iter(line) {
            if let Some(var_match) = cap.get(2) {
                readonly_vars.insert(var_match.as_str().to_string());
            }
        }
    }

    // Second pass: check for loops using readonly variables
    for (line_num, line) in source.lines().enumerate() {
        let line_num = line_num + 1;

        if line.trim_start().starts_with('#') {
            continue;
        }

        for cap in FOR_LOOP.captures_iter(line) {
            if let Some(var_match) = cap.get(1) {
                let var_name = var_match.as_str();

                if readonly_vars.contains(var_name) {
                    let start_col = var_match.start() + 1;
                    let end_col = var_match.end() + 1;

                    let diagnostic = Diagnostic::new(
                        "SC2132",
                        Severity::Error,
                        format!(
                            "'{}' was declared readonly. Can't use it as for loop variable",
                            var_name
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
    fn test_sc2132_readonly_in_for() {
        let code = r#"
readonly VAR
for VAR in 1 2 3; do
    echo "$VAR"
done
"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2132_declare_readonly() {
        let code = r#"
declare -r VAR=5
for VAR in a b c; do
    echo "$VAR"
done
"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2132_different_var_ok() {
        let code = r#"
readonly CONST
for VAR in 1 2 3; do
    echo "$VAR"
done
"#;
        let result = check(code);
        // Different variables
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2132_no_readonly_ok() {
        let code = r#"
for VAR in 1 2 3; do
    echo "$VAR"
done
"#;
        let result = check(code);
        // No readonly declaration
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2132_comment_ok() {
        let code = r#"
# readonly VAR
for VAR in 1 2 3; do
    echo "$VAR"
done
"#;
        let result = check(code);
        // Commented readonly doesn't count
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2132_declare_r() {
        let code = r#"
declare -r MYVAR
for MYVAR in x y z; do
    echo "$MYVAR"
done
"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2132_multiple_flags() {
        let code = r#"
declare -ir VAR=10
for VAR in 1 2 3; do
    echo "$VAR"
done
"#;
        let result = check(code);
        // -ir includes readonly
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2132_case_sensitive() {
        let code = r#"
readonly var
for VAR in 1 2 3; do
    echo "$VAR"
done
"#;
        let result = check(code);
        // var vs VAR are different
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2132_underscore_var() {
        let code = r#"
readonly _PRIVATE
for _PRIVATE in items; do
    echo "$_PRIVATE"
done
"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2132_order_matters() {
        let code = r#"
for VAR in 1 2 3; do
    echo "$VAR"
done
readonly VAR
"#;
        let result = check(code);
        // readonly after for loop still detected (stateful analysis)
        assert_eq!(result.diagnostics.len(), 1);
    }
}
