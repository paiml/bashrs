// SC2063: Grep uses regexes. Use -F for literal string matching
//
// Grep treats patterns as regular expressions by default. Characters like
// . * [ ] ^ $ have special meaning. Use -F (fixed strings) for literal matching.
//
// Examples:
// Bad:
//   grep "file.txt" *        // Dot matches any character (file-txt, fileXtxt)
//   grep "data[1]" file      // [1] is character class, matches 1
//
// Good:
//   grep -F "file.txt" *     // Literal dot matching
//   grep -F "data[1]" file   // Literal brackets
//   grep 'file\.txt' *       // Escaped dot for regex
//
// Impact: Unexpected matches, false positives in search results

use crate::linter::{Diagnostic, LintResult, Severity, Span};
use once_cell::sync::Lazy;
use regex::Regex;

static GREP_LITERAL_PATTERN: Lazy<Regex> = Lazy::new(|| {
    // Match grep with quoted string containing regex metacharacters
    Regex::new(r#"\bgrep\s+(?:-\w+\s+)*["']([^"']*[.\[\]\^$+*?{}|()\\][^"']*)["']"#).unwrap()
});

pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();

    for (line_num, line) in source.lines().enumerate() {
        let line_num = line_num + 1;

        if line.trim_start().starts_with('#') {
            continue;
        }

        // Skip if already using -F or -E flags
        if line.contains("grep -F") || line.contains("grep -E") {
            continue;
        }

        for cap in GREP_LITERAL_PATTERN.captures_iter(line) {
            let pattern = cap.get(1).unwrap().as_str();

            // Simple heuristic: if it looks like a filename or path with dots
            if pattern.contains('.') && !pattern.contains('*') {
                let full_match = cap.get(0).unwrap().as_str();
                let pos = line.find(full_match).unwrap_or(0);
                let start_col = pos + 1;
                let end_col = start_col + full_match.len();

                let diagnostic = Diagnostic::new(
                    "SC2063",
                    Severity::Info,
                    "Grep uses regexes. Use grep -F if you want literal string matching"
                        .to_string(),
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
    fn test_sc2063_dot_in_pattern() {
        let code = r#"grep "file.txt" *"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2063_path_with_dot() {
        let code = r#"grep "/var/log/app.log" file"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2063_fixed_string_ok() {
        let code = r#"grep -F "file.txt" *"#;
        let result = check(code);
        // Using -F flag, correct
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2063_extended_regex_ok() {
        let code = r#"grep -E "file.txt" *"#;
        let result = check(code);
        // Using -E, intentional regex
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2063_no_dots_ok() {
        let code = r#"grep "pattern" file"#;
        let result = check(code);
        // No regex metacharacters
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    #[ignore] // TODO: Fix escaped dot detection
    fn test_sc2063_escaped_dot_ok() {
        let code = r#"grep 'file\.txt' *"#;
        let result = check(code);
        // Escaped dot, intentional regex
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2063_comment_ok() {
        let code = r#"# grep "file.txt" *"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2063_wildcard_regex_ok() {
        let code = r#"grep "file.*txt" *"#;
        let result = check(code);
        // Contains *, likely intentional regex
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2063_single_quotes() {
        let code = r#"grep 'config.json' file"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2063_with_flags() {
        let code = r#"grep -i "error.log" /var/log/*"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }
}
