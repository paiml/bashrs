// SC2179: Use array+=("item") to append to arrays, not array=("${array[@]}" "item").
//
// Appending to arrays should use += syntax, not reconstruction.
//
// Examples:
// Bad:
//   array=("${array[@]}" "new")  // Reconstructs array
//   arr=("${arr[@]}" "$item")    // Inefficient
//
// Good:
//   array+=("new")               // Append syntax
//   array+=("item1" "item2")     // Multiple items
//
// Impact: Performance and readability

use crate::linter::{Diagnostic, LintResult, Severity, Span};
use once_cell::sync::Lazy;
use regex::Regex;

static ARRAY_RECONSTRUCTION: Lazy<Regex> = Lazy::new(|| {
    // Match: var=("${var[@]}" ...)
    // Can't use backreferences in Rust regex, so match the pattern and check manually
    Regex::new(r#"(\w+)=\(\s*"\$\{(\w+)\[@\]\}""#).unwrap()
});

pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();

    for (line_num, line) in source.lines().enumerate() {
        let line_num = line_num + 1;

        if line.trim_start().starts_with('#') {
            continue;
        }

        for cap in ARRAY_RECONSTRUCTION.captures_iter(line) {
            // Extract the two variable names
            let var1 = cap.get(1).map(|m| m.as_str()).unwrap_or("");
            let var2 = cap.get(2).map(|m| m.as_str()).unwrap_or("");

            // Only flag if the variable names match (reconstructing same array)
            if var1 == var2 {
                let mat = cap.get(0).unwrap();
                let start_col = mat.start() + 1;
                let end_col = mat.end() + 1;

                let diagnostic = Diagnostic::new(
                    "SC2179",
                    Severity::Info,
                    r#"Use array+=("item") to append, not array=("${array[@]}" "item")"#
                        .to_string(),
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
    fn test_sc2179_array_reconstruction() {
        let code = r#"array=("${array[@]}" "new")"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2179_array_append_ok() {
        let code = r#"array+=("new")"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2179_array_init_ok() {
        let code = r#"array=("item1" "item2")"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2179_comment_ok() {
        let code = r#"# array=("${array[@]}" "new")"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2179_different_array_ok() {
        let code = r#"arr2=("${arr1[@]}" "new")"#;
        let result = check(code);
        // Different arrays - this is OK
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2179_multiple_items() {
        let code = r#"array=("${array[@]}" "item1" "item2")"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2179_append_multiple_ok() {
        let code = r#"array+=("item1" "item2")"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2179_multiple() {
        let code = r#"
arr=("${arr[@]}" "a")
arr=("${arr[@]}" "b")
"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 2);
    }

    #[test]
    fn test_sc2179_var_reconstruction() {
        let code = r#"files=("${files[@]}" "$newfile")"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2179_star_syntax() {
        let code = r#"array=("${array[*]}" "new")"#;
        let result = check(code);
        // Different syntax, not matching pattern
        assert_eq!(result.diagnostics.len(), 0);
    }
}
