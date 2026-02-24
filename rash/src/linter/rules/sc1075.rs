// SC1075: Use `elif` not `else if`
//
// In shell, `else if` creates a nested if inside the else clause, requiring
// an extra `fi`. Use `elif` for proper else-if chains.
//
// Examples:
// Bad:
//   if [ "$x" -eq 1 ]; then
//       echo one
//   else if [ "$x" -eq 2 ]; then
//       echo two
//   fi
//   fi
//
// Good:
//   if [ "$x" -eq 1 ]; then
//       echo one
//   elif [ "$x" -eq 2 ]; then
//       echo two
//   fi

use crate::linter::{Diagnostic, LintResult, Severity, Span};
use once_cell::sync::Lazy;
use regex::Regex;

/// Matches `else if` (with optional whitespace) that should be `elif`
static ELSE_IF: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"\belse\s+if\b").unwrap()
});

pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();

    for (line_num, line) in source.lines().enumerate() {
        let line_num = line_num + 1;
        let trimmed = line.trim_start();

        if trimmed.starts_with('#') {
            continue;
        }

        for mat in ELSE_IF.find_iter(line) {
            let start_col = mat.start() + 1;
            let end_col = mat.end() + 1;

            let diagnostic = Diagnostic::new(
                "SC1075",
                Severity::Warning,
                "Use `elif` instead of `else if`".to_string(),
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
    fn test_sc1075_else_if() {
        let code = "else if [ \"$x\" -eq 2 ]; then";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
        assert_eq!(result.diagnostics[0].code, "SC1075");
        assert_eq!(result.diagnostics[0].severity, Severity::Warning);
    }

    #[test]
    fn test_sc1075_else_if_extra_spaces() {
        let code = "else   if [ \"$x\" -eq 2 ]; then";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc1075_elif_ok() {
        let code = "elif [ \"$x\" -eq 2 ]; then";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc1075_multiline() {
        let code = r#"
if [ "$x" -eq 1 ]; then
    echo one
else if [ "$x" -eq 2 ]; then
    echo two
fi
fi
"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc1075_comment_ok() {
        let code = "# else if should be elif";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }
}
