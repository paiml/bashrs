// SC2032: Use own script's variable. To set/use it, source script or remove shebang.
//
// Variables set in an executed script don't affect the calling shell.
// If you want to set variables that affect the caller, the script must be sourced.
//
// Examples:
// Bad:
//   # script.sh (with #!/bin/bash)
//   VAR=value
//   # Caller runs: ./script.sh
//   # Caller's $VAR is unaffected
//
// Good:
//   # config.sh (no shebang, meant to be sourced)
//   VAR=value
//   # Caller runs: source config.sh
//   # Caller's $VAR is now set
//
//   # OR: Remove shebang if meant to be sourced
//
// Note: This rule detects variable assignments in scripts with shebangs.
// If the script has a shebang, it's executed in a subshell and variables won't
// propagate to the caller.

use crate::linter::{Diagnostic, LintResult, Severity, Span};
use regex::Regex;

static VARIABLE_ASSIGNMENT: std::sync::LazyLock<Regex> = std::sync::LazyLock::new(|| {
    // Match: VAR=value (simple assignment)
    Regex::new(r"^([a-zA-Z_][a-zA-Z0-9_]*)=").unwrap()
});

/// Check if script has shebang line
fn has_shebang(lines: &[&str]) -> bool {
    !lines.is_empty() && lines[0].starts_with("#!")
}

/// Check if line is a comment
fn is_comment(line: &str) -> bool {
    line.trim_start().starts_with('#')
}

/// Check if line is an export statement
fn is_export_statement(line: &str) -> bool {
    line.trim_start().starts_with("export ")
}

/// Check if line is a local variable declaration
fn is_local_declaration(line: &str) -> bool {
    line.trim_start().starts_with("local ")
}

/// Check if line is a readonly declaration
fn is_readonly_declaration(line: &str) -> bool {
    line.trim_start().starts_with("readonly ")
}

/// Check if variable should be skipped (special shell variables)
fn is_special_variable(var_name: &str) -> bool {
    matches!(var_name, "PATH" | "IFS" | "PS1" | "HOME")
}

/// Calculate span for variable assignment
fn calculate_span(line: &str, var_name: &str, line_num: usize) -> Span {
    let pos = line.find(var_name).unwrap_or(0);
    let start_col = pos + 1;
    let end_col = start_col + var_name.len() + 1; // +1 for =
    Span::new(line_num, start_col, line_num, end_col)
}

/// Build diagnostic for variable assignment in shebang script
fn build_diagnostic(var_name: &str, span: Span) -> Diagnostic {
    Diagnostic::new(
        "SC2032",
        Severity::Info,
        format!(
            "Variable '{}' assigned in script with shebang. To affect caller, source this script (source {}) or remove shebang",
            var_name, "script.sh"
        ),
        span,
    )
}

pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();
    let lines: Vec<&str> = source.lines().collect();

    if !has_shebang(&lines) {
        return result;
    }

    for (line_num, line) in lines.iter().enumerate().skip(1) {
        let line_num = line_num + 1;

        if is_comment(line) {
            continue;
        }

        if !VARIABLE_ASSIGNMENT.is_match(line.trim_start()) {
            continue;
        }

        if is_export_statement(line) || is_local_declaration(line) || is_readonly_declaration(line)
        {
            continue;
        }

        if let Some(cap) = VARIABLE_ASSIGNMENT.captures(line.trim_start()) {
            let var_name = cap.get(1).unwrap().as_str();

            if is_special_variable(var_name) {
                continue;
            }

            let span = calculate_span(line, var_name, line_num);
            let diagnostic = build_diagnostic(var_name, span);
            result.add(diagnostic);
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sc2032_assignment_with_shebang() {
        let code = r#"#!/bin/bash
FOO=bar
"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
        assert_eq!(result.diagnostics[0].code, "SC2032");
        assert_eq!(result.diagnostics[0].severity, Severity::Info);
        assert!(result.diagnostics[0].message.contains("FOO"));
    }

    #[test]
    fn test_sc2032_no_shebang_ok() {
        let code = r#"# config.sh
FOO=bar
"#;
        let result = check(code);
        // No shebang means meant to be sourced, OK
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2032_export_ok() {
        let code = r#"#!/bin/bash
export PATH=/usr/bin
"#;
        let result = check(code);
        // export is meant for subprocesses, OK
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2032_local_ok() {
        let code = r#"#!/bin/bash
local foo=bar
"#;
        let result = check(code);
        // local is function-scoped, OK
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2032_readonly_ok() {
        let code = r#"#!/bin/bash
readonly VERSION=1.0
"#;
        let result = check(code);
        // readonly is constant, OK
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2032_path_ok() {
        let code = r#"#!/bin/bash
PATH=/usr/local/bin:$PATH
"#;
        let result = check(code);
        // PATH is commonly modified, OK
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2032_multiple_assignments() {
        let code = r#"#!/bin/bash
VAR1=a
VAR2=b
"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 2);
    }

    #[test]
    fn test_sc2032_comment_ok() {
        let code = r#"#!/bin/bash
# FOO=bar
"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2032_indented_assignment() {
        let code = r#"#!/bin/bash
    VAR=value
"#;
        let result = check(code);
        // Indented assignment (might be in function, but we detect anyway)
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2032_empty_file_ok() {
        let code = "";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }
}
