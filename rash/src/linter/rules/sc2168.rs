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

pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();
    let lines: Vec<&str> = source.lines().collect();
    let mut function_depth: usize = 0;

    for (i, line) in lines.iter().enumerate() {
        let line_num = i + 1;
        let trimmed = line.trim_start();

        // Skip comments
        if trimmed.starts_with('#') {
            continue;
        }

        // Track function depth with braces
        if FUNCTION_START.is_match(line) {
            // Look ahead to see if there's an opening brace
            if line.contains('{') {
                function_depth += 1;
            } else if i + 1 < lines.len() && lines[i + 1].contains('{') {
                // Brace on next line
                function_depth += 1;
            }
        }

        // Track closing braces
        if FUNCTION_END.is_match(line) && function_depth > 0 {
            function_depth = function_depth.saturating_sub(1);
        }

        // Count opening braces on current line
        function_depth += line.matches('{').count();
        // Subtract closing braces
        if function_depth > 0 {
            let closing = line.matches('}').count();
            function_depth = function_depth.saturating_sub(closing);
        }

        // Check for local keyword outside functions
        if let Some(mat) = LOCAL_KEYWORD.find(line) {
            if function_depth == 0 {
                let start_col = mat.start() + 1;
                let end_col = mat.end() + 1;

                let diagnostic = Diagnostic::new(
                    "SC2168",
                    Severity::Error,
                    "'local' is only valid in functions",
                    Span::new(line_num, start_col, line_num, end_col),
                );

                result.add(diagnostic);
            }
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
