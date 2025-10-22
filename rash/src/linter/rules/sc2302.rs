// SC2302: Prefer ${var// /} over tr for simple substitution
use crate::linter::{Diagnostic, LintResult, Severity, Span};
use once_cell::sync::Lazy;
use regex::Regex;

static TR_DELETE: Lazy<Regex> = Lazy::new(|| Regex::new(r#"tr\s+-d\s+['"].\s*['"]"#).unwrap());

pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();
    for (line_num, line) in source.lines().enumerate() {
        let line_num = line_num + 1;
        if line.trim_start().starts_with('#') {
            continue;
        }

        if TR_DELETE.is_match(line) && line.contains("<<<") {
            let diagnostic = Diagnostic::new(
                "SC2302",
                Severity::Info,
                "Consider using ${var//pattern/} for simple character removal".to_string(),
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
    fn test_sc2302_tr_delete() {
        let code = "tr -d ' ' <<< $var";
        assert_eq!(check(code).diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2302_param_expansion_ok() {
        let code = "result=${var// /}";
        assert_eq!(check(code).diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2302_tr_from_file_ok() {
        let code = "tr -d ' ' < file.txt";
        assert_eq!(check(code).diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2302_comment() {
        let code = "# tr -d ' ' <<< $var";
        assert_eq!(check(code).diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2302_empty() {
        assert_eq!(check("").diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2302_normal() {
        assert_eq!(check("echo test").diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2302_tr_translate_ok() {
        let code = "tr 'a-z' 'A-Z' <<< $var";
        assert_eq!(check(code).diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2302_double_quote() {
        let code = r#"tr -d "," <<< $csv"#;
        assert_eq!(check(code).diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2302_tr_squeeze_ok() {
        let code = "tr -s ' ' <<< $var";
        assert_eq!(check(code).diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2302_pipe_ok() {
        let code = "cat file | tr -d ' '";
        assert_eq!(check(code).diagnostics.len(), 0);
    }
}
