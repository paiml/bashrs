// SC2248: Prefer [[ ]] over [ ] for regex matching
use crate::linter::{Diagnostic, LintResult, Severity, Span};
use once_cell::sync::Lazy;
use regex::Regex;

static REGEX_IN_SINGLE_BRACKET: Lazy<Regex> = Lazy::new(|| {
    // Match: [ "$var" =~ pattern ] (regex in single bracket)
    Regex::new(r"\[\s+.*=~").unwrap()
});

pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();
    for (line_num, line) in source.lines().enumerate() {
        let line_num = line_num + 1;
        if line.trim_start().starts_with('#') {
            continue;
        }

        // Skip [[ ]]
        if line.contains("[[") {
            continue;
        }

        if REGEX_IN_SINGLE_BRACKET.is_match(line) {
            let diagnostic = Diagnostic::new(
                "SC2248",
                Severity::Warning,
                "Use [[ ]] instead of [ ] for regex matching with =~".to_string(),
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
    fn test_sc2248_regex_in_single_bracket() {
        let code = r#"[ "$var" =~ pattern ]"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }
    #[test]
    fn test_sc2248_double_bracket_ok() {
        let code = r#"[[ "$var" =~ pattern ]]"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }
    #[test]
    fn test_sc2248_equality_ok() {
        let code = r#"[ "$a" = "$b" ]"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }
    #[test]
    fn test_sc2248_in_if() {
        let code = r#"if [ "$str" =~ ^[0-9]+$ ]; then"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }
    #[test]
    fn test_sc2248_comment_skipped() {
        let code = r#"# [ "$var" =~ pattern ]"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }
    #[test]
    fn test_sc2248_regex_pattern() {
        let code = r#"[ $email =~ @.*\. ]"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }
    #[test]
    fn test_sc2248_no_code() {
        let code = "";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }
    #[test]
    fn test_sc2248_normal_command() {
        let code = "echo test";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }
    #[test]
    fn test_sc2248_numeric_test_ok() {
        let code = "[ $count -gt 5 ]";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }
    #[test]
    fn test_sc2248_file_test_ok() {
        let code = r#"[ -f "$file" ]"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }
}
