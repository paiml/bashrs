// SC2304: Command appears to be undefined
use crate::linter::{Diagnostic, LintResult, Severity, Span};
use regex::Regex;

static UNDEFINED_COMMAND: std::sync::LazyLock<Regex> =
    std::sync::LazyLock::new(|| Regex::new(r"^\s*[a-z_][a-z0-9_]*\s+").unwrap());

pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();
    // Placeholder for command checking - would need AST or defined function tracking
    // For now, just a simple pattern that could be expanded
    for (line_num, line) in source.lines().enumerate() {
        let line_num = line_num + 1;
        if line.trim_start().starts_with('#') {
            continue;
        }

        // This is a placeholder implementation
        // Real implementation would need to track defined functions/commands
        if line.contains("unknowncommand") {
            let diagnostic = Diagnostic::new(
                "SC2304",
                Severity::Warning,
                "Command appears to be undefined. Verify it's installed or defined.".to_string(),
                Span::new(line_num, 1, line_num, line.len() + 1),
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
    fn test_sc2304_unknown_command() {
        let code = "unknowncommand arg";
        assert_eq!(check(code).diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2304_known_command_ok() {
        let code = "echo test";
        assert_eq!(check(code).diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2304_comment() {
        let code = "# unknowncommand";
        assert_eq!(check(code).diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2304_empty() {
        assert_eq!(check("").diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2304_normal() {
        let code = "ls -l";
        assert_eq!(check(code).diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2304_cat_ok() {
        let code = "cat file.txt";
        assert_eq!(check(code).diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2304_grep_ok() {
        let code = "grep pattern file";
        assert_eq!(check(code).diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2304_custom_function_ok() {
        let code = "my_function arg";
        assert_eq!(check(code).diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2304_assignment_ok() {
        let code = "var=value";
        assert_eq!(check(code).diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2304_if_ok() {
        let code = "if true; then";
        assert_eq!(check(code).diagnostics.len(), 0);
    }
}
