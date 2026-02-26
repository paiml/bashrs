// SC2276: Avoid useless cat with here documents
use crate::linter::{Diagnostic, LintResult, Severity, Span};
use regex::Regex;

static CAT_HEREDOC: std::sync::LazyLock<Regex> =
    std::sync::LazyLock::new(|| Regex::new(r"cat\s*<<[^<]").unwrap());

pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();
    for (line_num, line) in source.lines().enumerate() {
        let line_num = line_num + 1;
        if line.trim_start().starts_with('#') {
            continue;
        }

        if CAT_HEREDOC.is_match(line) {
            let diagnostic = Diagnostic::new(
                "SC2276",
                Severity::Info,
                "Avoid useless cat - use here document directly".to_string(),
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
    fn test_sc2276_cat_heredoc() {
        let code = "cat << EOF";
        assert_eq!(check(code).diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2276_direct_heredoc_ok() {
        let code = "command << EOF";
        assert_eq!(check(code).diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2276_cat_file_ok() {
        let code = "cat file.txt";
        assert_eq!(check(code).diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2276_comment() {
        let code = "# cat << EOF";
        assert_eq!(check(code).diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2276_empty() {
        assert_eq!(check("").diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2276_normal() {
        assert_eq!(check("echo test").diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2276_cat_heredoc_dash() {
        let code = "cat <<- EOF";
        assert_eq!(check(code).diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2276_cat_here_string_ok() {
        let code = "cat <<< '$var'";
        assert_eq!(check(code).diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2276_concatenate_ok() {
        let code = "cat file1 file2";
        assert_eq!(check(code).diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2276_pipe_to_cat_ok() {
        let code = "echo test | cat";
        assert_eq!(check(code).diagnostics.len(), 0);
    }
}
