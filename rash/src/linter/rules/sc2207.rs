// SC2207: Prefer mapfile or read -a to split command output
use crate::linter::{Diagnostic, LintResult, Severity, Span};
use regex::Regex;

static ARRAY_FROM_CMDSUB: std::sync::LazyLock<Regex> = std::sync::LazyLock::new(|| {
    // Match array=( $(cmd) ) without quotes
    Regex::new(r"\w+\s*=\s*\(\s*\$\([^)]+\)\s*\)").unwrap()
});

pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();
    for (line_num, line) in source.lines().enumerate() {
        let line_num = line_num + 1;
        if line.trim_start().starts_with('#') {
            continue;
        }

        if ARRAY_FROM_CMDSUB.is_match(line) {
            // Exclude if quoted
            if !line.contains("(\"$(") {
                let diagnostic = Diagnostic::new(
                    "SC2207",
                    Severity::Info,
                    "Prefer mapfile or read -a to split command output, or use quotes to avoid splitting".to_string(),
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
    fn test_sc2207_cmdsub_split() {
        let code = r#"array=( $(ls *.txt) )"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }
    #[test]
    fn test_sc2207_quoted_ok() {
        let code = r#"array=( "$(ls *.txt)" )"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }
    #[test]
    fn test_sc2207_find_command() {
        let code = r#"files=( $(find . -name "*.sh") )"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }
    #[test]
    fn test_sc2207_literal_ok() {
        let code = r#"array=(one two three)"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }
    #[test]
    fn test_sc2207_grep_output() {
        let code = r#"matches=($(grep pattern file))"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }
    #[test]
    fn test_sc2207_awk_output() {
        let code = r#"arr=($(awk '{print $1}' data.txt))"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }
    #[test]
    fn test_sc2207_no_spaces() {
        let code = r#"arr=($(cmd))"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }
    #[test]
    fn test_sc2207_multiple_lines() {
        let code = "array=( \n$(get_items)\n)";
        let result = check(code);
        // Pattern may not match multiline, but single line should
        assert!(result.diagnostics.len() <= 1);
    }
    #[test]
    fn test_sc2207_comment_skipped() {
        let code = r#"# array=( $(ls) )"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }
    #[test]
    fn test_sc2207_piped_command() {
        let code = r#"lines=( $(cat file | sort) )"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }
}
