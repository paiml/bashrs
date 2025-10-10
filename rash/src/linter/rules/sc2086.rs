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

/// Check for unquoted variable expansions (SC2086)
pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();

    // Regex to find unquoted variables in command contexts
    // Matches: $VAR or ${VAR} not within quotes
    let var_pattern = Regex::new(r#"(?m)(?P<pre>[^"']|^)\$(?:\{(?P<brace>[A-Za-z_][A-Za-z0-9_]*)\}|(?P<simple>[A-Za-z_][A-Za-z0-9_]*))"#).unwrap();

    for (line_num, line) in source.lines().enumerate() {
        let line_num = line_num + 1; // 1-indexed

        // Skip comments
        if line.trim_start().starts_with('#') {
            continue;
        }

        // Skip variable assignments (VAR=value)
        if line.contains('=') && !line.contains("if [") && !line.contains("[ ") {
            // Simple heuristic: if line has = before any command, it's likely an assignment
            if let Some(eq_pos) = line.find('=') {
                if let Some(first_space) = line.find(' ') {
                    if eq_pos < first_space {
                        continue; // Assignment, not command
                    }
                }
            }
        }

        // Check for unquoted variables in arithmetic contexts (false positive prevention)
        let is_arithmetic = line.contains("$((") || line.contains("(( ");

        for cap in var_pattern.captures_iter(line) {
            let var_match = cap.get(0).unwrap();
            let var_name = cap.name("brace")
                .or_else(|| cap.name("simple"))
                .map(|m| m.as_str())
                .unwrap_or("");

            let col = var_match.start() + 1; // Account for the 'pre' capture group
            let end_col = var_match.end();

            // Skip if in arithmetic context
            if is_arithmetic {
                // Check if this variable is inside $(( ))
                let before = &line[..var_match.start()];
                let after = &line[var_match.end()..];
                if before.contains("$((") && after.contains("))") {
                    continue;
                }
            }

            // Check if already quoted
            let before_context = &line[..var_match.start()];
            let after_context = &line[var_match.end()..];

            // Simple quote detection
            let has_opening_quote = before_context.ends_with('"');
            let has_closing_quote = after_context.starts_with('"');

            if has_opening_quote && has_closing_quote {
                continue; // Already quoted
            }

            // Create diagnostic
            let span = Span::new(line_num, col, line_num, end_col);
            let var_text = if var_name.is_empty() {
                var_match.as_str().to_string()
            } else if cap.name("brace").is_some() {
                format!("${{{}}}", var_name)
            } else {
                format!("${}", var_name)
            };

            let fix = Fix::new(format!("\"{}\"", var_text));

            let diag = Diagnostic::new(
                "SC2086",
                Severity::Warning,
                format!("Double quote to prevent globbing and word splitting on {}", var_text),
                span,
            )
            .with_fix(fix);

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
        assert_eq!(result.diagnostics.len(), 1, "Should detect one unquoted variable");
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
        assert_eq!(result.diagnostics[0].fix.as_ref().unwrap().replacement, "\"$FILES\"");
    }

    #[test]
    fn test_sc2086_no_false_positive_arithmetic() {
        let bash_code = "result=$(( $x + $y ))";
        let result = check(bash_code);

        // Should NOT trigger SC2086 in arithmetic context
        assert_eq!(result.diagnostics.len(), 0, "Should not trigger in arithmetic context");
    }

    #[test]
    fn test_sc2086_multiple_violations() {
        let bash_code = r#"
rm -rf $DIR
cat $FILE1 $FILE2
"#;

        let result = check(bash_code);
        assert_eq!(result.diagnostics.len(), 3, "Should detect three unquoted variables");

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
        assert_eq!(result.diagnostics.len(), 1, "Should only detect variable in echo, not comment");
        assert!(result.diagnostics[0].message.contains("$ACTUAL"));
    }

    #[test]
    fn test_sc2086_skip_quoted() {
        let bash_code = r#"echo "$VAR""#;
        let result = check(bash_code);

        // Should NOT trigger - already quoted
        assert_eq!(result.diagnostics.len(), 0, "Should not trigger on already-quoted variables");
    }

    #[test]
    fn test_sc2086_mixed_quoted_unquoted() {
        let bash_code = r#"
echo "$VAR1"
echo $VAR2
echo "$VAR3"
"#;

        let result = check(bash_code);
        assert_eq!(result.diagnostics.len(), 1, "Should only detect unquoted $VAR2");
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
