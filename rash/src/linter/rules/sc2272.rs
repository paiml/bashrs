// SC2272: Prefer specific flags over combining find with pipes
use crate::linter::{Diagnostic, LintResult, Severity, Span};
use once_cell::sync::Lazy;
use regex::Regex;

static FIND_PIPE_XARGS: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"find\s+[^|]+\|\s*xargs\s+").unwrap()
});

static XARGS_WITH_0: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"xargs\s+-[a-z]*0").unwrap()
});

pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();
    for (line_num, line) in source.lines().enumerate() {
        let line_num = line_num + 1;
        if line.trim_start().starts_with('#') {
            continue;
        }

        if FIND_PIPE_XARGS.is_match(line) && !XARGS_WITH_0.is_match(line) {
            let diagnostic = Diagnostic::new(
                "SC2272",
                Severity::Warning,
                "Use find -print0 | xargs -0 or find -exec for filenames with spaces".to_string(),
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
    fn test_sc2272_find_pipe_xargs() {
        let code = "find . -name '*.txt' | xargs rm";
        assert_eq!(check(code).diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2272_find_print0_ok() {
        let code = "find . -name '*.txt' -print0 | xargs -0 rm";
        assert_eq!(check(code).diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2272_find_exec_ok() {
        let code = "find . -name '*.txt' -exec rm {} +";
        assert_eq!(check(code).diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2272_comment() {
        let code = "# find . | xargs rm";
        assert_eq!(check(code).diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2272_empty() {
        assert_eq!(check("").diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2272_normal() {
        assert_eq!(check("echo test").diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2272_find_alone_ok() {
        let code = "find . -name '*.txt'";
        assert_eq!(check(code).diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2272_xargs_0_ok() {
        let code = "find . | xargs -0 rm";
        assert_eq!(check(code).diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2272_find_delete_ok() {
        let code = "find . -name '*.tmp' -delete";
        assert_eq!(check(code).diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2272_pipe_grep_ok() {
        let code = "find . | grep pattern";
        assert_eq!(check(code).diagnostics.len(), 0);
    }
}
