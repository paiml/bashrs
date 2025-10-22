// SC2224: This function was already defined
use crate::linter::{Diagnostic, LintResult, Severity, Span};
use once_cell::sync::Lazy;
use regex::Regex;
use std::collections::HashMap;

static FUNCTION_DEF: Lazy<Regex> = Lazy::new(|| {
    // Match: function name { or name() {
    Regex::new(r"^\s*(?:function\s+(\w+)|(\w+)\s*\(\s*\))").unwrap()
});

pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();
    let mut defined_functions: HashMap<String, usize> = HashMap::new();

    for (line_num, line) in source.lines().enumerate() {
        let line_num = line_num + 1;
        if line.trim_start().starts_with('#') {
            continue;
        }

        if let Some(caps) = FUNCTION_DEF.captures(line) {
            // Check group 1 (function name) or group 2 (name())
            let func_name = caps.get(1).or(caps.get(2));
            if let Some(func_name) = func_name {
                let name = func_name.as_str().to_string();

                if let Some(&first_def_line) = defined_functions.get(&name) {
                    let diagnostic = Diagnostic::new(
                        "SC2224",
                        Severity::Warning,
                        format!(
                            "Function '{}' was already defined on line {}",
                            name, first_def_line
                        ),
                        Span::new(line_num, 1, line_num, line.len() + 1),
                    );
                    result.add(diagnostic);
                } else {
                    defined_functions.insert(name, line_num);
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
    fn test_sc2224_duplicate_function() {
        let code = "foo() { echo 1; }\nfoo() { echo 2; }";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
        assert!(result.diagnostics[0].message.contains("line 1"));
    }
    #[test]
    fn test_sc2224_single_function_ok() {
        let code = "foo() { echo bar; }";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }
    #[test]
    fn test_sc2224_different_functions_ok() {
        let code = "foo() { echo 1; }\nbar() { echo 2; }";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }
    #[test]
    fn test_sc2224_with_function_keyword() {
        let code = "function test { echo 1; }\ntest() { echo 2; }";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }
    #[test]
    fn test_sc2224_triple_definition() {
        let code = "foo() { :; }\nfoo() { :; }\nfoo() { :; }";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 2); // Second and third defs
    }
    #[test]
    fn test_sc2224_comment_skipped() {
        let code = "foo() { :; }\n# foo() { :; }";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }
    #[test]
    fn test_sc2224_underscore_names() {
        let code = "my_func() { :; }\nmy_func() { :; }";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }
    #[test]
    fn test_sc2224_no_code() {
        let code = "";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }
    #[test]
    fn test_sc2224_normal_commands() {
        let code = "echo test\nls -la";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }
    #[test]
    fn test_sc2224_multiline_function() {
        let code = "foo() {\n  echo 1\n}\nfoo() {\n  echo 2\n}";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }
}
