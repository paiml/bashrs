// SC2225: Backticks in assignments can interfere with line breaks
use crate::linter::{Diagnostic, LintResult, Severity, Span};
use regex::Regex;

static BACKTICK_ASSIGNMENT: std::sync::LazyLock<Regex> = std::sync::LazyLock::new(|| {
    // Match: var=`command` or var=`...`
    Regex::new(r"\b\w+\s*=\s*`").unwrap()
});

pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();
    for (line_num, line) in source.lines().enumerate() {
        let line_num = line_num + 1;
        if line.trim_start().starts_with('#') {
            continue;
        }

        if BACKTICK_ASSIGNMENT.is_match(line) {
            let diagnostic = Diagnostic::new(
                "SC2225",
                Severity::Info,
                "Use $(...) instead of backticks for command substitution in assignments"
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
    fn test_sc2225_backtick_assignment() {
        let code = r#"result=`date`"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }
    #[test]
    fn test_sc2225_dollar_paren_ok() {
        let code = r#"result=$(date)"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }
    #[test]
    fn test_sc2225_backtick_command_substitution() {
        let code = r#"output=`ls -la`"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }
    #[test]
    fn test_sc2225_nested_backticks() {
        let code = r#"var=`echo \`date\``"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }
    #[test]
    fn test_sc2225_comment_skipped() {
        let code = r#"# var=`date`"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }
    #[test]
    fn test_sc2225_no_assignment() {
        let code = r#"echo `date`"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0); // Not an assignment
    }
    #[test]
    fn test_sc2225_multiple_assignments() {
        let code = "a=`cmd1`\nb=`cmd2`";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 2);
    }
    #[test]
    fn test_sc2225_with_space() {
        let code = r#"x = `pwd`"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }
    #[test]
    fn test_sc2225_no_code() {
        let code = "";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }
    #[test]
    fn test_sc2225_literal_string() {
        let code = r#"var="literal value""#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }
}
