//! SC1007: Remove space after = in variable assignment
//!
//! Detects `VAR = value` patterns where spaces surround the `=` in what
//! looks like a variable assignment. In shell, `VAR = value` runs `VAR`
//! as a command with `=` and `value` as arguments.
//!
//! # Examples
//!
//! Bad:
//! ```bash
//! FOO = bar
//! MY_VAR = "hello"
//! ```
//!
//! Good:
//! ```bash
//! FOO=bar
//! MY_VAR="hello"
//! ```

use crate::linter::{Diagnostic, LintResult, Severity, Span};

/// Identifiers that are commands, not assignment targets.
const COMMAND_WORDS: &[&str] = &[
    "echo", "printf", "return", "exit", "export", "local", "readonly",
];

/// Lines that are test or conditional context, where `=` is a comparison.
fn is_test_context(trimmed: &str) -> bool {
    trimmed.starts_with('[')
        || trimmed.starts_with("if ")
        || trimmed.starts_with("while ")
        || trimmed.starts_with("until ")
        || trimmed.starts_with("elif ")
        || trimmed.starts_with("test ")
        || trimmed.contains("[ ")
        || trimmed.contains("[[ ")
        || trimmed.contains("==")
}

/// The 0-indexed offset of the `=` in a spaced assignment on `trimmed`, if the
/// line is one. `None` when the line is not an assignment worth reporting.
fn spaced_assignment(trimmed: &str) -> Option<usize> {
    let bytes = trimmed.as_bytes();

    // A name: alphabetic or `_`, then alphanumerics and `_`.
    let first = *bytes.first()?;
    if !(first.is_ascii_alphabetic() || first == b'_') {
        return None;
    }
    let mut i = 1;
    while i < bytes.len() && (bytes[i].is_ascii_alphanumeric() || bytes[i] == b'_') {
        i += 1;
    }
    let ident_end = i;

    let has_space_before = bytes.get(i) == Some(&b' ');
    while bytes.get(i) == Some(&b' ') {
        i += 1;
    }

    if bytes.get(i) != Some(&b'=') || bytes.get(i + 1) == Some(&b'=') {
        return None;
    }
    let eq_pos = i;
    let has_space_after = bytes.get(i + 1) == Some(&b' ');

    if !has_space_before && !has_space_after {
        return None;
    }

    let ident = &trimmed[..ident_end];
    if COMMAND_WORDS.contains(&ident) {
        return None;
    }
    // `IFS= read -r line` is the documented idiom for reading a line verbatim:
    // the empty value is deliberate, and the space is what separates it from
    // the command it prefixes. shellcheck exempts `IFS` by name for the same
    // reason and reports nothing at any severity.
    if ident == "IFS" && !has_space_before {
        return None;
    }

    Some(eq_pos)
}

pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();
    let mut continued = false;

    for (idx, line) in source.lines().enumerate() {
        let line_num = idx + 1;
        let trimmed = line.trim();

        // A line continued with `\` carries the previous command's arguments,
        // so `NAME=` there is an argument word, not a new assignment. Advanced
        // before the skips below so the flag never stalls.
        let was_continued = continued;
        continued = line.ends_with('\\');

        if was_continued || trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        if is_test_context(trimmed) {
            continue;
        }

        let Some(eq_pos) = spaced_assignment(trimmed) else {
            continue;
        };

        let col = line.find(trimmed).unwrap_or(0) + eq_pos + 1;
        result.add(Diagnostic::new(
            "SC1007",
            Severity::Error,
            "Remove space after = if this is intended as an assignment",
            Span::new(line_num, col, line_num, col + 1),
        ));
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sc1007_space_around_equals() {
        let result = check("FOO = bar");
        assert_eq!(result.diagnostics.len(), 1);
        assert_eq!(result.diagnostics[0].code, "SC1007");
        assert_eq!(result.diagnostics[0].severity, Severity::Error);
    }

    #[test]
    fn test_sc1007_space_after_equals() {
        let result = check("FOO= bar");
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc1007_space_before_equals() {
        let result = check("FOO =bar");
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc1007_no_space_ok() {
        let result = check("FOO=bar");
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc1007_test_context_not_flagged() {
        let result = check("[ $x = $y ]");
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc1007_double_bracket_not_flagged() {
        let result = check("[[ $x = $y ]]");
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc1007_if_context_not_flagged() {
        let result = check("if [ $x = $y ]; then");
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc1007_double_equals_not_flagged() {
        let result = check("x == y");
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc1007_comment_not_flagged() {
        let result = check("# FOO = bar");
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc1007_multiple_assignments() {
        let script = "A = 1\nB = 2\n";
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 2);
    }
}

#[cfg(test)]
mod tests_command_prefix_and_continuation {
    use super::*;

    /// `IFS= read -r x` is the documented idiom for reading a line without
    /// stripping whitespace. shellcheck exempts `IFS` by name and reports
    /// nothing at any severity; bashrs reported an ERROR.
    #[test]
    fn ifs_empty_assignment_is_the_read_idiom() {
        let result = check("IFS= read -r answer || answer=\"\"\n");
        assert_eq!(result.diagnostics.len(), 0, "got {:?}", result.diagnostics);
    }

    #[test]
    fn ifs_prefix_on_any_command_is_fine() {
        let result = check("IFS= cat /etc/hosts\n");
        assert_eq!(result.diagnostics.len(), 0, "got {:?}", result.diagnostics);
    }

    /// A line continued with `\` is not a fresh assignment context: its words
    /// are arguments to the command started on the previous line. shellcheck
    /// reports nothing at any severity here.
    #[test]
    fn continuation_line_words_are_arguments_not_assignments() {
        let script = "run_case \"no workspace vars\" no all \\\n    RUNNER_WORKSPACE= GITHUB_WORKSPACE= RUNNER_WORK_GLOB=\"/tmp/x\"\n";
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 0, "got {:?}", result.diagnostics);
    }

    /// MUST STILL FIRE: a space *before* `=` runs the name as a command. This
    /// is the defect the rule exists for and it stays an error.
    #[test]
    fn still_fires_on_space_before_equals() {
        let result = check("FOO = bar\n");
        assert_eq!(result.diagnostics.len(), 1, "got {:?}", result.diagnostics);
        assert_eq!(result.diagnostics[0].code, "SC1007");
        assert_eq!(result.diagnostics[0].severity, Severity::Error);
    }

    /// MUST STILL FIRE: a non-`IFS` empty assignment followed by a word is
    /// still reported — only the `IFS` idiom is exempt.
    #[test]
    fn still_fires_on_non_ifs_empty_assignment() {
        let result = check("FOO= bar\n");
        assert_eq!(result.diagnostics.len(), 1, "got {:?}", result.diagnostics);
    }

    /// MUST STILL FIRE: the continuation exemption is one line deep. A line
    /// after a continued line that itself does NOT continue is code again.
    #[test]
    fn still_fires_on_line_after_a_continuation_ends() {
        let script = "cmd one \\\n    two\nFOO = bar\n";
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 1, "got {:?}", result.diagnostics);
        assert_eq!(result.diagnostics[0].span.start_line, 3);
    }
}
