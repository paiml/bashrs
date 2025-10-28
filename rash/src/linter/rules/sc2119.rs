// SC2119: Use foo "$@" if function's $1 should mean script's $1
//
// When a function is called with arguments but doesn't reference $1, $2, etc.,
// it's likely that either the function call is wrong (passing unnecessary args)
// or the function should be using the arguments.
//
// Examples:
// Bad:
//   my_func() { echo "hello"; }
//   my_func "arg1" "arg2"  # Function doesn't use arguments
//
// Good:
//   my_func() { echo "hello $1"; }
//   my_func "arg1"  # Function uses arguments
//   my_func  # No arguments passed

use crate::linter::{Diagnostic, LintResult, Severity, Span};
use once_cell::sync::Lazy;
use regex::Regex;
use std::collections::{HashMap, HashSet};

static FUNCTION_DEF: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"([A-Za-z_][A-Za-z0-9_]*)\s*\(\s*\)\s*\{").unwrap()
});

static FUNCTION_CALL: Lazy<Regex> = Lazy::new(|| {
    // Match function calls with arguments
    Regex::new(r"^([A-Za-z_][A-Za-z0-9_]*)\s+[^;\|&<>]+").unwrap()
});

static ARG_REFERENCE: Lazy<Regex> = Lazy::new(|| {
    // Match $1, $2, $@, $*, etc.
    Regex::new(r"\$[@*#]|\$\{?[0-9]+\}?").unwrap()
});

/// Check if line is a comment
fn is_comment(line: &str) -> bool {
    line.trim().starts_with('#')
}

/// Update brace depth counter
fn update_brace_depth(line: &str, depth: usize) -> usize {
    let depth = depth + line.matches('{').count();
    depth.saturating_sub(line.matches('}').count())
}

/// Check if line contains argument references
fn has_arg_reference(line: &str) -> bool {
    ARG_REFERENCE.is_match(line)
}

/// Mark function as using arguments
fn mark_function_uses_args(
    functions: &mut HashMap<String, bool>,
    in_function: &Option<String>,
) {
    if let Some(ref func) = in_function {
        functions.insert(func.clone(), true);
    }
}

/// Find all functions and track which use positional parameters
fn find_functions_using_args(lines: &[&str]) -> HashMap<String, bool> {
    let mut functions_use_args: HashMap<String, bool> = HashMap::new();
    let mut in_function: Option<String> = None;
    let mut brace_depth = 0;

    for line in lines {
        if is_comment(line) {
            continue;
        }

        let trimmed = line.trim();

        if let Some(cap) = FUNCTION_DEF.captures(trimmed) {
            let func_name = cap.get(1).unwrap().as_str().to_string();
            in_function = Some(func_name.clone());
            functions_use_args.insert(func_name, false);
            brace_depth = 1;
            continue;
        }

        if in_function.is_some() {
            brace_depth = update_brace_depth(line, brace_depth);

            if brace_depth == 0 {
                in_function = None;
            } else if has_arg_reference(line) {
                mark_function_uses_args(&mut functions_use_args, &in_function);
            }
        }
    }

    functions_use_args
}

/// Build diagnostic for function call with unused arguments
fn build_diagnostic(func_name: &str, line_num: usize, start_col: usize, end_col: usize) -> Diagnostic {
    Diagnostic::new(
        "SC2119",
        Severity::Info,
        format!(
            "Use {} \"$@\" if function's $1 should mean script's $1",
            func_name
        ),
        Span::new(line_num, start_col, line_num, end_col),
    )
}

pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();
    let lines: Vec<&str> = source.lines().collect();

    let functions_use_args = find_functions_using_args(&lines);

    for (line_num, line) in lines.iter().enumerate() {
        let line_num = line_num + 1;
        let trimmed = line.trim();

        if is_comment(trimmed) || FUNCTION_DEF.is_match(trimmed) {
            continue;
        }

        if let Some(cap) = FUNCTION_CALL.captures(trimmed) {
            let func_name = cap.get(1).unwrap().as_str();

            if let Some(&uses_args) = functions_use_args.get(func_name) {
                if !uses_args {
                    let start_col = cap.get(0).unwrap().start() + 1;
                    let end_col = cap.get(0).unwrap().end() + 1;
                    let diagnostic = build_diagnostic(func_name, line_num, start_col, end_col);
                    result.add(diagnostic);
                }
            }
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sc2119_call_with_args_but_no_use() {
        let code = r#"
my_func() { echo "hello"; }
my_func "arg1" "arg2"
"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
        assert_eq!(result.diagnostics[0].code, "SC2119");
        assert_eq!(result.diagnostics[0].severity, Severity::Info);
        assert!(result.diagnostics[0].message.contains("my_func"));
    }

    #[test]
    fn test_sc2119_function_uses_args_ok() {
        let code = r#"
my_func() { echo "hello $1"; }
my_func "arg1"
"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2119_no_args_passed_ok() {
        let code = r#"
my_func() { echo "hello"; }
my_func
"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2119_function_uses_at_ok() {
        let code = r#"
my_func() { echo "$@"; }
my_func "arg1" "arg2"
"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2119_function_uses_star_ok() {
        let code = r#"
my_func() { echo "$*"; }
my_func "arg1"
"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2119_function_uses_numbered_param_ok() {
        let code = r#"
my_func() { local val="$2"; echo "$val"; }
my_func "a" "b"
"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2119_multiple_calls() {
        let code = r#"
bad_func() { echo "no args used"; }
bad_func "arg1"
bad_func "arg2" "arg3"
"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 2);
    }

    #[test]
    fn test_sc2119_nested_functions() {
        let code = r#"
outer() {
    inner() { echo "nested"; }
    inner "arg"
}
"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2119_function_undefined_ok() {
        let code = r#"
undefined_func "arg1"
"#;
        let result = check(code);
        // Function not defined in this file, no warning
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2119_braces_in_param_ref() {
        let code = r#"
my_func() { echo "${1}"; }
my_func "arg"
"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }
}
