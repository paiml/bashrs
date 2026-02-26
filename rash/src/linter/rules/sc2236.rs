// SC2236: Use -n instead of ! -z for positive tests
use crate::linter::{Diagnostic, LintResult, Severity, Span};
use regex::Regex;

static NEGATED_Z_TEST: std::sync::LazyLock<Regex> = std::sync::LazyLock::new(|| {
    // Match: [ ! -z $var ] or [[ ! -z $var ]]
    Regex::new(r"\[\[?\s*!\s+-z\s+").unwrap()
});

pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();
    for (line_num, line) in source.lines().enumerate() {
        let line_num = line_num + 1;
        if line.trim_start().starts_with('#') {
            continue;
        }

        if NEGATED_Z_TEST.is_match(line) {
            let diagnostic = Diagnostic::new(
                "SC2236",
                Severity::Info,
                "Use -n instead of ! -z for positive string length tests (more readable)"
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
    fn test_sc2236_negated_z() {
        let code = r#"[ ! -z "$var" ]"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }
    #[test]
    fn test_sc2236_double_bracket() {
        let code = r#"[[ ! -z "$str" ]]"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }
    #[test]
    fn test_sc2236_use_n_ok() {
        let code = r#"[ -n "$var" ]"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }
    #[test]
    fn test_sc2236_plain_z_ok() {
        let code = r#"[ -z "$var" ]"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }
    #[test]
    fn test_sc2236_comment_skipped() {
        let code = r#"# [ ! -z "$var" ]"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }
    #[test]
    fn test_sc2236_with_spaces() {
        let code = r#"[  !  -z  "$x"  ]"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }
    #[test]
    fn test_sc2236_in_if() {
        let code = r#"if [ ! -z "$value" ]; then"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }
    #[test]
    fn test_sc2236_no_code() {
        let code = "";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }
    #[test]
    fn test_sc2236_normal_command() {
        let code = "echo test";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }
    #[test]
    fn test_sc2236_other_negation() {
        let code = r#"[ ! -f "$file" ]"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0); // Not -z
    }
}
