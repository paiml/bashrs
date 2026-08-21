// SC2104: Missing space before ]
//
// Detects test expressions missing required whitespace before closing bracket.
// In POSIX shell, [ is a command and ] is its final argument, so spaces are required.
//
// Examples:
// Bad:
//   if [ "$var" = "value"]; then
//
// Good:
//   if [ "$var" = "value" ]; then

use crate::linter::rules::quoting::is_inside_quoted_string;
use crate::linter::{Diagnostic, Fix, LintResult, Severity, Span};
use regex::Regex;

static MISSING_SPACE_BEFORE_BRACKET: std::sync::LazyLock<Regex> = std::sync::LazyLock::new(|| {
    // Match: anything followed by ] without space before it
    Regex::new(r"[^\s\[]\]").unwrap()
});

static TEST_COMMAND: std::sync::LazyLock<Regex> =
    std::sync::LazyLock::new(|| Regex::new(r"\[\s+").unwrap());

/// Check if a position is inside a parameter expansion ${...}
fn is_inside_param_expansion(line: &str, pos: usize) -> bool {
    let chars: Vec<char> = line.chars().collect();
    let mut depth = 0;

    for i in 0..pos.min(chars.len()) {
        if i + 1 < chars.len() && chars[i] == '$' && chars[i + 1] == '{' {
            depth += 1;
        } else if chars[i] == '}' && depth > 0 {
            depth -= 1;
        }
    }

    depth > 0
}

/// Is this `]` real test syntax, or is it text that merely looks like it?
///
/// Three ways it is not:
///   * `]]` — the closing half of a double-bracket test.
///   * Inside `${...}` — e.g. `${#array[@]}`, `${var[$key]}` (issue #88).
///   * Inside a quoted string — a usage message such as
///     `'prog [--source|--videos]'` is documentation (issue #244).
fn is_real_missing_space(line: &str, end: usize) -> bool {
    if end < line.len() && line.chars().nth(end) == Some(']') {
        return false;
    }
    !is_inside_param_expansion(line, end - 1) && !is_inside_quoted_string(line, end - 1)
}

/// Build the fixed line with a space inserted before the `]`.
fn fixed_line(line: &str, start: usize, end: usize, matched: &str) -> String {
    let spaced = format!("{} ]", &matched[..matched.len() - 1]);
    format!("{}{}{}", &line[..start], spaced, &line[end..])
}

pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();

    for (i, line) in source.lines().enumerate() {
        let line_num = i + 1;

        // Comments are prose; `[[...]]` is bash's own construct, not SC2104.
        if line.trim_start().starts_with('#') || !TEST_COMMAND.is_match(line) || line.contains("[[")
        {
            continue;
        }

        for mat in MISSING_SPACE_BEFORE_BRACKET.find_iter(line) {
            if !is_real_missing_space(line, mat.end()) {
                continue;
            }
            result.add(
                Diagnostic::new(
                    "SC2104",
                    Severity::Error,
                    "Missing space before ]",
                    Span::new(line_num, mat.start() + 1, line_num, mat.end() + 1),
                )
                .with_fix(Fix::new(fixed_line(
                    line,
                    mat.start(),
                    mat.end(),
                    mat.as_str(),
                ))),
            );
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sc2104_missing_space_basic() {
        let code = r#"if [ "$var" = "value"]; then"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
        assert_eq!(result.diagnostics[0].code, "SC2104");
        assert_eq!(result.diagnostics[0].severity, Severity::Error);
        assert!(result.diagnostics[0].fix.is_some());
    }

    #[test]
    fn test_sc2104_autofix() {
        let code = r#"if [ "$var" = "value"]; then"#;
        let result = check(code);
        let fix = result.diagnostics[0].fix.as_ref().unwrap();
        assert!(fix.replacement.contains(" ]"));
        assert!(!fix.replacement.contains("\"]\"")); // Should not have "]" without space
        assert!(fix.replacement.contains("\" ]")); // Should have " ]" with space
    }

    #[test]
    fn test_sc2104_correct_spacing_ok() {
        let code = r#"if [ "$var" = "value" ]; then"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2104_double_bracket_ok() {
        let code = r#"if [[ "$var" = "value"]]; then"#;
        let result = check(code);
        // Should not trigger on [[...]] (bash extended test)
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2104_numeric_comparison() {
        let code = r#"if [ "$count" -eq 10]; then"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2104_string_comparison() {
        let code = r#"if [ "$str" != "test"]; then"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2104_file_test() {
        let code = r#"if [ -f "$file"]; then"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2104_multiple_conditions() {
        let code = r#"if [ "$a" = "1"] && [ "$b" = "2" ]; then"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2104_no_test_command() {
        let code = r#"echo "array[0]""#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2104_array_subscript_ok() {
        let code = r#"echo "${array[0]}""#;
        let result = check(code);
        // Should not trigger on array subscripts
        assert_eq!(result.diagnostics.len(), 0);
    }

    // Issue #88: SC2104 should NOT flag ] in array length/subscript syntax
    #[test]
    fn test_sc2104_issue_88_array_length_in_test() {
        // From issue #88 reproduction case
        let code = r#"if [ ${#PASSED_FILES[@]} -gt 0 ]; then"#;
        let result = check(code);
        assert_eq!(
            result.diagnostics.len(),
            0,
            "SC2104 must NOT flag ] inside ${{#array[@]}} - it's array subscript, not test bracket"
        );
    }

    #[test]
    fn test_sc2104_issue_88_associative_array_in_test() {
        // Another pattern from issue #88
        let code = r#"if [ -z "${SAMPLES[$errcode]:-}" ]; then"#;
        let result = check(code);
        assert_eq!(
            result.diagnostics.len(),
            0,
            "SC2104 must NOT flag ] inside ${{array[$key]:-}} - it's array subscript, not test bracket"
        );
    }

    #[test]
    fn test_sc2104_issue_88_array_expansion_in_test() {
        let code = r#"if [ "${#array[@]}" -ne 0 ]; then"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2104_issue_88_still_detects_real_issues() {
        // Should still detect actual missing space before test ]
        let code = r#"if [ "${#array[@]}" -gt 0]; then"#;
        let result = check(code);
        assert_eq!(
            result.diagnostics.len(),
            1,
            "SC2104 should still detect missing space before test ]"
        );
    }

    #[test]
    fn test_sc2104_nested_param_expansion() {
        let code = r#"if [ "${var[${idx}]}" = "test" ]; then"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    /// Issue #244: a `[` inside a quoted string is documentation, not test syntax.
    #[test]
    fn test_bracket_inside_single_quoted_string_is_not_a_test() {
        let code = r#"[ $# -gt 0 ] || { echo 'usage: prog [--source|--videos]' >&2; exit 2; }"#;
        let result = check(code);
        assert!(
            result.diagnostics.is_empty(),
            "SC2104 fired inside a single-quoted usage string: {:?}",
            result.diagnostics
        );
    }

    /// The same, double-quoted.
    #[test]
    fn test_bracket_inside_double_quoted_string_is_not_a_test() {
        let code = r#"[ -n "$x" ] && echo "opts: [--a|--b]""#;
        let result = check(code);
        assert!(
            result.diagnostics.is_empty(),
            "SC2104 fired inside a double-quoted string: {:?}",
            result.diagnostics
        );
    }

    /// Guard the guard: a REAL missing space must still be reported, so the
    /// quote-skip cannot be widened into "never fire".
    #[test]
    fn test_real_missing_space_still_detected_alongside_quotes() {
        let code = r#"if [ "$var" = "value"]; then"#;
        let result = check(code);
        assert_eq!(
            result.diagnostics.len(),
            1,
            "the genuine SC2104 must still fire: {:?}",
            result.diagnostics
        );
    }
}
