// SC2286: Prefer mapfile/readarray over read loops
use crate::linter::{Diagnostic, LintResult, Severity, Span};
use regex::Regex;

static READ_LOOP: std::sync::LazyLock<Regex> =
    std::sync::LazyLock::new(|| Regex::new(r"while\s+read\s+[^;]+;\s*do").unwrap());

pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();
    for (line_num, line) in source.lines().enumerate() {
        let line_num = line_num + 1;
        if line.trim_start().starts_with('#') {
            continue;
        }

        if READ_LOOP.is_match(line) {
            let diagnostic = Diagnostic::new(
                "SC2286",
                Severity::Info,
                "Consider using mapfile/readarray for reading files into arrays".to_string(),
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
    fn test_sc2286_while_read() {
        let code = "while read line; do";
        assert_eq!(check(code).diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2286_mapfile_ok() {
        let code = "mapfile -t array < file.txt";
        assert_eq!(check(code).diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2286_readarray_ok() {
        let code = "readarray -t array < file.txt";
        assert_eq!(check(code).diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2286_comment() {
        let code = "# while read line; do";
        assert_eq!(check(code).diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2286_empty() {
        assert_eq!(check("").diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2286_normal() {
        assert_eq!(check("echo test").diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2286_read_multiple_vars() {
        let code = "while read key value; do";
        assert_eq!(check(code).diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2286_for_loop_ok() {
        let code = "for item in list; do";
        assert_eq!(check(code).diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2286_while_condition_ok() {
        let code = "while [ $i -lt 10 ]; do";
        assert_eq!(check(code).diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2286_read_with_flags() {
        let code = "while read -r line; do";
        assert_eq!(check(code).diagnostics.len(), 1);
    }
}
