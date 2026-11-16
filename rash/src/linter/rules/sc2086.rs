// SC2086: Double quote to prevent globbing and word splitting - THIN SHIM
// All logic extracted to sc2086_logic.rs

use super::sc2086_logic::*;
use crate::linter::{Diagnostic, Fix, LintResult, Severity, Span};

/// Check for unquoted variable expansions (SC2086)
pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();
    let pattern = get_var_pattern();
    let cstyle_vars = get_cstyle_for_loop_vars(source);
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
