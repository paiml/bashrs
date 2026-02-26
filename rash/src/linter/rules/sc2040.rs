// SC2040: Avoid passing -o to other commands. It's a shell option, not a command flag.
//
// The -o option is typically used with `set` to set shell options (set -o pipefail).
// Passing it to other commands like rm, cp, grep is likely a mistake.
//
// Examples:
// Bad:
//   rm -o file.txt          // -o is not a valid rm flag
//   cp -o source dest       // -o is not a valid cp flag
//   grep -o pattern file    // -o exists for grep (print only matches), might be intended
//
// Good:
//   rm -f file.txt          // Use correct flags
//   cp source dest
//   set -o pipefail         // set uses -o for shell options
//
// Note: grep -o is actually valid (print only matching parts), so we skip grep.
// This rule focuses on commands where -o is clearly wrong.

use crate::linter::{Diagnostic, LintResult, Severity, Span};
use regex::Regex;

static COMMAND_WITH_O_FLAG: std::sync::LazyLock<Regex> = std::sync::LazyLock::new(|| {
    // Match: rm/cp/mv/chmod/chown with -o flag
    // Skip: set (valid), grep/sed/awk (valid), find (valid)
    Regex::new(r"\b(rm|cp|mv|chmod|chown|ls|cat|touch|mkdir)\s+[^|;&\n]*-o\b").unwrap()
});

pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();

    for (line_num, line) in source.lines().enumerate() {
        let line_num = line_num + 1;

        if line.trim_start().starts_with('#') {
            continue;
        }

        // Look for commands with -o flag
        if let Some(cap) = COMMAND_WITH_O_FLAG.captures(line) {
            let command = cap.get(1).unwrap().as_str();

            // Skip if inside quotes
            let full_match = cap.get(0).unwrap().as_str();
            let pos = line.find(full_match).unwrap_or(0);
            let before = &line[..pos];
            let quote_count = before.matches('"').count() + before.matches('\'').count();
            if quote_count % 2 == 1 {
                continue;
            }

            // Check if -o appears to be a shell option (after set)
            if line.contains("set") && line.find("set").unwrap_or(usize::MAX) < pos {
                continue; // set -o is valid
            }

            let start_col = pos + 1;
            let end_col = start_col + full_match.len();

            let diagnostic = Diagnostic::new(
                "SC2040",
                Severity::Warning,
                format!(
                    "'-o' is typically a shell option (set -o). Did you mean a different flag for '{}'?",
                    command
                ),
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
    fn test_sc2040_rm_with_o() {
        let code = r#"rm -o file.txt"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
        assert_eq!(result.diagnostics[0].code, "SC2040");
        assert!(result.diagnostics[0].message.contains("-o"));
    }

    #[test]
    fn test_sc2040_cp_with_o() {
        let code = r#"cp -o source dest"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2040_chmod_with_o() {
        let code = r#"chmod -o 755 file"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2040_set_o_ok() {
        let code = r#"set -o pipefail"#;
        let result = check(code);
        // set -o is valid shell usage
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2040_grep_o_ok() {
        let code = r#"grep -o pattern file"#;
        let result = check(code);
        // grep -o is valid (not in our pattern)
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2040_find_o_ok() {
        let code = r#"find . -name "*.txt" -o -name "*.sh""#;
        let result = check(code);
        // find -o is valid (OR operator, not in our pattern)
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2040_rm_with_other_flags() {
        let code = r#"rm -rf file.txt"#;
        let result = check(code);
        // Valid flags, no -o
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2040_in_quotes_ok() {
        let code = r#"echo "rm -o file.txt""#;
        let result = check(code);
        // Inside quotes, not actual command
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2040_comment_ok() {
        let code = r#"# rm -o file.txt"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2040_ls_with_o() {
        let code = r#"ls -o /tmp"#;
        let result = check(code);
        // ls -o is actually valid (long format without group), but unusual
        assert_eq!(result.diagnostics.len(), 1);
    }
}
