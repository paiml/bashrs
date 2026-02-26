// SC2020: tr replaces sets of chars, not words (e.g. 'yes' to 'no')
//
// tr replaces individual characters, not strings. Each character in the first
// set is mapped to the corresponding character in the second set.
//
// Examples:
// Bad:
//   tr 'yes' 'no'                   // Replaces y→n, e→o, s→o (not "yes"→"no")
//   tr 'foo' 'bar'                  // Replaces f→b, o→a, o→r
//   tr 'true' 'false'               // Doesn't work as expected
//
// Good:
//   sed 's/yes/no/g'                // Use sed for word replacement
//   awk '{gsub(/yes/,"no")}1'       // Use awk for word replacement
//   tr 'aeiou' 'AEIOU'              // OK - char-to-char mapping
//
// Explanation:
//   tr 'yes' 'no' means: y→n, e→o, s→o
//   So "yes" becomes "noo", not "no"!
//
// Note: Use sed or awk for word/string replacement. tr is for character sets.

use crate::linter::{Diagnostic, LintResult, Severity, Span};
use regex::Regex;

static TR_WORD_PATTERN: std::sync::LazyLock<Regex> = std::sync::LazyLock::new(|| {
    // Match: tr 'word' 'word' where both args are multi-char words
    // Look for tr with quoted strings that look like words (alphanumeric)
    Regex::new(r#"tr\s+['"]([a-z]{2,})['"]"#).unwrap()
});

pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();

    for (line_num, line) in source.lines().enumerate() {
        let line_num = line_num + 1;

        if line.trim_start().starts_with('#') {
            continue;
        }

        // Look for tr with word-like patterns
        for cap in TR_WORD_PATTERN.captures_iter(line) {
            let word = cap.get(1).unwrap().as_str();

            // Skip if it looks like a character range or class
            if word.contains('-') || word.contains(':') {
                continue;
            }

            // Skip single character or very short patterns
            if word.len() < 2 {
                continue;
            }

            // Warn if the word looks like a common word being replaced
            // Common patterns: yes/no, true/false, on/off, foo/bar, etc.
            let common_words = [
                "yes", "no", "true", "false", "on", "off", "foo", "bar", "old", "new",
            ];
            if common_words.contains(&word) {
                let start_col = line
                    .find(&format!("'{}'", word))
                    .or_else(|| line.find(&format!("\"{}\"", word)))
                    .map_or(1, |p| p + 1);
                let end_col = start_col + word.len() + 2; // +2 for quotes

                let diagnostic = Diagnostic::new(
                    "SC2020",
                    Severity::Info,
                    format!(
                        "tr replaces sets of chars, not words. To replace '{}', use sed or awk instead",
                        word
                    ),
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
    fn test_sc2020_yes_no() {
        let code = r#"tr 'yes' 'no'"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
        assert_eq!(result.diagnostics[0].code, "SC2020");
        assert_eq!(result.diagnostics[0].severity, Severity::Info);
        assert!(result.diagnostics[0].message.contains("sets of chars"));
    }

    #[test]
    fn test_sc2020_true_false() {
        let code = r#"tr 'true' 'false'"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2020_foo_bar() {
        let code = r#"tr 'foo' 'bar'"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2020_on_off() {
        let code = r#"tr 'on' 'off'"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2020_char_set_ok() {
        let code = r#"tr 'aeiou' 'AEIOU'"#;
        let result = check(code);
        // Not a common word pattern
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2020_range_ok() {
        let code = r#"tr 'a-z' 'A-Z'"#;
        let result = check(code);
        // Contains dash, looks like range
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2020_posix_class_ok() {
        let code = r#"tr '[:lower:]' '[:upper:]'"#;
        let result = check(code);
        // Contains colon, looks like POSIX class
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2020_single_char_ok() {
        let code = r#"tr 'a' 'b'"#;
        let result = check(code);
        // Single char is fine
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2020_multiple_issues() {
        let code = r#"
tr 'yes' 'no'
tr 'true' 'false'
"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 2);
    }

    #[test]
    fn test_sc2020_uncommon_word_ok() {
        let code = r#"tr 'xyz' 'abc'"#;
        let result = check(code);
        // Not in common words list
        assert_eq!(result.diagnostics.len(), 0);
    }
}
