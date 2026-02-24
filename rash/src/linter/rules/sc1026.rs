// SC1026: Use `||` and `&&` in `[[ ]]`, not `-a` and `-o`
//
// Inside [[ ]], the -a and -o operators are not supported as logical operators.
// Use && and || instead. Note: -a and -o ARE valid (though deprecated) in [ ].
//
// Examples:
// Bad:
//   [[ $x -eq 1 -o $y -eq 2 ]]
//   [[ -f file -a -r file ]]
//
// Good:
//   [[ $x -eq 1 || $y -eq 2 ]]
//   [[ -f file && -r file ]]
//   [ $x -eq 1 -o $y -eq 2 ]   # -o is valid in [ ] (SC2055 handles deprecation)

use crate::linter::{Diagnostic, LintResult, Severity, Span};
use once_cell::sync::Lazy;
use regex::Regex;

/// Matches [[ ... -a ... ]] or [[ ... -o ... ]]
static DOUBLE_BRACKET_AO: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"\[\[[^\]]*\s-([ao])\s[^\]]*\]\]").unwrap()
});

pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();

    for (line_num, line) in source.lines().enumerate() {
        let line_num = line_num + 1;
        let trimmed = line.trim_start();

        if trimmed.starts_with('#') {
            continue;
        }

        for caps in DOUBLE_BRACKET_AO.captures_iter(line) {
            let full = caps.get(0).unwrap();
            let op = caps.get(1).unwrap().as_str();
            let replacement = if op == "a" { "&&" } else { "||" };
            let start_col = full.start() + 1;
            let end_col = full.end() + 1;

            let diagnostic = Diagnostic::new(
                "SC1026",
                Severity::Warning,
                format!(
                    "Use `{}` in [[ ]], not `-{}`",
                    replacement, op
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
    fn test_sc1026_double_bracket_o() {
        let code = "[[ $x -eq 1 -o $y -eq 2 ]]";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
        assert_eq!(result.diagnostics[0].code, "SC1026");
        assert!(result.diagnostics[0].message.contains("||"));
    }

    #[test]
    fn test_sc1026_double_bracket_a() {
        let code = "[[ -f file -a -r file ]]";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
        assert!(result.diagnostics[0].message.contains("&&"));
    }

    #[test]
    fn test_sc1026_single_bracket_ok() {
        let code = "[ $x -eq 1 -o $y -eq 2 ]";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc1026_correct_syntax_ok() {
        let code = "[[ $x -eq 1 || $y -eq 2 ]]";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc1026_comment_ok() {
        let code = "# [[ $x -eq 1 -o $y -eq 2 ]]";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }
}
