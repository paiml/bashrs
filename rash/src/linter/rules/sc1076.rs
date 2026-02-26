// SC1076: Use $((...)) for arithmetic, not $[...]
//
// The $[...] syntax is a deprecated bash arithmetic expansion. It has been
// removed in modern shells and should be replaced with $((...)).
//
// Examples:
// Bad:
//   echo $[1+2]         # Deprecated syntax
//   x=$[a+b]            # Old arithmetic
//   echo $[ 5 * 3 ]     # Spaces don't help
//
// Good:
//   echo $((1+2))       # Standard arithmetic
//   x=$((a+b))          # Modern syntax
//   echo $(( 5 * 3 ))   # Works everywhere

use crate::linter::{Diagnostic, LintResult, Severity, Span};
use regex::Regex;

/// Matches $[ which starts the deprecated arithmetic syntax.
/// We must NOT confuse with ${arr[0]} array access.
static DEPRECATED_ARITH: std::sync::LazyLock<Regex> =
    std::sync::LazyLock::new(|| Regex::new(r"\$\[").expect("SC1076 regex must compile"));

pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();

    for (line_num, line) in source.lines().enumerate() {
        let line_num = line_num + 1;
        let trimmed = line.trim_start();
        if trimmed.starts_with('#') {
            continue;
        }

        for mat in DEPRECATED_ARITH.find_iter(line) {
            let start = mat.start();

            // Make sure this is not inside ${...[ (array access)
            // Array access looks like ${arr[...]} — the ${ comes before
            if start >= 1 && line.as_bytes()[start] == b'$' {
                // Check if there's a ${ pattern that would make this an array
                // Array: ${arr[0]} — the [ is inside braces, after a variable name
                // Deprecated arith: $[expr] — the [ immediately follows $
                // Since our regex matches $[, it won't match ${arr[0]} because
                // there the $ is followed by { not [
            }

            let start_col = start + 1;
            let end_col = mat.end() + 1;

            result.add(Diagnostic::new(
                "SC1076",
                Severity::Error,
                "Use $((...)) for arithmetic, not deprecated $[...] syntax.".to_string(),
                Span::new(line_num, start_col, line_num, end_col),
            ));
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sc1076_deprecated_arith() {
        let code = "echo $[1+2]";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
        assert_eq!(result.diagnostics[0].code, "SC1076");
        assert_eq!(result.diagnostics[0].severity, Severity::Error);
        assert!(result.diagnostics[0].message.contains("$((...))"));
    }

    #[test]
    fn test_sc1076_deprecated_with_spaces() {
        let code = "echo $[ 5 * 3 ]";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc1076_assignment() {
        let code = "x=$[a+b]";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc1076_modern_arith_ok() {
        let code = "echo $((1+2))";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc1076_array_access_ok() {
        let code = "echo ${arr[0]}";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc1076_comment_ok() {
        let code = "# echo $[1+2]";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc1076_multiple() {
        let code = "x=$[1+2]; y=$[3+4]";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 2);
    }
}
