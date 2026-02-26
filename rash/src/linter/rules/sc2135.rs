// SC2135: Unexpected 'then' after condition
//
// The 'then' keyword should appear after an 'if' condition, not in other contexts.
// This usually indicates a syntax error or misplaced keyword.
//
// Examples:
// Bad:
//   if [ -f file ] then echo "exists"; fi     // Missing semicolon before then
//   while true then echo "loop"; done         // 'then' with while (should be 'do')
//
// Good:
//   if [ -f file ]; then echo "exists"; fi    // Semicolon before then
//   if [ -f file ]                             // Or on separate line
//   then echo "exists"; fi
//   while true; do echo "loop"; done          // 'do' for while loops
//
// Impact: Syntax error, script will fail

use crate::linter::{Diagnostic, LintResult, Severity, Span};
use regex::Regex;

static MISSING_SEMICOLON_THEN: std::sync::LazyLock<Regex> = std::sync::LazyLock::new(|| {
    // Match: ] then (missing semicolon)
    Regex::new(r"\]\s+then\b").unwrap()
});

static WHILE_THEN: std::sync::LazyLock<Regex> = std::sync::LazyLock::new(|| {
    // Match: while ... then (should be do)
    Regex::new(r"\bwhile\b[^\n]*\bthen\b").unwrap()
});

static FOR_THEN: std::sync::LazyLock<Regex> = std::sync::LazyLock::new(|| {
    // Match: for ... then (should be do)
    Regex::new(r"\bfor\b[^\n]*\bthen\b").unwrap()
});

pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();

    for (line_num, line) in source.lines().enumerate() {
        let line_num = line_num + 1;

        if line.trim_start().starts_with('#') {
            continue;
        }

        // Check for ] then (missing semicolon)
        for mat in MISSING_SEMICOLON_THEN.find_iter(line) {
            let start_col = mat.start() + 1;
            let end_col = mat.end() + 1;

            let diagnostic = Diagnostic::new(
                "SC2135",
                Severity::Error,
                "Missing semicolon before 'then'. Use ]; then or put 'then' on next line"
                    .to_string(),
                Span::new(line_num, start_col, line_num, end_col),
            );

            result.add(diagnostic);
        }

        // Check for while ... then
        if WHILE_THEN.is_match(line) {
            if let Some(mat) = WHILE_THEN.find(line) {
                let start_col = mat.start() + 1;
                let end_col = mat.end() + 1;

                let diagnostic = Diagnostic::new(
                    "SC2135",
                    Severity::Error,
                    "'while' loops use 'do', not 'then'. Change 'then' to 'do'".to_string(),
                    Span::new(line_num, start_col, line_num, end_col),
                );

                result.add(diagnostic);
            }
        }

        // Check for for ... then
        if FOR_THEN.is_match(line) {
            if let Some(mat) = FOR_THEN.find(line) {
                let start_col = mat.start() + 1;
                let end_col = mat.end() + 1;

                let diagnostic = Diagnostic::new(
                    "SC2135",
                    Severity::Error,
                    "'for' loops use 'do', not 'then'. Change 'then' to 'do'".to_string(),
                    Span::new(line_num, start_col, line_num, end_col),
                );

                result.add(diagnostic);
            }
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sc2135_missing_semicolon() {
        let code = "if [ -f file ] then echo \"exists\"; fi";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
        assert!(result.diagnostics[0].message.contains("semicolon"));
    }

    #[test]
    fn test_sc2135_with_semicolon_ok() {
        let code = "if [ -f file ]; then echo \"exists\"; fi";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2135_while_then() {
        let code = "while true then echo \"loop\"; done";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
        assert!(result.diagnostics[0].message.contains("'do'"));
    }

    #[test]
    fn test_sc2135_while_do_ok() {
        let code = "while true; do echo \"loop\"; done";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2135_for_then() {
        let code = "for i in 1 2 3 then echo $i; done";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
        assert!(result.diagnostics[0].message.contains("'do'"));
    }

    #[test]
    fn test_sc2135_for_do_ok() {
        let code = "for i in 1 2 3; do echo $i; done";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2135_comment_ok() {
        let code = "# if [ -f file ] then echo \"test\"";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2135_multiline_if_ok() {
        let code = r#"
if [ -f file ]
then
    echo "exists"
fi
"#;
        let result = check(code);
        // Separate lines is OK
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2135_double_bracket_ok() {
        let code = "if [[ -f file ]]; then echo \"exists\"; fi";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2135_multiple_errors() {
        let code = r#"
if [ -f file ] then echo "1"; fi
while true then echo "2"; done
"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 2);
    }
}
