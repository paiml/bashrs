// SC2315: Use ${var:+replacement} to replace non-empty values
use crate::linter::{Diagnostic, LintResult, Severity, Span};
use once_cell::sync::Lazy;
use regex::Regex;

static CONDITIONAL_REPLACE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"\[\s+-n\s+\$\{?[a-zA-Z_][a-zA-Z0-9_]*\}?\s+\]\s*&&\s*echo").unwrap());

pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();
    for (line_num, line) in source.lines().enumerate() {
        let line_num = line_num + 1;
        if line.trim_start().starts_with('#') {
            continue;
        }

        if CONDITIONAL_REPLACE.is_match(line) {
            let diagnostic = Diagnostic::new(
                "SC2315",
                Severity::Info,
                "Consider using ${var:+value} for conditional replacement".to_string(),
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
    fn test_sc2315_conditional_echo() {
        let code = "[ -n $var ] && echo \"set\"";
        assert_eq!(check(code).diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2315_expansion_ok() {
        let code = "echo ${var:+set}";
        assert_eq!(check(code).diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2315_simple_echo_ok() {
        let code = "echo test";
        assert_eq!(check(code).diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2315_comment() {
        let code = "# [ -n $x ] && echo y";
        assert_eq!(check(code).diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2315_empty() {
        assert_eq!(check("").diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2315_normal() {
        assert_eq!(check("echo test").diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2315_z_test_ok() {
        let code = "[ -z $var ] && echo \"unset\"";
        assert_eq!(check(code).diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2315_braced_var() {
        let code = "[ -n ${config} ] && echo \"found\"";
        assert_eq!(check(code).diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2315_non_echo_ok() {
        let code = "[ -n $var ] && exit 1";
        assert_eq!(check(code).diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2315_multiple() {
        let code = r#"
[ -n $x ] && echo "x"
[ -n $y ] && echo "y"
"#;
        assert_eq!(check(code).diagnostics.len(), 2);
    }
}
