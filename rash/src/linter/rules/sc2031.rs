// SC2031: Variable was modified in subshell. Double check or use var=$(cmd).
//
// This is the complementary warning to SC2030. It detects when you use a variable
// that was previously assigned in a subshell, which means the value will be empty/wrong.
//
// Examples:
// Bad:
//   (foo=bar)
//   echo "$foo"  # SC2031: foo was assigned in subshell, will be empty
//
//   (x=5; y=10)
//   echo "$x"    # Empty
//
// Good:
//   foo=bar      # Assign in current shell
//   echo "$foo"
//
//   result=$(foo=bar; echo "$foo")  # Capture output
//   echo "$result"
//
// Note: This requires tracking variable assignments across lines (stateful analysis).
// For MVP, we detect the pattern heuristically: subshell assignment followed by usage.

use crate::linter::{Diagnostic, LintResult, Severity, Span};
use once_cell::sync::Lazy;
use regex::Regex;
use std::collections::HashSet;

static SUBSHELL_START: Lazy<Regex> = Lazy::new(|| Regex::new(r"\(").unwrap());

static ASSIGNMENT: Lazy<Regex> = Lazy::new(|| Regex::new(r"([a-zA-Z_][a-zA-Z0-9_]*)=").unwrap());

static VAR_USAGE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"\$\{?([a-zA-Z_][a-zA-Z0-9_]*)\}?").unwrap());

/// Check if line contains a subshell (standalone parentheses, not command substitution)
fn has_subshell(line: &str) -> bool {
    let chars: Vec<char> = line.chars().collect();
    for i in 0..chars.len() {
        if chars[i] == '(' {
            // Check if previous char is NOT $ (would be command substitution)
            if i == 0 || chars[i - 1] != '$' {
                return true;
            }
        }
    }
    false
}

/// Check if position in line is inside quotes (double or single)
fn is_in_quotes(line: &str, pos: usize) -> bool {
    let before = &line[..pos];
    let quote_count = before.matches('"').count() + before.matches('\'').count();
    quote_count % 2 == 1
}

/// Check if position in line is inside single quotes (where variables don't expand)
fn is_in_single_quotes(line: &str, pos: usize) -> bool {
    let before = &line[..pos];
    let single_quote_count = before.matches('\'').count();
    single_quote_count % 2 == 1
}

/// Check if variable usage is on same line as assignment (which is OK)
fn is_same_line_assignment(line: &str, var_name: &str) -> bool {
    line.contains('(') && line.contains(')') && line.contains(&format!("{}=", var_name))
}

/// Find subshell variable assignments on a line
fn find_subshell_assignments(line: &str) -> HashSet<String> {
    let mut vars = HashSet::new();

    if !line.contains('(') || !line.contains(')') {
        return vars;
    }

    if !has_subshell(line) {
        return vars;
    }

    // Find all variable assignments on this line
    for cap in ASSIGNMENT.captures_iter(line) {
        let var_name = cap.get(1).unwrap().as_str();
        let full_match = cap.get(0).unwrap().as_str();
        let pos = line.find(full_match).unwrap_or(0);

        // Skip if inside quotes
        if !is_in_quotes(line, pos) {
            vars.insert(var_name.to_string());
        }
    }

    vars
}

/// Create diagnostic for subshell variable usage
fn create_diagnostic(
    line_num: usize,
    var_name: &str,
    pos: usize,
    full_match_len: usize,
) -> Diagnostic {
    let start_col = pos + 1;
    let end_col = start_col + full_match_len;

    Diagnostic::new(
        "SC2031",
        Severity::Warning,
        format!(
            "Variable '{}' was assigned in a subshell. It will be empty here. Use var=$(cmd) or assign in current shell",
            var_name
        ),
        Span::new(line_num, start_col, line_num, end_col),
    )
}

pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();
    let mut subshell_vars: HashSet<String> = HashSet::new();

    for (line_num, line) in source.lines().enumerate() {
        let line_num = line_num + 1;

        if line.trim_start().starts_with('#') {
            continue;
        }

        // Track variables assigned in subshells
        subshell_vars.extend(find_subshell_assignments(line));

        // Check for usage of subshell-assigned variables
        for cap in VAR_USAGE.captures_iter(line) {
            let var_name = cap.get(1).unwrap().as_str();

            if !subshell_vars.contains(var_name) {
                continue;
            }

            // Skip if same line assignment or inside single quotes
            if is_same_line_assignment(line, var_name) {
                continue;
            }

            let full_match = cap.get(0).unwrap().as_str();
            let pos = line.find(full_match).unwrap_or(0);

            if is_in_single_quotes(line, pos) {
                continue;
            }

            let diagnostic = create_diagnostic(line_num, var_name, pos, full_match.len());
            result.add(diagnostic);
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sc2031_usage_after_subshell_assignment() {
        let code = r#"
(foo=bar)
echo "$foo"
"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
        assert_eq!(result.diagnostics[0].code, "SC2031");
        assert_eq!(result.diagnostics[0].severity, Severity::Warning);
        assert!(result.diagnostics[0].message.contains("foo"));
    }

    #[test]
    fn test_sc2031_multiple_vars() {
        let code = r#"
(x=5; y=10)
echo "$x"
echo "$y"
"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 2);
    }

    #[test]
    fn test_sc2031_direct_assignment_ok() {
        let code = r#"
foo=bar
echo "$foo"
"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2031_command_subst_ok() {
        let code = r#"
result=$(foo=bar; echo "$foo")
echo "$result"
"#;
        let result = check(code);
        // Command substitution captures output, result is OK
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2031_same_line_ok() {
        let code = r#"(foo=bar; echo "$foo")"#;
        let result = check(code);
        // Same line usage inside subshell is OK
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2031_in_quotes_ok() {
        let code = r#"
(foo=bar)
echo '(foo=bar)'
"#;
        let result = check(code);
        // Variable in quotes (literal string)
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2031_unrelated_var_ok() {
        let code = r#"
(foo=bar)
echo "$baz"
"#;
        let result = check(code);
        // Different variable
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2031_braced_var() {
        let code = r#"
(VAR=test)
echo "${VAR}"
"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2031_comment_ok() {
        let code = r#"
(foo=bar)
# echo "$foo"
"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2031_nested_subshell() {
        let code = r#"
((foo=bar))
echo "$foo"
"#;
        let result = check(code);
        // Still a subshell assignment
        assert_eq!(result.diagnostics.len(), 1);
    }
}
