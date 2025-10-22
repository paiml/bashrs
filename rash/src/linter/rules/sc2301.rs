// SC2301: Use [[ -v array[0] ]] to check if array element exists
use crate::linter::{Diagnostic, LintResult, Severity, Span};
use once_cell::sync::Lazy;
use regex::Regex;

static ARRAY_ELEMENT_CHECK: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r#"(?:^|[^\[])\[\s+-[nz]\s+"\$\{[a-zA-Z_][a-zA-Z0-9_]*\[\d+\]\}""#).unwrap()
});

pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();
    for (line_num, line) in source.lines().enumerate() {
        let line_num = line_num + 1;
        if line.trim_start().starts_with('#') {
            continue;
        }

        if ARRAY_ELEMENT_CHECK.is_match(line) {
            let diagnostic = Diagnostic::new(
                "SC2301",
                Severity::Info,
                "Use [[ -v array[index] ]] to check if array element exists".to_string(),
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
    fn test_sc2301_array_element_check() {
        let code = r#"[ -n "${arr[0]}" ]"#;
        assert_eq!(check(code).diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2301_v_flag_ok() {
        let code = "[[ -v arr[0] ]]";
        assert_eq!(check(code).diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2301_simple_var_ok() {
        let code = r#"[ -n "$var" ]"#;
        assert_eq!(check(code).diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2301_comment() {
        let code = r#"# [ -n "${arr[0]}" ]"#;
        assert_eq!(check(code).diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2301_empty() {
        assert_eq!(check("").diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2301_normal() {
        assert_eq!(check("echo test").diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2301_z_test() {
        let code = r#"[ -z "${arr[5]}" ]"#;
        assert_eq!(check(code).diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2301_double_bracket_ok() {
        let code = r#"[[ -n "${arr[0]}" ]]"#;
        assert_eq!(check(code).diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2301_array_all_ok() {
        let code = r#"[ -n "${arr[@]}" ]"#;
        assert_eq!(check(code).diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2301_different_index() {
        let code = r#"[ -n "${data[42]}" ]"#;
        assert_eq!(check(code).diagnostics.len(), 1);
    }
}
