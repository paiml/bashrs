// SC2156: Injecting filenames is fragile and insecure. Use * or ${array[@]}.
//
// Dynamically constructing filenames from command substitution is unsafe.
// Use glob patterns or arrays instead.
//
// Examples:
// Bad:
//   for file in $(ls); do           // Breaks on spaces, unsafe
//   rm $(find . -name "*.txt")      // Word splitting, unsafe
//
// Good:
//   for file in *; do               // Glob expansion, safe
//   find . -name "*.txt" -delete    // Direct action
//   files=(*.txt); rm "${files[@]}" // Array, safe
//
// Impact: Security and correctness issues with filenames

use crate::linter::{Diagnostic, LintResult, Severity, Span};
use regex::Regex;

static COMMAND_SUB_IN_FOR: std::sync::LazyLock<Regex> =
    std::sync::LazyLock::new(|| Regex::new(r"\bfor\s+\w+\s+in\s+\$\(").unwrap());

static UNQUOTED_COMMAND_SUB: std::sync::LazyLock<Regex> =
    std::sync::LazyLock::new(|| Regex::new(r"\b(rm|mv|cp)\s+\$\((find|ls)\b").unwrap());

pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();

    for (line_num, line) in source.lines().enumerate() {
        let line_num = line_num + 1;

        if line.trim_start().starts_with('#') {
            continue;
        }

        // Check for command substitution in for loops
        for mat in COMMAND_SUB_IN_FOR.find_iter(line) {
            let start_col = mat.start() + 1;
            let end_col = mat.end() + 1;

            let diagnostic = Diagnostic::new(
                "SC2156",
                Severity::Warning,
                "Injecting filenames is fragile. Use globs or arrays instead".to_string(),
                Span::new(line_num, start_col, line_num, end_col),
            );

            result.add(diagnostic);
        }

        // Check for unquoted command sub with file operations
        for mat in UNQUOTED_COMMAND_SUB.find_iter(line) {
            let start_col = mat.start() + 1;
            let end_col = mat.end() + 1;

            let diagnostic = Diagnostic::new(
                "SC2156",
                Severity::Warning,
                "Injecting filenames is fragile. Use -exec or xargs instead".to_string(),
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
    fn test_sc2156_for_ls() {
        let code = "for file in $(ls); do";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2156_for_find() {
        let code = "for file in $(find . -name '*.txt'); do";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2156_rm_find() {
        let code = "rm $(find . -name '*.txt')";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2156_for_glob_ok() {
        let code = "for file in *.txt; do";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2156_for_array_ok() {
        let code = r#"for file in "${files[@]}"; do"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2156_find_delete_ok() {
        let code = "find . -name '*.txt' -delete";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2156_comment_ok() {
        let code = "# for file in $(ls); do";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2156_mv_find() {
        let code = "mv $(find . -type f) /tmp/";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2156_cp_ls() {
        let code = "cp $(ls *.txt) backup/";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2156_multiple() {
        let code = "for f in $(ls)\nrm $(find .)";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 2);
    }
}
