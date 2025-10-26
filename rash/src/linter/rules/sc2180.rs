// SC2180: Bash doesn't support multidimensional arrays. Use associative arrays.
//
// Bash only supports 1D arrays. Multidimensional access won't work.
//
// Examples:
// Bad:
//   array[0][1]=value            // Not supported
//   echo "${array[0][1]}"        // Won't work
//
// Good:
//   declare -A array             // Associative array
//   array["0,1"]=value           // Use key like "row,col"
//   echo "${array["0,1"]}"       // Access with key
//
// Impact: Syntax error or unexpected behavior

use crate::linter::{Diagnostic, LintResult, Severity, Span};
use once_cell::sync::Lazy;
use regex::Regex;

static MULTIDIM_ARRAY: Lazy<Regex> = Lazy::new(|| Regex::new(r"\[\d+\]\[\d+\]").unwrap());

pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();

    for (line_num, line) in source.lines().enumerate() {
        let line_num = line_num + 1;

        if line.trim_start().starts_with('#') {
            continue;
        }

        for mat in MULTIDIM_ARRAY.find_iter(line) {
            let start_col = mat.start() + 1;
            let end_col = mat.end() + 1;

            let diagnostic = Diagnostic::new(
                "SC2180",
                Severity::Error,
                "Bash doesn't support multidimensional arrays. Use associative arrays".to_string(),
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
    fn test_sc2180_multidim_access() {
        let code = r#"echo "${array[0][1]}""#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2180_multidim_assignment() {
        let code = "array[0][1]=value";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2180_single_dim_ok() {
        let code = r#"echo "${array[0]}""#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2180_assoc_array_ok() {
        let code = r#"declare -A array; array["0,1"]=value"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2180_comment_ok() {
        let code = "# array[0][1]=value";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2180_multiple() {
        let code = "array[0][1]=a\narray[2][3]=b";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 2);
    }

    #[test]
    fn test_sc2180_higher_dimensions() {
        let code = "array[0][1][2]=value";
        let result = check(code);
        // Matches [0][1]
        assert!(!result.diagnostics.is_empty());
    }

    #[test]
    fn test_sc2180_nested_var_ok() {
        let code = r#"echo "${array[${index}]}""#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2180_arithmetic_ok() {
        let code = "result=$((array[0] + array[1]))";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2180_matrix_access() {
        let code = "value=${matrix[5][10]}";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }
}
