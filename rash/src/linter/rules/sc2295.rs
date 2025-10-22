// SC2295: Expansions inside ${} need to be quoted separately
use crate::linter::{Diagnostic, LintResult, Severity, Span};
use once_cell::sync::Lazy;
use regex::Regex;

static UNQUOTED_EXPANSION_IN_BRACE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"\$\{[a-zA-Z_][a-zA-Z0-9_]*:-\$[a-zA-Z_]").unwrap());

pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();
    for (line_num, line) in source.lines().enumerate() {
        let line_num = line_num + 1;
        if line.trim_start().starts_with('#') {
            continue;
        }

        if UNQUOTED_EXPANSION_IN_BRACE.is_match(line) {
            let diagnostic = Diagnostic::new(
                "SC2295",
                Severity::Warning,
                r#"Expansions inside ${} should be quoted: use ${var:-"$default"} not ${var:-$default}"#.to_string(),
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
    fn test_sc2295_unquoted_expansion() {
        let code = "${VAR:-$DEFAULT}";
        assert_eq!(check(code).diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2295_quoted_ok() {
        let code = r#"${VAR:-"$DEFAULT"}"#;
        assert_eq!(check(code).diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2295_literal_default_ok() {
        let code = "${VAR:-default}";
        assert_eq!(check(code).diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2295_comment() {
        let code = "# ${VAR:-$DEFAULT}";
        assert_eq!(check(code).diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2295_empty() {
        assert_eq!(check("").diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2295_normal() {
        assert_eq!(check("echo test").diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2295_simple_var_ok() {
        let code = "$VAR";
        assert_eq!(check(code).diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2295_assign_default_ok() {
        let code = "${VAR:=default}";
        assert_eq!(check(code).diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2295_braced_default() {
        let code = "${CONFIG:-${HOME}/default}";
        assert_eq!(check(code).diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2295_in_command() {
        let code = "echo ${PATH:-$HOME}";
        assert_eq!(check(code).diagnostics.len(), 1);
    }
}
