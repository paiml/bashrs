// SC2324: Use ${var:+value} for conditional value based on isset
use crate::linter::{Diagnostic, LintResult, Severity, Span};
use once_cell::sync::Lazy;
use regex::Regex;

static ISSET_CONDITIONAL: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"\[\[\s+-v\s+[a-zA-Z_][a-zA-Z0-9_]*\s+\]\]\s*&&\s*echo").unwrap()
});

pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();
    for (line_num, line) in source.lines().enumerate() {
        let line_num = line_num + 1;
        if line.trim_start().starts_with('#') {
            continue;
        }

        if ISSET_CONDITIONAL.is_match(line) {
            let diagnostic = Diagnostic::new(
                "SC2324",
                Severity::Info,
                "Consider using ${var:+value} for conditional output based on variable being set".to_string(),
                Span::new(line_num, 1, line_num, line.len() + 1),
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
    fn test_sc2324_isset_echo() {
        let code = "[[ -v VAR ]] && echo \"set\"";
        assert_eq!(check(code).diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2324_expansion_ok() {
        let code = "echo ${VAR:+set}";
        assert_eq!(check(code).diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2324_non_echo_ok() {
        let code = "[[ -v VAR ]] && exit 1";
        assert_eq!(check(code).diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2324_comment() {
        let code = "# [[ -v X ]] && echo y";
        assert_eq!(check(code).diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2324_empty() {
        assert_eq!(check("").diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2324_normal() {
        assert_eq!(check("echo test").diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2324_not_v_ok() {
        let code = "[[ ! -v VAR ]] && echo \"unset\"";
        assert_eq!(check(code).diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2324_n_test_ok() {
        let code = "[[ -n $VAR ]] && echo \"nonempty\"";
        assert_eq!(check(code).diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2324_multiple_vars() {
        let code = r#"
[[ -v X ]] && echo "x"
[[ -v Y ]] && echo "y"
"#;
        assert_eq!(check(code).diagnostics.len(), 2);
    }

    #[test]
    fn test_sc2324_single_bracket_ok() {
        let code = "[ -v VAR ] && echo \"set\"";
        assert_eq!(check(code).diagnostics.len(), 0);
    }
}
