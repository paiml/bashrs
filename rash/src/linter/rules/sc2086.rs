//! SC2086: Double quote to prevent globbing and word splitting
//!
//! Detects unquoted variable expansions that could cause:
//! - Word splitting on IFS characters (space, tab, newline)
//! - Glob expansion of *, ?, [...] patterns
//!
//! References:
//! - https://www.shellcheck.net/wiki/SC2086
//! - POSIX Shell Command Language Section 2.6.2

use crate::linter::{Diagnostic, Fix, LintResult, Severity, Span};
use regex::Regex;

/// Check if line should be skipped (comments or assignments)
fn should_skip_line(line: &str) -> bool {
    // Skip comments
    if line.trim_start().starts_with('#') {
        return true;
    }

    // Skip variable assignments (VAR=value)
    if line.contains('=') && !line.contains("if [") && !line.contains("[ ") {
        if let Some(eq_pos) = line.find('=') {
            if let Some(first_space) = line.find(' ') {
                if eq_pos < first_space {
                    return true; // Assignment, not command
                }
            }
        }
    }

    false
}

/// Find the position of $ character before a variable
fn find_dollar_position(line: &str, var_start: usize) -> usize {
    line[..var_start].rfind('$').unwrap_or(var_start)
}

/// Calculate end column for variable span, including closing brace if present
fn calculate_end_column(line: &str, var_end: usize, is_braced: bool) -> usize {
    if is_braced {
        let after_var = &line[var_end..];
        if let Some(brace_pos) = after_var.find('}') {
            var_end + brace_pos + 2 // +1 for }, +1 for 1-indexing
        } else {
            var_end + 1 // Fallback
        }
    } else {
        var_end + 1 // Simple $VAR case
    }
}

/// Check if variable is in arithmetic context (inside $(( )))
fn is_in_arithmetic_context(line: &str, dollar_pos: usize, var_end: usize) -> bool {
    let before = &line[..dollar_pos];
    let after = &line[var_end..];
    before.contains("$((") && after.contains("))")
}

/// Check if variable is already quoted
fn is_already_quoted(line: &str, dollar_pos: usize, var_end: usize) -> bool {
    let before_context = &line[..dollar_pos];
    let after_context = &line[var_end..];
    before_context.ends_with('"') && after_context.starts_with('"')
}

/// Build diagnostic for unquoted variable
fn build_diagnostic(
    line_num: usize,
    col: usize,
    end_col: usize,
    var_name: &str,
    is_braced: bool,
) -> Diagnostic {
    let span = Span::new(line_num, col, line_num, end_col);
    let var_text = if is_braced {
        format!("${{{}}}", var_name)
    } else {
        format!("${}", var_name)
    };

    let fix = Fix::new(format!("\"{}\"", var_text));

    Diagnostic::new(
        "SC2086",
        Severity::Warning,
        format!(
            "Double quote to prevent globbing and word splitting on {}",
            var_text
        ),
        span,
    )
    .with_fix(fix)
}

/// Check for unquoted variable expansions (SC2086)
pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();

    // Regex to find unquoted variables in command contexts
    let var_pattern = Regex::new(r#"(?m)(?P<pre>[^"']|^)\$(?:\{(?P<brace>[A-Za-z_][A-Za-z0-9_]*)\}|(?P<simple>[A-Za-z_][A-Za-z0-9_]*))"#).unwrap();

    for (line_num, line) in source.lines().enumerate() {
        let line_num = line_num + 1; // 1-indexed

        if should_skip_line(line) {
            continue;
        }

        let is_arithmetic = line.contains("$((") || line.contains("(( ");

        for cap in var_pattern.captures_iter(line) {
            let var_capture = match cap.name("brace").or_else(|| cap.name("simple")) {
                Some(v) => v,
                None => continue,
            };

            let var_name = var_capture.as_str();
            let dollar_pos = find_dollar_position(line, var_capture.start());
            let col = dollar_pos + 1; // 1-indexed

            let is_braced = cap.name("brace").is_some();
            let end_col = calculate_end_column(line, var_capture.end(), is_braced);

            // Skip if in arithmetic context or already quoted
            if is_arithmetic && is_in_arithmetic_context(line, dollar_pos, var_capture.end()) {
                continue;
            }

            if is_already_quoted(line, dollar_pos, var_capture.end()) {
                continue;
            }

            let diag = build_diagnostic(line_num, col, end_col, var_name, is_braced);
            result.add(diag);
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sc2086_basic_detection() {
        let bash_code = r#"
#!/bin/bash
FILES=$1
ls $FILES
"#;

        let result = check(bash_code);
        assert_eq!(
            result.diagnostics.len(),
            1,
            "Should detect one unquoted variable"
        );
        assert_eq!(result.diagnostics[0].code, "SC2086");
        assert!(result.diagnostics[0].message.contains("Double quote"));
        assert!(result.diagnostics[0].message.contains("$FILES"));
    }

    #[test]
    fn test_sc2086_autofix() {
        let bash_code = "ls $FILES";
        let result = check(bash_code);

        assert_eq!(result.diagnostics.len(), 1);
        assert!(result.diagnostics[0].fix.is_some());
        assert_eq!(
            result.diagnostics[0].fix.as_ref().unwrap().replacement,
            "\"$FILES\""
        );
    }

    #[test]
    fn test_sc2086_no_false_positive_arithmetic() {
        let bash_code = "result=$(( $x + $y ))";
        let result = check(bash_code);

        // Should NOT trigger SC2086 in arithmetic context
        assert_eq!(
            result.diagnostics.len(),
            0,
            "Should not trigger in arithmetic context"
        );
    }

    #[test]
    fn test_sc2086_multiple_violations() {
        let bash_code = r#"
rm -rf $DIR
cat $FILE1 $FILE2
"#;

        let result = check(bash_code);
        assert_eq!(
            result.diagnostics.len(),
            3,
            "Should detect three unquoted variables"
        );

        let codes: Vec<&str> = result.diagnostics.iter().map(|d| d.code.as_str()).collect();
        assert_eq!(codes, vec!["SC2086", "SC2086", "SC2086"]);
    }

    #[test]
    fn test_sc2086_braced_variables() {
        let bash_code = r#"echo ${VAR}"#;
        let result = check(bash_code);

        assert_eq!(result.diagnostics.len(), 1);
        assert!(result.diagnostics[0].message.contains("${VAR}"));
    }

    #[test]
    fn test_sc2086_skip_comments() {
        let bash_code = r#"
# This is a comment with $VAR
echo $ACTUAL
"#;

        let result = check(bash_code);
        assert_eq!(
            result.diagnostics.len(),
            1,
            "Should only detect variable in echo, not comment"
        );
        assert!(result.diagnostics[0].message.contains("$ACTUAL"));
    }

    #[test]
    fn test_sc2086_skip_quoted() {
        let bash_code = r#"echo "$VAR""#;
        let result = check(bash_code);

        // Should NOT trigger - already quoted
        assert_eq!(
            result.diagnostics.len(),
            0,
            "Should not trigger on already-quoted variables"
        );
    }

    #[test]
    fn test_sc2086_mixed_quoted_unquoted() {
        let bash_code = r#"
echo "$VAR1"
echo $VAR2
echo "$VAR3"
"#;

        let result = check(bash_code);
        assert_eq!(
            result.diagnostics.len(),
            1,
            "Should only detect unquoted $VAR2"
        );
        assert!(result.diagnostics[0].message.contains("$VAR2"));
    }

    #[test]
    fn test_sc2086_severity() {
        let bash_code = "ls $FILES";
        let result = check(bash_code);

        assert_eq!(result.diagnostics[0].severity, Severity::Warning);
    }

    #[test]
    fn test_sc2086_span_accuracy() {
        let bash_code = "ls $FILES";
        let result = check(bash_code);

        let span = result.diagnostics[0].span;
        assert_eq!(span.start_line, 1);
        assert_eq!(span.end_line, 1);
        // Column positions should point to $FILES
        assert!(span.start_col <= 4); // "ls " is 3 chars
        assert!(span.end_col >= span.start_col);
    }
}
