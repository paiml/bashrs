// SC2062: Quote the grep pattern so the shell won't interpret it

use crate::linter::{Diagnostic, LintResult, Severity, Span};
use once_cell::sync::Lazy;
use regex::Regex;

static GREP_UNQUOTED: Lazy<Regex> = Lazy::new(|| {
    // Match grep followed by optional flags, then a pattern with glob chars
    // (?:\s+-\S+)* handles flags like -r, -i, --recursive, etc.
    Regex::new(r"\bgrep(?:\s+-\S+)*\s+\S*[\*\?\[]").unwrap()
});

pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();

    for (line_num, line) in source.lines().enumerate() {
        let line_num = line_num + 1;

        if line.trim_start().starts_with('#') {
            continue;
        }

        // Simple check: if line has quoted grep, skip
        if line.contains("grep '") || line.contains("grep \"") {
            continue;
        }

        if let Some(mat) = GREP_UNQUOTED.find(line) {
            let start_col = mat.start() + 1;
            let end_col = mat.end() + 1;

            let diagnostic = Diagnostic::new(
                "SC2062",
                Severity::Warning,
                "Quote the grep pattern so the shell won't interpret it".to_string(),
                Span::new(line_num, start_col, line_num, end_col),
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
    fn test_sc2062_unquoted_glob() {
        let code = r#"grep *.txt file"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2062_bracket_expression() {
        let code = r#"grep [0-9]+ data"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2062_question_mark() {
        let code = r#"grep file?.txt data"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2062_quoted_single_ok() {
        let code = r#"grep '*.txt' file"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2062_quoted_double_ok() {
        let code = r#"grep "*.txt" file"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2062_no_special_chars_ok() {
        let code = r#"grep pattern file"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2062_comment_ok() {
        let code = r#"# grep *.txt file"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2062_with_flags() {
        let code = r#"grep -r *.log ."#;
        let result = check(code);
        // Regex now handles optional flags before pattern
        assert_eq!(result.diagnostics.len(), 1); // Has glob, should warn
    }

    #[test]
    fn test_sc2062_pipe() {
        let code = r#"cat file | grep [ERROR]"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2062_variable_ok() {
        let code = r#"grep "$pattern" file"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }
}
