// SC2109: In [[ ]], use || instead of -o
//
// The -o operator is deprecated in POSIX test. Use || instead.
// In [[ ]] test expressions, -o is confusing with shell options (-o flag).
//
// Examples:
// Bad:
//   [[ $x -eq 1 -o $y -eq 2 ]]    // Deprecated -o
//   [[ -f file -o -d dir ]]       // Confusing
//
// Good:
//   [[ $x -eq 1 || $y -eq 2 ]]    // Modern ||
//   [[ -f file || -d dir ]]       // Clear intent
//
// Impact: Deprecated syntax, confusing with shell options

use crate::linter::{Diagnostic, LintResult, Severity, Span};
use regex::Regex;

static DOUBLE_BRACKET_WITH_O: std::sync::LazyLock<Regex> = std::sync::LazyLock::new(|| {
    // Match: [[ ... -o ... ]]
    Regex::new(r"\[\[[^\]]*\s-o\s[^\]]*\]\]").unwrap()
});

pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();

    for (line_num, line) in source.lines().enumerate() {
        let line_num = line_num + 1;

        if line.trim_start().starts_with('#') {
            continue;
        }

        for mat in DOUBLE_BRACKET_WITH_O.find_iter(line) {
            let matched = mat.as_str();

            // Find ALL positions of -o within the match
            let mut search_pos = 0;
            while let Some(o_pos) = matched[search_pos..].find(" -o ") {
                let actual_pos = search_pos + o_pos;
                let start_col = mat.start() + actual_pos + 1;
                let end_col = start_col + 3; // length of " -o"

                let diagnostic = Diagnostic::new(
                    "SC2109",
                    Severity::Warning,
                    "In [[ ]], use || instead of -o".to_string(),
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
    fn test_sc2109_double_bracket_o() {
        let code = "[[ $x -eq 1 -o $y -eq 2 ]]";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2109_file_tests() {
        let code = "[[ -f file -o -d dir ]]";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2109_or_ok() {
        let code = "[[ $x -eq 1 || $y -eq 2 ]]";
        let result = check(code);
        // || is correct
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2109_single_bracket_ok() {
        let code = "[ $x -eq 1 -o $y -eq 2 ]";
        let result = check(code);
        // Single bracket not flagged by this rule (SC2056 handles that)
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2109_comment_ok() {
        let code = "# [[ $x -eq 1 -o $y -eq 2 ]]";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2109_multiple() {
        let code = "[[ $a -eq 1 -o $b -eq 2 -o $c -eq 3 ]]";
        let result = check(code);
        // Multiple -o operators
        assert_eq!(result.diagnostics.len(), 2);
    }

    #[test]
    fn test_sc2109_multiline() {
        let code = r#"
if [[ $x -eq 1 -o $y -eq 2 ]]; then
    echo "test"
fi
"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2109_with_negation() {
        let code = "[[ ! -f file -o -d dir ]]";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2109_string_test() {
        let code = r#"[[ "$str" = "test" -o -z "$other" ]]"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2109_nested() {
        let code = "[[ ( $x -eq 1 -o $y -eq 2 ) ]]";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }
}
