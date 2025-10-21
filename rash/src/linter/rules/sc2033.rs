// SC2033: Shell can't see variables exported in a subshell
//
// When you export a variable inside a subshell (created by parentheses, command
// substitution, or pipe), the export only affects that subshell. The parent shell
// won't see the exported variable.
//
// Examples:
// Bad:
//   (export VAR=value)          // Export in subshell - parent can't see it
//   cat file | export VAR=value // Export in pipeline subshell
//   $(export PATH=$PATH:/new)   // Export in command substitution
//
// Good:
//   export VAR=value            // Export in current shell
//   VAR=value; export VAR       // Export in current shell
//
// Note: This is a common mistake that leads to confusion about why environment
// variables aren't being set correctly.

use crate::linter::{Diagnostic, LintResult, Severity, Span};
use once_cell::sync::Lazy;
use regex::Regex;

static EXPORT_IN_SUBSHELL: Lazy<Regex> = Lazy::new(|| {
    // Match: (export ...) - any export statement in subshell
    Regex::new(r"\(\s*export\b").unwrap()
});

static EXPORT_IN_PIPE: Lazy<Regex> = Lazy::new(|| {
    // Match: | export ... - any export statement in pipeline
    Regex::new(r"\|\s*export\b").unwrap()
});

static EXPORT_IN_COMMAND_SUBST: Lazy<Regex> = Lazy::new(|| {
    // Match: $(export ...) - any export statement in command substitution
    Regex::new(r"\$\(\s*export\b").unwrap()
});

pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();

    for (line_num, line) in source.lines().enumerate() {
        let line_num = line_num + 1;

        if line.trim_start().starts_with('#') {
            continue;
        }

        // Check for export in (subshell) - but not $(command substitution)
        if let Some(m) = EXPORT_IN_SUBSHELL.find(line) {
            // Skip if this is actually $(export ...) (command substitution)
            let match_start = m.start();
            let is_command_subst =
                match_start > 0 && line.chars().nth(match_start - 1) == Some('$');

            if !is_command_subst {
                let start_col = match_start + 1;
                let end_col = m.end() + 1;

                let diagnostic = Diagnostic::new(
                    "SC2033",
                    Severity::Warning,
                    "Shell can't see variables exported in a subshell. Remove parentheses or export in the current shell".to_string(),
                    Span::new(line_num, start_col, line_num, end_col),
                );

                result.add(diagnostic);
            }
        }

        // Check for export in pipeline
        if let Some(m) = EXPORT_IN_PIPE.find(line) {
            let start_col = m.start() + 1;
            let end_col = m.end() + 1;

            let diagnostic = Diagnostic::new(
                "SC2033",
                Severity::Warning,
                "Shell can't see variables exported in a pipeline. The export only affects the pipeline subshell".to_string(),
                Span::new(line_num, start_col, line_num, end_col),
            );

            result.add(diagnostic);
        }

        // Check for export in command substitution
        if let Some(m) = EXPORT_IN_COMMAND_SUBST.find(line) {
            let start_col = m.start() + 1;
            let end_col = m.end() + 1;

            let diagnostic = Diagnostic::new(
                "SC2033",
                Severity::Warning,
                "Shell can't see variables exported in command substitution. Export in the current shell instead".to_string(),
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
    fn test_sc2033_export_in_subshell() {
        let code = r#"(export VAR=value)"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
        assert_eq!(result.diagnostics[0].code, "SC2033");
        assert_eq!(result.diagnostics[0].severity, Severity::Warning);
        assert!(result.diagnostics[0].message.contains("subshell"));
    }

    #[test]
    fn test_sc2033_export_in_subshell_with_spaces() {
        let code = r#"( export PATH=$PATH:/new )"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2033_export_in_pipeline() {
        let code = r#"cat file | export VAR=value"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
        assert!(result.diagnostics[0].message.contains("pipeline"));
    }

    #[test]
    fn test_sc2033_export_in_command_subst() {
        let code = r#"result=$(export PATH=$PATH:/new)"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
        assert!(result.diagnostics[0]
            .message
            .contains("command substitution"));
    }

    #[test]
    fn test_sc2033_export_normal_ok() {
        let code = r#"export VAR=value"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2033_export_after_assignment_ok() {
        let code = r#"VAR=value; export VAR"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2033_subshell_without_export_ok() {
        let code = r#"(VAR=value; echo $VAR)"#;
        let result = check(code);
        // No export, just assignment in subshell - different issue
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2033_multiple_issues() {
        let code = r#"
(export VAR1=a)
cat file | export VAR2=b
result=$(export VAR3=c)
"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 3);
    }

    #[test]
    fn test_sc2033_export_function_call() {
        let code = r#"(export -f my_function)"#;
        let result = check(code);
        // -f exports functions, but still wrong in subshell
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2033_lowercase_var() {
        let code = r#"(export path=/usr/bin)"#;
        let result = check(code);
        // Lowercase variables are less common but still affected by the same issue
        assert_eq!(result.diagnostics.len(), 1); // Now correctly catches all exports
    }
}
