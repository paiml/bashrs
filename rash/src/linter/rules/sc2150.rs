// SC2150: -exec is non-portable. Consider using -execdir or find -print0 | xargs -0.
//
// The -exec action in find works but has performance issues with many files
// and can be non-portable. Better alternatives exist.
//
// Examples:
// Bad:
//   find . -name "*.txt" -exec rm {} \;     // Spawns rm for each file
//   find . -type f -exec chmod 644 {} \;    // Inefficient
//
// Good:
//   find . -name "*.txt" -delete            // Built-in delete
//   find . -name "*.txt" -exec rm {} +      // Batch mode
//   find . -name "*.txt" -print0 | xargs -0 rm   // xargs for efficiency
//   find . -type f -execdir chmod 644 {} +  // Execute in directory
//
// Impact: Performance - -exec \; spawns process per file

use crate::linter::{Diagnostic, LintResult, Severity, Span};
use regex::Regex;

static FIND_EXEC_SEMICOLON: std::sync::LazyLock<Regex> = std::sync::LazyLock::new(|| {
    // Match: find ... -exec command {} \;
    // The \; indicates one process per file (inefficient)
    // Match literal backslash-semicolon: \\;
    Regex::new(r"\bfind\b.*-exec\s+.*\{\}\s*\\;").unwrap()
});

pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();

    for (line_num, line) in source.lines().enumerate() {
        let line_num = line_num + 1;

        if line.trim_start().starts_with('#') {
            continue;
        }

        // Check for find -exec with \; (one process per file)
        for mat in FIND_EXEC_SEMICOLON.find_iter(line) {
            let start_col = mat.start() + 1;
            let end_col = mat.end() + 1;

            let diagnostic = Diagnostic::new(
                "SC2150",
                Severity::Info,
                r"-exec with \; is inefficient. Use + for batch mode or -print0 | xargs -0"
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
    fn test_sc2150_find_exec_semicolon() {
        let code = r#"find . -name "*.txt" -exec rm {} \;"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
        assert!(result.diagnostics[0].message.contains("inefficient"));
    }

    #[test]
    fn test_sc2150_find_exec_chmod() {
        let code = r#"find . -type f -exec chmod 644 {} \;"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2150_find_exec_plus_ok() {
        let code = r#"find . -name "*.txt" -exec rm {} +"#;
        let result = check(code);
        // Using + for batch mode is OK
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2150_find_delete_ok() {
        let code = r#"find . -name "*.txt" -delete"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2150_find_xargs_ok() {
        let code = r#"find . -name "*.txt" -print0 | xargs -0 rm"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2150_find_execdir_ok() {
        let code = r#"find . -type f -execdir chmod 644 {} +"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2150_comment_ok() {
        let code = r#"# find . -name "*.txt" -exec rm {} \;"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2150_multiple() {
        let code = r#"
find . -name "*.log" -exec rm {} \;
find /tmp -type f -exec chmod 644 {} \;
"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 2);
    }

    #[test]
    fn test_sc2150_exec_with_command() {
        let code = r#"find . -name "*.txt" -exec grep "pattern" {} \;"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2150_without_find_ok() {
        let code = r#"exec rm file.txt"#;
        let result = check(code);
        // Not a find command
        assert_eq!(result.diagnostics.len(), 0);
    }
}
