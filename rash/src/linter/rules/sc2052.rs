// SC2052: Use [[ .. ]] instead of [ .. ] when using != with globs.
//
// In [ ], the != operator does literal string comparison.
// In [[ ]], the != operator performs pattern matching (glob).
// If you're using wildcards, you likely want [[ ]].
//
// Examples:
// Bad (likely incorrect):
//   [ "$file" != *.txt ]      // Compares to literal string "*.txt"
//   [ "$name" != foo* ]       // Literal comparison, not pattern match
//
// Good (pattern matching):
//   [[ "$file" != *.txt ]]    // Pattern match: doesn't end with .txt
//   [[ "$name" != foo* ]]     // Pattern match: doesn't start with foo
//
// Good (literal):
//   [ "$file" != "*.txt" ]    // Quote if you want literal "*.txt"
//   [ "$str" = "literal" ]    // Use [ ] for literal comparisons
//
// Note: This also applies to = with patterns, but != is more common.

use crate::linter::{Diagnostic, LintResult, Severity, Span};
use regex::Regex;

static SINGLE_BRACKET_WITH_GLOB: std::sync::LazyLock<Regex> = std::sync::LazyLock::new(|| {
    // Match: [ ... != pattern* ] or [ ... = pattern* ]
    // Look for glob characters in comparisons
    Regex::new(r"\[\s+[^]]*(!?=)\s*([^\s\]]*[\*\?][^\s\]]*)").unwrap()
});

pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();

    for (line_num, line) in source.lines().enumerate() {
        let line_num = line_num + 1;

        if line.trim_start().starts_with('#') {
            continue;
        }

        // Skip [[ ]] (they handle globs correctly)
        if line.contains("[[") {
            continue;
        }

        // Look for [ ] with glob patterns
        for cap in SINGLE_BRACKET_WITH_GLOB.captures_iter(line) {
            let operator = cap.get(1).unwrap().as_str();
            let pattern = cap.get(2).unwrap().as_str();

            // Skip if the pattern is quoted (literal match intended)
            if (pattern.starts_with('"') && pattern.ends_with('"'))
                || (pattern.starts_with('\'') && pattern.ends_with('\''))
            {
                continue; // Quoted pattern, literal match intended
            }

            let full_match = cap.get(0).unwrap().as_str();
            let pos = line.find(full_match).unwrap_or(0);

            let start_col = pos + 1;
            let end_col = start_col + full_match.len();

            let diagnostic = Diagnostic::new(
                "SC2052",
                Severity::Warning,
                format!(
                    "Use [[ ]] instead of [ ] when using {} with glob '{}' for pattern matching",
                    operator, pattern
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
    fn test_sc2052_single_bracket_not_equal_glob() {
        let code = r#"[ "$file" != *.txt ]"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
        assert_eq!(result.diagnostics[0].code, "SC2052");
        assert!(result.diagnostics[0].message.contains("[["));
    }

    #[test]
    fn test_sc2052_single_bracket_equal_glob() {
        let code = r#"[ "$name" = foo* ]"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2052_question_mark_glob() {
        let code = r#"[ "$char" != ? ]"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2052_double_bracket_ok() {
        let code = r#"[[ "$file" != *.txt ]]"#;
        let result = check(code);
        // [[ ]] handles globs, correct
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2052_quoted_pattern_ok() {
        let code = r#"[ "$file" != "*.txt" ]"#;
        let result = check(code);
        // Quoted pattern = literal match, correct
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2052_literal_comparison_ok() {
        let code = r#"[ "$str" = "literal" ]"#;
        let result = check(code);
        // No glob characters, OK
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2052_comment_ok() {
        let code = r#"# [ "$file" != *.txt ]"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2052_complex_glob() {
        let code = r#"[ "$path" != /usr/bin/* ]"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2052_multiple_globs() {
        let code = r#"[ "$f" != *.txt ] && [ "$f" != *.log ]"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 2);
    }

    #[test]
    fn test_sc2052_single_quote_pattern_ok() {
        let code = r#"[ "$file" != '*.txt' ]"#;
        let result = check(code);
        // Single quoted, literal
        assert_eq!(result.diagnostics.len(), 0);
    }
}
