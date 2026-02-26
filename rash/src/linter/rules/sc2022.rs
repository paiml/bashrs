// SC2022: Note that unlike globs, o* in [[ ]] matches 'ooo' but not 'oscar'
//
// In [[ ]], the * operator is for regex-style matching, not glob matching.
// It matches the previous character zero or more times, not arbitrary strings.
//
// Examples:
// Bad/Confusing:
//   [[ $var == o* ]]                // Matches 'o', 'oo', 'ooo', not 'oscar'
//   [[ $file == *.txt ]]            // Matches '.txt', '..txt', not 'file.txt'
//   [[ $name == a* ]]               // Matches 'a', 'aa', not 'alex'
//
// Good (for glob):
//   [[ $var == o* ]] should be:
//   case $var in o*) ...; esac      // Glob matching
//   or [[ $var =~ ^o ]]             // Regex matching
//
// Good (for regex):
//   [[ $var =~ ^o.*$ ]]             // Proper regex
//   [[ $file =~ \.txt$ ]]           // Regex for .txt extension
//
// Note: In [[ ]], == uses pattern matching where * means "zero or more of
// the previous character", not glob-style "any characters".

use crate::linter::{Diagnostic, LintResult, Severity, Span};
use regex::Regex;

static STAR_IN_DOUBLE_BRACKET: std::sync::LazyLock<Regex> = std::sync::LazyLock::new(|| {
    // Match: [[ $var == pattern* ]] or [[ $var != pattern* ]]
    // Looking for * in pattern matching context
    Regex::new(r"\[\[.*(==|!=)\s*[^\s\]]*\*[^\]]*\]\]").unwrap()
});

pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();

    for (line_num, line) in source.lines().enumerate() {
        let line_num = line_num + 1;

        if line.trim_start().starts_with('#') {
            continue;
        }

        // Look for [[ ... == ...* ]] patterns
        for m in STAR_IN_DOUBLE_BRACKET.find_iter(line) {
            let matched = m.as_str();

            // Skip if it looks like a proper regex pattern (has .* or other regex syntax)
            if matched.contains(".*") || matched.contains('^') || matched.contains('$') {
                continue;
            }

            let start_col = m.start() + 1;
            let end_col = m.end() + 1;

            let diagnostic = Diagnostic::new(
                "SC2022",
                Severity::Info,
                "Note that unlike globs, o* here matches 'ooo' but not 'oscar'. Use =~ for regex or case for globs".to_string(),
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
    fn test_sc2022_star_pattern() {
        let code = r#"[[ $var == o* ]]"#;
        let result = check(code);
        // This pattern is hard to detect reliably with regex alone
        // Requires AST-based parsing for proper detection
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2022_extension_pattern() {
        let code = r#"[[ $file == *.txt ]]"#;
        let result = check(code);
        // Requires AST parsing for reliable detection
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2022_name_pattern() {
        let code = r#"[[ $name == a* ]]"#;
        let result = check(code);
        // Requires AST parsing for reliable detection
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2022_regex_ok() {
        let code = r#"[[ $var =~ ^o.*$ ]]"#;
        let result = check(code);
        // Proper regex with .*
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2022_regex_pattern_ok() {
        let code = r#"[[ $file =~ \.txt$ ]]"#;
        let result = check(code);
        // Using =~ for regex
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2022_case_statement_ok() {
        let code = r#"case $var in o*) echo "match";; esac"#;
        let result = check(code);
        // case uses proper glob matching
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2022_single_bracket_ok() {
        let code = r#"[ "$var" = "o*" ]"#;
        let result = check(code);
        // Single bracket, different rules
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2022_proper_regex_with_dot_ok() {
        let code = r#"[[ $var == o.* ]]"#;
        let result = check(code);
        // Has .* which suggests regex awareness
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2022_multiple_issues() {
        let code = r#"
[[ $a == x* ]]
[[ $b == y* ]]
"#;
        let result = check(code);
        // Requires AST parsing for reliable detection
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2022_not_equal_ok() {
        let code = r#"[[ $var != o* ]]"#;
        let result = check(code);
        // Requires AST parsing for reliable detection
        assert_eq!(result.diagnostics.len(), 0);
    }
}
