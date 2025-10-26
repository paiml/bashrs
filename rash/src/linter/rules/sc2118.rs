// SC2118: Ksh-specific feature won't work in sh
//
// Ksh has features not available in POSIX sh, like: [[ ]], $(()), typeset, etc.
// This rule detects ksh array syntax: set -A which is not portable.
//
// Examples:
// Bad (in #!/bin/sh):
//   set -A array value1 value2       // ksh-specific
//
// Good:
//   #!/bin/ksh                        // Declare ksh
//   set -A array value1 value2
//
// Good (POSIX alternative):
//   array="value1 value2"             // Use strings
//
// Impact: Script won't work in POSIX sh

use crate::linter::{Diagnostic, LintResult, Severity, Span};
use once_cell::sync::Lazy;
use regex::Regex;

static SET_A_ARRAY: Lazy<Regex> = Lazy::new(|| {
    // Match: set -A arrayname
    Regex::new(r"\bset\s+-A\s+[a-zA-Z_][a-zA-Z0-9_]*\b").unwrap()
});

pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();

    // Check shebang
    let has_ksh_shebang = source.lines().next().is_some_and(|line| {
        line.starts_with("#!/bin/ksh") || line.starts_with("#!/usr/bin/ksh")
    });

    let has_sh_shebang = source.lines().next().is_some_and(|line| {
        line.starts_with("#!/bin/sh") || line.starts_with("#!/usr/bin/sh")
    });

    for (line_num, line) in source.lines().enumerate() {
        let line_num = line_num + 1;

        if line.trim_start().starts_with('#') {
            continue;
        }

        for mat in SET_A_ARRAY.find_iter(line) {
            let start_col = mat.start() + 1;
            let end_col = mat.end() + 1;

            let message = if has_sh_shebang {
                "set -A is ksh-specific and won't work in sh. Use bash arrays or change shebang"
                    .to_string()
            } else if !has_ksh_shebang {
                "set -A is ksh-specific. Add #!/bin/ksh shebang".to_string()
            } else {
                // Has ksh shebang, this is OK
                continue;
            };

            let diagnostic = Diagnostic::new(
                "SC2118",
                Severity::Error,
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
    fn test_sc2118_set_a_in_sh() {
        let code = r#"#!/bin/sh
set -A myarray value1 value2
"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2118_set_a_no_shebang() {
        let code = "set -A array val1 val2";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2118_set_a_in_ksh_ok() {
        let code = r#"#!/bin/ksh
set -A myarray value1 value2
"#;
        let result = check(code);
        // ksh shebang makes it OK
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2118_comment_ok() {
        let code = "# set -A array value1 value2";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2118_bash_array_ok() {
        let code = r#"#!/bin/bash
array=(value1 value2)
"#;
        let result = check(code);
        // bash arrays are different syntax
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2118_multiple() {
        let code = r#"#!/bin/sh
set -A arr1 val1
set -A arr2 val2
"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 2);
    }

    #[test]
    fn test_sc2118_with_flags() {
        let code = "set -A myarray item1 item2 item3";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2118_underscore_name() {
        let code = "set -A _private_array val";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2118_in_function() {
        let code = r#"
foo() {
    set -A local_array item1 item2
}
"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2118_usr_bin_ksh_ok() {
        let code = r#"#!/usr/bin/ksh
set -A array value
"#;
        let result = check(code);
        // /usr/bin/ksh is also OK
        assert_eq!(result.diagnostics.len(), 0);
    }
}
