// SC2122: '>=' is not a valid comparison operator. Use -ge
//
// In [ ] test, use -ge for numeric >= comparison, not >=.
// The >= syntax is for string comparison in [[ ]], not arithmetic in [ ].
//
// Examples:
// Bad:
//   [ $x >= 10 ]              // Wrong operator in [ ]
//   if [ $count >= 5 ]        // Won't work as expected
//
// Good:
//   [ $x -ge 10 ]             // Arithmetic comparison
//   [[ $str >= "abc" ]]       // String comparison (but rarely useful)
//
// Impact: Comparison doesn't work correctly, treated as string

use crate::linter::{Diagnostic, LintResult, Severity, Span};
use regex::Regex;

static SINGLE_BRACKET_GE: std::sync::LazyLock<Regex> = std::sync::LazyLock::new(|| {
    // Match: [ ... >= ... ] (not [[]])
    Regex::new(r"\[\s[^\]]*>=\s*[^\]]*\]").unwrap()
});

pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();

    for (line_num, line) in source.lines().enumerate() {
        let line_num = line_num + 1;

        if line.trim_start().starts_with('#') {
            continue;
        }

        // Skip lines with [[ ]] (double brackets allow >=)
        if line.contains("[[") {
            continue;
        }

        for mat in SINGLE_BRACKET_GE.find_iter(line) {
            let matched = mat.as_str();

            // Find position of >= within match
            if let Some(ge_pos) = matched.find(">=") {
                let start_col = mat.start() + ge_pos + 1;
                let end_col = start_col + 2; // length of ">="

                let diagnostic = Diagnostic::new(
                    "SC2122",
                    Severity::Error,
                    "'>=' is not a valid operator in [ ]. Use -ge for numeric, or [[ ]] for lexical comparison".to_string(),
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
    fn test_sc2122_single_bracket_ge() {
        let code = "[ $x >= 10 ]";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2122_ge_correct_ok() {
        let code = "[ $x -ge 10 ]";
        let result = check(code);
        // -ge is correct
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2122_double_bracket_ok() {
        let code = "[[ $x >= 10 ]]";
        let result = check(code);
        // [[ ]] allows >= for lexical comparison
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2122_comment_ok() {
        let code = "# [ $x >= 10 ]";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2122_in_if() {
        let code = "if [ $count >= 5 ]; then";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2122_with_variable() {
        let code = r#"[ "$VAR" >= "$MIN" ]"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2122_spaces() {
        let code = "[ $x  >=  10 ]";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2122_multiline() {
        let code = r#"
if [ $value >= 100 ]; then
    echo "Large"
fi
"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2122_gt_ok() {
        let code = "[ $x -gt 10 ]";
        let result = check(code);
        // -gt is correct, not testing >=
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2122_string_comparison() {
        let code = r#"[ "$str1" >= "$str2" ]"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }
}
