// SC2276: Avoid useless cat with here documents
//
// PMAT-248 (#242): `cat <<EOF ... EOF` with no pipe is the ordinary,
// idiomatic way to emit a block of text to stdout (or to a redirect target
// given elsewhere on the line) — there is no simpler construct that replaces
// it. It is only "useless" when its output is piped into another command,
// since the heredoc could be fed to that command directly without `cat` at
// all: `cat <<EOF | grep x` should be `grep x <<EOF`.
use crate::linter::{Diagnostic, LintResult, Severity, Span};
use regex::Regex;

static CAT_HEREDOC: std::sync::LazyLock<Regex> =
    std::sync::LazyLock::new(|| Regex::new(r"cat\s*<<[^<]").unwrap());

/// Is there a pipe to another command after `at`? `||` (logical or) does not
/// count.
fn pipes_to_another_command(rest: &str) -> bool {
    let bytes = rest.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'|' {
            if bytes.get(i + 1) == Some(&b'|') {
                i += 2;
                continue;
            }
            return true;
        }
        i += 1;
    }
    false
}

pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();
    for (line_num, line) in source.lines().enumerate() {
        let line_num = line_num + 1;
        if line.trim_start().starts_with('#') {
            continue;
        }

        let Some(mat) = CAT_HEREDOC.find(line) else {
            continue;
        };
        if !pipes_to_another_command(&line[mat.end()..]) {
            continue;
        }

        let diagnostic = Diagnostic::new(
            "SC2276",
            Severity::Info,
            "Avoid useless cat - use here document directly".to_string(),
            Span::new(line_num, 1, line_num, line.len() + 1),
        );
        result.add(diagnostic);
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sc2276_cat_heredoc_to_stdout_ok() {
        // PMAT-248 (#242): a cat whose heredoc goes to stdout is the ordinary
        // way to emit a block of text — it is not useless. This test used to
        // assert `len() == 1`; that encoded the old, overly broad behavior.
        let code = "cat << EOF";
        assert_eq!(check(code).diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2276_cat_heredoc_piped_flagged() {
        let code = "cat <<EOF | grep x";
        assert_eq!(check(code).diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2276_cat_heredoc_quoted_delim_piped_flagged() {
        let code = "cat <<'EOF' | grep x";
        assert_eq!(check(code).diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2276_cat_heredoc_piped_no_spaces_flagged() {
        let code = "cat<<EOF|grep x";
        assert_eq!(check(code).diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2276_cat_heredoc_to_redirect_ok() {
        // The heredoc's target is a file, not another command's stdin.
        let code = "cat <<EOF > out.txt";
        assert_eq!(check(code).diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2276_cat_heredoc_logical_or_not_a_pipe() {
        // `||` is logical-or, not a pipe to another command.
        let code = "cat <<EOF || true";
        assert_eq!(check(code).diagnostics.len(), 0);
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
        // PMAT-248 (#242): no pipe, so not useless — see
        // `test_sc2276_cat_heredoc_to_stdout_ok`.
        let code = "cat <<- EOF";
        assert_eq!(check(code).diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2276_cat_heredoc_dash_piped_flagged() {
        let code = "cat <<- EOF | grep x";
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
