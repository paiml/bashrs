// SC2268: Avoid unnecessary subshells
use crate::linter::{Diagnostic, LintResult, Severity, Span};
use once_cell::sync::Lazy;
use regex::Regex;

static UNNECESSARY_SUBSHELL: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"\(\s*[a-zA-Z_][a-zA-Z0-9_]*=[^;)]+\s*\)").unwrap()
});

pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();
    for (line_num, line) in source.lines().enumerate() {
        let line_num = line_num + 1;
        if line.trim_start().starts_with('#') {
            continue;
        }

        if UNNECESSARY_SUBSHELL.is_match(line) {
            let diagnostic = Diagnostic::new(
                "SC2268",
                Severity::Info,
                "Avoid unnecessary subshells for simple assignments".to_string(),
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
    fn test_sc2268_unnecessary_subshell() {
        let code = "( var=value )";
        assert_eq!(check(code).diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2268_direct_assignment_ok() {
        let code = "var=value";
        assert_eq!(check(code).diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2268_needed_subshell_ok() {
        let code = "( cd /tmp && make )";
        assert_eq!(check(code).diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2268_comment() {
        let code = "# ( var=value )";
        assert_eq!(check(code).diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2268_empty() {
        assert_eq!(check("").diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2268_normal() {
        assert_eq!(check("echo test").diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2268_command_substitution_ok() {
        let code = "result=$(command)";
        assert_eq!(check(code).diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2268_arithmetic_ok() {
        let code = "(( x = y + 1 ))";
        assert_eq!(check(code).diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2268_test_ok() {
        let code = "if ( test -f file ); then";
        assert_eq!(check(code).diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2268_multiple_statements_ok() {
        let code = "( var1=a; var2=b )";
        assert_eq!(check(code).diagnostics.len(), 0);
    }
}
