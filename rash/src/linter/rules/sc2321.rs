// SC2321: This && is not a logical AND but part of [[ ]]
use crate::linter::{Diagnostic, LintResult, Severity, Span};
use once_cell::sync::Lazy;
use regex::Regex;

static AND_OUTSIDE_BRACKET: Lazy<Regex> = Lazy::new(|| Regex::new(r"\]\]\s*&&\s*\[\[").unwrap());

pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();
    for (line_num, line) in source.lines().enumerate() {
        let line_num = line_num + 1;
        if line.trim_start().starts_with('#') {
            continue;
        }

        if AND_OUTSIDE_BRACKET.is_match(line) {
            let diagnostic = Diagnostic::new(
                "SC2321",
                Severity::Info,
                "Use [[ condition && condition ]] instead of [[ condition ]] && [[ condition ]]"
                    .to_string(),
                Span::new(line_num, 1, line_num, line.len() + 1),
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
    fn test_sc2321_separate_conditions() {
        let code = "[[ $x -eq 1 ]] && [[ $y -eq 2 ]]";
        assert_eq!(check(code).diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2321_combined_ok() {
        let code = "[[ $x -eq 1 && $y -eq 2 ]]";
        assert_eq!(check(code).diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2321_command_ok() {
        let code = "[[ $x -eq 1 ]] && echo ok";
        assert_eq!(check(code).diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2321_comment() {
        let code = "# [[ $x ]] && [[ $y ]]";
        assert_eq!(check(code).diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2321_empty() {
        assert_eq!(check("").diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2321_normal() {
        assert_eq!(check("echo test").diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2321_single_bracket_ok() {
        let code = "[ $x -eq 1 ] && [ $y -eq 2 ]";
        assert_eq!(check(code).diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2321_or_operator_ok() {
        let code = "[[ $x -eq 1 ]] || [[ $y -eq 2 ]]";
        assert_eq!(check(code).diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2321_three_conditions() {
        let code = "[[ $a ]] && [[ $b ]] && [[ $c ]]";
        // Should match first instance
        assert_eq!(check(code).diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2321_multiline() {
        let code = r#"
[[ $x -eq 1 ]] &&
[[ $y -eq 2 ]]
"#;
        assert_eq!(check(code).diagnostics.len(), 0);
    }
}
