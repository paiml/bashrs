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

use crate::linter::rules::posix_bracket;
use crate::linter::{Diagnostic, LintResult, Severity, Span};

/// Tokens that are valid after ]
const VALID_AFTER_BRACKET: &[&str] = &[
    "&&", "||", "|", ";", ")", "then", "do", "else", "elif", "fi", "done", "esac", "{", "}", ">>",
    ">", "<", "2>", "&>", "2>&1", "#", "\\",
];

pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();

    for (line_num, line) in source.lines().enumerate() {
        let line_num = line_num + 1;
        if line.trim_start().starts_with('#') {
            continue;
        }
        if let Some(diag) = line_diagnostic(line, line_num) {
            result.add(diag);
        }
    }

    result
}

/// GH-226: this used to accept any `[` as a test opener and any `]` as its
/// close, so an associative-array assignment (`M[k]=v`) or a case pattern
/// (`[0-7][0-7])`) produced `Severity::Error` findings. `posix_bracket` applies
/// the POSIX word rules, so only a real `[ … ]` command is considered.
fn line_diagnostic(line: &str, line_num: usize) -> Option<Diagnostic> {
    let open = *posix_bracket::openers(line).first()?;
    let close = posix_bracket::close_of(line, open)?;

    let after = line.get(close + 1..)?;
    let after_trimmed = after.trim_start();
    let first_token = after_trimmed.split_whitespace().next()?;

    if is_valid_after_bracket(first_token) {
        return None;
    }

    let col = close + 1 + (after.len() - after_trimmed.len());
    let end_col = col + first_token.len();
    Some(Diagnostic::new(
        "SC1140",
        Severity::Error,
        format!("Unexpected token '{first_token}' after ]. Did you forget && or || ?"),
        Span::new(line_num, col + 1, line_num, end_col + 1),
    ))
}

fn is_valid_after_bracket(token: &str) -> bool {
    VALID_AFTER_BRACKET
        .iter()
        .any(|&valid| token == valid || token.starts_with(valid))
        || token.starts_with([';', '#', '|', '&', '>', '<'])
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
