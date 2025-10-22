// SC2281: Don't use $@ in double quotes, it breaks word splitting
use crate::linter::{Diagnostic, LintResult, Severity, Span};
use once_cell::sync::Lazy;
use regex::Regex;

static QUOTED_AT: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r#""[^"]*\$@[^"]*""#).unwrap()
});

pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();
    for (line_num, line) in source.lines().enumerate() {
        let line_num = line_num + 1;
        if line.trim_start().starts_with('#') {
            continue;
        }

        if QUOTED_AT.is_match(line) {
            let diagnostic = Diagnostic::new(
                "SC2281",
                Severity::Warning,
                r#"Use "$*" or ${array[*]} for string concatenation, or ${array[@]} for separate elements"#.to_string(),
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
    fn test_sc2281_quoted_at() {
        let code = r#"msg="$@""#;
        assert_eq!(check(code).diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2281_quoted_star_ok() {
        let code = r#"msg="$*""#;
        assert_eq!(check(code).diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2281_unquoted_at_ok() {
        let code = r#"cmd $@"#;
        assert_eq!(check(code).diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2281_comment() {
        let code = r#"# "$@""#;
        assert_eq!(check(code).diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2281_empty() {
        assert_eq!(check("").diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2281_normal() {
        assert_eq!(check("echo test").diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2281_in_command() {
        let code = r#"echo "Args: $@""#;
        assert_eq!(check(code).diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2281_array_ok() {
        let code = r#"echo "${array[@]}""#;
        assert_eq!(check(code).diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2281_concatenation() {
        let code = r#"str="prefix $@ suffix""#;
        assert_eq!(check(code).diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2281_assignment() {
        let code = r#"all_args="$@""#;
        assert_eq!(check(code).diagnostics.len(), 1);
    }
}
