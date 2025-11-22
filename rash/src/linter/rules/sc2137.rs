// SC2137: Unexpected braces in arithmetic context
//
// Variables in arithmetic contexts $(( )) don't need braces {}.
// Using braces can cause unexpected behavior or syntax errors.
//
// Examples:
// Bad:
//   echo $(( ${var} + 1 ))          // Unnecessary braces
//   result=$(( ${x} * ${y} ))       // Braces not needed
//
// Good:
//   echo $(( $var + 1 ))            // Simple $ prefix
//   result=$(( x + 1 ))             // Or no $ at all in arithmetic
//   echo $(( $x * $y ))             // Simpler without braces
//
// Note: Braces are fine for ${arr[i]} or ${#var}, but not needed for simple vars
// Impact: Style/clarity issue, may cause confusion

use crate::linter::{Diagnostic, LintResult, Severity, Span};
use once_cell::sync::Lazy;
use regex::Regex;

static ARITH_EXPR: Lazy<Regex> = Lazy::new(|| {
    // Match: $(( ... )) arithmetic expressions
    Regex::new(r"\$\(\(([^)]+)\)\)").unwrap()
});

static BRACED_VAR: Lazy<Regex> = Lazy::new(|| {
    // Match: ${var} braced variables
    Regex::new(r"\$\{([a-zA-Z_][a-zA-Z0-9_]*)\}").unwrap()
});

/// Check if a braced variable should be skipped (array or length syntax)
fn should_skip_braced_var(matched: &str) -> bool {
    matched.contains('[') || matched.contains('#')
}

/// Create diagnostic for unnecessary braces in arithmetic
fn create_braces_diagnostic(
    var_name: &str,
    abs_start: usize,
    abs_end: usize,
    line_num: usize,
) -> Diagnostic {
    Diagnostic::new(
        "SC2137",
        Severity::Info,
        format!(
            "Braces are unnecessary in arithmetic. Use ${} instead of ${{{}}}",
            var_name, var_name
        ),
        Span::new(line_num, abs_start + 1, line_num, abs_end + 1),
    )
}

/// Process a single braced variable capture within arithmetic expression
fn process_braced_var(
    var_cap: regex::Captures,
    arith_start: usize,
    line_num: usize,
    result: &mut LintResult,
) {
    let full_match = match var_cap.get(0) {
        Some(m) => m,
        None => return,
    };

    let matched = full_match.as_str();
    if should_skip_braced_var(matched) {
        return;
    }

    let var_name = match var_cap.get(1) {
        Some(m) => m.as_str(),
        None => return,
    };

    let var_pos = full_match.start();
    let abs_start = arith_start + var_pos;
    let abs_end = abs_start + matched.len();

    let diagnostic = create_braces_diagnostic(var_name, abs_start, abs_end, line_num);
    result.add(diagnostic);
}

/// Check a single arithmetic expression for unnecessary braces
fn check_arithmetic_expression(
    arith_content: &str,
    arith_start: usize,
    line_num: usize,
    result: &mut LintResult,
) {
    for var_cap in BRACED_VAR.captures_iter(arith_content) {
        process_braced_var(var_cap, arith_start, line_num, result);
    }
}

pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();

    for (line_num, line) in source.lines().enumerate() {
        let line_num = line_num + 1;

        if line.trim_start().starts_with('#') {
            continue;
        }

        // Find all arithmetic expressions
        for arith_mat in ARITH_EXPR.find_iter(line) {
            let arith_content = &line[arith_mat.start()..arith_mat.end()];
            check_arithmetic_expression(arith_content, arith_mat.start(), line_num, &mut result);
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sc2137_braced_variable() {
        let code = "echo $(( ${var} + 1 ))";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
        assert!(result.diagnostics[0].message.contains("$var"));
    }

    #[test]
    fn test_sc2137_simple_variable_ok() {
        let code = "echo $(( $var + 1 ))";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2137_no_dollar_ok() {
        let code = "echo $(( var + 1 ))";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2137_array_index_ok() {
        let code = "echo $(( ${arr[i]} + 1 ))";
        let result = check(code);
        // Array syntax needs braces
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2137_length_ok() {
        let code = "len=$(( ${#str} ))";
        let result = check(code);
        // Length syntax needs braces
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2137_multiple_braced() {
        let code = "result=$(( ${x} * ${y} ))";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 2);
    }

    #[test]
    fn test_sc2137_comment_ok() {
        let code = "# echo $(( ${var} + 1 ))";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2137_complex_expression() {
        let code = "val=$(( ${a} + ${b} * ${c} ))";
        let result = check(code);
        // Three braced variables
        assert_eq!(result.diagnostics.len(), 3);
    }

    #[test]
    fn test_sc2137_mixed_ok_and_bad() {
        let code = "result=$(( $x + ${y} ))";
        let result = check(code);
        // Only ${y} is flagged
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2137_multiline() {
        let code = r#"
x=$(( ${foo} ))
y=$(( ${bar} + 1 ))
"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 2);
    }
}
