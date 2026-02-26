// SC2270: Prefer getopts over manual argument parsing
use crate::linter::{Diagnostic, LintResult, Severity, Span};
use regex::Regex;

static MANUAL_ARG_PARSE: std::sync::LazyLock<Regex> =
    std::sync::LazyLock::new(|| Regex::new(r#"\[\s+"\$[0-9]+"\s*==?\s*"-[a-zA-Z]""#).unwrap());

pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();
    for (line_num, line) in source.lines().enumerate() {
        let line_num = line_num + 1;
        if line.trim_start().starts_with('#') {
            continue;
        }

        if MANUAL_ARG_PARSE.is_match(line) {
            let diagnostic = Diagnostic::new(
                "SC2270",
                Severity::Info,
                "Consider using getopts for option parsing instead of manual checks".to_string(),
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
    fn test_sc2270_manual_flag_check() {
        let code = r#"[ "$1" = "-h" ]"#;
        assert_eq!(check(code).diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2270_getopts_ok() {
        let code = "while getopts 'h' opt; do";
        assert_eq!(check(code).diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2270_case_statement_ok() {
        let code = r#"case "$1" in"#;
        assert_eq!(check(code).diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2270_comment() {
        let code = r#"# [ "$1" = "-h" ]"#;
        assert_eq!(check(code).diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2270_empty() {
        assert_eq!(check("").diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2270_normal() {
        assert_eq!(check("echo test").diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2270_double_equals() {
        let code = r#"[ "$1" == "-v" ]"#;
        assert_eq!(check(code).diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2270_positional_param_ok() {
        let code = r#"[ "$1" = "value" ]"#;
        assert_eq!(check(code).diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2270_variable_comparison_ok() {
        let code = r#"[ "$var" = "-h" ]"#;
        assert_eq!(check(code).diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2270_arg2() {
        let code = r#"[ "$2" = "-f" ]"#;
        assert_eq!(check(code).diagnostics.len(), 1);
    }
}
