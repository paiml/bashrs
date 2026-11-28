// SC2206: Quote to prevent word splitting/globbing in arrays
use crate::linter::{Diagnostic, LintResult, Severity, Span};
use regex::Regex;

static UNQUOTED_ARRAY_EXPANSION: std::sync::LazyLock<Regex> = std::sync::LazyLock::new(|| {
    // Match array=($var) or array=($(cmd)) or array=($a $b) without quotes
    Regex::new(r"\w+\s*=\s*\([^)]*\$").unwrap()
});

/// bashrs#677: an array whose whole value is one bare command substitution,
/// `array=( $(cmd) )`. ShellCheck 0.8.0 reports only SC2207 for that shape, so
/// it is blanked out before the SC2206 match. Other elements on the line stay.
static BARE_ARRAY_CMDSUB: std::sync::LazyLock<Regex> =
    std::sync::LazyLock::new(|| Regex::new(r"\w+\s*=\s*\(\s*\$\([^)]+\)\s*\)").unwrap());

pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();
    for (line_num, line) in source.lines().enumerate() {
        let line_num = line_num + 1;
        if line.trim_start().starts_with('#') {
            continue;
        }

        let unquoted_part = BARE_ARRAY_CMDSUB.replace_all(line, " ");
        if UNQUOTED_ARRAY_EXPANSION.is_match(&unquoted_part) {
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
    // bashrs#677: a bare command-substitution value is SC2207's, not SC2206's.
    #[test]
    fn test_sc2206_bare_cmdsub_in_array_is_sc2207_only() {
        let code = r#"array=($(find . -name "*.txt"))"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
        let code = r#"declare -a arr=($(ls))"#;
        assert_eq!(check(code).diagnostics.len(), 0);
    }
    #[test]
    fn test_sc2206_cmdsub_plus_variable_still_flagged() {
        let code = r#"arr=($(ls) $extra)"#;
        assert_eq!(check(code).diagnostics.len(), 1);
        let code = r#"x=($(ls)); y=($v)"#;
        assert_eq!(check(code).diagnostics.len(), 1);
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
