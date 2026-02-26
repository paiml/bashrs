// SC2293: Use += to append to arrays
use crate::linter::{Diagnostic, LintResult, Severity, Span};
use regex::Regex;

static ARRAY_APPEND_PATTERN: std::sync::LazyLock<Regex> = std::sync::LazyLock::new(|| {
    Regex::new(r#"([a-zA-Z_][a-zA-Z0-9_]*)=\(\s*"\$\{([a-zA-Z_][a-zA-Z0-9_]*)\[@\]\}""#).unwrap()
});

pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();
    for (line_num, line) in source.lines().enumerate() {
        let line_num = line_num + 1;
        if line.trim_start().starts_with('#') {
            continue;
        }

        if let Some(caps) = ARRAY_APPEND_PATTERN.captures(line) {
            // Extract variable names to check if same array is being reassigned
            let assign_name = &caps[1];
            let expand_name = &caps[2];

            // Only warn if reassigning the same array to itself
            if assign_name == expand_name {
                let diagnostic = Diagnostic::new(
                    "SC2293",
                    Severity::Info,
                    "Use array+=(elements) instead of array=(...old...new) for appending"
                        .to_string(),
                    Span::new(line_num, 1, line_num, line.len() + 1),
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
    fn test_sc2293_array_reassign() {
        let code = r#"arr=("${arr[@]}" "new")"#;
        assert_eq!(check(code).diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2293_plus_equals_ok() {
        let code = r#"arr+=("new")"#;
        assert_eq!(check(code).diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2293_comment() {
        let code = r#"# arr=("${arr[@]}" "new")"#;
        assert_eq!(check(code).diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2293_empty() {
        assert_eq!(check("").diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2293_normal() {
        assert_eq!(check("echo test").diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2293_simple_assign_ok() {
        let code = r#"arr=("a" "b" "c")"#;
        assert_eq!(check(code).diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2293_string_concat_ok() {
        let code = r#"str="${str}new""#;
        assert_eq!(check(code).diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2293_indexed_assign_ok() {
        let code = r#"arr[0]="value""#;
        assert_eq!(check(code).diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2293_copy_ok() {
        let code = r#"new=("${old[@]}")"#;
        assert_eq!(check(code).diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2293_list_assign() {
        let code = r#"list=("${list[@]}" "$item")"#;
        assert_eq!(check(code).diagnostics.len(), 1);
    }
}
