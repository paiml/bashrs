// SC2095: Redirections only apply to the commands they precede
//
// In bash, redirections are part of the command they're attached to, not the
// surrounding context like if/while/for statements.
//
// Examples:
// Bad:
//   if foo > file; then echo "Redirected"; fi
//   # Only 'foo' output is redirected, not 'echo'
//
//   while read line > output.txt; do
//     echo "$line"
//   done
//   # Only 'read' is redirected, not the loop body
//
// Good:
//   if foo; then echo "Not redirected"; fi > file
//   # Entire if block redirected
//
//   {
//     while read line; do
//       echo "$line"
//     done
//   } > output.txt
//   # Entire loop redirected

use crate::linter::{Diagnostic, LintResult, Severity, Span};
use once_cell::sync::Lazy;
use regex::Regex;

static IF_WITH_REDIRECT: Lazy<Regex> = Lazy::new(|| {
    // Match: if command > file; then
    Regex::new(r"\bif\s+[^;]+>\s*[^\s;]+\s*;").unwrap()
});

static WHILE_WITH_REDIRECT: Lazy<Regex> = Lazy::new(|| {
    // Match: while command > file; do
    Regex::new(r"\bwhile\s+[^;]+>\s*[^\s;]+\s*;").unwrap()
});

static FOR_WITH_REDIRECT: Lazy<Regex> = Lazy::new(|| {
    // Match: for var in ... > file; do
    Regex::new(r"\bfor\s+[^;]+>\s*[^\s;]+\s*;").unwrap()
});

pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();

    for (line_num, line) in source.lines().enumerate() {
        let line_num = line_num + 1;

        if line.trim_start().starts_with('#') {
            continue;
        }

        // Check for 'if condition > file; then'
        if let Some(cap) = IF_WITH_REDIRECT.captures(line) {
            let start_col = cap.get(0).unwrap().start() + 1;
            let end_col = cap.get(0).unwrap().end() + 1;

            let diagnostic = Diagnostic::new(
                "SC2095",
                Severity::Info,
                "Redirections only apply to the condition command, not the if block. Move redirection after 'fi' to redirect entire block".to_string(),
                Span::new(line_num, start_col, line_num, end_col),
            );

            result.add(diagnostic);
        }

        // Check for 'while condition > file; do'
        if let Some(cap) = WHILE_WITH_REDIRECT.captures(line) {
            let start_col = cap.get(0).unwrap().start() + 1;
            let end_col = cap.get(0).unwrap().end() + 1;

            let diagnostic = Diagnostic::new(
                "SC2095",
                Severity::Info,
                "Redirections only apply to the condition command, not the loop body. Wrap loop in { } and redirect after closing brace".to_string(),
                Span::new(line_num, start_col, line_num, end_col),
            );

            result.add(diagnostic);
        }

        // Check for 'for var in ... > file; do'
        if let Some(cap) = FOR_WITH_REDIRECT.captures(line) {
            let start_col = cap.get(0).unwrap().start() + 1;
            let end_col = cap.get(0).unwrap().end() + 1;

            let diagnostic = Diagnostic::new(
                "SC2095",
                Severity::Info,
                "Redirections only apply to the for statement itself, not the loop body. Wrap loop in { } and redirect after closing brace".to_string(),
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
    fn test_sc2095_if_with_redirect() {
        let code = r#"if foo > file.txt; then echo "test"; fi"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
        assert_eq!(result.diagnostics[0].code, "SC2095");
        assert_eq!(result.diagnostics[0].severity, Severity::Info);
        assert!(result.diagnostics[0].message.contains("condition"));
    }

    #[test]
    fn test_sc2095_while_with_redirect() {
        let code = r#"while read line > output.txt; do echo "$line"; done"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
        assert!(result.diagnostics[0].message.contains("condition"));
    }

    #[test]
    fn test_sc2095_for_with_redirect() {
        let code = r#"for i in 1 2 3 > nums.txt; do echo $i; done"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2095_if_redirect_after_fi_ok() {
        let code = r#"if foo; then echo "test"; fi > file.txt"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2095_while_wrapped_ok() {
        let code = r#"{ while read line; do echo "$line"; done; } > output.txt"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2095_for_wrapped_ok() {
        let code = r#"{ for i in 1 2 3; do echo $i; done; } > nums.txt"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2095_simple_command_ok() {
        let code = r#"echo "test" > file.txt"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2095_if_no_redirect_ok() {
        let code = r#"if foo; then echo "test"; fi"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2095_while_no_redirect_ok() {
        let code = r#"while read line; do echo "$line"; done"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2095_multiple_issues() {
        let code = r#"
if test > a.txt; then echo "1"; fi
while read x > b.txt; do echo "2"; done
"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 2);
    }
}
