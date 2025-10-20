// SC2123: PATH is the shell search path. Use another name.
//
// Setting PATH to a literal value (not appending/prepending) will break
// most commands because the shell won't be able to find them.
//
// Examples:
// Bad:
//   PATH="/my/dir"              # Breaks ls, cat, grep, etc.
//   PATH="./bin"                # Most commands won't work
//
// Good:
//   PATH="/my/dir:$PATH"        # Prepend to PATH
//   PATH="$PATH:/my/dir"        # Append to PATH
//   MY_PATH="/my/dir"           # Use different variable name

use crate::linter::{Diagnostic, LintResult, Severity, Span};
use once_cell::sync::Lazy;
use regex::Regex;

static PATH_ASSIGNMENT: Lazy<Regex> = Lazy::new(|| {
    // Match: PATH=value where value doesn't reference $PATH
    Regex::new(r"^\s*PATH=([^$\s]+)").unwrap()
});

pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();

    for (line_num, line) in source.lines().enumerate() {
        let line_num = line_num + 1;

        if line.trim_start().starts_with('#') {
            continue;
        }

        if let Some(cap) = PATH_ASSIGNMENT.captures(line) {
            let value = cap.get(1).unwrap().as_str();

            // Skip if it references $PATH
            if line.contains("$PATH") || line.contains("${PATH}") {
                continue;
            }

            // Skip export PATH (might be intentional in minimal environments)
            if line.trim_start().starts_with("export PATH") {
                continue;
            }

            let start_col = cap.get(0).unwrap().start() + 1;
            let end_col = cap.get(0).unwrap().end() + 1;

            let diagnostic = Diagnostic::new(
                "SC2123",
                Severity::Warning,
                "PATH is the shell search path. Use another name or append/prepend to $PATH instead",
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
    fn test_sc2123_path_set_to_literal() {
        let code = r#"PATH="/my/dir""#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
        assert_eq!(result.diagnostics[0].code, "SC2123");
        assert_eq!(result.diagnostics[0].severity, Severity::Warning);
        assert!(result.diagnostics[0].message.contains("PATH"));
    }

    #[test]
    fn test_sc2123_path_relative() {
        let code = r#"PATH="./bin""#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2123_path_prepend_ok() {
        let code = r#"PATH="/my/dir:$PATH""#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2123_path_append_ok() {
        let code = r#"PATH="$PATH:/my/dir""#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2123_path_with_braces_ok() {
        let code = r#"PATH="/usr/bin:${PATH}""#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2123_other_variable_ok() {
        let code = r#"MY_PATH="/my/dir""#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2123_path_in_middle_ok() {
        let code = r#"PATH="/bin:$PATH:/usr/bin""#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2123_export_path_ok() {
        let code = r#"export PATH="/usr/local/bin""#;
        let result = check(code);
        // Export might be intentional in minimal environments
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2123_multiple_paths() {
        let code = r#"PATH="/bin:/usr/bin""#;
        let result = check(code);
        // Still a literal value, should warn
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2123_empty_path() {
        let code = r#"PATH="""#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }
}
