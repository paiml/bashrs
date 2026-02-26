// SC2282: Use {var:?} to require variables to be set
use crate::linter::{Diagnostic, LintResult, Severity, Span};
use regex::Regex;

static VAR_OR_EMPTY: std::sync::LazyLock<Regex> =
    std::sync::LazyLock::new(|| Regex::new(r"\$\{[a-zA-Z_][a-zA-Z0-9_]*:-\}").unwrap());

pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();
    for (line_num, line) in source.lines().enumerate() {
        let line_num = line_num + 1;
        if line.trim_start().starts_with('#') {
            continue;
        }

        if VAR_OR_EMPTY.is_match(line) {
            let diagnostic = Diagnostic::new(
                "SC2282",
                Severity::Info,
                "Use ${var:?} to fail if variable is unset, rather than defaulting to empty"
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
    fn test_sc2282_default_empty() {
        let code = "${var:-}";
        assert_eq!(check(code).diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2282_require_ok() {
        let code = "${var:?}";
        assert_eq!(check(code).diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2282_default_value_ok() {
        let code = "${var:-default}";
        assert_eq!(check(code).diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2282_comment() {
        let code = "# ${var:-}";
        assert_eq!(check(code).diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2282_empty() {
        assert_eq!(check("").diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2282_normal() {
        assert_eq!(check("echo test").diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2282_plain_var_ok() {
        let code = "$var";
        assert_eq!(check(code).diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2282_assign_default_ok() {
        let code = "${var:=default}";
        assert_eq!(check(code).diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2282_alternative_ok() {
        let code = "${var:+value}";
        assert_eq!(check(code).diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2282_in_command() {
        let code = "echo ${CONFIG:-}";
        assert_eq!(check(code).diagnostics.len(), 1);
    }
}
