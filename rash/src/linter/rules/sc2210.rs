// SC2210: Don't use arithmetic shortcuts like x=++y
use crate::linter::{Diagnostic, LintResult, Severity, Span};
use regex::Regex;

static ARITHMETIC_SHORTCUT: std::sync::LazyLock<Regex> = std::sync::LazyLock::new(|| {
    // Match x=++y or x=--y (C-style prefix operators in assignment)
    Regex::new(r"\w+\s*=\s*(\+\+|--)\w+").unwrap()
});

pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();
    for (line_num, line) in source.lines().enumerate() {
        let line_num = line_num + 1;
        if line.trim_start().starts_with('#') {
            continue;
        }

        // Skip if in arithmetic context
        if line.contains("$((") || line.contains("((") {
            continue;
        }

        if ARITHMETIC_SHORTCUT.is_match(line) {
            let diagnostic = Diagnostic::new(
                "SC2210",
                Severity::Error,
                "Prefix operators (++/--) only work in arithmetic context. Use x=$((y + 1))"
                    .to_string(),
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
    fn test_sc2210_prefix_increment() {
        let code = r#"x=++y"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }
    #[test]
    fn test_sc2210_prefix_decrement() {
        let code = r#"count=--total"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }
    #[test]
    fn test_sc2210_arithmetic_context_ok() {
        let code = r#"x=$((++y))"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }
    #[test]
    fn test_sc2210_double_paren_ok() {
        let code = r#"((x=++y))"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }
    #[test]
    fn test_sc2210_normal_assignment_ok() {
        let code = r#"x=$y"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }
    #[test]
    fn test_sc2210_addition_ok() {
        let code = r#"x=$((y + 1))"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }
    #[test]
    fn test_sc2210_spaces() {
        let code = r#"val = ++count"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }
    #[test]
    fn test_sc2210_comment_skipped() {
        let code = r#"# x=++y"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }
    #[test]
    fn test_sc2210_string_literal_ok() {
        let code = r#"text="++value""#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0); // String, not arithmetic
    }
    #[test]
    fn test_sc2210_underscore_var() {
        let code = r#"_new=++_old"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }
}

/// GH-370: an assignment is a property of a word's POSITION in a simple
/// command. The old line regex `\w+\s*=\s*(\+\+|--)\w+` let `=` and the
/// operator sit in different shell words, so the empty `ps` format `pid=`
/// followed by the long option `--ppid` read as `pid=--ppid`.
#[cfg(test)]
mod gh370_tests {
    use super::*;

    fn count(code: &str) -> usize {
        check(code).diagnostics.len()
    }

    #[test]
    fn test_gh370_ps_ppid_in_command_substitution_is_not_an_assignment() {
        assert_eq!(count(r#"c="$(ps -o pid= --ppid "$p")""#), 0);
    }

    #[test]
    fn test_gh370_ps_ppid_bare_is_not_an_assignment() {
        assert_eq!(count("ps -o pid= --ppid 1"), 0);
    }

    #[test]
    fn test_gh370_long_option_value_is_not_an_assignment() {
        // `--format=--x` begins with `-`: it cannot be a NAME=value word.
        assert_eq!(count("git log --format=--x"), 0);
    }

    #[test]
    fn test_gh370_quoted_argument_is_not_an_assignment() {
        assert_eq!(count(r#"echo "x=--y""#), 0);
    }

    #[test]
    fn test_gh370_let_is_already_arithmetic() {
        assert_eq!(count("let x=++y"), 0);
    }

    // Negative controls: the shapes the rule exists for still fire.
    #[test]
    fn test_gh370_control_prefix_decrement_still_fires() {
        assert_eq!(count("x=--y"), 1);
    }

    #[test]
    fn test_gh370_control_prefix_before_command_still_fires() {
        assert_eq!(count("x=++y cmd"), 1);
    }

    #[test]
    fn test_gh370_control_local_still_fires() {
        assert_eq!(count("local n=--total"), 1);
    }

    #[test]
    fn test_gh370_control_after_if_still_fires() {
        assert_eq!(count("if true; then x=++y; fi"), 1);
    }

    #[test]
    fn test_gh370_control_inside_command_substitution_still_fires() {
        assert_eq!(count(r#"out="$(x=--y; echo "$x")""#), 1);
    }

    #[test]
    fn test_gh370_control_empty_prefix_then_operator_command_still_fires() {
        // `x= --y` assigns '' and then RUNS `--y`: still the defect.
        assert_eq!(count("x= --y"), 1);
    }
}
