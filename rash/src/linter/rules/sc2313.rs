// SC2313: Quote array indices to prevent globbing
use crate::linter::{Diagnostic, LintResult, Severity, Span};
use regex::Regex;

static UNQUOTED_INDEX: std::sync::LazyLock<Regex> =
    std::sync::LazyLock::new(|| Regex::new(r"\$\{[a-zA-Z_][a-zA-Z0-9_]*\[\*\]\}").unwrap());

pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();
    for (line_num, line) in source.lines().enumerate() {
        let line_num = line_num + 1;
        if line.trim_start().starts_with('#') {
            continue;
        }

        if UNQUOTED_INDEX.is_match(line) && !line.contains("\"${") {
            let diagnostic = Diagnostic::new(
                "SC2313",
                Severity::Warning,
                "Quote array expansions to prevent globbing: \"${array[*]}\"".to_string(),
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
    fn test_sc2313_unquoted_star() {
        let code = r#"echo ${arr[*]}"#;
        assert_eq!(check(code).diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2313_quoted_ok() {
        let code = r#"echo "${arr[*]}""#;
        assert_eq!(check(code).diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2313_at_ok() {
        let code = r#"echo ${arr[@]}"#;
        assert_eq!(check(code).diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2313_comment() {
        let code = r#"# echo ${arr[*]}"#;
        assert_eq!(check(code).diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2313_empty() {
        assert_eq!(check("").diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2313_normal() {
        assert_eq!(check("echo test").diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2313_index_ok() {
        let code = r#"echo ${arr[0]}"#;
        assert_eq!(check(code).diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2313_in_assignment() {
        let code = r#"result=${arr[*]}"#;
        assert_eq!(check(code).diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2313_quoted_star_ok() {
        let code = r#"result="${arr[*]}""#;
        assert_eq!(check(code).diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2313_multiple() {
        let code = r#"
echo ${x[*]}
echo ${y[*]}
"#;
        assert_eq!(check(code).diagnostics.len(), 2);
    }
}
