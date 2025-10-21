// SC2216: Piping to 'rm' is dangerous - use xargs
use crate::linter::{Diagnostic, LintResult, Severity, Span};
use once_cell::sync::Lazy;
use regex::Regex;

static PIPE_TO_RM: Lazy<Regex> = Lazy::new(|| {
    // Match any command piped to rm
    Regex::new(r"\|\s*rm\b").unwrap()
});

pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();
    for (line_num, line) in source.lines().enumerate() {
        let line_num = line_num + 1;
        if line.trim_start().starts_with('#') {
            continue;
        }

        if PIPE_TO_RM.is_match(line) {
            let diagnostic = Diagnostic::new(
                "SC2216",
                Severity::Warning,
                "Piping to 'rm' is dangerous and may not work. Use 'xargs rm' or 'while read' instead".to_string(),
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
    fn test_sc2216_pipe_to_rm() {
        let code = r#"find . -name "*.tmp" | rm"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }
    #[test]
    fn test_sc2216_xargs_ok() {
        let code = r#"find . -name "*.tmp" | xargs rm"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }
    #[test]
    fn test_sc2216_pipe_rm_with_flags() {
        let code = r#"ls *.bak | rm -f"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }
    #[test]
    fn test_sc2216_grep_to_rm() {
        let code = r#"grep -l pattern *.txt | rm"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }
    #[test]
    fn test_sc2216_find_exec_ok() {
        let code = r#"find . -name "*.tmp" -exec rm {} +"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }
    #[test]
    fn test_sc2216_while_read_ok() {
        let code = r#"find . -name "*.tmp" | while read f; do rm "$f"; done"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0); // While read, not piping directly to rm
    }
    #[test]
    fn test_sc2216_comment_skipped() {
        let code = r#"# find . | rm"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }
    #[test]
    fn test_sc2216_rm_not_piped_ok() {
        let code = r#"rm *.tmp"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }
    #[test]
    fn test_sc2216_pipe_to_other_cmd_ok() {
        let code = r#"find . | sort"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }
    #[test]
    fn test_sc2216_multiple_pipes() {
        let code = r#"find . | grep pattern | rm -f"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }
}
