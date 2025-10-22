// SC2259: Prefer [[ ]] over grep for simple string checks
use crate::linter::{Diagnostic, LintResult, Severity, Span};
use once_cell::sync::Lazy;
use regex::Regex;

static GREP_FOR_SUBSTRING: Lazy<Regex> =
    Lazy::new(|| Regex::new(r#"echo\s+[^|]+\|\s*grep\s+-q"#).unwrap());

pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();
    for (line_num, line) in source.lines().enumerate() {
        let line_num = line_num + 1;
        if line.trim_start().starts_with('#') {
            continue;
        }

        if GREP_FOR_SUBSTRING.is_match(line) {
            let diagnostic = Diagnostic::new(
                "SC2259",
                Severity::Info,
                "Consider using [[ $var == *substring* ]] instead of echo/grep for substring matching".to_string(),
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
    fn test_sc2259_echo_grep() {
        assert_eq!(
            check(r#"echo "$var" | grep -q pattern"#).diagnostics.len(),
            1
        );
    }
    #[test]
    fn test_sc2259_use_bracket_ok() {
        assert_eq!(check(r#"[[ "$var" == *pattern* ]]"#).diagnostics.len(), 0);
    }
    #[test]
    fn test_sc2259_file_grep_ok() {
        assert_eq!(check("grep -q pattern file").diagnostics.len(), 0);
    }
    #[test]
    fn test_sc2259_comment() {
        assert_eq!(check("# comment").diagnostics.len(), 0);
    }
    #[test]
    fn test_sc2259_empty() {
        assert_eq!(check("").diagnostics.len(), 0);
    }
    #[test]
    fn test_sc2259_normal() {
        assert_eq!(check("echo test").diagnostics.len(), 0);
    }
    #[test]
    fn test_sc2259_cmd_pipe_grep() {
        assert_eq!(check("ls | grep pattern").diagnostics.len(), 0);
    }
    #[test]
    fn test_sc2259_echo_literal() {
        assert_eq!(check(r#"echo "test" | grep -q foo"#).diagnostics.len(), 1);
    }
    #[test]
    fn test_sc2259_case_ok() {
        assert_eq!(check("case $x in *pattern*) ;;").diagnostics.len(), 0);
    }
    #[test]
    fn test_sc2259_regex_ok() {
        assert_eq!(check(r#"[[ "$x" =~ pattern ]]"#).diagnostics.len(), 0);
    }
}
