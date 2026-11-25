//! SC2043: Use direct command instead of for loop with single element
//!
//! # Examples
//!
//! Bad:
//! ```bash
//! for i in "$var"; do
//!   echo "$i"
//! done
//! ```
//!
//! Good:
//! ```bash
//! echo "$var"
//! ```
//!
//! # Rationale
//!
//! A for loop with a single quoted variable is unnecessary complexity.
//! Just use the variable directly.
//!
//! # Auto-fix
//!
//! Suggest simplifying to direct command
//!
//! A single double-quoted literal (`for i in "a b c"`, no expansion and no `*`)
//! also runs once and is reported, matching ShellCheck. A single quoted
//! variable (`for i in "$var"`) is not reported: ShellCheck 0.8.0 reports that
//! case as SC2066, not SC2043.

use crate::linter::{Diagnostic, LintResult, Severity, Span};
use regex::Regex;

/// A `for` loop over one double-quoted literal word (no `$`, backtick or `*`),
/// in command position and followed by `;`, a comment or end of line.
static LITERAL_PATTERN: std::sync::LazyLock<Regex> = std::sync::LazyLock::new(|| {
    Regex::new(
        r#"(?:^|[;&|({]|\b(?:then|do|else)\b)\s*(for\s+[A-Za-z_]\w*\s+in\s+"(?:[^"\\$`*]|\\[^*])*"\s*(?:;|$|#))"#,
    )
    .unwrap()
});

fn single_diagnostic(line_num: usize, start: usize, end: usize) -> Diagnostic {
    Diagnostic::new(
        "SC2043",
        Severity::Info,
        "This for loop will only run once. Use direct command instead.",
        Span::new(line_num, start + 1, line_num, end + 1),
    )
}

pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();

    for (line_num, line) in source.lines().enumerate() {
        let line_num = line_num + 1;

        if line.trim_start().starts_with('#') {
            continue;
        }

        for cap in LITERAL_PATTERN.captures_iter(line) {
            let m = cap.get(1).unwrap();
            result.add(single_diagnostic(line_num, m.start(), m.end()));
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sc2043_basic_detection() {
        let script = r#"for i in "$var"; do echo "$i"; done"#;
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2043_single_variable() {
        let script = r#"for f in "$file"; do cat "$f"; done"#;
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2043_false_positive_multiple_vars() {
        let script = r#"for f in "$var1" "$var2"; do echo "$f"; done"#;
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2043_false_positive_unquoted() {
        let script = r#"for f in $files; do echo "$f"; done"#;
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 0); // Unquoted might expand to multiple
    }

    #[test]
    fn test_sc2043_false_positive_array() {
        let script = r#"for f in "${array[@]}"; do echo "$f"; done"#;
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2043_false_positive_glob() {
        let script = r#"for f in *.txt; do echo "$f"; done"#;
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2043_multiline() {
        let script = r#"
for item in "$single_item"; do
    echo "$item"
done
"#;
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2043_different_variable_names() {
        let script = r#"for x in "$value"; do process "$x"; done"#;
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2043_false_positive_command_subst() {
        let script = r#"for f in "$(ls)"; do echo "$f"; done"#;
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 0); // Command substitution might return multiple
    }

    #[test]
    fn test_sc2043_nested_var() {
        let script = r#"for i in "$myvar"; do echo "$i"; done"#;
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 0);
    }

    fn count(script: &str) -> usize {
        check(script).diagnostics.len()
    }

    #[test]
    fn test_sc2043_literal_with_spaces() {
        assert_eq!(count(r#"for i in "a b c"; do echo "$i"; done"#), 1);
        assert_eq!(count(r#"for i in "abc"; do echo "$i"; done"#), 1);
        assert_eq!(count(r#"for i in ""; do echo "$i"; done"#), 1);
        assert_eq!(count(r#"for i in "a  b"; do :; done"#), 1);
    }

    #[test]
    fn test_sc2043_literal_whitespace_forms() {
        assert_eq!(count("  for i in \"a b\"; do :; done"), 1);
        assert_eq!(count("\tfor i in \"a b\"; do :; done"), 1);
        assert_eq!(count("for\ti\tin\t\"a b\"\t;\tdo :; done"), 1);
        assert_eq!(count("for   i   in   \"a b\"   ; do :; done"), 1);
        assert_eq!(count("for i in \"a b\";do :; done"), 1);
        assert_eq!(count("for i in \"a b\"\ndo :; done"), 1);
        assert_eq!(count("for i in \"a b\" # note\ndo :; done"), 1);
    }

    #[test]
    fn test_sc2043_literal_special_chars() {
        assert_eq!(count(r#"for i in "a;b"; do :; done"#), 1);
        assert_eq!(count(r#"for i in "a?b"; do :; done"#), 1);
        assert_eq!(count(r#"for i in "a[b]"; do :; done"#), 1);
        assert_eq!(count(r#"for i in "a{b,c}"; do :; done"#), 1);
        assert_eq!(count(r#"for i in "a\"b c"; do :; done"#), 1);
        assert_eq!(count(r#"for i in "a\$b"; do :; done"#), 1);
        assert_eq!(count("for i in \"a'b\"; do :; done"), 1);
    }

    #[test]
    fn test_sc2043_literal_not_flagged() {
        // oracle: expansions, globs and multiple words are not single literals
        assert_eq!(count(r#"for i in "a b" "c"; do :; done"#), 0);
        assert_eq!(count(r#"for i in "a" "b"; do :; done"#), 0);
        assert_eq!(count(r#"for i in "a*"; do :; done"#), 0);
        assert_eq!(count(r#"for i in "a\*"; do :; done"#), 0);
        assert_eq!(count(r#"for i in "a$x b"; do :; done"#), 0);
        assert_eq!(count(r#"for i in "a$(ls) b"; do :; done"#), 0);
        assert_eq!(count(r#"for i in "a`ls` b"; do :; done"#), 0);
        assert_eq!(count(r#"for i in "a ${x} b"; do :; done"#), 0);
        assert_eq!(count("for i in 'a b'; do :; done"), 0);
        assert_eq!(count(r#"for i in "a b" c; do :; done"#), 0);
    }

    #[test]
    fn test_sc2043_literal_not_command_position() {
        assert_eq!(count(r#"echo "for i in \"a b\"; do :; done""#), 0);
        assert_eq!(count(r#"echo "x" for i in "a b"; do :; done"#), 0);
        assert_eq!(count(r#"# for i in "a b"; do :; done"#), 0);
    }

    #[test]
    fn test_sc2043_literal_nested_contexts() {
        assert_eq!(count(r#"echo x; for i in "a b"; do :; done"#), 1);
        assert_eq!(count(r#"if x; then for i in "a b"; do :; done; fi"#), 1);
        assert_eq!(count(r#"while x; do for i in "a b"; do :; done; done"#), 1);
        assert_eq!(count(r#"x && for i in "a b"; do :; done"#), 1);
        assert_eq!(
            count("for i in \"a b\"; do :; done; for j in \"c d\"; do :; done"),
            2
        );
    }

    #[test]
    fn test_sc2043_literal_span() {
        let r = check("x\n\tfor i in \"a b\"; do :; done");
        assert_eq!(r.diagnostics.len(), 1);
        assert_eq!(r.diagnostics[0].span.start_line, 2);
        assert_eq!(r.diagnostics[0].span.start_col, 2);
    }

    #[test]
    fn test_sc2043_var_regression() {
        assert_eq!(count(r#"for i in "$var"; do :; done"#), 0);
        assert_eq!(count(r#"for i in "$var" ; do :; done"#), 0);
    }

    #[test]
    fn test_sc2043_quoted_var_not_flagged_like_shellcheck() {
        // shellcheck 0.8.0 is silent on SC2043 for a single quoted variable
        // (it reports SC2066 instead); SC2043 is literal-only.
        assert_eq!(count(r#"for i in "$var"; do echo "$i"; done"#), 0);
        assert_eq!(count(r#"for f in "$file"; do cat "$f"; done"#), 0);
        assert_eq!(count(r#"for i in "$var" ; do :; done"#), 0);
    }
}
