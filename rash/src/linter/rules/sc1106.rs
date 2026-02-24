// SC1106: Use -lt/-gt instead of </>  for numeric comparison in [ ]
//
// In single brackets [ ], < and > are interpreted as shell redirections,
// not comparison operators. Use -lt, -gt, -le, -ge for numeric comparisons.
//
// Examples:
// Bad:
//   [ $x < $y ]       # Redirects stdin from file $y
//   [ $a > $b ]       # Redirects stdout to file $b
//   [ "$x" < "$y" ]   # Still a redirection
//
// Good:
//   [ $x -lt $y ]     # Less than
//   [ $a -gt $b ]     # Greater than
//   [ $x -le $y ]     # Less than or equal
//   [ $a -ge $b ]     # Greater than or equal
//   [[ $x < $y ]]     # OK in [[ ]] (lexicographic comparison)

use crate::linter::{Diagnostic, LintResult, Severity, Span};
use once_cell::sync::Lazy;
use regex::Regex;

/// Matches [ ... < ... ] or [ ... > ... ] (single brackets with < or >)
/// This pattern looks for single [ followed by content with < or > then ]
/// but NOT [[ which is a different construct.
static SINGLE_BRACKET_COMPARE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?:^|[;\s&|])\[\s+.*\s+([<>])\s+.*\s+\](?:\s|;|$|\||\&)")
        .expect("SC1106 regex must compile")
});

pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();

    for (line_num, line) in source.lines().enumerate() {
        let line_num = line_num + 1;
        let trimmed = line.trim_start();
        if trimmed.starts_with('#') {
            continue;
        }

        // Skip lines with [[ ]] — double brackets handle < > correctly
        if trimmed.contains("[[") {
            continue;
        }

        // Look for single bracket test with < or >
        // We do a simpler approach: find [ ... ] then check for < > inside
        if let Some(bracket_content) = extract_single_bracket_content(line) {
            for (offset, ch) in bracket_content.content.char_indices() {
                if ch == '<' || ch == '>' {
                    // Check it's surrounded by whitespace (an operator, not part of a word)
                    let before_ok = offset == 0
                        || bracket_content.content.as_bytes()[offset - 1] == b' ';
                    let after_ok = offset + 1 >= bracket_content.content.len()
                        || bracket_content.content.as_bytes()[offset + 1] == b' ';

                    if before_ok && after_ok {
                        let replacement = if ch == '<' { "-lt" } else { "-gt" };
                        let col = bracket_content.start_col + offset;

                        result.add(Diagnostic::new(
                            "SC1106",
                            Severity::Warning,
                            format!(
                                "In [ ], use {} instead of '{}' for numeric comparison. The '{}' is a shell redirection in [ ].",
                                replacement, ch, ch
                            ),
                            Span::new(line_num, col + 1, line_num, col + 2),
                        ));
                    }
                }
            }
        }
    }

    result
}

struct BracketContent {
    content: String,
    start_col: usize,
}

fn extract_single_bracket_content(line: &str) -> Option<BracketContent> {
    // Find [ that is not [[
    let bytes = line.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'[' {
            // Check it's not [[
            if i + 1 < bytes.len() && bytes[i + 1] == b'[' {
                i += 2;
                continue;
            }
            // Found single [, now find matching ]
            let start = i + 1;
            let mut j = start;
            while j < bytes.len() {
                if bytes[j] == b']' {
                    // Make sure it's not ]]
                    if j + 1 < bytes.len() && bytes[j + 1] == b']' {
                        j += 2;
                        continue;
                    }
                    let content = line[start..j].to_string();
                    return Some(BracketContent {
                        content,
                        start_col: start,
                    });
                }
                j += 1;
            }
        }
        i += 1;
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sc1106_less_than_in_bracket() {
        let code = "[ $x < $y ]";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
        assert_eq!(result.diagnostics[0].code, "SC1106");
        assert_eq!(result.diagnostics[0].severity, Severity::Warning);
        assert!(result.diagnostics[0].message.contains("-lt"));
    }

    #[test]
    fn test_sc1106_greater_than_in_bracket() {
        let code = "[ $a > $b ]";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
        assert!(result.diagnostics[0].message.contains("-gt"));
    }

    #[test]
    fn test_sc1106_quoted_vars() {
        let code = r#"[ "$x" < "$y" ]"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc1106_double_bracket_ok() {
        let code = "[[ $x < $y ]]";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc1106_correct_operators_ok() {
        let code = "[ $x -lt $y ]";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc1106_gt_operator_ok() {
        let code = "[ $x -gt $y ]";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc1106_comment_ok() {
        let code = "# [ $x < $y ]";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }
}
