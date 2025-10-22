// SC2277: Prefer process substitution over temporary files
use crate::linter::{Diagnostic, LintResult, Severity, Span};
use once_cell::sync::Lazy;
use regex::Regex;

static TEMP_FILE_PATTERN: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"mktemp.*;\s*\w+\s+[^;]+;\s*rm").unwrap()
});

pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();
    
    // Check across multiple lines for temp file pattern
    if TEMP_FILE_PATTERN.is_match(source) {
        for (line_num, line) in source.lines().enumerate() {
            let line_num = line_num + 1;
            if line.contains("mktemp") {
                let diagnostic = Diagnostic::new(
                    "SC2277",
                    Severity::Info,
                    "Consider using process substitution <(...) instead of temporary files".to_string(),
                    Span::new(line_num, 1, line_num, line.len() + 1),
                );
                result.add(diagnostic);
                break;
            }
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sc2277_temp_file_pattern() {
        let code = "tmp=$(mktemp); echo data > $tmp; cat $tmp; rm $tmp";
        assert_eq!(check(code).diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2277_process_subst_ok() {
        let code = "diff <(cmd1) <(cmd2)";
        assert_eq!(check(code).diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2277_mktemp_alone_ok() {
        let code = "tmp=$(mktemp)";
        assert_eq!(check(code).diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2277_comment() {
        let code = "# mktemp; cat; rm";
        assert_eq!(check(code).diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2277_empty() {
        assert_eq!(check("").diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2277_normal() {
        assert_eq!(check("echo test").diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2277_mktemp_kept_ok() {
        let code = "tmp=$(mktemp); echo data > $tmp";
        assert_eq!(check(code).diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2277_no_mktemp_ok() {
        let code = "cat file; rm file";
        assert_eq!(check(code).diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2277_heredoc_ok() {
        let code = "cat << EOF\ndata\nEOF";
        assert_eq!(check(code).diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2277_pipe_ok() {
        let code = "echo data | cat";
        assert_eq!(check(code).diagnostics.len(), 0);
    }
}
