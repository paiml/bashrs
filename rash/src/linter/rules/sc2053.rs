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
use regex::Regex;

static UNQUOTED_RHS_WITH_SPECIAL: std::sync::LazyLock<Regex> = std::sync::LazyLock::new(|| {
    // Match: [ "..." = unquoted_rhs ] where RHS contains glob chars
    // Match any token after = that contains *, ?, or [
    Regex::new(r#"=\s+([^\s\]"']*[\*\?\[][^\s\]"']*)"#).unwrap()
});

/// Check if a line is a comment
fn is_comment_line(line: &str) -> bool {
    line.trim_start().starts_with('#')
}

/// Check if line contains double brackets
fn has_double_bracket(line: &str) -> bool {
    line.contains("[[")
}

/// Check if line should be checked (has [ and =)
fn should_check_line(line: &str) -> bool {
    line.contains('[') && line.contains('=')
}

/// Check if RHS is already quoted
fn is_already_quoted(line: &str, absolute_rhs_pos: usize) -> bool {
    if absolute_rhs_pos > 0 {
        let before_rhs = &line[..absolute_rhs_pos];
        before_rhs.ends_with('"') || before_rhs.ends_with('\'')
    } else {
        false
    }
}

/// Calculate absolute position of RHS in line
fn calculate_absolute_rhs_pos(line: &str, full_match: &str, rhs: &str, match_pos: usize) -> usize {
    let rhs_pos = full_match.rfind(rhs).unwrap();
    match_pos + rhs_pos
}

/// Create diagnostic for unquoted glob pattern
fn create_unquoted_glob_diagnostic(
    rhs: &str,
    line_num: usize,
    start_col: usize,
    end_col: usize,
) -> Diagnostic {
    Diagnostic::new(
        "SC2053",
        Severity::Warning,
        format!(
            "Quote the RHS '{}' in [ ] to prevent glob matching, or use [[ ]] for patterns",
            rhs
        ),
        Span::new(line_num, start_col, line_num, end_col),
    )
}

pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();

    for (line_num, line) in source.lines().enumerate() {
        let line_num = line_num + 1;

        if is_comment_line(line) || has_double_bracket(line) || !should_check_line(line) {
            continue;
        }

        // Look for unquoted RHS with special characters in [ ]
        for cap in UNQUOTED_RHS_WITH_SPECIAL.captures_iter(line) {
            let rhs = cap.get(1).unwrap().as_str();
            let full_match = cap.get(0).unwrap().as_str();
            let match_pos = line.find(full_match).unwrap_or(0);

            let absolute_rhs_pos = calculate_absolute_rhs_pos(line, full_match, rhs, match_pos);

            if is_already_quoted(line, absolute_rhs_pos) {
                continue;
            }

            let start_col = absolute_rhs_pos + 1;
            let end_col = start_col + rhs.len();

            let diagnostic = create_unquoted_glob_diagnostic(rhs, line_num, start_col, end_col);
            result.add(diagnostic);
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    // ===== Manual Property Tests =====

    #[test]
    fn prop_sc2053_comments_never_diagnosed() {
        // Property: Comment lines should never produce diagnostics
        let test_cases = vec![
            "# [ \"$var\" = *.txt ]",
            "  # [ \"$x\" = foo* ]",
            "\t# [ \"$c\" = ? ]",
        ];

        for code in test_cases {
            let result = check(code);
            assert_eq!(result.diagnostics.len(), 0);
        }
    }

    #[test]
    fn prop_sc2053_double_brackets_never_diagnosed() {
        // Property: Double brackets [[ ]] should never be diagnosed (patterns are intentional)
        let test_cases = vec![
            "[[ \"$var\" = *.txt ]]",
            "[[ \"$name\" = foo* ]]",
            "[[ \"$c\" = ? ]]",
            "[[ \"$x\" = [abc] ]]",
        ];

        for code in test_cases {
            let result = check(code);
            assert_eq!(result.diagnostics.len(), 0);
        }
    }

    #[test]
    fn prop_sc2053_quoted_rhs_never_diagnosed() {
        // Property: Quoted RHS should never be diagnosed
        let test_cases = vec![
            "[ \"$var\" = \"*.txt\" ]",
            "[ \"$name\" = 'foo*' ]",
            "[ \"$c\" = \"?\" ]",
            "[ \"$x\" = '[abc]' ]",
        ];

        for code in test_cases {
            let result = check(code);
            assert_eq!(result.diagnostics.len(), 0);
        }
    }

    #[test]
    fn prop_sc2053_no_glob_chars_never_diagnosed() {
        // Property: RHS without glob characters should never be diagnosed
        let test_cases = vec![
            "[ \"$x\" = \"literal\" ]",
            "[ \"$y\" = simple ]",
            "[ \"$z\" = value123 ]",
        ];

        for code in test_cases {
            let result = check(code);
            assert_eq!(result.diagnostics.len(), 0);
        }
    }

    #[test]
    fn prop_sc2053_unquoted_globs_always_diagnosed() {
        // Property: Unquoted RHS with glob characters should always be diagnosed
        let test_cases = vec![
            "[ \"$var\" = *.txt ]",
            "[ \"$name\" = foo* ]",
            "[ \"$c\" = ? ]",
            "[ \"$x\" = [abc] ]",
        ];

        for code in test_cases {
            let result = check(code);
            assert_eq!(result.diagnostics.len(), 1, "Should diagnose: {}", code);
            assert!(result.diagnostics[0].message.contains("Quote"));
        }
    }

    #[test]
    fn prop_sc2053_diagnostic_code_always_sc2053() {
        // Property: All diagnostics must have code "SC2053"
        let code = "[ \"$a\" = *.txt ] && [ \"$b\" = foo* ]";
        let result = check(code);

        for diagnostic in &result.diagnostics {
            assert_eq!(&diagnostic.code, "SC2053");
        }
    }

    #[test]
    fn prop_sc2053_diagnostic_severity_always_warning() {
        // Property: All diagnostics must be Warning severity
        let code = "[ \"$var\" = *.txt ]";
        let result = check(code);

        for diagnostic in &result.diagnostics {
            assert_eq!(diagnostic.severity, Severity::Warning);
        }
    }

    #[test]
    fn prop_sc2053_message_suggests_alternatives() {
        // Property: Message should suggest quoting or using [[ ]]
        let code = "[ \"$var\" = *.txt ]";
        let result = check(code);

        assert_eq!(result.diagnostics.len(), 1);
        let msg = &result.diagnostics[0].message;
        assert!(msg.contains("Quote") || msg.contains("[["));
    }

    #[test]
    fn prop_sc2053_multiple_globs_all_diagnosed() {
        // Property: Multiple unquoted globs should all be diagnosed
        let code = "[ \"$a\" = *.txt ] && [ \"$b\" = foo* ]";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 2);
    }

    #[test]
    fn prop_sc2053_empty_source_no_diagnostics() {
        // Property: Empty source should produce no diagnostics
        let result = check("");
        assert_eq!(result.diagnostics.len(), 0);
    }

    // ===== Original Unit Tests =====

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
