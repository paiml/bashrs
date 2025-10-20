// SC2201: Brace expansion doesn't happen in assignments. Use a loop or array
//
// Brace expansions are NOT performed in simple variable assignments.
// The braces remain literal.
//
// Examples:
// Bad:
//   files={a,b,c}.txt          # files="{a,b,c}.txt" (literal)
//   dirs=/path/{foo,bar}       # dirs="/path/{foo,bar}" (literal)
//
// Good:
//   files=(a.txt b.txt c.txt)  # Array assignment
//   for dir in /path/{foo,bar}; do ... done  # Loop with expansion
//   files="a.txt b.txt c.txt"  # Space-separated if appropriate

use crate::linter::{Diagnostic, LintResult, Severity, Span};
use once_cell::sync::Lazy;
use regex::Regex;

static ASSIGNMENT_WITH_BRACES: Lazy<Regex> = Lazy::new(|| {
    // Match: var={...} or var=.../...{...}
    Regex::new(r"^([A-Za-z_][A-Za-z0-9_]*)=([^=\s]*\{[a-zA-Z0-9_,./\-]+\}[^\s]*)").unwrap()
});

pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();

    for (line_num, line) in source.lines().enumerate() {
        let line_num = line_num + 1;
        let trimmed = line.trim();

        if trimmed.starts_with('#') {
            continue;
        }

        // Skip array assignments (those with =( ))
        if trimmed.contains("=(") {
            continue;
        }

        if let Some(cap) = ASSIGNMENT_WITH_BRACES.captures(trimmed) {
            let var_name = cap.get(1).unwrap().as_str();
            let value = cap.get(2).unwrap().as_str();
            let start_col = cap.get(0).unwrap().start() + 1;
            let end_col = cap.get(0).unwrap().end() + 1;

            let diagnostic = Diagnostic::new(
                "SC2201",
                Severity::Warning,
                format!(
                    "Brace expansion doesn't happen in assignments. Use an array {}=(...) or a loop instead",
                    var_name
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
    fn test_sc2201_brace_in_assignment() {
        let code = r#"files={a,b,c}.txt"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
        assert_eq!(result.diagnostics[0].code, "SC2201");
        assert_eq!(result.diagnostics[0].severity, Severity::Warning);
        assert!(result.diagnostics[0].message.contains("Brace expansion"));
    }

    #[test]
    fn test_sc2201_path_with_braces() {
        let code = r#"dirs=/path/{foo,bar}"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2201_range_in_assignment() {
        let code = r#"nums={1..10}"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2201_array_assignment_ok() {
        let code = r#"files=(a.txt b.txt c.txt)"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2201_array_with_braces_ok() {
        let code = r#"files=({a,b,c}.txt)"#;
        let result = check(code);
        // Array assignment with braces - expansion happens
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2201_simple_assignment_ok() {
        let code = r#"file=test.txt"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2201_quoted_braces() {
        let code = r#"pattern="{a,b}""#;
        let result = check(code);
        // Quoted, might be intentional literal
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2201_in_loop_ok() {
        let code = r#"for file in {a,b,c}.txt; do echo $file; done"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2201_multiple_assignments() {
        let code = r#"
a={x,y}.log
b={1,2,3}
"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 2);
    }

    #[test]
    fn test_sc2201_path_expansion() {
        let code = r#"backup=/backup/{daily,weekly}"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }
}
