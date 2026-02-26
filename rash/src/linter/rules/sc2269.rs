// SC2269: Use read -r to preserve backslashes
use crate::linter::{Diagnostic, LintResult, Severity, Span};
use regex::Regex;

static READ_WITHOUT_R: std::sync::LazyLock<Regex> =
    std::sync::LazyLock::new(|| Regex::new(r"\bread\s+").unwrap());

static READ_WITH_R: std::sync::LazyLock<Regex> =
    std::sync::LazyLock::new(|| Regex::new(r"\bread\s+-[a-z]*r").unwrap());

pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();
    for (line_num, line) in source.lines().enumerate() {
        let line_num = line_num + 1;
        if line.trim_start().starts_with('#') {
            continue;
        }

        // Check if 'read' is present without '-r' flag
        if READ_WITHOUT_R.is_match(line) && !READ_WITH_R.is_match(line) {
            let diagnostic = Diagnostic::new(
                "SC2269",
                Severity::Warning,
                "Use 'read -r' to prevent backslash interpretation".to_string(),
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
    fn test_sc2269_read_without_r() {
        let code = "read line";
        assert_eq!(check(code).diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2269_read_with_r_ok() {
        let code = "read -r line";
        assert_eq!(check(code).diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2269_read_with_flags() {
        let code = "read -p 'Enter: ' var";
        assert_eq!(check(code).diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2269_read_with_r_and_flags_ok() {
        let code = "read -r -p 'Enter: ' var";
        assert_eq!(check(code).diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2269_comment() {
        let code = "# read line";
        assert_eq!(check(code).diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2269_empty() {
        assert_eq!(check("").diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2269_normal() {
        assert_eq!(check("echo test").diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2269_readonly_ok() {
        let code = "readonly VAR=value";
        assert_eq!(check(code).diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2269_while_read() {
        let code = "while read line; do";
        assert_eq!(check(code).diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2269_while_read_r_ok() {
        let code = "while read -r line; do";
        assert_eq!(check(code).diagnostics.len(), 0);
    }
}
