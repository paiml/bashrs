// SC2051: Bash doesn't support variables in brace range expansions.
//
// Brace expansion happens before variable expansion, so variables in {$a..$b}
// don't work. Use seq or a C-style for loop instead.
//
// Examples:
// Bad:
//   for i in {$start..$end}; do    // Variables not expanded in braces
//     echo "$i"                     // Will literally print "{$start..$end}"
//   done
//
//   echo {$a..$b}                   // Doesn't work, prints literally
//
// Good:
//   for i in $(seq $start $end); do  // Use seq for variable ranges
//     echo "$i"
//   done
//
//   for ((i=start; i<=end; i++)); do  // C-style for loop (bash)
//     echo "$i"
//   done
//
//   # POSIX sh compatible:
//   i=$start
//   while [ $i -le $end ]; do
//     echo "$i"
//     i=$((i + 1))
//   done
//
// Note: Brace expansion {1..10} is parsed before variables are substituted.

use crate::linter::{Diagnostic, LintResult, Severity, Span};
use once_cell::sync::Lazy;
use regex::Regex;

static BRACE_WITH_VAR: Lazy<Regex> = Lazy::new(|| {
    // Match: {$var..something} or {something..$var}
    Regex::new(r"\{\$[a-zA-Z_][a-zA-Z0-9_]*\.\.[^}]*\}|\{[^}]*\.\.\$[a-zA-Z_][a-zA-Z0-9_]*\}")
        .unwrap()
});

pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();

    for (line_num, line) in source.lines().enumerate() {
        let line_num = line_num + 1;

        if line.trim_start().starts_with('#') {
            continue;
        }

        // Look for brace expansion with variables
        for mat in BRACE_WITH_VAR.find_iter(line) {
            // Skip if inside single quotes (no expansion at all)
            let pos = mat.start();
            let before = &line[..pos];
            let single_quote_count = before.matches('\'').count();
            if single_quote_count % 2 == 1 {
                continue;
            }

            let start_col = pos + 1;
            let end_col = mat.end() + 1;

            let diagnostic = Diagnostic::new(
                "SC2051",
                Severity::Warning,
                "Bash doesn't expand variables in brace ranges. Use seq or a for loop instead."
                    .to_string(),
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
    fn test_sc2051_var_in_brace_range() {
        let code = r#"for i in {$start..$end}; do echo "$i"; done"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
        assert_eq!(result.diagnostics[0].code, "SC2051");
        assert!(result.diagnostics[0].message.contains("variables"));
    }

    #[test]
    fn test_sc2051_echo_range() {
        let code = r#"echo {$a..$b}"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2051_var_at_end() {
        let code = r#"for i in {1..$max}; do echo "$i"; done"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2051_var_at_start() {
        let code = r#"for i in {$min..10}; do echo "$i"; done"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2051_literal_range_ok() {
        let code = r#"for i in {1..10}; do echo "$i"; done"#;
        let result = check(code);
        // Literal range is OK
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2051_seq_ok() {
        let code = r#"for i in $(seq $start $end); do echo "$i"; done"#;
        let result = check(code);
        // Using seq with variables is correct
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2051_c_style_loop_ok() {
        let code = r#"for ((i=start; i<=end; i++)); do echo "$i"; done"#;
        let result = check(code);
        // C-style loop is correct
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2051_in_single_quotes_ok() {
        let code = r#"echo 'example: {$start..$end}'"#;
        let result = check(code);
        // Inside single quotes, no expansion
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2051_comment_ok() {
        let code = r#"# for i in {$start..$end}; do"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2051_both_vars() {
        let code = r#"echo {$x..$y}"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }
}
