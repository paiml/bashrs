// SC2299: Parameter expansion only allows literals here
use crate::linter::{Diagnostic, LintResult, Severity, Span};
use once_cell::sync::Lazy;
use regex::Regex;

static PARAM_WITH_VAR: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"\$\{[a-zA-Z_][a-zA-Z0-9_]*:[^}]*\$").unwrap());

pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();
    for (line_num, line) in source.lines().enumerate() {
        let line_num = line_num + 1;
        if line.trim_start().starts_with('#') {
            continue;
        }

        if PARAM_WITH_VAR.is_match(line) {
            let diagnostic = Diagnostic::new(
                "SC2299",
                Severity::Error,
                "Parameter expansions can't use variables in offset/length".to_string(),
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
    fn test_sc2299_variable_in_offset() {
        let code = "${var:$start:5}";
        assert_eq!(check(code).diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2299_literal_ok() {
        let code = "${var:0:5}";
        assert_eq!(check(code).diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2299_arithmetic_ok() {
        let code = "${var:0}";
        assert_eq!(check(code).diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2299_comment() {
        let code = "# ${var:$n:5}";
        assert_eq!(check(code).diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2299_empty() {
        assert_eq!(check("").diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2299_normal() {
        assert_eq!(check("echo test").diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2299_default_ok() {
        let code = "${var:-default}";
        assert_eq!(check(code).diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2299_simple_var_ok() {
        let code = "$var";
        assert_eq!(check(code).diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2299_length_ok() {
        let code = "${#var}";
        assert_eq!(check(code).diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2299_variable_in_length() {
        let code = "${var:0:$len}";
        assert_eq!(check(code).diagnostics.len(), 1);
    }
}
