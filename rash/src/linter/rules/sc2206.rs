// SC2206: Quote to prevent word splitting/globbing in arrays
use crate::linter::{Diagnostic, LintResult, Severity, Span};
use once_cell::sync::Lazy;
use regex::Regex;

static UNQUOTED_ARRAY_EXPANSION: Lazy<Regex> = Lazy::new(|| {
    // Match array=($var) or array=($(cmd)) or array=($a $b) without quotes
    Regex::new(r"\w+\s*=\s*\([^)]*\$").unwrap()
});

pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();
    for (line_num, line) in source.lines().enumerate() {
        let line_num = line_num + 1;
        if line.trim_start().starts_with('#') {
            continue;
        }

        if UNQUOTED_ARRAY_EXPANSION.is_match(line) {
            // Exclude if already quoted: array=("$var")
            if !line.contains("(\"$") && !line.contains("(\"$(") {
                let diagnostic = Diagnostic::new(
                    "SC2206",
                    Severity::Warning,
                    "Quote to prevent word splitting/globbing in arrays. Use array=(\"$var\") or mapfile".to_string(),
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
    fn test_sc2206_unquoted_var() {
        let code = r#"array=($var)"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }
    #[test]
    fn test_sc2206_unquoted_cmdsub() {
        let code = r#"files=$(ls *.txt)"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0); // Not array assignment
    }
    #[test]
    fn test_sc2206_quoted_ok() {
        let code = r#"array=("$var")"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }
    #[test]
    fn test_sc2206_literal_ok() {
        let code = r#"array=(one two three)"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }
    #[test]
    fn test_sc2206_expansion_in_array() {
        let code = r#"arr=($VAR)"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }
    #[test]
    fn test_sc2206_braced_var() {
        let code = r#"array=(${items})"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }
    #[test]
    fn test_sc2206_cmdsub_in_array() {
        let code = r#"array=($(find . -name "*.txt"))"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }
    #[test]
    fn test_sc2206_multiple_vars() {
        let code = r#"arr=($a $b $c)"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }
    #[test]
    fn test_sc2206_quoted_cmdsub_ok() {
        let code = r#"array=("$(get_items)")"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }
    #[test]
    fn test_sc2206_comment_skipped() {
        let code = r#"# array=($var)"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }
}
