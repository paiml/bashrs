// SC2259: This redirection overrides piped input (`cmd | cmd2 < file`)
use crate::linter::{Diagnostic, LintResult, Severity, Span};
use regex::Regex;

static GREP_FOR_SUBSTRING: std::sync::LazyLock<Regex> =
    std::sync::LazyLock::new(|| Regex::new(r"\|[^|;&<]*[^<\s]\s+<\s*[^\s<(]").unwrap());

pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();
    for (line_num, line) in source.lines().enumerate() {
        let line_num = line_num + 1;
        if line.trim_start().starts_with('#') {
            continue;
        }

        if GREP_FOR_SUBSTRING.is_match(line) {
            let diagnostic = Diagnostic::new(
                "SC2259",
                Severity::Info,
                "This redirection overrides piped input; the pipe is ignored".to_string(),
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
    fn test_sc2259_echo_grep() {
        assert_eq!(
            check(r#"echo "$var" | grep -q pattern"#).diagnostics.len(),
            0
        );
    }
    #[test]
    fn test_sc2259_use_bracket_ok() {
        assert_eq!(check(r#"[[ "$var" == *pattern* ]]"#).diagnostics.len(), 0);
    }
    #[test]
    fn test_sc2259_file_grep_ok() {
        assert_eq!(check("grep -q pattern file").diagnostics.len(), 0);
    }
    #[test]
    fn test_sc2259_comment() {
        assert_eq!(check("# comment").diagnostics.len(), 0);
    }
    #[test]
    fn test_sc2259_empty() {
        assert_eq!(check("").diagnostics.len(), 0);
    }
    #[test]
    fn test_sc2259_normal() {
        assert_eq!(check("echo test").diagnostics.len(), 0);
    }
    #[test]
    fn test_sc2259_cmd_pipe_grep() {
        assert_eq!(check("ls | grep pattern").diagnostics.len(), 0);
    }
    #[test]
    fn test_sc2259_echo_literal() {
        assert_eq!(check(r#"echo "test" | grep -q foo"#).diagnostics.len(), 0);
    }
    #[test]
    fn test_sc2259_case_ok() {
        assert_eq!(check("case $x in *pattern*) ;;").diagnostics.len(), 0);
    }
    #[test]
    fn test_sc2259_regex_ok() {
        assert_eq!(check(r#"[[ "$x" =~ pattern ]]"#).diagnostics.len(), 0);
    }
}

#[cfg(test)]
mod redirect_override_tests {
    use super::*;
    #[test]
    fn test_sc2259_pipe_then_input_redirect_fires() {
        assert_eq!(check("echo x | grep -q a < file").diagnostics.len(), 1);
    }
    #[test]
    fn test_sc2259_pipe_without_redirect_ok() {
        assert_eq!(check(r#"echo "$x" | grep -q a"#).diagnostics.len(), 0);
    }
}
