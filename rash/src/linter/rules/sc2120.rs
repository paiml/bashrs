// SC2120: foo references arguments, but none are ever passed
//
// This is the complement of SC2119. When a function uses positional parameters
// ($1, $2, $@, etc.) but is never called with arguments, it's likely a bug.
//
// Examples:
// Bad:
//   my_func() { echo "hello $1"; }
//   my_func  # Should pass an argument
//
// Good:
//   my_func() { echo "hello $1"; }
//   my_func "world"  # Argument provided
//
//   my_func() { echo "hello"; }
//   my_func  # No arguments needed

use crate::linter::{Diagnostic, LintResult, Severity, Span};
use once_cell::sync::Lazy;
use regex::Regex;
use std::collections::{HashMap, HashSet};

static FUNCTION_DEF: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"([A-Za-z_][A-Za-z0-9_]*)\s*\(\s*\)\s*\{").unwrap()
});

static FUNCTION_CALL: Lazy<Regex> = Lazy::new(|| {
    // Match function calls (with or without arguments)
    Regex::new(r"^([A-Za-z_][A-Za-z0-9_]*)\b").unwrap()
});

static ARG_REFERENCE: Lazy<Regex> = Lazy::new(|| {
    // Match $1, $2, $@, $*, etc.
    Regex::new(r"\$[@*#]|\$\{?[0-9]+\}?").unwrap()
});

/// Check if text after function name indicates arguments are passed
fn has_arguments_after_name(call_text: &str, func_name: &str) -> bool {
    if call_text.len() <= func_name.len() + 1 {
        return false;
    }

    let after_func = &call_text[func_name.len()..].trim_start();
    !after_func.is_empty()
        && !after_func.starts_with(';')
        && !after_func.starts_with('|')
        && !after_func.starts_with('&')
        && !after_func.starts_with('<')
        && !after_func.starts_with('>')
}

/// Update function to mark that it uses arguments
fn mark_function_uses_args(
    function_defs: &mut HashMap<String, (usize, bool)>,
    in_function: &Option<String>,
) {
    if let Some(ref func) = in_function {
        if let Some(entry) = function_defs.get_mut(func) {
            entry.1 = true;
        }
    }
}

/// Find all function definitions and track which use positional parameters
fn find_function_definitions(lines: &[&str]) -> HashMap<String, (usize, bool)> {
    let mut function_defs: HashMap<String, (usize, bool)> = HashMap::new();
    let mut in_function: Option<String> = None;
    let mut brace_depth = 0;

    for (idx, line) in lines.iter().enumerate() {
        let trimmed = line.trim();

        if trimmed.starts_with('#') {
            continue;
        }

        // Track function definitions
        if let Some(cap) = FUNCTION_DEF.captures(trimmed) {
            let func_name = cap.get(1).unwrap().as_str().to_string();
            in_function = Some(func_name.clone());
            let func_def_line = idx + 1;
            function_defs.insert(func_name, (func_def_line, false));
            brace_depth = 1;
            continue;
        }

        // Track brace depth and argument usage
        if in_function.is_some() {
            brace_depth += line.matches('{').count();
            brace_depth = brace_depth.saturating_sub(line.matches('}').count());

            if brace_depth == 0 {
                in_function = None;
            } else if ARG_REFERENCE.is_match(line) {
                mark_function_uses_args(&mut function_defs, &in_function);
            }
        }
    }

    function_defs
}

/// Find all functions that are called with arguments
fn find_functions_called_with_args(
    lines: &[&str],
    function_defs: &HashMap<String, (usize, bool)>,
) -> HashSet<String> {
    let mut functions_called_with_args: HashSet<String> = HashSet::new();

    for line in lines {
        let trimmed = line.trim();

        if trimmed.starts_with('#') || FUNCTION_DEF.is_match(trimmed) {
            continue;
        }

        if let Some(cap) = FUNCTION_CALL.captures(trimmed) {
            let func_name = cap.get(1).unwrap().as_str();

            if function_defs.contains_key(func_name)
                && has_arguments_after_name(trimmed, func_name)
            {
                functions_called_with_args.insert(func_name.to_string());
            }
        }
    }

    functions_called_with_args
}

/// Generate diagnostics for functions using args but never called with them
fn generate_diagnostics(
    function_defs: HashMap<String, (usize, bool)>,
    functions_called_with_args: &HashSet<String>,
) -> LintResult {
    let mut result = LintResult::new();

    for (func_name, (line_num, uses_args)) in function_defs {
        if uses_args && !functions_called_with_args.contains(&func_name) {
            let diagnostic = Diagnostic::new(
                "SC2120",
                Severity::Info,
                format!(
                    "{} references arguments, but none are ever passed",
                    func_name
                ),
                Span::new(line_num, 1, line_num, func_name.len() + 1),
            );

            result.add(diagnostic);
        }
    }

    result
}

pub fn check(source: &str) -> LintResult {
    let lines: Vec<&str> = source.lines().collect();

    // First pass: Find functions and check if they use positional parameters
    let function_defs = find_function_definitions(&lines);

    // Second pass: Find function calls and check if they pass arguments
    let functions_called_with_args = find_functions_called_with_args(&lines, &function_defs);

    // Generate warnings for functions that use args but are never called with them
    generate_diagnostics(function_defs, &functions_called_with_args)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sc2120_uses_args_but_never_passed() {
        let code = r#"
my_func() { echo "hello $1"; }
my_func
"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
        assert_eq!(result.diagnostics[0].code, "SC2120");
        assert_eq!(result.diagnostics[0].severity, Severity::Info);
        assert!(result.diagnostics[0].message.contains("my_func"));
        assert!(result.diagnostics[0].message.contains("arguments"));
    }

    #[test]
    fn test_sc2120_args_passed_ok() {
        let code = r#"
my_func() { echo "hello $1"; }
my_func "world"
"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2120_no_args_used_ok() {
        let code = r#"
my_func() { echo "hello"; }
my_func
"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2120_uses_at() {
        let code = r#"
my_func() { echo "$@"; }
my_func
"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2120_uses_star() {
        let code = r#"
my_func() { echo "$*"; }
my_func
"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2120_uses_numbered_param() {
        let code = r#"
my_func() { local val="$2"; echo "$val"; }
my_func
"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2120_braces_in_param_ref() {
        let code = r#"
my_func() { echo "${1}"; }
my_func
"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2120_multiple_functions() {
        let code = r#"
good_func() { echo "hello $1"; }
bad_func() { echo "world $1"; }
good_func "arg"
"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
        assert!(result.diagnostics[0].message.contains("bad_func"));
    }

    #[test]
    fn test_sc2120_called_without_and_with_args() {
        let code = r#"
my_func() { echo "$1"; }
my_func
my_func "arg"
"#;
        let result = check(code);
        // Called with args at least once, so OK
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2120_function_call_with_pipe() {
        let code = r#"
my_func() { echo "$1"; }
my_func | grep test
"#;
        let result = check(code);
        // Called without args (piped), should warn
        assert_eq!(result.diagnostics.len(), 1);
    }
}
