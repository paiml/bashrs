// SC2035: Use ./* so names with dashes won't become options
//
// When using globs like *.txt with commands, files starting with dashes
// can be interpreted as options. Using ./*.txt prevents this.
//
// Examples:
// Bad:
//   rm *.txt           # File named "-rf.txt" would expand to "rm -rf.txt"
//   cat *.log          # File named "-n.log" would be treated as option
//   grep pattern *.sh  # File named "-v.sh" could cause issues
//
// Good:
//   rm ./*.txt         # Safe: "./-rf.txt" is clearly a file
//   cat ./*.log        # Safe: files won't be treated as options
//   grep pattern ./*.sh

use crate::linter::{Diagnostic, LintResult, Severity, Span};
use once_cell::sync::Lazy;
use regex::Regex;

#[allow(clippy::unwrap_used)] // Compile-time regex, panic on invalid pattern is acceptable
static UNSAFE_COMMAND: Lazy<Regex> = Lazy::new(|| {
    // Match commands that take file arguments
    Regex::new(r"^(?:.*\s+)?(rm|cat|grep|ls|mv|cp|chmod|chown|find|xargs|echo)\b").unwrap()
});

#[allow(clippy::expect_used)] // Compile-time regex, panic on invalid pattern is acceptable
static BARE_GLOB: Lazy<Regex> = Lazy::new(|| {
    // Match bare globs (*.ext) that aren't prefixed with ./ or / or $
    Regex::new(r"\*\.[a-zA-Z0-9]+\b").expect("valid bare glob regex")
});

/// Issue #96: Regex to detect find pattern arguments that are quoted
/// Matches: -name 'pattern', -iname "pattern", -path 'pattern'
#[allow(clippy::expect_used)] // Compile-time regex, panic on invalid pattern is acceptable
static FIND_PATTERN_ARG: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r#"-(name|iname|path)\s+['"]([^'"]+)['"]"#).expect("valid find pattern regex")
});

/// Issue #104: Regex to detect grep/egrep/fgrep pattern arguments that are quoted
/// Matches: grep 'pattern', grep -e 'pattern', grep -E 'pattern', egrep 'pattern'
#[allow(clippy::expect_used)] // Compile-time regex, panic on invalid pattern is acceptable
static GREP_PATTERN_ARG: Lazy<Regex> = Lazy::new(|| {
    // Match grep/egrep/fgrep followed by optional flags then a quoted pattern
    // Pattern: (e|f)?grep ... ['"]pattern['"]
    Regex::new(r#"\b[ef]?grep\s+(?:-[a-zA-Z0-9]+\s+)*['"]([^'"]+)['"]"#)
        .expect("valid grep pattern regex")
});

/// FP018: Regex to detect stderr redirect to /dev/null
/// Matches: 2>/dev/null, 2> /dev/null, &>/dev/null, &> /dev/null
#[allow(clippy::expect_used)] // Compile-time regex, panic on invalid pattern is acceptable
static STDERR_REDIRECT_DEVNULL: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"(?:2|&)>\s*/dev/null").expect("valid stderr redirect regex"));

/// Check if glob is safe (prefixed with ./ or / or $)
fn is_glob_safe(line: &str, glob_start: usize) -> bool {
    if glob_start == 0 {
        return false;
    }

    let before = &line[..glob_start];
    before.ends_with("./") || before.ends_with('/') || before.ends_with('$')
}

/// Issue #96: Check if glob position is inside a quoted find -name/-iname/-path argument
/// These patterns are for find, not shell expansion, so they're safe when quoted
fn is_inside_find_pattern(line: &str, glob_start: usize, glob_end: usize) -> bool {
    // Check if this line has a find command with quoted pattern arguments
    for cap in FIND_PATTERN_ARG.captures_iter(line) {
        if let Some(pattern_match) = cap.get(2) {
            // Check if the glob falls within this pattern match
            if glob_start >= pattern_match.start() && glob_end <= pattern_match.end() {
                return true;
            }
        }
    }
    false
}

/// Issue #104: Check if glob position is inside a quoted grep pattern argument
/// These patterns are regex patterns, not shell globs, so they're safe when quoted
fn is_inside_grep_pattern(line: &str, glob_start: usize, glob_end: usize) -> bool {
    // Check if this line has a grep command with quoted pattern arguments
    for cap in GREP_PATTERN_ARG.captures_iter(line) {
        if let Some(pattern_match) = cap.get(1) {
            // Check if the glob falls within this pattern match
            if glob_start >= pattern_match.start() && glob_end <= pattern_match.end() {
                return true;
            }
        }
    }
    false
}

/// FP018: Check if stderr is redirected to /dev/null
/// When stderr is redirected, user is handling the "no files match" case
fn has_stderr_redirect_to_devnull(line: &str) -> bool {
    STDERR_REDIRECT_DEVNULL.is_match(line)
}

/// Create diagnostic for unsafe glob pattern
fn create_unsafe_glob_diagnostic(
    glob_start: usize,
    glob_end: usize,
    line_num: usize,
) -> Diagnostic {
    let start_col = glob_start + 1;
    let end_col = glob_end + 1;

    Diagnostic::new(
        "SC2035",
        Severity::Warning,
        "Use ./* so names with dashes won't become options. Example: rm ./*.txt instead of rm *.txt",
        Span::new(line_num, start_col, line_num, end_col),
    )
}

/// Check if line should be processed (has unsafe command and not a comment)
fn should_check_line(line: &str) -> bool {
    !line.trim_start().starts_with('#') && UNSAFE_COMMAND.is_match(line)
}

pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();

    for (line_num, line) in source.lines().enumerate() {
        let line_num = line_num + 1;

        if !should_check_line(line) {
            continue;
        }

        // FP018: Skip if stderr is redirected to /dev/null
        // User is already handling the "no files match" case
        if has_stderr_redirect_to_devnull(line) {
            continue;
        }

        // Find all bare globs on this line
        for mat in BARE_GLOB.find_iter(line) {
            let glob_start = mat.start();
            let glob_end = mat.end();

            // Skip if glob is safe (prefixed with ./ or / or $)
            if is_glob_safe(line, glob_start) {
                continue;
            }

            // Issue #96: Skip if glob is inside a quoted find -name/-iname/-path argument
            if is_inside_find_pattern(line, glob_start, glob_end) {
                continue;
            }

            // Issue #104: Skip if glob is inside a quoted grep pattern argument
            if is_inside_grep_pattern(line, glob_start, glob_end) {
                continue;
            }

            let diagnostic = create_unsafe_glob_diagnostic(glob_start, glob_end, line_num);
            result.add(diagnostic);
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sc2035_rm_glob() {
        let code = r#"rm *.txt"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
        assert_eq!(result.diagnostics[0].code, "SC2035");
        assert_eq!(result.diagnostics[0].severity, Severity::Warning);
        assert!(result.diagnostics[0].message.contains("./*"));
    }

    #[test]
    fn test_sc2035_cat_glob() {
        let code = r#"cat *.log"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2035_grep_glob() {
        let code = r#"grep pattern *.sh"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2035_safe_dotslash_ok() {
        let code = r#"rm ./*.txt"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2035_absolute_path_ok() {
        let code = r#"rm /tmp/*.txt"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2035_variable_path_ok() {
        let code = r#"rm "$dir"/*.txt"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2035_multiple_globs() {
        let code = r#"rm *.txt *.log"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 2);
    }

    #[test]
    fn test_sc2035_mv_glob() {
        let code = r#"mv *.bak /backup/"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2035_chmod_glob() {
        let code = r#"chmod 644 *.conf"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2035_no_glob_ok() {
        let code = r#"rm file.txt"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    // ===== Issue #96: find -name pattern tests =====
    // Patterns after find -name/-iname/-path are for find, not shell expansion

    #[test]
    fn test_FP_096_find_name_not_flagged() {
        let code = r#"find . -name '*.json'"#;
        let result = check(code);
        assert_eq!(
            result.diagnostics.len(),
            0,
            "SC2035 must NOT flag find -name patterns"
        );
    }

    #[test]
    fn test_FP_096_find_iname_not_flagged() {
        let code = r#"find /tmp -iname '*.TXT'"#;
        let result = check(code);
        assert_eq!(
            result.diagnostics.len(),
            0,
            "SC2035 must NOT flag find -iname patterns"
        );
    }

    #[test]
    fn test_FP_096_find_path_not_flagged() {
        let code = r#"find . -path '*.log'"#;
        let result = check(code);
        assert_eq!(
            result.diagnostics.len(),
            0,
            "SC2035 must NOT flag find -path patterns"
        );
    }

    #[test]
    fn test_FP_096_find_name_unquoted_still_flagged() {
        // Unquoted glob after find -name IS dangerous (shell expands before find sees it)
        let code = r#"find . -name *.json"#;
        let result = check(code);
        assert_eq!(
            result.diagnostics.len(),
            1,
            "SC2035 SHOULD flag unquoted find -name patterns"
        );
    }

    #[test]
    fn test_FP_096_find_name_double_quoted_not_flagged() {
        let code = r#"find . -name "*.json""#;
        let result = check(code);
        assert_eq!(
            result.diagnostics.len(),
            0,
            "SC2035 must NOT flag double-quoted find -name patterns"
        );
    }

    // ===== Issue #104: grep pattern tests =====
    // Patterns after grep are regex patterns, not shell globs

    #[test]
    fn test_FP_104_grep_quoted_pattern_not_flagged() {
        let code = r#"grep -r '*.c' ."#;
        let result = check(code);
        assert_eq!(
            result.diagnostics.len(),
            0,
            "SC2035 must NOT flag quoted grep patterns"
        );
    }

    #[test]
    fn test_FP_104_grep_e_pattern_not_flagged() {
        let code = r#"grep -e '*.log' files.txt"#;
        let result = check(code);
        assert_eq!(
            result.diagnostics.len(),
            0,
            "SC2035 must NOT flag grep -e patterns"
        );
    }

    #[test]
    fn test_FP_104_grep_E_pattern_not_flagged() {
        let code = r#"grep -E '.*\.txt' ."#;
        let result = check(code);
        assert_eq!(
            result.diagnostics.len(),
            0,
            "SC2035 must NOT flag grep -E patterns"
        );
    }

    #[test]
    fn test_FP_104_egrep_pattern_not_flagged() {
        let code = r#"egrep '*.json' file"#;
        let result = check(code);
        assert_eq!(
            result.diagnostics.len(),
            0,
            "SC2035 must NOT flag egrep patterns"
        );
    }

    #[test]
    fn test_FP_104_grep_unquoted_still_flagged() {
        // Unquoted glob after grep IS a shell glob
        let code = r#"grep pattern *.txt"#;
        let result = check(code);
        assert_eq!(
            result.diagnostics.len(),
            1,
            "SC2035 SHOULD flag unquoted globs as file args to grep"
        );
    }

    // ===== FP018: Stderr redirect handling =====
    // When stderr is redirected to /dev/null, user is handling "no match" case

    #[test]
    fn test_FP018_glob_with_stderr_redirect_not_flagged() {
        // User redirects stderr - they're handling "no files match" scenario
        let code = r#"ls *.txt 2>/dev/null"#;
        let result = check(code);
        assert_eq!(
            result.diagnostics.len(),
            0,
            "SC2035 must NOT flag globs when stderr is redirected to /dev/null"
        );
    }

    #[test]
    fn test_FP018_glob_with_stderr_redirect_space_not_flagged() {
        let code = r#"ls *.txt 2> /dev/null"#;
        let result = check(code);
        assert_eq!(
            result.diagnostics.len(),
            0,
            "SC2035 must NOT flag globs with spaced stderr redirect"
        );
    }

    #[test]
    fn test_FP018_glob_with_stderr_ampersand_redirect_not_flagged() {
        // &>/dev/null redirects both stdout and stderr
        let code = r#"ls *.txt &>/dev/null"#;
        let result = check(code);
        assert_eq!(
            result.diagnostics.len(),
            0,
            "SC2035 must NOT flag globs with &>/dev/null"
        );
    }

    #[test]
    fn test_FP018_glob_with_stderr_redirect_or_not_flagged() {
        // cmd || true also handles errors
        let code = r#"ls *.txt 2>/dev/null || true"#;
        let result = check(code);
        assert_eq!(
            result.diagnostics.len(),
            0,
            "SC2035 must NOT flag globs with stderr redirect and || true"
        );
    }

    #[test]
    fn test_FP018_glob_without_redirect_still_flagged() {
        // Without redirect, should still flag
        let code = r#"ls *.txt"#;
        let result = check(code);
        assert_eq!(
            result.diagnostics.len(),
            1,
            "SC2035 SHOULD flag globs without error handling"
        );
    }
}

#[cfg(test)]
mod property_tests {
    use super::*;
    use proptest::prelude::*;

    // Property: Any command with 2>/dev/null should NOT trigger SC2035
    proptest! {
        #![proptest_config(proptest::test_runner::Config::with_cases(10))]
        #[test]
        fn prop_stderr_redirect_never_flags(
            cmd in "(ls|cat|rm|mv|cp|chmod|grep)",
            ext in "[a-z]{1,4}"
        ) {
            let code = format!("{} *.{} 2>/dev/null", cmd, ext);
            let result = check(&code);
            prop_assert!(
                result.diagnostics.is_empty(),
                "SC2035 must NOT flag with stderr redirect: {}",
                code
            );
        }

        #[test]
        fn prop_ampersand_redirect_never_flags(
            cmd in "(ls|cat|rm|mv|cp|chmod|grep)",
            ext in "[a-z]{1,4}"
        ) {
            let code = format!("{} *.{} &>/dev/null", cmd, ext);
            let result = check(&code);
            prop_assert!(
                result.diagnostics.is_empty(),
                "SC2035 must NOT flag with &>/dev/null: {}",
                code
            );
        }

        #[test]
        fn prop_stderr_redirect_with_space_never_flags(
            cmd in "(ls|cat|rm|mv|cp|chmod|grep)",
            ext in "[a-z]{1,4}"
        ) {
            let code = format!("{} *.{} 2> /dev/null", cmd, ext);
            let result = check(&code);
            prop_assert!(
                result.diagnostics.is_empty(),
                "SC2035 must NOT flag with spaced stderr redirect: {}",
                code
            );
        }

        #[test]
        fn prop_no_redirect_still_flags(
            cmd in "(ls|cat|rm|mv|cp|chmod|grep)",
            ext in "[a-z]{1,4}"
        ) {
            let code = format!("{} *.{}", cmd, ext);
            let result = check(&code);
            prop_assert!(
                !result.diagnostics.is_empty(),
                "SC2035 SHOULD flag without redirect: {}",
                code
            );
        }

        #[test]
        fn prop_safe_prefix_never_flags(
            cmd in "(ls|cat|rm|mv|cp|chmod|grep)",
            ext in "[a-z]{1,4}"
        ) {
            let code = format!("{} ./*.{}", cmd, ext);
            let result = check(&code);
            prop_assert!(
                result.diagnostics.is_empty(),
                "SC2035 must NOT flag with ./ prefix: {}",
                code
            );
        }
    }
}
