// SC2200: Brace expansion doesn't happen in [[ ]]. Use separate statements or [ ]
//
// Brace expansions like {a,b,c} are NOT performed inside [[ ]] test expressions.
// They remain literal strings, which is usually not intended.
//
// Examples:
// Bad:
//   [[ $var = {foo,bar} ]]      # Checks if $var equals literal "{foo,bar}"
//   [[ -f file.{txt,log} ]]     # Checks if file named "file.{txt,log}" exists
//
// Good:
//   [[ $var = foo || $var = bar ]]  # Use separate comparisons
//   case $var in foo|bar) ... esac  # Use case statement
//   for ext in txt log; do [[ -f file.$ext ]] && ...; done  # Loop

use crate::linter::{Diagnostic, LintResult, Severity, Span};
use regex::Regex;

static BRACE_EXPANSION: std::sync::LazyLock<Regex> = std::sync::LazyLock::new(|| {
    // Match: {word,word} or {a..z} patterns (but not ${var} parameter expansion)
    // Must have a comma or consecutive dots (..) to be brace expansion
    Regex::new(r"\{[a-zA-Z0-9_/.]+([,]|\.\.)[a-zA-Z0-9_/.]*\}").unwrap()
});

static DOUBLE_BRACKET: std::sync::LazyLock<Regex> =
    std::sync::LazyLock::new(|| Regex::new(r"\[\[.*?\]\]").unwrap());

pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();

    for (line_num, line) in source.lines().enumerate() {
        let line_num = line_num + 1;

        if line.trim_start().starts_with('#') {
            continue;
        }

        // Only check lines with [[ ]]
        if !line.contains("[[") {
            continue;
        }

        // Issue #124: Skip if this is a regex match with =~
        // Regex quantifiers like {1,3} are valid in [[ $var =~ pattern{1,3} ]]
        // Use word boundary check to avoid matching "=" comparison
        if line.contains(" =~ ") || line.contains("]=~") {
            continue;
        }

        // Extract [[ ]] blocks
        for bracket_match in DOUBLE_BRACKET.find_iter(line) {
            let bracket_text = bracket_match.as_str();

            // Check for brace expansions
            if BRACE_EXPANSION.is_match(bracket_text) {
                let start_col = line.find(bracket_text).unwrap_or(0) + 1;
                let end_col = start_col + bracket_text.len();

                let diagnostic = Diagnostic::new(
                    "SC2200",
                    Severity::Warning,
                    "Brace expansion doesn't happen in [[ ]]. Use separate comparisons or a case statement instead".to_string(),
                    Span::new(line_num, start_col, line_num, end_col),
                );

                result.add(diagnostic);
                break; // Only warn once per [[ ]] block
            }
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sc2200_brace_in_comparison() {
        let code = r#"[[ $var = {foo,bar} ]]"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
        assert_eq!(result.diagnostics[0].code, "SC2200");
        assert_eq!(result.diagnostics[0].severity, Severity::Warning);
        assert!(result.diagnostics[0].message.contains("Brace expansion"));
    }

    #[test]
    fn test_sc2200_brace_in_file_test() {
        let code = r#"[[ -f file.{txt,log} ]]"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2200_range_expansion() {
        let code = r#"[[ $num = {1..10} ]]"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2200_separate_comparisons_ok() {
        let code = r#"[[ $var = foo || $var = bar ]]"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2200_single_bracket_ok() {
        let code = r#"[ $var = {foo,bar} ]"#;
        let result = check(code);
        // Single bracket also doesn't expand, but that's expected
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2200_outside_brackets_ok() {
        let code = r#"for file in {a,b,c}.txt; do echo $file; done"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2200_variable_braces_ok() {
        let code = r#"[[ ${var} = test ]]"#;
        let result = check(code);
        // ${var} is parameter expansion, not brace expansion
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2200_multiple_braces() {
        let code = r#"[[ $a = {x,y} && $b = {1,2} ]]"#;
        let result = check(code);
        // Only warns once per [[ ]]
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2200_path_with_braces() {
        let code = r#"[[ -d /path/{dir1,dir2} ]]"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2200_alpha_range() {
        let code = r#"[[ $letter = {a..z} ]]"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    // Issue #124: Regex quantifiers should NOT be flagged
    #[test]
    fn test_issue_124_regex_quantifier_not_flagged() {
        // {1,3} is a valid regex quantifier in =~ context
        let code = r#"[[ $var =~ ^[0-9]{1,3}$ ]]"#;
        let result = check(code);
        assert_eq!(
            result.diagnostics.len(),
            0,
            "SC2200 must NOT flag regex quantifiers in =~ context"
        );
    }

    #[test]
    fn test_issue_124_regex_with_range_not_flagged() {
        let code = r#"[[ $ip =~ ^[0-9]{1,3}\.[0-9]{1,3}\.[0-9]{1,3}\.[0-9]{1,3}$ ]]"#;
        let result = check(code);
        assert_eq!(
            result.diagnostics.len(),
            0,
            "SC2200 must NOT flag IP address regex pattern"
        );
    }

    #[test]
    fn test_issue_124_brace_without_regex_still_flagged() {
        // Brace expansion without =~ should still be flagged
        // Note: Use 2-element brace pattern as regex matches {a,b} format
        let code = r#"[[ $var = {foo,bar} ]]"#;
        let result = check(code);
        assert_eq!(
            result.diagnostics.len(),
            1,
            "SC2200 should still flag brace expansion in non-regex context"
        );
    }
}
