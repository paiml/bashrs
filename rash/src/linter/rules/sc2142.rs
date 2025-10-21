// SC2142: Aliases can't use positional parameters. Use a function.
//
// Aliases are simple text replacements and can't handle arguments like $1, $2.
// Use functions instead for parameterized commands.
//
// Examples:
// Bad:
//   alias greet="echo Hello $1"          // $1 won't work in alias
//   alias copy="cp $1 $2"                // Parameters don't work
//   alias show='echo First: $1'          // Even in single quotes
//
// Good:
//   greet() { echo "Hello $1"; }         // Function with parameter
//   copy() { cp "$1" "$2"; }             // Function for multi-arg
//   show() { echo "First: $1"; }         // Functions handle args
//
// Impact: Command won't work as expected - parameters will be empty

use crate::linter::{Diagnostic, LintResult, Severity, Span};
use once_cell::sync::Lazy;
use regex::Regex;

static ALIAS_WITH_POSITIONAL_DOUBLE: Lazy<Regex> = Lazy::new(|| {
    // Match: alias name="..$1.."
    Regex::new(r#"\balias\s+\w+="[^"]*\$[0-9@*][^"]*""#).unwrap()
});

static ALIAS_WITH_POSITIONAL_SINGLE: Lazy<Regex> = Lazy::new(|| {
    // Match: alias name='..$1..'
    Regex::new(r"\balias\s+\w+='[^']*\$[0-9@*][^']*'").unwrap()
});

pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();

    for (line_num, line) in source.lines().enumerate() {
        let line_num = line_num + 1;

        if line.trim_start().starts_with('#') {
            continue;
        }

        // Check for aliases with positional parameters in double quotes
        for mat in ALIAS_WITH_POSITIONAL_DOUBLE.find_iter(line) {
            let start_col = mat.start() + 1;
            let end_col = mat.end() + 1;

            let diagnostic = Diagnostic::new(
                "SC2142",
                Severity::Error,
                "Aliases can't use positional parameters. Use a function instead".to_string(),
                Span::new(line_num, start_col, line_num, end_col),
            );

            result.add(diagnostic);
        }

        // Check for aliases with positional parameters in single quotes
        for mat in ALIAS_WITH_POSITIONAL_SINGLE.find_iter(line) {
            let start_col = mat.start() + 1;
            let end_col = mat.end() + 1;

            let diagnostic = Diagnostic::new(
                "SC2142",
                Severity::Error,
                "Aliases can't use positional parameters. Use a function instead".to_string(),
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
    fn test_sc2142_alias_with_dollar1() {
        let code = r#"alias greet="echo Hello $1""#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
        assert!(result.diagnostics[0].message.contains("function"));
    }

    #[test]
    fn test_sc2142_alias_with_dollar2() {
        let code = r#"alias copy="cp $1 $2""#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2142_single_quotes() {
        let code = "alias show='echo First: $1'";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2142_function_ok() {
        let code = "greet() { echo \"Hello $1\"; }";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2142_alias_no_param_ok() {
        let code = r#"alias ll="ls -la""#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2142_comment_ok() {
        let code = r#"# alias greet="echo Hello $1""#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2142_dollar_at() {
        let code = r#"alias run="echo $@""#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2142_dollar_star() {
        let code = r#"alias all="echo $*""#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2142_dollar0() {
        let code = r#"alias name="echo $0""#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2142_multiple() {
        let code = r#"
alias greet="echo Hello $1"
alias copy="cp $1 $2"
"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 2);
    }
}
