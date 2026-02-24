// SC1105: Shells don't support $ followed by space before (
//
// A space between $ and ( means the $ is literal, and ( starts a subshell.
// If you meant command substitution, use $(cmd) without a space.
// If you meant a subshell, drop the $.
//
// Examples:
// Bad:
//   x=$ (cmd)          # $ is literal, ( starts subshell
//   echo $ (date)      # Space breaks command substitution
//
// Good:
//   x=$(cmd)           # Correct command substitution
//   (cmd)              # Correct subshell
//   echo $(date)       # Correct command substitution

use crate::linter::{Diagnostic, LintResult, Severity, Span};
use once_cell::sync::Lazy;
use regex::Regex;

/// Matches $ followed by one or more spaces then ( — likely a broken
/// command substitution attempt.
static DOLLAR_SPACE_PAREN: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"\$\s+\(").expect("SC1105 regex must compile")
});

pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();

    for (line_num, line) in source.lines().enumerate() {
        let line_num = line_num + 1;
        let trimmed = line.trim_start();
        if trimmed.starts_with('#') {
            continue;
        }

        for mat in DOLLAR_SPACE_PAREN.find_iter(line) {
            let start_col = mat.start() + 1;
            let end_col = mat.end() + 1;

            result.add(Diagnostic::new(
                "SC1105",
                Severity::Error,
                "Remove the space between $ and ( for command substitution, or drop the $ for a subshell.".to_string(),
                Span::new(line_num, start_col, line_num, end_col),
            ));
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sc1105_dollar_space_paren() {
        let code = "x=$ (cmd)";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
        assert_eq!(result.diagnostics[0].code, "SC1105");
        assert_eq!(result.diagnostics[0].severity, Severity::Error);
    }

    #[test]
    fn test_sc1105_echo_dollar_space() {
        let code = "echo $ (date)";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc1105_multiple_spaces() {
        let code = "x=$  (cmd)";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc1105_correct_cmd_subst_ok() {
        let code = "x=$(cmd)";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc1105_subshell_ok() {
        let code = "(cmd)";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc1105_comment_ok() {
        let code = "# x=$ (cmd)";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc1105_normal_dollar_ok() {
        let code = "echo $var (something)";
        let result = check(code);
        // $var is separate from (something), no space between $ and (
        assert_eq!(result.diagnostics.len(), 0);
    }
}
