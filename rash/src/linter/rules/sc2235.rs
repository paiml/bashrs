// SC2235: Quote arguments to unalias to prevent word splitting
use crate::linter::{Diagnostic, LintResult, Severity, Span};
use once_cell::sync::Lazy;
use regex::Regex;

static UNQUOTED_UNALIAS: Lazy<Regex> = Lazy::new(|| {
    // Match: unalias $var or unalias $(cmd)
    Regex::new(r"\bunalias\s+\$").unwrap()
});

pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();
    for (line_num, line) in source.lines().enumerate() {
        let line_num = line_num + 1;
        if line.trim_start().starts_with('#') {
            continue;
        }

        if UNQUOTED_UNALIAS.is_match(line) {
            if !line.contains("\"$") {
                let diagnostic = Diagnostic::new(
                    "SC2235",
                    Severity::Warning,
                    "Quote arguments to unalias to prevent word splitting and glob expansion"
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
    fn test_sc2235_unquoted_var() {
        let code = "unalias $alias_name";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }
    #[test]
    fn test_sc2235_quoted_var_ok() {
        let code = r#"unalias "$alias_name""#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }
    #[test]
    fn test_sc2235_literal_ok() {
        let code = "unalias ls";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }
    #[test]
    fn test_sc2235_multiple_aliases() {
        let code = "unalias $a $b";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }
    #[test]
    fn test_sc2235_comment_skipped() {
        let code = "# unalias $var";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }
    #[test]
    fn test_sc2235_unalias_all() {
        let code = "unalias -a";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }
    #[test]
    fn test_sc2235_braced_var() {
        let code = "unalias ${alias}";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }
    #[test]
    fn test_sc2235_command_substitution() {
        let code = "unalias $(get_alias)";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }
    #[test]
    fn test_sc2235_no_code() {
        let code = "";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }
    #[test]
    fn test_sc2235_normal_command() {
        let code = "echo test";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }
}
