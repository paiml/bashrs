// SC2053: Quote the right-hand side of = in [ ] to prevent glob matching.
//
// In [ ], an unquoted string on the right side of = is treated as a glob pattern.
// This usually isn't intended and can cause unexpected matches.
//
// Examples:
// Bad:
//   [ "$var" = *.txt ]        // Glob pattern match (usually unintended)
//   [ "$x" = $pattern ]       // If pattern contains globs, unexpected
//   [ "$name" = foo* ]        // Pattern match, not literal
//
// Good (literal comparison):
//   [ "$var" = "*.txt" ]      // Literal string "*.txt"
//   [ "$x" = "$pattern" ]     // Quote both sides for safety
//   [ "$name" = "foo*" ]      // Literal string "foo*"
//
// Good (pattern matching):
//   [[ "$var" = *.txt ]]      // Use [[ ]] for deliberate patterns
//   [[ "$name" = foo* ]]      // Clear intent: pattern matching
//
// Note: If you want glob matching, use [[ ]] to make intent clear.
// In [ ], always quote literal strings.

use crate::linter::{Diagnostic, LintResult, Severity, Span};
use once_cell::sync::Lazy;
use regex::Regex;

static UNQUOTED_RHS_WITH_SPECIAL: Lazy<Regex> = Lazy::new(|| {
    // Match: [ "..." = unquoted_rhs ] where RHS contains glob chars
    // Match any token after = that contains *, ?, or [
    Regex::new(r#"=\s+([^\s\]"']*[\*\?\[][^\s\]"']*)"#).unwrap()
});

pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();

    for (line_num, line) in source.lines().enumerate() {
        let line_num = line_num + 1;

        if line.trim_start().starts_with('#') {
            continue;
        }

        // Skip [[ ]] (they handle patterns intentionally)
        if line.contains("[[") {
            continue;
        }

        // Only check [ ] contexts
        if !line.contains("[") || !line.contains("=") {
            continue;
        }

        // Look for unquoted RHS with special characters in [ ]
        for cap in UNQUOTED_RHS_WITH_SPECIAL.captures_iter(line) {
            let rhs = cap.get(1).unwrap().as_str();
            let full_match = cap.get(0).unwrap().as_str();
            let pos = line.find(full_match).unwrap_or(0);

            // Check if RHS is already quoted (shouldn't match our regex, but double-check)
            let rhs_pos = full_match.rfind(rhs).unwrap();
            let absolute_rhs_pos = pos + rhs_pos;

            if absolute_rhs_pos > 0 {
                let before_rhs = &line[..absolute_rhs_pos];
                if before_rhs.ends_with('"') || before_rhs.ends_with('\'') {
                    continue; // Already quoted
                }
            }

            let start_col = absolute_rhs_pos + 1;
            let end_col = start_col + rhs.len();

            let diagnostic = Diagnostic::new(
                "SC2053",
                Severity::Warning,
                format!(
                    "Quote the RHS '{}' in [ ] to prevent glob matching, or use [[ ]] for patterns",
                    rhs
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
    fn test_sc2053_unquoted_glob_rhs() {
        let code = r#"[ "$var" = *.txt ]"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
        assert_eq!(result.diagnostics[0].code, "SC2053");
        assert!(result.diagnostics[0].message.contains("Quote"));
    }

    #[test]
    fn test_sc2053_unquoted_var_with_glob() {
        let code = r#"[ "$x" = $pattern ]"#;
        let result = check(code);
        // $pattern doesn't have glob chars in the literal text, won't detect
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2053_unquoted_foo_star() {
        let code = r#"[ "$name" = foo* ]"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2053_quoted_rhs_ok() {
        let code = r#"[ "$var" = "*.txt" ]"#;
        let result = check(code);
        // Quoted RHS, literal comparison
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2053_double_bracket_ok() {
        let code = r#"[[ "$var" = *.txt ]]"#;
        let result = check(code);
        // [[ ]] handles patterns, OK
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2053_literal_string_ok() {
        let code = r#"[ "$x" = "literal" ]"#;
        let result = check(code);
        // No special chars, OK
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2053_comment_ok() {
        let code = r#"# [ "$var" = *.txt ]"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2053_question_mark() {
        let code = r#"[ "$char" = ? ]"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2053_bracket_char_class() {
        let code = r#"[ "$c" = [abc] ]"#;
        let result = check(code);
        // Character class in glob
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2053_single_quote_ok() {
        let code = r#"[ "$var" = '*.txt' ]"#;
        let result = check(code);
        // Single quoted, literal
        assert_eq!(result.diagnostics.len(), 0);
    }
}
