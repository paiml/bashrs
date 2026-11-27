// SC2086: Double quote to prevent globbing and word splitting - THIN SHIM
// All logic extracted to sc2086_logic.rs

use super::sc2086_logic::*;
use crate::linter::{Diagnostic, Fix, LintResult, Severity, Span};

/// Check for unquoted variable expansions (SC2086)
pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();
    let pattern = get_var_pattern();
    // PMAT-654: a variable only ever assigned an integer literal cannot word-split; shellcheck 0.8.0 agrees.
    let mut cstyle_vars = get_cstyle_for_loop_vars(source);
    cstyle_vars.extend(get_integer_literal_vars(source));
    let body = super::sc2086_heredoc::body_command_substitution_bytes(source);
    for (line_num, line) in source.lines().enumerate() {
        let subst = body.get(&(line_num + 1));
        for uv in find_unquoted_vars(line, &pattern, &cstyle_vars) {
            if subst.is_some_and(|s| !s.contains(&(uv.col - 1))) {
                continue;
            }
            let span = Span::new(line_num + 1, uv.col, line_num + 1, uv.end_col);
            let var_text = format_var_text(&uv.var_name, uv.is_braced);
            result.add(
                Diagnostic::new(
                    "SC2086",
                    Severity::Warning,
                    format!(
                        "Double quote to prevent globbing and word splitting on {}",
                        var_text
                    ),
                    span,
                )
                .with_fix(Fix::new(format_quoted_var(&uv.var_name, uv.is_braced))),
            );
        }
    }
    result
}

#[cfg(test)]
#[path = "sc2086_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "sc2086_tests_heredoc.rs"]
mod tests_heredoc;

#[cfg(test)]
mod integer_literal_tests_654 {
    use super::check;

    // PMAT-654 / bashrs#654: shellcheck 0.8.0 does not flag $n when n is assigned an integer literal.
    #[test]
    fn echo_of_integer_literal_var_is_not_flagged() {
        let r = check("n=1\necho $n\n");
        assert!(
            r.diagnostics.is_empty(),
            "SC2086 fired on integer-literal var: {:?}",
            r.diagnostics
                .iter()
                .map(|d| d.message.clone())
                .collect::<Vec<_>>()
        );
    }

    #[test]
    fn test_bracket_of_integer_literal_var_is_not_flagged() {
        let r = check("n=1\nif [ $n -le 1 ]; then\n  echo ok\nfi\n");
        assert!(
            r.diagnostics.is_empty(),
            "SC2086 fired in [ ] on integer-literal var"
        );
    }

    #[test]
    fn non_literal_var_is_still_flagged() {
        let r = check("n=$(date)\necho $n\n");
        assert!(!r.diagnostics.is_empty());
    }

    // Shellcheck 0.8.0 flags an unquoted positional parameter ($1..$9) with SC2086.
    #[test]
    fn unquoted_positional_param_is_flagged_sc2086() {
        let r = check("#!/bin/bash\necho $1 $2\n");
        let codes: Vec<String> = r.diagnostics.iter().map(|d| d.code.clone()).collect();
        assert_eq!(
            codes.iter().filter(|c| *c == "SC2086").count(),
            2,
            "SC2086 must fire on $1 and $2, got {:?}",
            r.diagnostics
                .iter()
                .map(|d| (d.code.clone(), d.message.clone()))
                .collect::<Vec<_>>()
        );
    }
}
