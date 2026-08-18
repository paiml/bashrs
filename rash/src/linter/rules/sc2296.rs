// SC2296: Parameter expansions can't be nested
use crate::linter::{Diagnostic, LintResult, Severity, Span};
use regex::Regex;

// GH-233 dogfood review: the old pattern `\$\{[^}]*\$\{` matched a `${` ANYWHERE
// before a second `${`, so `${BUILD_DIR:-/tmp/builds/${BUILD_ID}}` — a parameter
// expansion whose DEFAULT VALUE contains another expansion, which is completely
// valid POSIX shell and executes correctly under both dash and bash — was reported
// as a hard parse error. Real shellcheck 0.8.0 does not flag it either.
//
// What genuinely cannot be nested is the operand itself: `${${name}}` attempts to
// use the result of one expansion as the NAME of another, which is a parse error
// in both dash and bash ("bad substitution"). That shape is `${` followed
// immediately (mod whitespace) by another `${`, with no operator in between.
static NESTED_EXPANSION: std::sync::LazyLock<Regex> =
    std::sync::LazyLock::new(|| Regex::new(r"\$\{\s*\$\{").unwrap());

pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();
    for (line_num, line) in source.lines().enumerate() {
        let line_num = line_num + 1;
        if line.trim_start().starts_with('#') {
            continue;
        }

        if NESTED_EXPANSION.is_match(line) {
            let diagnostic = Diagnostic::new(
                "SC2296",
                Severity::Error,
                "Parameter expansions can't be nested. Use separate expansions.".to_string(),
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
    fn test_GH233_sc2296_true_nesting_still_flagged() {
        // `${${name}}` nests the OPERAND: it tries to use one expansion's result
        // as the name of another, which is a parse error ("bad substitution") in
        // both dash and bash.
        let code = "${${name}}";
        assert_eq!(check(code).diagnostics.len(), 1);
    }

    #[test]
    fn test_GH233_sc2296_default_value_expansion_is_not_nesting() {
        // A parameter expansion whose DEFAULT VALUE contains another expansion is
        // ordinary, valid POSIX shell — real shellcheck does not flag it, and both
        // dash and bash execute it correctly. This was the false positive found
        // dogfooding v6.67.0: examples/cicd-integration/purified.sh (a file meant
        // to be clean output) failed its own linter on this exact shape.
        for code in [
            "${var:-${default}}",
            "BUILD_DIR=${BUILD_DIR:-/tmp/builds/${BUILD_ID}}",
            "${var:=${default}}",
            "${var:+${other}}",
            "${var-${default}}",
        ] {
            assert_eq!(
                check(code).diagnostics.len(),
                0,
                "default-value nesting must not be flagged: {code}"
            );
        }
    }

    #[test]
    fn test_sc2296_separate_ok() {
        let code = "default=${DEFAULT}\nvar=${var:-$default}";
        assert_eq!(check(code).diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2296_simple_expansion_ok() {
        let code = "${var:-default}";
        assert_eq!(check(code).diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2296_comment() {
        let code = "# ${var:-${def}}";
        assert_eq!(check(code).diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2296_empty() {
        assert_eq!(check("").diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2296_normal() {
        assert_eq!(check("echo test").diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2296_multiple_vars_ok() {
        let code = "${var1} ${var2}";
        assert_eq!(check(code).diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2296_command_subst_ok() {
        let code = "${var:-$(command)}";
        assert_eq!(check(code).diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2296_nested_in_assign() {
        // Was asserting the exact false positive fixed in GH-233:
        // `x=${a:-${b}}` is default-value nesting, valid POSIX shell, not flagged
        // by real shellcheck. See test_GH233_sc2296_default_value_expansion_is_not_nesting.
        let code = "x=${a:-${b}}";
        assert_eq!(check(code).diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2296_arithmetic_ok() {
        let code = "$((x + y))";
        assert_eq!(check(code).diagnostics.len(), 0);
    }
}
