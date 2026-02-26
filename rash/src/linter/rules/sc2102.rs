// SC2102: Ranges can only match single characters (mentioned using * or +)
//
// Character ranges in shell patterns match single characters, not strings.
// [0-9]+ doesn't work in shell patterns (it's regex syntax).
//
// Examples:
// Bad:
//   [[ $var = [0-9]+ ]]          // + doesn't work in shell patterns
//   case $x in [a-z]*+) ;;       // Invalid syntax
//
// Good:
//   [[ $var =~ [0-9]+ ]]         // Use =~ for regex
//   [[ $var = [0-9]* ]]          // * is OK for glob
//
// Impact: Pattern doesn't match as expected

use crate::linter::{Diagnostic, LintResult, Severity, Span};
use regex::Regex;

static RANGE_WITH_PLUS: std::sync::LazyLock<Regex> = std::sync::LazyLock::new(|| {
    // Match: [range]+ or [[:posix:]]+ in glob context (not =~)
    // Handles both simple ranges [0-9]+ and POSIX classes [[:digit:]]+
    Regex::new(r"\[(?:[^\]]|\[:.*?:\])+\]\+").unwrap()
});

/// Check if a flag argument contains an ERE flag (E or P)
/// Handles combined flags like -oE, -cE, -PE, etc.
fn has_ere_flag(arg: &str) -> bool {
    if !arg.starts_with('-') || arg.starts_with("--") {
        // Long options are handled separately
        return false;
    }
    // Check if any character in the flag is E or P (for -E, -P, -oE, -cE, etc.)
    arg.chars().skip(1).any(|c| c == 'E' || c == 'P')
}

/// Check if line uses extended regex (ERE) context where + is valid
fn is_ere_context(line: &str) -> bool {
    // Bash regex match operator
    if line.contains("=~") {
        return true;
    }

    // Issue #92: grep with ERE flags (-E, -P, --extended-regexp, --perl-regexp)
    // Common patterns: grep -E, grep -oE, grep -cE, egrep, etc.
    if line.contains("grep") {
        // Check for long options first
        if line.contains("--extended-regexp") || line.contains("--perl-regexp") {
            return true;
        }
        // Check for short flags (including combined flags like -oE)
        for word in line.split_whitespace() {
            if has_ere_flag(word) {
                return true;
            }
        }
    }

    // egrep is extended regex by default
    if line.contains("egrep") {
        return true;
    }

    // sed with ERE (-E or -r flag)
    if line.contains("sed") {
        for word in line.split_whitespace() {
            if word.starts_with('-')
                && !word.starts_with("--")
                && (word.contains('E') || word.contains('r'))
            {
                return true;
            }
        }
    }

    // awk uses ERE by default
    if line.contains("awk") || line.contains("gawk") {
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

        // Issue #92: Skip ERE contexts where + is a valid quantifier
        if is_ere_context(line) {
            continue;
        }

        for mat in RANGE_WITH_PLUS.find_iter(line) {
            let start_col = mat.start() + 1;
            let end_col = mat.end() + 1;

            let diagnostic = Diagnostic::new(
                "SC2102",
                Severity::Warning,
                "Ranges can only match single chars (to match + literally, use \\+)".to_string(),
                Span::new(line_num, start_col, line_num, end_col),
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
    fn test_sc2102_range_plus() {
        let code = "[[ $var = [0-9]+ ]]";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2102_case_range_plus() {
        let code = "case $x in [a-z]+) ;;";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2102_regex_ok() {
        let code = "[[ $var =~ [0-9]+ ]]";
        let result = check(code);
        // =~ uses regex, + is valid
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2102_glob_star_ok() {
        let code = "[[ $var = [0-9]* ]]";
        let result = check(code);
        // * is valid in globs
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2102_comment_ok() {
        let code = "# [[ $var = [0-9]+ ]]";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2102_literal_plus() {
        let code = "[[ $var = [0-9]\\+ ]]";
        let result = check(code);
        // Escaped + is literal
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2102_multiple_ranges() {
        let code = "case $x in [a-z]+|[0-9]+) ;;";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 2);
    }

    #[test]
    fn test_sc2102_in_test() {
        let code = "[ \"$var\" = [0-9]+ ]";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2102_posix_class() {
        let code = "[[ $var = [[:digit:]]+ ]]";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2102_find_name() {
        let code = "find . -name \"[0-9]+\"";
        let result = check(code);
        // In find -name patterns
        assert_eq!(result.diagnostics.len(), 1);
    }

    // Issue #92: SC2102 should NOT flag + in ERE contexts
    #[test]
    fn test_sc2102_issue_92_grep_e_flag() {
        // From issue #92 reproduction case
        let code = r#"grep -oE 'error\[E[0-9]+\]' "$COMPILE_LOG""#;
        let result = check(code);
        assert_eq!(
            result.diagnostics.len(),
            0,
            "SC2102 must NOT flag [0-9]+ in grep -E (ERE context)"
        );
    }

    #[test]
    fn test_sc2102_issue_92_grep_extended_regexp() {
        let code = "grep --extended-regexp '[0-9]+' file.txt";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2102_issue_92_grep_p_flag() {
        let code = "grep -P '[0-9]+' file.txt";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2102_issue_92_egrep() {
        let code = "egrep '[0-9]+' file.txt";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2102_issue_92_sed_e_flag() {
        let code = "sed -E 's/[0-9]+/X/g' file.txt";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2102_issue_92_awk() {
        let code = "awk '/[0-9]+/ { print }' file.txt";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2102_issue_92_basic_grep_still_flagged() {
        // Basic grep without -E should still flag
        let code = "grep '[0-9]+' file.txt";
        let result = check(code);
        assert_eq!(
            result.diagnostics.len(),
            1,
            "Basic grep (BRE) should still flag + quantifier"
        );
    }
}
