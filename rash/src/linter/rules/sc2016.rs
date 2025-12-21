// SC2016: Expressions don't expand in single quotes, use double quotes for that
//
// Single quotes preserve literal strings - variables and command substitutions
// don't expand inside them. Use double quotes if you want expansion.
//
// Examples:
// Bad:
//   echo 'Hello $USER'              // Prints literal "$USER"
//   msg='Value: $value'             // $value doesn't expand
//   cmd='$(date)'                   // Command doesn't run
//
// Good:
//   echo "Hello $USER"              // Variable expands
//   msg="Value: $value"             // Variable expands
//   cmd="$(date)"                   // Command runs
//   literal='$50 per item'          // OK - intentional literal
//
// Note: This rule detects likely mistakes where users expect expansion
// but use single quotes. Intentional literals with $ are acceptable.

use crate::linter::{Diagnostic, LintResult, Severity, Span};
use once_cell::sync::Lazy;
use regex::Regex;

#[allow(clippy::unwrap_used)] // Compile-time regex, panic on invalid pattern is acceptable
static SINGLE_QUOTE_WITH_VAR: Lazy<Regex> = Lazy::new(|| {
    // Match: '...$var...' or '...${var}...' or '...$(cmd)...'
    Regex::new(r"'[^']*(\$[a-zA-Z_][a-zA-Z0-9_]*|\$\{[^}]+\}|\$\([^)]+\))[^']*'").unwrap()
});

/// F025: Check if the variable in single quotes is wrapped in double quotes
/// Pattern: '"$var"' or '${var}' inside quotes means it's likely documentation/example
fn is_documentation_pattern(matched: &str) -> bool {
    // Check if the variable is wrapped in double quotes (e.g., '"$var"')
    // This is a common pattern for showing shell syntax as literal text
    if matched.contains("\"$") || matched.contains("\"${") || matched.contains("\"$(") {
        return true;
    }

    // Check for JSON-like patterns (e.g., '{"key": "$val"}')
    if matched.contains("{") && matched.contains(":") && matched.contains("\"$") {
        return true;
    }

    false
}

pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();

    for (line_num, line) in source.lines().enumerate() {
        let line_num = line_num + 1;

        if line.trim_start().starts_with('#') {
            continue;
        }

        // Look for single-quoted strings with $ expressions
        for m in SINGLE_QUOTE_WITH_VAR.find_iter(line) {
            let matched = m.as_str();
            let start_col = m.start() + 1;
            let end_col = m.end() + 1;

            // Skip some common false positives
            // Skip if it's clearly a price/money (like '$50')
            if matched.contains("$0") || matched.contains("$1") && matched.len() < 10 {
                continue;
            }

            // F025: Skip if variable is in double quotes (documentation pattern)
            // e.g., 'Value: "$var"' or 'Use "$(cmd)"' are intentional literals
            if is_documentation_pattern(matched) {
                continue;
            }

            let diagnostic = Diagnostic::new(
                "SC2016",
                Severity::Info,
                "Expressions don't expand in single quotes, use double quotes for that".to_string(),
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
    fn test_sc2016_var_in_single_quotes() {
        let code = r#"echo 'Hello $USER'"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
        assert_eq!(result.diagnostics[0].code, "SC2016");
        assert_eq!(result.diagnostics[0].severity, Severity::Info);
        assert!(result.diagnostics[0].message.contains("single quotes"));
    }

    #[test]
    fn test_sc2016_var_with_braces() {
        let code = r#"msg='Value: ${value}'"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2016_command_substitution() {
        let code = r#"cmd='$(date)'"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2016_multiple_vars() {
        let code = r#"text='$name is $age years old'"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2016_double_quotes_ok() {
        let code = r#"echo "Hello $USER""#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2016_literal_dollar_ok() {
        let code = r#"price='$50 per item'"#;
        let result = check(code);
        // Price pattern, likely intentional
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2016_no_dollar_ok() {
        let code = r#"msg='Hello World'"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2016_escaped_dollar_ok() {
        let code = r#"echo "\$USER""#;
        let result = check(code);
        // Not in single quotes
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2016_multiple_issues() {
        let code = r#"
msg1='Hello $USER'
msg2='Today is $(date)'
"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 2);
    }

    #[test]
    fn test_sc2016_in_assignment() {
        let code = r#"VAR='Current path: $PWD'"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    // ===== F025: Literal quotes in single-quoted strings =====
    // When single-quoted string contains quoted variables like "$var",
    // it's likely documentation or intentional literal output

    #[test]
    fn test_FP_025_literal_quoted_var_not_flagged() {
        // User wants to output literal '"$var"' - documentation pattern
        let code = r#"echo 'Value: "$var"'"#;
        let result = check(code);
        assert_eq!(
            result.diagnostics.len(),
            0,
            "SC2016 must NOT flag quotes inside literal (documentation pattern)"
        );
    }

    #[test]
    fn test_FP_025_syntax_example_not_flagged() {
        // Common pattern: showing shell syntax as literal text
        let code = r#"echo 'The syntax is: "$variable"'"#;
        let result = check(code);
        assert_eq!(
            result.diagnostics.len(),
            0,
            "SC2016 must NOT flag syntax examples in single quotes"
        );
    }

    #[test]
    fn test_FP_025_json_pattern_not_flagged() {
        // JSON-like patterns with quoted variables
        let code = r#"echo '{"name": "$name"}'"#;
        let result = check(code);
        assert_eq!(
            result.diagnostics.len(),
            0,
            "SC2016 must NOT flag JSON-like patterns"
        );
    }

    #[test]
    fn test_FP_025_bare_var_still_flagged() {
        // Bare variable without quotes should still be flagged (likely mistake)
        let code = r#"echo 'Value: $var'"#;
        let result = check(code);
        assert_eq!(
            result.diagnostics.len(),
            1,
            "SC2016 SHOULD flag bare variable in single quotes"
        );
    }

    #[test]
    fn test_FP_025_command_subst_in_quotes_not_flagged() {
        // Documentation showing command substitution syntax
        let code = r#"echo 'Use "$(command)" for substitution'"#;
        let result = check(code);
        assert_eq!(
            result.diagnostics.len(),
            0,
            "SC2016 must NOT flag quoted command substitution in docs"
        );
    }
}
