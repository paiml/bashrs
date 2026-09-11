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
use regex::Regex;

#[allow(clippy::unwrap_used)] // Compile-time regex, panic on invalid pattern is acceptable
static UNSAFE_COMMAND: std::sync::LazyLock<Regex> = std::sync::LazyLock::new(|| {
    // Match commands that take file arguments
    Regex::new(r"^(?:.*\s+)?(rm|cat|grep|ls|mv|cp|chmod|chown|find|xargs|echo)\b").unwrap()
});

#[allow(clippy::expect_used)] // Compile-time regex, panic on invalid pattern is acceptable
static BARE_GLOB: std::sync::LazyLock<Regex> = std::sync::LazyLock::new(|| {
    // Match bare globs (*.ext) that aren't prefixed with ./ or / or $
    Regex::new(r"\*\.[a-zA-Z0-9]+\b").expect("valid bare glob regex")
});

/// Issue #96: Regex to detect find pattern arguments that are quoted
/// Matches: -name 'pattern', -iname "pattern", -path 'pattern'
#[allow(clippy::expect_used)] // Compile-time regex, panic on invalid pattern is acceptable
static FIND_PATTERN_ARG: std::sync::LazyLock<Regex> = std::sync::LazyLock::new(|| {
    Regex::new(r#"-(name|iname|path)\s+['"]([^'"]+)['"]"#).expect("valid find pattern regex")
});

/// Issue #104: Regex to detect grep/egrep/fgrep pattern arguments that are quoted
/// Matches: grep 'pattern', grep -e 'pattern', grep -E 'pattern', egrep 'pattern'
#[allow(clippy::expect_used)] // Compile-time regex, panic on invalid pattern is acceptable
static GREP_PATTERN_ARG: std::sync::LazyLock<Regex> = std::sync::LazyLock::new(|| {
    // Match grep/egrep/fgrep followed by optional flags then a quoted pattern
    // Pattern: (e|f)?grep ... ['"]pattern['"]
    Regex::new(r#"\b[ef]?grep\s+(?:-[a-zA-Z0-9]+\s+)*['"]([^'"]+)['"]"#)
        .expect("valid grep pattern regex")
});

/// FP018: Regex to detect stderr redirect to /dev/null
/// Matches: 2>/dev/null, 2> /dev/null, &>/dev/null, &> /dev/null
#[allow(clippy::expect_used)] // Compile-time regex, panic on invalid pattern is acceptable
static STDERR_REDIRECT_DEVNULL: std::sync::LazyLock<Regex> = std::sync::LazyLock::new(|| {
    Regex::new(r"(?:2|&)>\s*/dev/null").expect("valid stderr redirect regex")
});

/// Check if glob is safe (prefixed with ./ or / or $)
fn is_glob_safe(line: &str, glob_start: usize) -> bool {
    if glob_start == 0 {
        return false;
    }

    let before = &line[..glob_start];
    before.ends_with("./") || before.ends_with('/') || before.ends_with('$')
}

/// GH-311: A glob character sequence inside a quoted word is never expanded
/// by the shell — it reaches the command as a literal string (e.g. a git
/// pathspec that git itself expands). SC2035 must not fire on it.
fn is_inside_quotes(line: &str, pos: usize) -> bool {
    let before = &line[..pos];
    let quote_count = before.matches('"').count() + before.matches('\'').count();
    quote_count % 2 == 1
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

/// Decide whether a bare-glob match at `[glob_start, glob_end)` on `line`
/// should be skipped (i.e. it is not actually a shell glob that will be
/// expanded unquoted).
fn should_skip_glob(line: &str, glob_start: usize, glob_end: usize) -> bool {
    // Skip if glob is safe (prefixed with ./ or / or $)
    if is_glob_safe(line, glob_start) {
        return true;
    }

    // GH-311: Skip if the glob is inside a quoted word — the shell never
    // globs quoted text, regardless of which command receives it.
    if is_inside_quotes(line, glob_start) {
        return true;
    }

    // Issue #96: Skip if glob is inside a quoted find -name/-iname/-path argument
    if is_inside_find_pattern(line, glob_start, glob_end) {
        return true;
    }

    // Issue #104: Skip if glob is inside a quoted grep pattern argument
    is_inside_grep_pattern(line, glob_start, glob_end)
}

/// Find all bare globs on `line` and add a diagnostic for each unsafe one.
fn check_globs_on_line(line: &str, line_num: usize, result: &mut LintResult) {
    for mat in BARE_GLOB.find_iter(line) {
        let glob_start = mat.start();
        let glob_end = mat.end();

        if should_skip_glob(line, glob_start, glob_end) {
            continue;
        }

        result.add(create_unsafe_glob_diagnostic(
            glob_start, glob_end, line_num,
        ));
    }
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

        check_globs_on_line(line, line_num, &mut result);
    }

    result
}

#[cfg(test)]
#[path = "sc2035_tests_sc2035_rm.rs"]
mod tests_extracted;

#[cfg(test)]
mod pmat257_gh311_tests {
    use super::*;

    /// GH-311: `git ls-files '*.sh'` — the glob is single-quoted, so the shell
    /// never expands it; it reaches git as a literal pathspec that git itself
    /// expands. SC2035 must not fire on a quoted word.
    #[test]
    fn test_PMAT257_gh311_sc2035_quoted_pathspec_is_not_a_shell_glob() {
        let code = r#"git ls-files '*.sh'"#;
        let result = check(code);
        assert_eq!(
            result.diagnostics.len(),
            0,
            "SC2035 must NOT flag a quoted pathspec passed to git ls-files: {:?}",
            result.diagnostics
        );
    }

    /// Companion: a genuinely unquoted glob is still a real shell glob and
    /// dash-prone filenames can still be misread as options.
    #[test]
    fn test_PMAT257_gh311_sc2035_unquoted_glob_still_fires() {
        let code = r#"rm *.txt"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
        assert_eq!(result.diagnostics[0].code, "SC2035");
    }
}
