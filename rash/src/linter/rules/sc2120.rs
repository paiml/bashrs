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

pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();
    let lines: Vec<&str> = source.lines().collect();

    // Track which functions use arguments and their definition locations
    let mut function_defs: HashMap<String, (usize, bool)> = HashMap::new();  // (line_num, uses_args)
    let mut functions_called_with_args: HashSet<String> = HashSet::new();

    let mut in_function: Option<String> = None;
    let mut func_def_line: usize = 0;
    let mut brace_depth = 0;

    // First pass: Find functions and check if they use positional parameters
    for (idx, line) in lines.iter().enumerate() {
        let trimmed = line.trim();

        if trimmed.starts_with('#') {
            continue;
        }

        // Track function definitions
        if let Some(cap) = FUNCTION_DEF.captures(trimmed) {
            let func_name = cap.get(1).unwrap().as_str().to_string();
            in_function = Some(func_name.clone());
            func_def_line = idx + 1;
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
                // Function uses positional parameters
                if let Some(ref func) = in_function {
                    if let Some(entry) = function_defs.get_mut(func) {
                        entry.1 = true;
                    }
                }
            }
        }
    }

    // Second pass: Find function calls and check if they pass arguments
    for line in &lines {
        let trimmed = line.trim();

        if trimmed.starts_with('#') {
            continue;
        }

        // Skip function definitions
        if FUNCTION_DEF.is_match(trimmed) {
            continue;
        }

        if let Some(cap) = FUNCTION_CALL.captures(trimmed) {
            let func_name = cap.get(1).unwrap().as_str();

            // Check if this is a function call with arguments
            if function_defs.contains_key(func_name) {
                let call_text = trimmed;
                // Simple heuristic: if there's text after the function name, it has args
                if call_text.len() > func_name.len() + 1 {
                    let after_func = &call_text[func_name.len()..].trim_start();
                    if !after_func.is_empty() && !after_func.starts_with(';') &&
                       !after_func.starts_with('|') && !after_func.starts_with('&') &&
                       !after_func.starts_with('<') && !after_func.starts_with('>') {
                        functions_called_with_args.insert(func_name.to_string());
                    }
                }
            }
        }
    }

    // Generate warnings for functions that use args but are never called with them
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
