// SC2154: Variable referenced but not assigned - THIN SHIM
// All logic extracted to sc2154_logic.rs

use super::sc2154_logic::*;
use crate::linter::{Diagnostic, LintResult, Severity, Span};

/// Check for variables referenced but not assigned
pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();
    let patterns = create_patterns();
    let builtins = get_builtins();
    let (mut assigned, used_vars) = collect_variable_info(source, &patterns);
    assigned.extend(collect_case_statement_variables(source));
    for (var_name, line_num, col) in find_undefined_variables(&assigned, &used_vars, &builtins) {
        result.add(Diagnostic::new(
            "SC2154",
            Severity::Warning,
            format!("Variable '{}' is referenced but not assigned", var_name),
            Span::new(line_num, col, line_num, col + var_name.len() + 1),
        ));
    }
    result
}

#[cfg(test)]
#[path = "sc2154_tests_extracted.rs"]
mod tests_extracted;
