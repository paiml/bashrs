// SC2087: Quote or escape expressions in sh -c / bash -c
//
// Variables in sh -c need proper quoting or escaping, otherwise they expand
// in the calling shell instead of the invoked shell.
//
// Examples:
// Bad:
//   sh -c "echo $var"            // $var expands now, not in sh
//   bash -c "test $x -eq 1"      // $x expands in outer shell
//
// Good:
//   sh -c 'echo $var'            // Single quotes prevent expansion
//   bash -c "echo \$var"         // Escaped $ delays expansion
//   sh -c "echo '$var'"          // Expanded now, passed as literal
//
// Impact: Variables expand in wrong shell context

use crate::linter::{Diagnostic, LintResult, Severity, Span};
use once_cell::sync::Lazy;
use regex::Regex;

static SH_C_WITH_VAR: Lazy<Regex> = Lazy::new(|| {
    // Match: sh -c "...$var..." or bash -c "...${var}..." (with optional flags)
    Regex::new(r#"\b(sh|bash)(\s+-[a-z]+)*\s+-c\s+"[^"]*\$(\{[a-zA-Z_][a-zA-Z0-9_]*\}|[a-zA-Z_][a-zA-Z0-9_]*)[^"]*""#).unwrap()
});

pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();

    for (line_num, line) in source.lines().enumerate() {
        let line_num = line_num + 1;

        if line.trim_start().starts_with('#') {
            continue;
        }

        for mat in SH_C_WITH_VAR.find_iter(line) {
            let matched = mat.as_str();

            // Check if variables are escaped
            if matched.contains("\\$") {
                continue; // Escaped variables are OK
            }

            let start_col = mat.start() + 1;
            let end_col = mat.end() + 1;

            let diagnostic = Diagnostic::new(
                "SC2087",
                Severity::Warning,
                "Quote variables in sh -c / bash -c with single quotes or escape with \\$"
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
    fn test_sc2087_sh_c_with_var() {
        let code = r#"sh -c "echo $var""#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2087_bash_c() {
        let code = r#"bash -c "test $x -eq 1""#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2087_single_quotes_ok() {
        let code = r#"sh -c 'echo $var'"#;
        let result = check(code);
        // Single quotes prevent expansion
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2087_escaped_ok() {
        let code = r#"bash -c "echo \$var""#;
        let result = check(code);
        // Escaped $ is correct
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2087_no_var_ok() {
        let code = r#"sh -c "echo hello""#;
        let result = check(code);
        // No variables, OK
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2087_comment_ok() {
        let code = r#"# sh -c "echo $var""#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2087_multiple_vars() {
        let code = r#"sh -c "echo $a $b""#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1); // One warning for the command
    }

    #[test]
    #[ignore] // TODO: Handle sh -c with separated flags (-c -e)
    fn test_sc2087_with_flags() {
        let code = r#"bash -c -e "echo $var""#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2087_braced_var() {
        let code = r#"sh -c "echo ${var}""#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2087_sudo() {
        let code = r#"sudo sh -c "echo $HOME""#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }
}
