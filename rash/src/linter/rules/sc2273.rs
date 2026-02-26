// SC2273: Prefer [[ ]] for test operations with variables
use crate::linter::{Diagnostic, LintResult, Severity, Span};
use regex::Regex;

static SINGLE_BRACKET_VAR: std::sync::LazyLock<Regex> = std::sync::LazyLock::new(|| {
    Regex::new(r"(?:^|[^\[])\[\s+\$\{?[a-zA-Z_][a-zA-Z0-9_]*\}?\s+-[a-z]{2}\s+").unwrap()
});

pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();
    for (line_num, line) in source.lines().enumerate() {
        let line_num = line_num + 1;
        if line.trim_start().starts_with('#') {
            continue;
        }

        if SINGLE_BRACKET_VAR.is_match(line) {
            let diagnostic = Diagnostic::new(
                "SC2273",
                Severity::Info,
                "Prefer [[ ]] over [ ] for robustness with variables".to_string(),
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
    fn test_sc2273_single_bracket() {
        let code = "[ $var -gt 10 ]";
        assert_eq!(check(code).diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2273_double_bracket_ok() {
        let code = "[[ $var -gt 10 ]]";
        assert_eq!(check(code).diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2273_braced_var() {
        let code = "[ ${count} -eq 5 ]";
        assert_eq!(check(code).diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2273_comment() {
        let code = "# [ $var -gt 10 ]";
        assert_eq!(check(code).diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2273_empty() {
        assert_eq!(check("").diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2273_normal() {
        assert_eq!(check("echo test").diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2273_literal_ok() {
        let code = r#"[ "value" = "test" ]"#;
        assert_eq!(check(code).diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2273_quoted_var_ok() {
        let code = r#"[ "$var" -gt 10 ]"#;
        assert_eq!(check(code).diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2273_test_command() {
        let code = "[ $x -lt 100 ]";
        assert_eq!(check(code).diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2273_file_test() {
        let code = "[ $file -nt $other ]";
        assert_eq!(check(code).diagnostics.len(), 1);
    }
}
