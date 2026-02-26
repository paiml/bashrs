// SC2281: Don't use $@ in double quotes for string concatenation
// Issue #122: But "$@" is perfectly valid when passing arguments to commands
use crate::linter::{Diagnostic, LintResult, Severity, Span};
use regex::Regex;

// Match assignment patterns: var="..." or var+="..."
static ASSIGNMENT_WITH_QUOTED_AT: std::sync::LazyLock<Regex> = std::sync::LazyLock::new(|| {
    Regex::new(r#"[a-zA-Z_][a-zA-Z0-9_]*\+?="[^"]*\$@[^"]*""#).unwrap()
});

// Match concatenation patterns: "$prefix$@" or "$@suffix" or "text $@ text"
// But NOT: command "$@" (valid) or func "$@" (valid)
static CONCAT_QUOTED_AT: std::sync::LazyLock<Regex> = std::sync::LazyLock::new(|| {
    // Pattern matches "$@" with adjacent content (not just "$@" alone)
    Regex::new(r#""([^"]+\$@|\$@[^"]+)""#).unwrap()
});

/// Issue #122: Check if line is a function call or command with "$@"
/// cmd "$@" and func "$@" are perfectly valid - they pass all arguments
fn is_valid_args_pass(line: &str) -> bool {
    // Skip if "$@" is used as arguments to a command (no concatenation)
    // This matches: cmd "$@", func "$@", etc.
    let trimmed = line.trim();

    // Pattern: command/function call with "$@" as arguments
    // Valid: cmd "$@", echo "$@", func "$@"
    // Invalid: var="$@", str="prefix$@", str="$@suffix"
    if trimmed.contains(r#""$@""#) && !trimmed.contains('=') {
        return true;
    }

    // Check if it's in a case statement pattern line
    if trimmed.contains("case") || trimmed.contains("esac") || trimmed.ends_with(')') {
        return true;
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

        // Issue #122: Skip valid uses of "$@" as command arguments
        if is_valid_args_pass(line) {
            continue;
        }

        // Check for assignment with "$@" - this is the main problematic case
        if ASSIGNMENT_WITH_QUOTED_AT.is_match(line) {
            let diagnostic = Diagnostic::new(
                "SC2281",
                Severity::Warning,
                r#"Use "$*" or ${array[*]} for string concatenation, or ${array[@]} for separate elements"#.to_string(),
                Span::new(line_num, 1, line_num, line.len() + 1),
            );
            result.add(diagnostic);
            continue;
        }

        // Check for concatenation patterns
        if CONCAT_QUOTED_AT.is_match(line) && !is_valid_args_pass(line) {
            let diagnostic = Diagnostic::new(
                "SC2281",
                Severity::Warning,
                r#"Use "$*" or ${array[*]} for string concatenation, or ${array[@]} for separate elements"#.to_string(),
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
    fn test_sc2281_quoted_at() {
        let code = r#"msg="$@""#;
        assert_eq!(check(code).diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2281_quoted_star_ok() {
        let code = r#"msg="$*""#;
        assert_eq!(check(code).diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2281_unquoted_at_ok() {
        let code = r#"cmd $@"#;
        assert_eq!(check(code).diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2281_comment() {
        let code = r#"# "$@""#;
        assert_eq!(check(code).diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2281_empty() {
        assert_eq!(check("").diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2281_normal() {
        assert_eq!(check("echo test").diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2281_concatenation_in_echo() {
        // Concatenation with prefix/suffix is problematic
        let code = r#"echo "Args: $@""#;
        assert_eq!(check(code).diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2281_array_ok() {
        let code = r#"echo "${array[@]}""#;
        assert_eq!(check(code).diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2281_concatenation() {
        let code = r#"str="prefix $@ suffix""#;
        assert_eq!(check(code).diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2281_assignment() {
        let code = r#"all_args="$@""#;
        assert_eq!(check(code).diagnostics.len(), 1);
    }

    // Issue #122: Valid uses of "$@" should NOT be flagged
    #[test]
    fn test_issue_122_function_call_not_flagged() {
        // Passing "$@" to a function/command is valid
        let code = r#"my_function "$@""#;
        assert_eq!(
            check(code).diagnostics.len(),
            0,
            "SC2281 must NOT flag \"$@\" passed to function"
        );
    }

    #[test]
    fn test_issue_122_command_args_not_flagged() {
        let code = r#"exec "$@""#;
        assert_eq!(
            check(code).diagnostics.len(),
            0,
            "SC2281 must NOT flag exec \"$@\""
        );
    }

    #[test]
    fn test_issue_122_echo_args_not_flagged() {
        // echo "$@" is valid - it prints all arguments
        let code = r#"echo "$@""#;
        assert_eq!(
            check(code).diagnostics.len(),
            0,
            "SC2281 must NOT flag echo \"$@\""
        );
    }

    #[test]
    fn test_issue_122_case_pattern_not_flagged() {
        let code = r#"case "$@" in *) ;; esac"#;
        assert_eq!(
            check(code).diagnostics.len(),
            0,
            "SC2281 must NOT flag \"$@\" in case statement"
        );
    }

    #[test]
    fn test_issue_122_assignment_still_flagged() {
        // Assignment with "$@" is still problematic
        let code = r#"args="$@""#;
        assert_eq!(
            check(code).diagnostics.len(),
            1,
            "SC2281 should still flag assignment with \"$@\""
        );
    }
}
