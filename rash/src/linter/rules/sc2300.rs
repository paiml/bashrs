// SC2300: Use ${var:?} for required environment variables
use crate::linter::{Diagnostic, LintResult, Severity, Span};
use once_cell::sync::Lazy;
use regex::Regex;

static UNCHECKED_ENV: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"=\s*\$\{?[A-Z_][A-Z0-9_]*\}?\s*$").unwrap()
});

pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();
    for (line_num, line) in source.lines().enumerate() {
        let line_num = line_num + 1;
        if line.trim_start().starts_with('#') {
            continue;
        }

        // Check for environment variable usage without error handling
        if line.contains("export") || line.starts_with("export") {
            continue; // Skip export statements
        }

        if UNCHECKED_ENV.is_match(line) && !line.contains(":?") {
            let diagnostic = Diagnostic::new(
                "SC2300",
                Severity::Info,
                "Consider using ${VAR:?} to ensure required environment variables are set".to_string(),
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
    fn test_sc2300_unchecked_env() {
        let code = "path=$HOME";
        assert_eq!(check(code).diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2300_with_check_ok() {
        let code = "path=${HOME:?}";
        assert_eq!(check(code).diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2300_export_ok() {
        let code = "export PATH=$PATH";
        assert_eq!(check(code).diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2300_comment() {
        let code = "# x=$HOME";
        assert_eq!(check(code).diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2300_empty() {
        assert_eq!(check("").diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2300_normal() {
        assert_eq!(check("echo test").diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2300_lowercase_var_ok() {
        let code = "path=$home";
        assert_eq!(check(code).diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2300_with_default_ok() {
        let code = "path=${HOME:-/default}";
        assert_eq!(check(code).diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2300_config_var() {
        let code = "db=$DATABASE_URL";
        assert_eq!(check(code).diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2300_in_command_ok() {
        let code = "echo $HOME";
        assert_eq!(check(code).diagnostics.len(), 0);
    }
}
