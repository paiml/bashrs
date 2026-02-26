// SC2145: Argument mixin in arrays - $@ inside quotes with other text.
//
// When using $@ inside double quotes alongside other text, the array elements
// join with spaces which is often unintended. Use "$*" for explicit concatenation
// or "$@" alone for separate arguments.
//
// NOTE: Using $* inside quotes IS the correct way to concatenate arguments.
// This rule only warns about $@ inside quotes (which should be $* for concatenation).
//
// Examples:
// Bad:
//   echo "Args: $@"              // Elements concatenate with spaces (use $*)
//   msg="Files: $@"               // Array elements join incorrectly
//
// Good:
//   echo "Args: $*"               // Explicit concatenation - CORRECT
//   log_info() { echo "$*"; }     // $* for log functions - CORRECT
//   printf '%s\n' "$@"            // "$@" alone for separate args
//   for arg in "$@"; do           // Proper iteration
//
// Impact: Incorrect argument handling

use crate::linter::{Diagnostic, LintResult, Severity, Span};
use regex::Regex;

static UNQUOTED_AT_IN_QUOTES: std::sync::LazyLock<Regex> = std::sync::LazyLock::new(|| {
    // Match: "...$@..." (unquoted $@ inside double quotes with other content)
    // Look for $@ that's NOT immediately preceded by opening quote or space-quote
    Regex::new(r#""[^"]*\$@[^"]*""#).unwrap()
});

pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();

    for (line_num, line) in source.lines().enumerate() {
        let line_num = line_num + 1;

        if line.trim_start().starts_with('#') {
            continue;
        }

        // Check for $@ in double quotes mixed with other text
        // Using $* in this context is CORRECT (concatenation), so we don't warn about it
        for mat in UNQUOTED_AT_IN_QUOTES.find_iter(line) {
            let matched = mat.as_str();

            // Skip if it's properly quoted: "$@" (the entire quoted string is just "$@")
            if matched == r#""$@""# {
                continue;
            }

            let start_col = mat.start() + 1;
            let end_col = mat.end() + 1;

            let diagnostic = Diagnostic::new(
                "SC2145",
                Severity::Warning,
                "Argument mixin: Use \"$*\" for concatenation or \"$@\" as separate arguments"
                    .to_string(),
                Span::new(line_num, start_col, line_num, end_col),
            );

            result.add(diagnostic);
        }

        // NOTE: We do NOT warn about $* inside quotes - that IS the correct usage
        // for concatenation. The previous implementation was incorrect.
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sc2145_unquoted_at_in_quotes() {
        let code = r#"echo "Args: $@""#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
        assert!(result.diagnostics[0].message.contains("Argument mixin"));
    }

    #[test]
    fn test_sc2145_star_in_quotes_is_correct() {
        // $* inside quotes is the CORRECT way to concatenate arguments
        // This should NOT trigger a warning (Issue #129)
        let code = r#"echo "All: $*""#;
        let result = check(code);
        assert_eq!(
            result.diagnostics.len(),
            0,
            "$* in quotes is correct for concatenation"
        );
    }

    #[test]
    fn test_sc2145_log_function_with_star() {
        // Common log function pattern - $* is correct here (Issue #129)
        let code = r#"log_info() { echo -e "${GREEN}[INFO]${NC} $*"; }"#;
        let result = check(code);
        assert_eq!(
            result.diagnostics.len(),
            0,
            "$* in log functions is correct"
        );
    }

    #[test]
    fn test_sc2145_quoted_at_ok() {
        let code = r#"printf '%s\n' "$@""#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2145_quoted_star_ok() {
        let code = r#"echo "$*""#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2145_unquoted_at_ok() {
        let code = r#"for arg in $@; do"#;
        let result = check(code);
        // Unquoted outside of quotes is a different issue (SC2068)
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2145_in_assignment() {
        let code = r#"msg="Files: $@""#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2145_comment_ok() {
        let code = r#"# echo "Args: $@""#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2145_multiple_at_only() {
        // Only $@ in quotes should warn, not $*
        let code = r#"
echo "Args: $@"
msg="All: $*"
"#;
        let result = check(code);
        // Only the $@ line should warn (line 2), not the $* line
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2145_at_beginning() {
        let code = r#"echo "$@ are the args""#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2145_at_end() {
        let code = r#"echo "Arguments: $@""#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2145_star_concatenation_multiple() {
        // All of these use $* correctly for concatenation
        let code = r#"
log_info() { echo "[INFO] $*"; }
log_warn() { echo "[WARN] $*" >&2; }
log_error() { echo "[ERROR] $*" >&2; }
"#;
        let result = check(code);
        assert_eq!(
            result.diagnostics.len(),
            0,
            "All $* concatenations should be allowed"
        );
    }
}
