// SC2065: This is interpreted as a shell file redirection, not a comparison
//
// When > or < appear outside proper test contexts, they're interpreted as
// redirections, not comparison operators. This can cause confusing errors.
//
// Examples:
// Bad:
//   cmd && echo "Success > $file"    // Redirects to $file (confusing)
//   if cmd > /dev/null && echo done  // Redirect, then echo (might be unclear)
//
// Good:
//   cmd && echo "Success: $file"     // Clear message
//   cmd > /dev/null && echo done     // Clearly separate redirect from echo
//   [ $a -gt $b ] && echo "Greater"  // Use proper comparison
//
// Impact: Confusing behavior, unintended redirections

use crate::linter::{Diagnostic, LintResult, Severity, Span};
use regex::Regex;

static REDIRECT_IN_STRING: std::sync::LazyLock<Regex> = std::sync::LazyLock::new(|| {
    // Match strings with > or < that might be confusing
    Regex::new(r#"echo\s+"[^"]*[<>][^"]*\$"#).unwrap()
});

pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();

    for (line_num, line) in source.lines().enumerate() {
        let line_num = line_num + 1;

        if line.trim_start().starts_with('#') {
            continue;
        }

        for mat in REDIRECT_IN_STRING.find_iter(line) {
            let matched_text = mat.as_str();

            // Skip >> and << (append/heredoc redirects are less confusing)
            if matched_text.contains(">>") || matched_text.contains("<<") {
                continue;
            }

            let start_col = mat.start() + 1;
            let end_col = mat.end() + 1;

            let diagnostic = Diagnostic::new(
                "SC2065",
                Severity::Info,
                "This may be interpreted as a shell file redirection. Quote or escape if intentional".to_string(),
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
    fn test_sc2065_redirect_in_message() {
        let code = r#"echo "Success > $file""#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2065_input_redirect() {
        let code = r#"echo "Reading < $input""#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2065_clear_message_ok() {
        let code = r#"echo "Success: $file""#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2065_actual_redirect_ok() {
        let code = r#"echo "Success" > $file"#;
        let result = check(code);
        // Actual redirect (outside quotes), OK
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2065_comparison_ok() {
        let code = r#"[ $a -gt $b ] && echo "Greater""#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2065_comment_ok() {
        let code = r#"# echo "Success > $file""#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2065_single_quotes_ok() {
        let code = r#"echo 'Output > file'"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2065_no_variable_ok() {
        let code = r#"echo "A > B""#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2065_append_redirect() {
        let code = r#"echo "Appended >> $log""#;
        let result = check(code);
        // >> (append redirect) is less confusing than single >, should not warn
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2065_multiple() {
        let code = r#"echo "In: $in > Out: $out""#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }
}
