// SC2183: Variable used as command name - potential code injection.
use crate::linter::{Diagnostic, LintResult, Severity, Span};
use once_cell::sync::Lazy;
use regex::Regex;

static VAR_AS_COMMAND: Lazy<Regex> = Lazy::new(|| Regex::new(r"^\s*\$\{?\w+\}?\s+").unwrap());

pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();
    for (line_num, line) in source.lines().enumerate() {
        let line_num = line_num + 1;
        if line.trim_start().starts_with('#') {
            continue;
        }

        if VAR_AS_COMMAND.is_match(line) {
            let diagnostic = Diagnostic::new(
                "SC2183",
                Severity::Warning,
                "Variable used as command name - potential injection risk".to_string(),
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
    fn test_sc2183_var_command() {
        let code = "$cmd arg";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }
    #[test]
    fn test_sc2183_normal_ok() {
        let code = "echo test";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }
}
