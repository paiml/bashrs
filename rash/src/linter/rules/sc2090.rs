// SC2090: Quotes/backslashes in expansion will be treated literally. Use array
//
// When expanding variables containing quotes, those quotes are literal
// characters, not shell quoting. Use arrays for proper word splitting.
//
// Examples:
// Bad:
//   args="-name '*.txt'"
//   find . $args                 // '*.txt' including quotes
//
// Good:
//   args=(-name '*.txt')
//   find . "${args[@]}"          // Proper expansion
//
// Impact: Arguments parsed incorrectly, literal quotes passed

use crate::linter::{Diagnostic, LintResult, Severity, Span};
use regex::Regex;

static VAR_EXPANSION_UNQUOTED: std::sync::LazyLock<Regex> = std::sync::LazyLock::new(|| {
    // Match: command ... $var (unquoted variable expansion)
    // Simplified: just look for command followed by unquoted $var
    Regex::new(r"\b(find|grep|ssh|rsync|curl|wget)\s+\S*\s*\$[a-zA-Z_0-9]+").unwrap()
});

pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();

    for (line_num, line) in source.lines().enumerate() {
        let line_num = line_num + 1;

        if line.trim_start().starts_with('#') {
            continue;
        }

        // This is informational - only flag if we see potential issues
        // Don't flag every unquoted expansion, only suspicious ones
        for mat in VAR_EXPANSION_UNQUOTED.find_iter(line) {
            let matched = mat.as_str();

            // Skip if the variable appears to be quoted
            if matched.contains(r#""$"#) || matched.contains(r"'$") {
                continue;
            }

            let start_col = mat.start() + 1;
            let end_col = mat.end() + 1;

            let diagnostic = Diagnostic::new(
                "SC2090",
                Severity::Info,
                "Quotes/backslashes in this variable will be treated literally. Use array"
                    .to_string(),
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
    fn test_sc2090_find_with_var() {
        let code = "find . $args";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2090_grep_with_var() {
        let code = "grep $pattern file.txt";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2090_array_expansion_ok() {
        let code = r#"find . "${args[@]}""#;
        let result = check(code);
        // Array expansion is correct
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2090_quoted_ok() {
        let code = r#"find . "$args""#;
        let result = check(code);
        // Quoted expansion (though may not work as expected)
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2090_echo_ok() {
        let code = "echo $var";
        let result = check(code);
        // echo doesn't need special handling
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2090_comment_ok() {
        let code = "# find . $args";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2090_ssh_with_var() {
        let code = "ssh host $cmd";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2090_multiple() {
        let code = "find . $opts && grep $pattern file";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 2);
    }

    #[test]
    fn test_sc2090_curl() {
        let code = "curl $options http://example.com";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2090_positional() {
        let code = "find . $1";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }
}
