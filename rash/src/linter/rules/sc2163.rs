// SC2163: export/readonly doesn't work with arrays in most shells
//
// In bash, you cannot export or make arrays readonly directly.
// This syntax is invalid in POSIX sh and most shells.
//
// Examples:
// Bad:
//   export array=(a b c)
//   readonly items=(1 2 3)
//
// Good:
//   array=(a b c)
//   # Arrays can't be exported in POSIX

use crate::linter::{Diagnostic, LintResult, Severity, Span};
use once_cell::sync::Lazy;
use regex::Regex;

static EXPORT_READONLY_ARRAY: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"\b(export|readonly)\s+([A-Za-z_][A-Za-z0-9_]*)=\(").unwrap());

pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();

    for (line_num, line) in source.lines().enumerate() {
        let line_num = line_num + 1;

        if line.trim_start().starts_with('#') {
            continue;
        }

        for cap in EXPORT_READONLY_ARRAY.captures_iter(line) {
            if let Some(keyword) = cap.get(1) {
                if let Some(var_name) = cap.get(2) {
                    let start_col = keyword.start() + 1;
                    let end_col = keyword.end() + 1;

                    let diagnostic = Diagnostic::new(
                        "SC2163",
                        Severity::Error,
                        format!(
                            "{} doesn't work with arrays. Declare separately.",
                            keyword.as_str()
                        ),
                        Span::new(line_num, start_col, line_num, end_col),
                    );

                    result.add(diagnostic);
                }
            }
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sc2163_export_array() {
        let code = r#"export array=(a b c)"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
        assert_eq!(result.diagnostics[0].code, "SC2163");
        assert_eq!(result.diagnostics[0].severity, Severity::Error);
        assert!(result.diagnostics[0].message.contains("export"));
    }

    #[test]
    fn test_sc2163_readonly_array() {
        let code = r#"readonly items=(1 2 3)"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
        assert!(result.diagnostics[0].message.contains("readonly"));
    }

    #[test]
    fn test_sc2163_normal_array_ok() {
        let code = r#"array=(a b c)"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2163_export_scalar_ok() {
        let code = r#"export VAR="value""#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2163_readonly_scalar_ok() {
        let code = r#"readonly CONST=42"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2163_declare_array_ok() {
        let code = r#"declare -a array=(a b c)"#;
        let result = check(code);
        // declare is OK for arrays
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2163_multiline() {
        let code = r#"
export files=(
    file1.txt
    file2.txt
)
"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2163_mixed_content() {
        let code = r#"
export VAR="ok"
readonly array=(not ok)
export PATH="/usr/bin"
"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2163_empty_array() {
        let code = r#"export empty=()"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2163_local_array_ok() {
        let code = r#"local array=(a b c)"#;
        let result = check(code);
        // local is OK for arrays
        assert_eq!(result.diagnostics.len(), 0);
    }
}
