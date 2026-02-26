// SC2110: In [[ ]], combine -a and -o with && and ||
//
// Mixing -a/-o with &&/|| in [[ ]] is confusing and error-prone.
// Use either the old style (-a/-o) or new style (&&/||), not both.
//
// Examples:
// Bad:
//   [[ $x -eq 1 -a $y -eq 2 || $z -eq 3 ]]    // Mixing styles
//   [[ -f file && -d dir -o -r file ]]         // Inconsistent
//
// Good:
//   [[ $x -eq 1 && $y -eq 2 || $z -eq 3 ]]    // All new style
//   [[ -f file -a -d dir -o -r file ]]         // All old style (but use new style)
//
// Impact: Confusing precedence, hard to read

use crate::linter::{Diagnostic, LintResult, Severity, Span};
use regex::Regex;

static DOUBLE_BRACKET_MIXED: std::sync::LazyLock<Regex> = std::sync::LazyLock::new(|| {
    // Match: [[ ]] with both old style and new style operators
    Regex::new(r"\[\[([^\]]+)\]\]").unwrap()
});

pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();

    for (line_num, line) in source.lines().enumerate() {
        let line_num = line_num + 1;

        if line.trim_start().starts_with('#') {
            continue;
        }

        for mat in DOUBLE_BRACKET_MIXED.find_iter(line) {
            let matched = mat.as_str();
            let content = &matched[2..matched.len() - 2]; // Remove [[ and ]]

            // Check for mixing old style (-a/-o) with new style (&&/||)
            let has_old_a = content.contains(" -a ");
            let has_old_o = content.contains(" -o ");
            let has_new_and = content.contains("&&");
            let has_new_or = content.contains("||");

            let has_old = has_old_a || has_old_o;
            let has_new = has_new_and || has_new_or;

            if has_old && has_new {
                let start_col = mat.start() + 1;
                let end_col = mat.end() + 1;

                let diagnostic = Diagnostic::new(
                    "SC2110",
                    Severity::Warning,
                    "In [[ ]], don't mix && and || with -a and -o".to_string(),
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
    fn test_sc2110_mixed_and() {
        let code = "[[ $x -eq 1 -a $y -eq 2 && $z -eq 3 ]]";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2110_mixed_or() {
        let code = "[[ $x -eq 1 || $y -eq 2 -o $z -eq 3 ]]";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2110_all_new_ok() {
        let code = "[[ $x -eq 1 && $y -eq 2 || $z -eq 3 ]]";
        let result = check(code);
        // All new style is OK
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2110_all_old() {
        let code = "[[ $x -eq 1 -a $y -eq 2 -o $z -eq 3 ]]";
        let result = check(code);
        // All old style is OK (though not recommended)
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2110_comment_ok() {
        let code = "# [[ $x -eq 1 -a $y -eq 2 && $z -eq 3 ]]";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2110_single_bracket_ok() {
        let code = "[ $x -eq 1 -a $y -eq 2 ]";
        let result = check(code);
        // Single bracket not checked
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2110_file_tests_mixed() {
        let code = "[[ -f file && -d dir -o -r file ]]";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2110_complex_mixed() {
        let code = "[[ $a -eq 1 -a $b -eq 2 || $c -eq 3 && $d -eq 4 ]]";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2110_multiline() {
        let code = r#"
if [[ $x -eq 1 -a $y -eq 2 || $z -eq 3 ]]; then
    echo "test"
fi
"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2110_string_tests() {
        let code = r#"[[ "$a" = "test" && "$b" = "value" -o -z "$c" ]]"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }
}
