// SC2112: 'function' keyword is non-standard. Delete it
//
// Even if your shell supports 'function', it's better to use POSIX syntax.
// The 'function' keyword is bash/ksh-specific and not portable.
//
// Examples:
// Bad:
//   #!/bin/bash
//   function myfunc { echo "test"; }
//
// Good:
//   #!/bin/bash
//   myfunc() { echo "test"; }
//
// Impact: Portability - script won't work in POSIX sh

use crate::linter::{Diagnostic, LintResult, Severity, Span};
use regex::Regex;

static FUNCTION_KEYWORD_ANY: std::sync::LazyLock<Regex> = std::sync::LazyLock::new(|| {
    // Match: function name (no parens) or function name()
    Regex::new(r"\bfunction\s+([a-zA-Z_][a-zA-Z0-9_]*)").unwrap()
});

pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();

    // Check if script has bash/ksh shebang
    let has_bash_shebang = source.lines().next().is_some_and(|line| {
        line.starts_with("#!/bin/bash")
            || line.starts_with("#!/usr/bin/bash")
            || line.starts_with("#!/bin/ksh")
            || line.starts_with("#!/usr/bin/ksh")
    });

    for (line_num, line) in source.lines().enumerate() {
        let line_num = line_num + 1;

        if line.trim_start().starts_with('#') {
            continue;
        }

        for mat in FUNCTION_KEYWORD_ANY.find_iter(line) {
            let start_col = mat.start() + 1;
            let end_col = mat.start() + 9; // length of "function "

            let message = if has_bash_shebang {
                "'function' keyword is non-standard. Delete it for better portability".to_string()
            } else {
                "'function' keyword is non-standard. Delete it".to_string()
            };

            let diagnostic = Diagnostic::new(
                "SC2112",
                Severity::Info,
                message,
                Span::new(line_num, start_col, line_num, end_col),
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
    fn test_sc2112_function_keyword() {
        let code = "function foo { echo \"bar\"; }";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2112_bash_shebang() {
        let code = r#"#!/bin/bash
function deploy { echo "test"; }
"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
        assert!(result.diagnostics[0].message.contains("better portability"));
    }

    #[test]
    fn test_sc2112_sh_shebang() {
        let code = r#"#!/bin/sh
function deploy { echo "test"; }
"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
        assert!(!result.diagnostics[0].message.contains("better portability"));
    }

    #[test]
    fn test_sc2112_posix_ok() {
        let code = "foo() { echo \"bar\"; }";
        let result = check(code);
        // POSIX style is OK
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2112_comment_ok() {
        let code = "# function foo { echo \"bar\"; }";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2112_multiple() {
        let code = r#"
function foo { echo "foo"; }
function bar { echo "bar"; }
"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 2);
    }

    #[test]
    fn test_sc2112_with_parens() {
        let code = "function test() { echo \"test\"; }";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2112_multiline() {
        let code = r#"
function process_data {
    local input=$1
    echo "$input"
}
"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2112_ksh_shebang() {
        let code = r#"#!/bin/ksh
function myfunc { echo "ksh"; }
"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2112_no_shebang() {
        let code = "function helper { echo \"help\"; }";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }
}
