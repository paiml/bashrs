// SC2139: This expands when defined, not when called. Consider escaping.
//
// Variables in alias/function definitions expand at definition time unless escaped.
// This is usually not what you want.
//
// Examples:
// Bad:
//   alias ll="ls -la $PWD"           // $PWD expands now, not when ll is used
//   function greet() { echo "Hello $USER"; }  // $USER expands at definition
//
// Good:
//   alias ll='ls -la $PWD'           // Single quotes prevent expansion
//   function greet() { echo "Hello \$USER"; } // Escaped
//   greet() { local user="$USER"; echo "Hello $user"; } // Store in local
//
// Impact: Unexpected behavior - variable value frozen at definition time

use crate::linter::{Diagnostic, LintResult, Severity, Span};
use once_cell::sync::Lazy;
use regex::Regex;

static ALIAS_WITH_VAR: Lazy<Regex> = Lazy::new(|| {
    // Match: alias name="...$VAR..."
    Regex::new(r#"\balias\s+\w+="[^"]*\$[A-Z_][A-Z0-9_]*[^"]*""#).unwrap()
});

pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();

    for (line_num, line) in source.lines().enumerate() {
        let line_num = line_num + 1;

        if line.trim_start().starts_with('#') {
            continue;
        }

        // Check for alias with variables in double quotes
        for mat in ALIAS_WITH_VAR.find_iter(line) {
            let start_col = mat.start() + 1;
            let end_col = mat.end() + 1;

            let diagnostic = Diagnostic::new(
                "SC2139",
                Severity::Warning,
                "This expands when defined, not when called. Use single quotes to prevent expansion".to_string(),
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
    fn test_sc2139_alias_with_var() {
        let code = r#"alias ll="ls -la $PWD""#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
        assert!(result.diagnostics[0]
            .message
            .contains("expands when defined"));
    }

    #[test]
    fn test_sc2139_single_quotes_ok() {
        let code = "alias ll='ls -la $PWD'";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2139_no_var_ok() {
        let code = r#"alias ll="ls -la""#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2139_comment_ok() {
        let code = r#"# alias ll="ls -la $PWD""#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2139_home_var() {
        let code = r#"alias gohs="cd $HOME""#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2139_user_var() {
        let code = r#"alias greet="echo Hello $USER""#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2139_path_var() {
        let code = r#"alias showpath="echo $PATH""#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2139_lowercase_var_ok() {
        let code = r#"alias test="echo $myvar""#;
        let result = check(code);
        // Lowercase variables are often intentional
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2139_multiple() {
        let code = r#"
alias ll="ls -la $PWD"
alias greet="echo $USER"
"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 2);
    }

    #[test]
    fn test_sc2139_mixed_case() {
        let code = r#"alias test="echo $HOME_DIR""#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }
}
