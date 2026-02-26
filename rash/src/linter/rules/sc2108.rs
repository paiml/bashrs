// SC2108: In [[ ]], use && instead of -a
//
// The -a operator is deprecated in POSIX test. Use && instead.
// In [[ ]] test expressions, -a is ambiguous with file test -a.
//
// Examples:
// Bad:
//   [[ $x -eq 1 -a $y -eq 2 ]]    // Deprecated -a
//   [[ -f file -a -r file ]]      // Ambiguous
//
// Good:
//   [[ $x -eq 1 && $y -eq 2 ]]    // Modern &&
//   [[ -f file && -r file ]]      // Clear intent
//
// Impact: Deprecated syntax, POSIX compliance

use crate::linter::{Diagnostic, LintResult, Severity, Span};
use regex::Regex;

static DOUBLE_BRACKET_WITH_A: std::sync::LazyLock<Regex> = std::sync::LazyLock::new(|| {
    // Match: [[ ... -a ... ]]
    Regex::new(r"\[\[[^\]]*\s-a\s[^\]]*\]\]").unwrap()
});

pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();

    for (line_num, line) in source.lines().enumerate() {
        let line_num = line_num + 1;

        if line.trim_start().starts_with('#') {
            continue;
        }

        for mat in DOUBLE_BRACKET_WITH_A.find_iter(line) {
            let matched = mat.as_str();

            // Find ALL positions of -a within the match
            let mut search_pos = 0;
            while let Some(a_pos) = matched[search_pos..].find(" -a ") {
                let actual_pos = search_pos + a_pos;
                let start_col = mat.start() + actual_pos + 1;
                let end_col = start_col + 3; // length of " -a"

                let diagnostic = Diagnostic::new(
                    "SC2108",
                    Severity::Warning,
                    "In [[ ]], use && instead of -a".to_string(),
                    Span::new(line_num, start_col, line_num, end_col),
                );

                result.add(diagnostic);
                search_pos = actual_pos + 3; // Move past this match
            }
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sc2108_double_bracket_a() {
        let code = "[[ $x -eq 1 -a $y -eq 2 ]]";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2108_file_tests() {
        let code = "[[ -f file -a -r file ]]";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2108_and_ok() {
        let code = "[[ $x -eq 1 && $y -eq 2 ]]";
        let result = check(code);
        // && is correct
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2108_single_bracket_ok() {
        let code = "[ $x -eq 1 -a $y -eq 2 ]";
        let result = check(code);
        // Single bracket not flagged by this rule (SC2055 handles that)
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2108_comment_ok() {
        let code = "# [[ $x -eq 1 -a $y -eq 2 ]]";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2108_multiple() {
        let code = "[[ $a -eq 1 -a $b -eq 2 -a $c -eq 3 ]]";
        let result = check(code);
        // Multiple -a operators
        assert_eq!(result.diagnostics.len(), 2);
    }

    #[test]
    fn test_sc2108_multiline() {
        let code = r#"
if [[ $x -eq 1 -a $y -eq 2 ]]; then
    echo "test"
fi
"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2108_with_negation() {
        let code = "[[ ! -f file -a -r dir ]]";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2108_string_test() {
        let code = r#"[[ "$str" = "test" -a -n "$other" ]]"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2108_nested() {
        let code = "[[ ( $x -eq 1 -a $y -eq 2 ) ]]";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }
}
