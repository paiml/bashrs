// SC2038: Use -print0/-0 or find -exec + instead of for loop over find output.
//
// Using `for` to iterate over `find` output is unsafe because:
// 1. Filenames with spaces/newlines will break word splitting
// 2. Glob characters (*?[) in filenames will expand unexpectedly
//
// Examples:
// Bad:
//   for file in $(find . -name "*.txt"); do
//     echo "$file"  # Breaks on spaces, globs expand
//   done
//
// Good:
//   find . -name "*.txt" -print0 | while IFS= read -r -d '' file; do
//     echo "$file"  # Safe with -print0/-d ''
//   done
//
//   # Or use -exec (most portable):
//   find . -name "*.txt" -exec echo {} +
//
// Note: -print0 requires GNU find or BSD find with -print0 support.

use crate::linter::{Diagnostic, LintResult, Severity, Span};
use once_cell::sync::Lazy;
use regex::Regex;

static FOR_FIND_LOOP: Lazy<Regex> = Lazy::new(|| {
    // Match: for var in $(find ...) or for var in `find ...`
    Regex::new(r"for\s+\w+\s+in\s+(?:\$\(|`)find\s+").unwrap()
});

pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();

    for (line_num, line) in source.lines().enumerate() {
        let line_num = line_num + 1;

        if line.trim_start().starts_with('#') {
            continue;
        }

        // Look for for loops over find output
        if FOR_FIND_LOOP.is_match(line) {
            // Skip if inside quotes
            if let Some(mat) = FOR_FIND_LOOP.find(line) {
                let pos = mat.start();
                let before = &line[..pos];
                let quote_count = before.matches('"').count() + before.matches('\'').count();
                if quote_count % 2 == 1 {
                    continue;
                }

                let start_col = pos + 1;
                let end_col = mat.end() + 1;

                let diagnostic = Diagnostic::new(
                    "SC2038",
                    Severity::Warning,
                    "Use find -exec or -print0/-0 instead of for loop over find output. Filenames with spaces/newlines will break.".to_string(),
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
    fn test_sc2038_for_find_command_subst() {
        let code = r#"for file in $(find . -name "*.txt"); do echo "$file"; done"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
        assert_eq!(result.diagnostics[0].code, "SC2038");
        assert_eq!(result.diagnostics[0].severity, Severity::Warning);
        assert!(result.diagnostics[0].message.contains("-print0"));
    }

    #[test]
    fn test_sc2038_for_find_backticks() {
        let code = r#"for f in `find . -type f`; do rm "$f"; done"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2038_find_exec_ok() {
        let code = r#"find . -name "*.txt" -exec echo {} +"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2038_find_print0_ok() {
        let code = r#"find . -name "*.txt" -print0 | while IFS= read -r -d '' file; do echo "$file"; done"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2038_regular_for_ok() {
        let code = r#"for file in *.txt; do echo "$file"; done"#;
        let result = check(code);
        // Regular glob is OK
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2038_in_quotes_ok() {
        let code = r#"echo "for file in $(find . -name '*.txt')""#;
        let result = check(code);
        // Inside quotes, not actual loop
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2038_comment_ok() {
        let code = r#"# for file in $(find . -name "*.txt"); do"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2038_multiline_find() {
        let code = r#"
for item in $(find /var/log \
  -name "*.log"); do
  cat "$item"
done
"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2038_while_read_ok() {
        let code = r#"while IFS= read -r line; do echo "$line"; done < <(find . -type f)"#;
        let result = check(code);
        // while read with process substitution is OK
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2038_array_assignment() {
        let code = r#"files=($(find . -name "*.sh"))"#;
        let result = check(code);
        // Array assignment from find (also unsafe, but different pattern)
        assert_eq!(result.diagnostics.len(), 0);
    }
}
