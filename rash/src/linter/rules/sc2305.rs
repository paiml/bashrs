// SC2305: Use ${var:=value} to assign default value
use crate::linter::{Diagnostic, LintResult, Severity, Span};
use once_cell::sync::Lazy;
use regex::Regex;

static REDUNDANT_DEFAULT: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"\[\s+-z\s+\$\{?([a-zA-Z_][a-zA-Z0-9_]*)\}?\s+\]\s*&&\s*([a-zA-Z_][a-zA-Z0-9_]*)=")
        .unwrap()
});

pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();
    for (line_num, line) in source.lines().enumerate() {
        let line_num = line_num + 1;
        if line.trim_start().starts_with('#') {
            continue;
        }

        if let Some(caps) = REDUNDANT_DEFAULT.captures(line) {
            let test_var = &caps[1];
            let assign_var = &caps[2];

            // Only warn if testing and assigning the same variable
            if test_var == assign_var {
                let diagnostic = Diagnostic::new(
                    "SC2305",
                    Severity::Info,
                    "Use ${var:=default} instead of [ -z $var ] && var=default".to_string(),
                    Span::new(line_num, 1, line_num, line.len() + 1),
                );
                result.add(diagnostic);
            }
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sc2305_redundant_default() {
        let code = "[ -z $var ] && var=default";
        assert_eq!(check(code).diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2305_colon_equals_ok() {
        let code = ": ${var:=default}";
        assert_eq!(check(code).diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2305_simple_assignment_ok() {
        let code = "var=value";
        assert_eq!(check(code).diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2305_comment() {
        let code = "# [ -z $x ] && x=5";
        assert_eq!(check(code).diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2305_empty() {
        assert_eq!(check("").diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2305_normal() {
        assert_eq!(check("echo test").diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2305_n_test_ok() {
        let code = "[ -n $var ] && var=value";
        assert_eq!(check(code).diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2305_braced_var() {
        let code = "[ -z ${config} ] && config=def";
        assert_eq!(check(code).diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2305_different_var_ok() {
        let code = "[ -z $x ] && y=value";
        assert_eq!(check(code).diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2305_with_quotes() {
        let code = r#"[ -z "$var" ] && var=default"#;
        assert_eq!(check(code).diagnostics.len(), 0);
    }
}
