// SC1087: Use braces when expanding arrays: ${arr[0]} not $arr[0]
//
// Without braces, $arr[0] is interpreted as the value of $arr followed by
// literal [0]. You must use ${arr[0]} to access array elements.
//
// Examples:
// Bad:
//   echo $arr[0]       # Interpreted as "$arr" followed by literal "[0]"
//   echo $list[1]      # Same problem
//   x=$data[5]         # Gets $data then literal [5]
//
// Good:
//   echo ${arr[0]}     # Correct array access
//   echo "${arr[1]}"   # Correct and quoted
//   x=${data[5]}       # Correct

use crate::linter::{Diagnostic, LintResult, Severity, Span};
use regex::Regex;

/// Matches $varname[ where varname is a valid identifier and it's NOT
/// already inside ${...}. We look for $identifier[ that isn't preceded by ${.
static UNBRACED_ARRAY: std::sync::LazyLock<Regex> = std::sync::LazyLock::new(|| {
    Regex::new(r"\$([A-Za-z_]\w*)\[").expect("SC1087 regex must compile")
});

pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();

    for (line_num, line) in source.lines().enumerate() {
        let line_num = line_num + 1;
        let trimmed = line.trim_start();
        if trimmed.starts_with('#') {
            continue;
        }

        for caps in UNBRACED_ARRAY.captures_iter(line) {
            let mat = caps.get(0).unwrap();
            let start = mat.start();

            // If this is preceded by ${ then it's already braced — skip
            // e.g., ${arr[0]} — the regex would match arr[ inside ${arr[0]}
            if start >= 1 {
                let before = &line[..start];
                if before.ends_with("${") || before.ends_with('{') {
                    continue;
                }
            }

            // Also skip if inside ${ ... } context
            if is_inside_braced_var(line, start) {
                continue;
            }

            let var_name = caps.get(1).map_or("", |m| m.as_str());
            let start_col = start + 1;
            let end_col = mat.end() + 1;

            result.add(Diagnostic::new(
                "SC1087",
                Severity::Error,
                format!(
                    "Use braces when expanding arrays: ${{{}[n]}} instead of ${}[n].",
                    var_name, var_name
                ),
                Span::new(line_num, start_col, line_num, end_col),
            ));
        }
    }

    result
}

/// Check if position is inside a ${...} expansion
fn is_inside_braced_var(line: &str, pos: usize) -> bool {
    let bytes = line.as_bytes();
    let mut i = 0;
    let mut depth = 0i32;
    while i < pos {
        if i + 1 < bytes.len() && bytes[i] == b'$' && bytes[i + 1] == b'{' {
            depth += 1;
            i += 2;
            continue;
        }
        if bytes[i] == b'}' && depth > 0 {
            depth -= 1;
        }
        i += 1;
    }
    depth > 0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sc1087_unbraced_array() {
        let code = "echo $arr[0]";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
        assert_eq!(result.diagnostics[0].code, "SC1087");
        assert_eq!(result.diagnostics[0].severity, Severity::Error);
        assert!(result.diagnostics[0].message.contains("${arr[n]}"));
    }

    #[test]
    fn test_sc1087_unbraced_list() {
        let code = "echo $list[1]";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc1087_assignment() {
        let code = "x=$data[5]";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc1087_braced_array_ok() {
        let code = "echo ${arr[0]}";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc1087_braced_quoted_ok() {
        let code = r#"echo "${arr[1]}""#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc1087_comment_ok() {
        let code = "# echo $arr[0]";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc1087_no_array_ok() {
        let code = "echo $var";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc1087_multiple() {
        let code = "echo $a[0] $b[1]";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 2);
    }
}
