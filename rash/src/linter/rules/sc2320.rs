// SC2320: This $N expands to the parameter, not a separate word
use crate::linter::{Diagnostic, LintResult, Severity, Span};
use regex::Regex;

static UNQUOTED_PARAM: std::sync::LazyLock<Regex> =
    std::sync::LazyLock::new(|| Regex::new(r"(?:=|:)\s*\$[0-9]+").unwrap());

pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();
    for (line_num, line) in source.lines().enumerate() {
        let line_num = line_num + 1;
        if line.trim_start().starts_with('#') {
            continue;
        }

        if UNQUOTED_PARAM.is_match(line) && !line.contains("\"$") {
            let diagnostic = Diagnostic::new(
                "SC2320",
                Severity::Info,
                "Positional parameter expands as single word. Quote to prevent word splitting: \"$1\"".to_string(),
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
    fn test_sc2320_unquoted_param() {
        let code = "file=$1";
        assert_eq!(check(code).diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2320_quoted_ok() {
        let code = r#"file="$1""#;
        assert_eq!(check(code).diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2320_in_test_ok() {
        let code = "[ -f $1 ]";
        assert_eq!(check(code).diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2320_comment() {
        let code = "# file=$1";
        assert_eq!(check(code).diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2320_empty() {
        assert_eq!(check("").diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2320_normal() {
        assert_eq!(check("echo test").diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2320_param_2() {
        let code = "arg=$2";
        assert_eq!(check(code).diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2320_in_default() {
        let code = "val=${VAR:=$1}";
        assert_eq!(check(code).diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2320_command_substitution_ok() {
        let code = "result=$(cmd $1)";
        assert_eq!(check(code).diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2320_multiple() {
        let code = r#"
x=$1
y=$2
"#;
        assert_eq!(check(code).diagnostics.len(), 2);
    }
}
