//! SC1095: Space between function name and `()` with `function` keyword
//!
//! When using the `function` keyword, having a space between the function
//! name and `()` is non-standard and can cause issues in some shells.
//! Use either `function f { ... }` or `f() { ... }`, not `function f () { ... }`.
//!
//! # Examples
//!
//! Bad:
//! ```bash
//! function greet () {
//!     echo hello
//! }
//! ```
//!
//! Good:
//! ```bash
//! function greet {
//!     echo hello
//! }
//! # or
//! greet() {
//!     echo hello
//! }
//! ```

use crate::linter::{Diagnostic, LintResult, Severity, Span};

pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();

    for (line_num, line) in source.lines().enumerate() {
        let line_num = line_num + 1;
        let trimmed = line.trim();

        if trimmed.starts_with('#') || trimmed.is_empty() {
            continue;
        }

        // Look for: function <name> ()
        // The pattern is: "function" followed by spaces, an identifier, spaces, then "()"
        if let Some(rest) = trimmed.strip_prefix("function ") {
            let rest = rest.trim_start();

            // Extract function name (identifier)
            let name_end = rest
                .find(|c: char| !c.is_ascii_alphanumeric() && c != '_')
                .unwrap_or(rest.len());

            if name_end == 0 {
                continue;
            }

            let after_name = &rest[name_end..];
            let after_name_trimmed = after_name.trim_start();

            // Check if there's "()" after the name AND there was whitespace between name and ()
            if after_name_trimmed.starts_with("()") && after_name.len() != after_name_trimmed.len()
            {
                let line_offset = line.find("function").unwrap_or(0);
                let col = line_offset + 1;
                result.add(Diagnostic::new(
                    "SC1095",
                    Severity::Warning,
                    "Use 'function name { .. }' or 'name() { .. }', not 'function name () { .. }'",
                    Span::new(line_num, col, line_num, col + trimmed.len()),
                ));
            }
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sc1095_function_with_space_parens() {
        let result = check("function greet () {");
        assert_eq!(result.diagnostics.len(), 1);
        assert_eq!(result.diagnostics[0].code, "SC1095");
        assert_eq!(result.diagnostics[0].severity, Severity::Warning);
    }

    #[test]
    fn test_sc1095_function_without_parens_ok() {
        let result = check("function greet {");
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc1095_plain_function_def_ok() {
        let result = check("greet() {");
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc1095_function_nospace_parens_ok() {
        // function name() is borderline but not this rule's target
        let result = check("function greet() {");
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc1095_comment_not_flagged() {
        let result = check("# function greet () {");
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc1095_multiple_spaces() {
        let result = check("function myFunc   () {");
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc1095_with_body() {
        let script = "function test_func () {\n    echo hello\n}";
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 1);
    }
}
