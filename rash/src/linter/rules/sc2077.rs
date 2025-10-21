// SC2077: Quote regex parameter in =~ to prevent word splitting
//
// When using =~ in [[ ]], variables containing regex patterns should typically
// be unquoted to allow pattern matching. However, if you want literal matching,
// quote the pattern or use a variable.
//
// Examples:
// Bad (if pattern has spaces):
//   pattern="foo bar"
//   [[ $text =~ $pattern ]]    // Word splits, matches "foo" OR "bar"
//
// Good:
//   [[ $text =~ "$pattern" ]]  // Literal match for "foo bar"
//   [[ $text =~ foo\ bar ]]    // Escaped space in pattern
//
// Impact: Unexpected pattern matching behavior

use crate::linter::{Diagnostic, LintResult, Severity, Span};
use once_cell::sync::Lazy;
use regex::Regex;

static REGEX_WITH_UNQUOTED_VAR: Lazy<Regex> = Lazy::new(|| {
    // Match: [[ ... =~ $var ]] or [[ ... =~ ${var} ]] (unquoted variable in regex position)
    Regex::new(r"\[\[[^\]]*=~\s+\$(\{[a-zA-Z_][a-zA-Z0-9_]*\}|[a-zA-Z_][a-zA-Z0-9_]*)[^\]]*\]\]")
        .unwrap()
});

pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();

    for (line_num, line) in source.lines().enumerate() {
        let line_num = line_num + 1;

        if line.trim_start().starts_with('#') {
            continue;
        }

        // This is a style preference - flagging for awareness
        // Some prefer unquoted for regex, quoted for literal
        for mat in REGEX_WITH_UNQUOTED_VAR.find_iter(line) {
            let start_col = mat.start() + 1;
            let end_col = mat.end() + 1;

            let diagnostic = Diagnostic::new(
                "SC2077",
                Severity::Info,
                "Regex variable pattern may word split. Quote for literal match or ensure no spaces"
                    .to_string(),
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
    fn test_sc2077_unquoted_var() {
        let code = r#"[[ $text =~ $pattern ]]"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2077_with_condition() {
        let code = r#"if [[ $text =~ $regex ]]; then echo "match"; fi"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2077_quoted_ok() {
        let code = r#"[[ $text =~ "$pattern" ]]"#;
        let result = check(code);
        // Quoted is OK (literal match)
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2077_literal_pattern_ok() {
        let code = r#"[[ $text =~ [0-9]+ ]]"#;
        let result = check(code);
        // Literal regex pattern, not a variable
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2077_comment_ok() {
        let code = r#"# [[ $text =~ $pattern ]]"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2077_braced_var() {
        let code = r#"[[ $text =~ ${pattern} ]]"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2077_negation() {
        let code = r#"[[ ! $text =~ $pattern ]]"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2077_escaped_space() {
        let code = r#"[[ $text =~ foo\ bar ]]"#;
        let result = check(code);
        // Escaped space in literal pattern
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2077_multiple() {
        let code = r#"[[ $a =~ $p1 ]] && [[ $b =~ $p2 ]]"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 2);
    }

    #[test]
    fn test_sc2077_single_bracket_ignored() {
        let code = r#"[ $text = $pattern ]"#;
        let result = check(code);
        // Single bracket uses different rule
        assert_eq!(result.diagnostics.len(), 0);
    }
}
