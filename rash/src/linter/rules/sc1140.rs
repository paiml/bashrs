// SC1140: Unexpected extra token after ]
//
// When using [ ] (test command), extra tokens after the closing ]
// are unexpected and usually indicate a syntax error.
//
// Examples:
// Bad:
//   [ -f file ] extra         # 'extra' is unexpected
//   [ $x -eq 1 ] foo          # 'foo' after ] is wrong
//   [ -n "$var" ] bar baz     # Extra tokens
//
// Good:
//   [ -f file ] && echo yes   # && is valid after ]
//   [ $x -eq 1 ] || exit 1   # || is valid
//   [ -n "$var" ]; then       # ; then is valid
//   [ -f file ] | cat         # pipe is valid

use crate::linter::{Diagnostic, LintResult, Severity, Span};
use regex::Regex;

/// Matches ] followed by whitespace and then an unexpected token.
/// Valid tokens after ] include: ;, &&, ||, |, ), then, do, else, elif, fi,
/// done, esac, end-of-line, and comments.
static BRACKET_EXTRA: std::sync::LazyLock<Regex> =
    std::sync::LazyLock::new(|| Regex::new(r"\]\s+(\S+)").expect("SC1140 regex must compile"));

/// Tokens that are valid after ]
const VALID_AFTER_BRACKET: &[&str] = &[
    "&&", "||", "|", ";", ")", "then", "do", "else", "elif", "fi", "done", "esac", "{", "}", ">>",
    ">", "<", "2>", "&>", "2>&1", "#", "\\",
];

pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();

    for (line_num, line) in source.lines().enumerate() {
        let line_num = line_num + 1;
        let trimmed = line.trim_start();
        if trimmed.starts_with('#') {
            continue;
        }

        // Skip [[ ]] — only check single brackets
        if trimmed.contains("[[") || trimmed.contains("]]") {
            continue;
        }

        // Find single bracket test commands
        if let Some(bracket_end) = find_single_bracket_close(line) {
            let after = &line[bracket_end + 1..];
            let after_trimmed = after.trim_start();

            if after_trimmed.is_empty() {
                continue;
            }

            // Get the first token after ]
            let first_token: &str = after_trimmed.split_whitespace().next().unwrap_or("");

            if first_token.is_empty() {
                continue;
            }

            // Check if the token starts with a valid sequence
            let is_valid = VALID_AFTER_BRACKET
                .iter()
                .any(|&valid| first_token == valid || first_token.starts_with(valid))
                || first_token.starts_with(';')
                || first_token.starts_with('#')
                || first_token.starts_with('|')
                || first_token.starts_with('&')
                || first_token.starts_with('>')
                || first_token.starts_with('<');

            if !is_valid {
                let col = bracket_end + 1 + (after.len() - after_trimmed.len());
                let end_col = col + first_token.len();

                result.add(Diagnostic::new(
                    "SC1140",
                    Severity::Error,
                    format!(
                        "Unexpected token '{}' after ]. Did you forget && or || ?",
                        first_token
                    ),
                    Span::new(line_num, col + 1, line_num, end_col + 1),
                ));
            }
        }
    }

    result
}

/// Find the position of the closing ] of a single bracket test command.
/// Returns None if no valid single bracket test found.
fn find_single_bracket_close(line: &str) -> Option<usize> {
    let bytes = line.as_bytes();
    let mut i = 0;

    while i < bytes.len() {
        if bytes[i] == b'[' {
            // Skip [[ — double brackets
            if i + 1 < bytes.len() && bytes[i + 1] == b'[' {
                i += 2;
                // Skip past ]]
                while i < bytes.len() {
                    if i + 1 < bytes.len() && bytes[i] == b']' && bytes[i + 1] == b']' {
                        i += 2;
                        break;
                    }
                    i += 1;
                }
                continue;
            }

            // Found single [, find matching ]
            let mut j = i + 1;
            while j < bytes.len() {
                if bytes[j] == b']' {
                    // Make sure it's not ]]
                    if j + 1 < bytes.len() && bytes[j + 1] == b']' {
                        j += 2;
                        continue;
                    }
                    return Some(j);
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
    fn test_sc1140_extra_token() {
        let code = "[ -f file ] extra";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
        assert_eq!(result.diagnostics[0].code, "SC1140");
        assert_eq!(result.diagnostics[0].severity, Severity::Error);
        assert!(result.diagnostics[0].message.contains("extra"));
    }

    #[test]
    fn test_sc1140_extra_word_after_test() {
        let code = "[ $x -eq 1 ] foo";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
        assert!(result.diagnostics[0].message.contains("foo"));
    }

    #[test]
    fn test_sc1140_and_ok() {
        let code = "[ -f file ] && echo yes";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc1140_or_ok() {
        let code = "[ -f file ] || exit 1";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc1140_semicolon_then_ok() {
        let code = "[ -f file ]; then";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc1140_pipe_ok() {
        let code = "[ -f file ] | cat";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc1140_end_of_line_ok() {
        let code = "[ -f file ]";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc1140_comment_ok() {
        let code = "# [ -f file ] extra";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc1140_then_ok() {
        let code = "if [ -f file ] then";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }
}
