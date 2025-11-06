// SC2168: 'local' is only valid in functions
//
// The `local` keyword can only be used inside shell functions.
// Using it at the top level is a syntax error.
//
// Examples:
// Bad:
//   local var="value"  # At top level - ERROR
//
// Good:
//   function test() {
//       local var="value"  # Inside function - OK
//   }

use crate::linter::{Diagnostic, LintResult, Severity, Span};
use once_cell::sync::Lazy;
use regex::Regex;

static LOCAL_KEYWORD: Lazy<Regex> = Lazy::new(|| Regex::new(r"\blocal\s+").unwrap());

static FUNCTION_START: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"\b(function\s+[A-Za-z_][A-Za-z0-9_]*|[A-Za-z_][A-Za-z0-9_]*\s*\(\s*\))").unwrap()
});

static FUNCTION_END: Lazy<Regex> = Lazy::new(|| Regex::new(r"^\s*\}").unwrap());

/// Check if a line is a comment
fn is_comment_line(line: &str) -> bool {
    line.trim_start().starts_with('#')
}

/// Check if a position in a line is inside single or double quotes
///
/// This function tracks quote state to determine if a position is inside a quoted string.
/// It handles both single quotes ('...') and double quotes ("...").
///
/// # Examples
/// ```
/// assert!(is_inside_quotes("echo 'local'", 7));  // Inside 'local'
/// assert!(is_inside_quotes("echo \"local\"", 7));  // Inside "local"
/// assert!(!is_inside_quotes("echo local", 5));    // Not inside quotes
/// ```
fn is_inside_quotes(line: &str, pos: usize) -> bool {
    let mut in_single_quote = false;
    let mut in_double_quote = false;
    let mut prev_char = '\0';

    for (i, ch) in line.chars().enumerate() {
        if i >= pos {
            break;
        }

        // Handle escape sequences
        if prev_char == '\\' {
            prev_char = ch;
            continue;
        }

        match ch {
            '\'' if !in_double_quote => in_single_quote = !in_single_quote,
            '"' if !in_single_quote => in_double_quote = !in_double_quote,
            _ => {}
        }

        prev_char = ch;
    }

    in_single_quote || in_double_quote
}

/// Check if line starts a function
fn is_function_start(line: &str) -> bool {
    FUNCTION_START.is_match(line)
}

/// Check if line has opening brace
fn has_opening_brace(line: &str) -> bool {
    line.contains('{')
}

/// Check if next line has opening brace
fn has_opening_brace_next_line(lines: &[&str], i: usize) -> bool {
    i + 1 < lines.len() && lines[i + 1].contains('{')
}

/// Count opening braces in line
fn count_opening_braces(line: &str) -> usize {
    line.matches('{').count()
}

/// Count closing braces in line
fn count_closing_braces(line: &str) -> usize {
    line.matches('}').count()
}

/// Check if line is function end
fn is_function_end(line: &str) -> bool {
    FUNCTION_END.is_match(line)
}

/// Update function depth for function start
fn update_depth_for_function_start(
    function_depth: &mut usize,
    line: &str,
    lines: &[&str],
    i: usize,
) {
    if is_function_start(line) {
        // Look ahead to see if there's an opening brace
        if has_opening_brace(line) {
            *function_depth += 1;
        } else if has_opening_brace_next_line(lines, i) {
            // Brace on next line
            *function_depth += 1;
        }
    }
}

/// Update function depth for braces
fn update_depth_for_braces(function_depth: &mut usize, line: &str) {
    // Track closing braces
    if is_function_end(line) && *function_depth > 0 {
        *function_depth = function_depth.saturating_sub(1);
    }

    // Count opening braces on current line
    *function_depth += count_opening_braces(line);
    // Subtract closing braces
    if *function_depth > 0 {
        let closing = count_closing_braces(line);
        *function_depth = function_depth.saturating_sub(closing);
    }
}

/// Create diagnostic for local outside function
fn create_local_outside_function_diagnostic(
    line_num: usize,
    start_col: usize,
    end_col: usize,
) -> Diagnostic {
    Diagnostic::new(
        "SC2168",
        Severity::Error,
        "'local' is only valid in functions",
        Span::new(line_num, start_col, line_num, end_col),
    )
}

pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();
    let lines: Vec<&str> = source.lines().collect();
    let mut function_depth: usize = 0;

    for (i, line) in lines.iter().enumerate() {
        let line_num = i + 1;

        if is_comment_line(line) {
            continue;
        }

        // Track function depth
        update_depth_for_function_start(&mut function_depth, line, &lines, i);
        update_depth_for_braces(&mut function_depth, line);

        // Check for local keyword outside functions
        if let Some(mat) = LOCAL_KEYWORD.find(line) {
            // Skip if 'local' is inside quotes (false positive)
            if is_inside_quotes(line, mat.start()) {
                continue;
            }

            if function_depth == 0 {
                let start_col = mat.start() + 1;
                let end_col = mat.end() + 1;

                let diagnostic =
                    create_local_outside_function_diagnostic(line_num, start_col, end_col);

                result.add(diagnostic);
            }
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    // ===== Manual Property Tests =====

    #[test]
    fn prop_sc2168_comments_never_diagnosed() {
        // Property: Comment lines should never produce diagnostics
        let test_cases = vec![
            "# local var=\"value\"",
            "  # local x=5",
            "\t# local name=\"test\"",
        ];

        for code in test_cases {
            let result = check(code);
            assert_eq!(result.diagnostics.len(), 0);
        }
    }

    #[test]
    fn prop_sc2168_local_in_bash_function_never_diagnosed() {
        // Property: local in bash-style function never diagnosed
        let test_cases = vec![
            "function test() {\n    local var=\"value\"\n}",
            "function myFunc {\n    local x=5\n}",
            "function foo() {\nlocal name=\"test\"\n}",
        ];

        for code in test_cases {
            let result = check(code);
            assert_eq!(result.diagnostics.len(), 0);
        }
    }

    #[test]
    fn prop_sc2168_local_in_posix_function_never_diagnosed() {
        // Property: local in POSIX-style function never diagnosed
        let test_cases = vec![
            "test() {\n    local var=\"value\"\n}",
            "myFunc() {\n    local x=5\n}",
            "foo() {\nlocal name=\"test\"\n}",
        ];

        for code in test_cases {
            let result = check(code);
            assert_eq!(result.diagnostics.len(), 0);
        }
    }

    #[test]
    fn prop_sc2168_local_in_nested_function_never_diagnosed() {
        // Property: local in nested functions never diagnosed
        let test_cases = vec![
            "outer() {\n    local var1=\"outer\"\n    inner() {\n        local var2=\"inner\"\n    }\n}",
            "function outer() {\n    function inner() {\n        local x=5\n    }\n    local y=10\n}",
        ];

        for code in test_cases {
            let result = check(code);
            assert_eq!(result.diagnostics.len(), 0);
        }
    }

    #[test]
    fn prop_sc2168_local_at_top_level_always_diagnosed() {
        // Property: local at top level should always be diagnosed
        let test_cases = vec![
            "local var=\"value\"",
            "local x=5",
            "local name=\"test\"",
            "local -r CONST=\"value\"",
        ];

        for code in test_cases {
            let result = check(code);
            assert_eq!(result.diagnostics.len(), 1, "Should diagnose: {}", code);
            assert!(result.diagnostics[0].message.contains("local"));
        }
    }

    #[test]
    fn prop_sc2168_multiple_top_level_locals_all_diagnosed() {
        // Property: Multiple top-level locals should all be diagnosed
        let code = "local var1=\"a\"\nlocal var2=\"b\"\nlocal var3=\"c\"";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 3);
    }

    #[test]
    fn prop_sc2168_diagnostic_code_always_sc2168() {
        // Property: All diagnostics must have code "SC2168"
        let code = "local a=\"x\"\nlocal b=\"y\"";
        let result = check(code);

        for diagnostic in &result.diagnostics {
            assert_eq!(&diagnostic.code, "SC2168");
        }
    }

    #[test]
    fn prop_sc2168_diagnostic_severity_always_error() {
        // Property: All diagnostics must be Error severity
        let code = "local var=\"value\"";
        let result = check(code);

        for diagnostic in &result.diagnostics {
            assert_eq!(diagnostic.severity, Severity::Error);
        }
    }

    #[test]
    fn prop_sc2168_no_local_keyword_never_diagnosed() {
        // Property: Code without 'local' keyword should never be diagnosed
        let test_cases = vec![
            "var=\"value\"",
            "function test() {\n    var=\"value\"\n}",
            "echo \"hello\"",
            "# Just a comment",
        ];

        for code in test_cases {
            let result = check(code);
            assert_eq!(result.diagnostics.len(), 0);
        }
    }

    #[test]
    fn prop_sc2168_empty_source_no_diagnostics() {
        // Property: Empty source should produce no diagnostics
        let result = check("");
        assert_eq!(result.diagnostics.len(), 0);
    }

    // ===== Helper Function Tests =====

    #[test]
    fn test_is_inside_quotes_single_quotes() {
        let line = "echo 'local'";
        assert!(is_inside_quotes(line, 6)); // 'l' of local
        assert!(is_inside_quotes(line, 7)); // 'o' of local
        assert!(is_inside_quotes(line, 10)); // 'l' of local
        assert!(!is_inside_quotes(line, 0)); // 'e' of echo
        assert!(!is_inside_quotes(line, 5)); // opening quote
    }

    #[test]
    fn test_is_inside_quotes_double_quotes() {
        let line = "echo \"local\"";
        assert!(is_inside_quotes(line, 6)); // 'l' of local
        assert!(is_inside_quotes(line, 7)); // 'o' of local
        assert!(is_inside_quotes(line, 10)); // 'l' of local
        assert!(!is_inside_quotes(line, 0)); // 'e' of echo
        assert!(!is_inside_quotes(line, 5)); // opening quote
    }

    #[test]
    fn test_is_inside_quotes_no_quotes() {
        let line = "echo local";
        assert!(!is_inside_quotes(line, 0)); // 'e' of echo
        assert!(!is_inside_quotes(line, 5)); // 'l' of local
        assert!(!is_inside_quotes(line, 9)); // 'l' of local
    }

    #[test]
    fn test_is_inside_quotes_mixed_quotes() {
        let line = "echo 'text' \"local\" 'more'";
        assert!(is_inside_quotes(line, 6)); // Inside 'text'
        assert!(is_inside_quotes(line, 13)); // Inside "local"
        assert!(is_inside_quotes(line, 21)); // Inside 'more'
        assert!(!is_inside_quotes(line, 11)); // Between quotes
    }

    #[test]
    fn test_is_inside_quotes_escaped_quotes() {
        let line = "echo 'it\\'s' local";
        assert!(is_inside_quotes(line, 8)); // Inside 'it\'s'
        assert!(!is_inside_quotes(line, 13)); // 'local' outside quotes
    }

    // ===== Issue #16 Tests =====

    #[test]
    fn test_sc2168_issue_016_local_in_printf_single_quotes() {
        let code = r#"@printf 'Starting local server on port 8080...\n'"#;
        let result = check(code);
        assert_eq!(
            result.diagnostics.len(),
            0,
            "Should not trigger on 'local' in single-quoted string"
        );
    }

    #[test]
    fn test_sc2168_issue_016_local_in_echo_double_quotes() {
        let code = r#"@echo "Connecting to local database""#;
        let result = check(code);
        assert_eq!(
            result.diagnostics.len(),
            0,
            "Should not trigger on 'local' in double-quoted string"
        );
    }

    #[test]
    fn test_sc2168_issue_016_local_in_various_string_contexts() {
        let code = r#"
@echo "local variable"
@printf 'local server\n'
@echo 'localhost'
@echo "locale settings"
"#;
        let result = check(code);
        assert_eq!(
            result.diagnostics.len(),
            0,
            "Should not trigger on 'local' as part of words in strings"
        );
    }

    #[test]
    fn test_sc2168_actual_local_keyword_still_caught() {
        // Verify we still catch real errors
        let code = r#"
local var="bad"
echo "local in string is ok"
"#;
        let result = check(code);
        assert_eq!(
            result.diagnostics.len(),
            1,
            "Should still catch actual 'local' keyword at top level"
        );
    }

    // ===== Original Unit Tests =====

    #[test]
    fn test_sc2168_local_at_top_level() {
        let code = r#"local var="value""#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
        assert_eq!(result.diagnostics[0].code, "SC2168");
        assert_eq!(result.diagnostics[0].severity, Severity::Error);
    }

    #[test]
    fn test_sc2168_local_in_function_ok() {
        let code = r#"
function test() {
    local var="value"
}
"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2168_local_in_posix_function_ok() {
        let code = r#"
test() {
    local var="value"
}
"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2168_multiple_locals_top_level() {
        let code = r#"
local var1="a"
local var2="b"
"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 2);
    }

    #[test]
    fn test_sc2168_nested_function() {
        let code = r#"
outer() {
    local var1="outer"
    inner() {
        local var2="inner"
    }
}
"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2168_after_function() {
        let code = r#"
test() {
    local var="inside"
}
local outside="error"
"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2168_mixed() {
        let code = r#"
local bad="top level"
function good() {
    local ok="inside"
}
local bad2="also top level"
"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 2);
    }

    #[test]
    fn test_sc2168_one_line_function() {
        let code = r#"test() { local var="ok"; }"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2168_function_keyword() {
        let code = r#"
function myFunc {
    local var="value"
}
"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2168_no_local() {
        let code = r#"
var="global"
function test() {
    echo "$var"
}
"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    // ===== Property Tests for Quote Tracking (Issue #16) =====

    #[test]
    fn prop_sc2168_local_in_any_single_quoted_string_never_diagnosed() {
        // Property: 'local' in single quotes should never trigger, regardless of surrounding text
        let test_cases = vec![
            "echo 'local'",
            "printf 'Starting local server'",
            "'local variable'",
            "echo 'before' 'local' 'after'",
            "test 'local test case'",
            "@printf 'Starting local server on port 8080...\n'",
        ];

        for code in test_cases {
            let result = check(code);
            assert_eq!(
                result.diagnostics.len(),
                0,
                "Should not diagnose 'local' in single quotes: {}",
                code
            );
        }
    }

    #[test]
    fn prop_sc2168_local_in_any_double_quoted_string_never_diagnosed() {
        // Property: 'local' in double quotes should never trigger, regardless of surrounding text
        let test_cases = vec![
            "echo \"local\"",
            "printf \"Starting local server\"",
            "\"local variable\"",
            "echo \"before\" \"local\" \"after\"",
            "test \"local test case\"",
            "@echo \"Connecting to local database\"",
        ];

        for code in test_cases {
            let result = check(code);
            assert_eq!(
                result.diagnostics.len(),
                0,
                "Should not diagnose 'local' in double quotes: {}",
                code
            );
        }
    }

    #[test]
    fn prop_sc2168_local_substring_in_quotes_never_diagnosed() {
        // Property: 'local' as part of another word in quotes should never trigger
        let test_cases = vec![
            "echo 'localhost'",
            "echo 'locale'",
            "echo 'localtime'",
            "echo \"localhost\"",
            "echo \"locale\"",
            "echo \"localtime\"",
            "@echo 'localhost settings'",
            "@echo \"locale configuration\"",
        ];

        for code in test_cases {
            let result = check(code);
            assert_eq!(
                result.diagnostics.len(),
                0,
                "Should not diagnose 'local' substring in quotes: {}",
                code
            );
        }
    }

    #[test]
    fn prop_sc2168_mixed_quotes_only_quoted_local_ignored() {
        // Property: Only 'local' in quotes should be ignored, unquoted 'local' should be caught
        let test_cases = vec![
            ("echo 'local'", 0),              // In quotes - OK
            ("local var=\"value\"", 1),       // Not in quotes - ERROR
            ("echo 'local' && local x=5", 1), // Mixed: quoted OK, unquoted ERROR
            ("echo \"local\" \"test\"", 0),   // All quoted - OK
        ];

        for (code, expected_count) in test_cases {
            let result = check(code);
            assert_eq!(
                result.diagnostics.len(),
                expected_count,
                "Wrong diagnostic count for: {}",
                code
            );
        }
    }

    #[test]
    fn prop_sc2168_quote_escaping_handled_correctly() {
        // Property: Escaped quotes should be handled correctly
        let test_cases = vec![
            ("echo 'it\\'s local'", 0),             // 'local' still in quotes
            ("echo \"she said \\\"local\\\"\"", 0), // 'local' still in quotes
        ];

        for (code, expected_count) in test_cases {
            let result = check(code);
            assert_eq!(
                result.diagnostics.len(),
                expected_count,
                "Wrong diagnostic count for: {}",
                code
            );
        }
    }
}
