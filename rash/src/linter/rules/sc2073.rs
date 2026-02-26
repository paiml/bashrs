// SC2073: Escape backslashes in character classes
//
// In character classes like [a-z\d], the backslash needs to be escaped
// to match a literal backslash. Otherwise it's interpreted as an escape sequence.
//
// Examples:
// Bad:
//   [[ $var =~ [a-z\d] ]]      // \d treated as escape, not literal
//   case $x in [\w]*) ;;       // \w not valid in case patterns
//
// Good:
//   [[ $var =~ [a-z\\d] ]]     // Literal backslash + d
//   [[ $var =~ [a-z0-9] ]]     // Use explicit range instead
//
// Impact: Pattern doesn't match as intended, unexpected behavior

use crate::linter::{Diagnostic, LintResult, Severity, Span};
use regex::Regex;

static UNESCAPED_BACKSLASH_IN_CLASS: std::sync::LazyLock<Regex> = std::sync::LazyLock::new(|| {
    // Match character classes with unescaped backslashes followed by letters
    // [something\d] or [something\w] etc
    Regex::new(r"\[([^\]]*\\[a-zA-Z][^\]]*)\]").unwrap()
});

pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();

    for (line_num, line) in source.lines().enumerate() {
        let line_num = line_num + 1;

        if line.trim_start().starts_with('#') {
            continue;
        }

        for cap in UNESCAPED_BACKSLASH_IN_CLASS.captures_iter(line) {
            let class_content = cap.get(1).unwrap().as_str();

            // Check if it contains \d, \w, \s (not \\d, \\w, \\s - those are escaped)
            let has_single_backslash = (class_content.contains("\\d")
                || class_content.contains("\\w")
                || class_content.contains("\\s"))
                && !class_content.contains("\\\\d")
                && !class_content.contains("\\\\w")
                && !class_content.contains("\\\\s");

            if has_single_backslash {
                let full_match = cap.get(0).unwrap();
                let start_col = full_match.start() + 1;
                let end_col = full_match.end() + 1;

                let diagnostic = Diagnostic::new(
                    "SC2073",
                    Severity::Warning,
                    "Escape backslashes in character classes or use POSIX classes like [:digit:]"
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
    fn test_sc2073_digit_class() {
        let code = r#"[[ $var =~ [a-z\d] ]]"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2073_word_class() {
        let code = r#"[[ $var =~ [\w+] ]]"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2073_space_class() {
        let code = r#"[[ $var =~ [a-z\s] ]]"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2073_escaped_ok() {
        let code = r#"[[ $var =~ [a-z\\d] ]]"#;
        let result = check(code);
        // Double backslash is OK (literal backslash)
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2073_posix_ok() {
        let code = r#"[[ $var =~ [[:digit:]] ]]"#;
        let result = check(code);
        // POSIX character class is correct
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2073_plain_range_ok() {
        let code = r#"[[ $var =~ [a-z0-9] ]]"#;
        let result = check(code);
        // Plain range without backslashes
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2073_case_statement() {
        let code = r#"case $x in [\w]*) echo "word";; esac"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2073_comment_ok() {
        let code = r#"# [[ $var =~ [\d+] ]]"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2073_multiple() {
        let code = r#"[[ $a =~ [\d] ]] && [[ $b =~ [\w] ]]"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 2);
    }

    #[test]
    fn test_sc2073_in_string() {
        let code = r#"pattern="[\d+]""#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }
}
