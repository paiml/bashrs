// SC2082: To expand via indirection, use arrays, ${!name}, or (for sh) eval
//
// Using $$var for indirection doesn't work as expected. It expands $ twice,
// which gives the PID followed by var literally.
//
// Examples:
// Bad:
//   value=$$var                // Expands to PID + literal "var"
//   echo $$myvar               // Wrong: PID + "myvar"
//
// Good:
//   value=${!var}              // Indirect expansion (bash)
//   eval "value=\$$var"        // POSIX-compatible indirection
//   declare -n ref=var; value=$ref  // Name reference (bash 4.3+)
//
// Impact: Wrong values, PID instead of variable content

use crate::linter::{Diagnostic, LintResult, Severity, Span};
use once_cell::sync::Lazy;
use regex::Regex;

static DOUBLE_DOLLAR_VAR: Lazy<Regex> = Lazy::new(|| {
    // Match: $$var (not $$ alone, which is PID)
    Regex::new(r"\$\$[a-zA-Z_][a-zA-Z0-9_]*").unwrap()
});

pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();

    for (line_num, line) in source.lines().enumerate() {
        let line_num = line_num + 1;

        if line.trim_start().starts_with('#') {
            continue;
        }

        for mat in DOUBLE_DOLLAR_VAR.find_iter(line) {
            let start_col = mat.start() + 1;
            let end_col = mat.end() + 1;

            let diagnostic = Diagnostic::new(
                "SC2082",
                Severity::Warning,
                "To expand via indirection, use ${!name} or eval. $$ is the PID, not indirection"
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
    fn test_sc2082_double_dollar_var() {
        let code = r#"value=$$var"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2082_echo() {
        let code = r#"echo $$myvar"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2082_indirect_ok() {
        let code = r#"value=${!var}"#;
        let result = check(code);
        // Correct indirection syntax
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2082_pid_ok() {
        let code = r#"echo $$"#;
        let result = check(code);
        // $$ alone is PID (correct)
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    #[ignore] // TODO: Don't flag escaped $$ in eval
    fn test_sc2082_eval_ok() {
        let code = r#"eval "value=\$$var""#;
        let result = check(code);
        // eval with escaped $ is correct but hard to detect
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2082_comment_ok() {
        let code = r#"# value=$$var"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2082_nameref_ok() {
        let code = r#"declare -n ref=var; value=$ref"#;
        let result = check(code);
        // Name reference is correct
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2082_multiple() {
        let code = r#"a=$$var1; b=$$var2"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 2);
    }

    #[test]
    fn test_sc2082_in_string() {
        let code = r#"msg="Value is $$value""#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    #[ignore] // TODO: Handle braced variables after $$
    fn test_sc2082_braced() {
        let code = r#"value=$${var}"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }
}
