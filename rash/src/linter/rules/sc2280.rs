// SC2280: Use proper array initialization syntax
use crate::linter::{Diagnostic, LintResult, Severity, Span};
use regex::Regex;

static WRONG_ARRAY_INIT: std::sync::LazyLock<Regex> =
    std::sync::LazyLock::new(|| Regex::new(r"[a-zA-Z_][a-zA-Z0-9_]*=\(\s*\)").unwrap());

pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();
    for (line_num, line) in source.lines().enumerate() {
        let line_num = line_num + 1;
        if line.trim_start().starts_with('#') {
            continue;
        }

        if WRONG_ARRAY_INIT.is_match(line) && !line.contains("declare") && !line.contains("local") {
            let diagnostic = Diagnostic::new(
                "SC2280",
                Severity::Info,
                "Consider using 'declare -a' or 'local -a' for array initialization".to_string(),
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
    fn test_sc2280_plain_array_init() {
        let code = "array=()";
        assert_eq!(check(code).diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2280_declare_ok() {
        let code = "declare -a array=()";
        assert_eq!(check(code).diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2280_local_ok() {
        let code = "local -a array=()";
        assert_eq!(check(code).diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2280_comment() {
        let code = "# array=()";
        assert_eq!(check(code).diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2280_empty() {
        assert_eq!(check("").diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2280_normal() {
        assert_eq!(check("echo test").diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2280_with_values_ok() {
        let code = "array=(val1 val2)";
        assert_eq!(check(code).diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2280_string_var_ok() {
        let code = "var=''";
        assert_eq!(check(code).diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2280_command_subst_ok() {
        let code = "result=$(command)";
        assert_eq!(check(code).diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2280_readonly_array() {
        let code = "my_array=()";
        assert_eq!(check(code).diagnostics.len(), 1);
    }
}
