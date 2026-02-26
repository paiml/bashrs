//! SC2116: Useless echo wrapping in command substitution
//!
//! Detects patterns like $(echo $var) which can be simplified to just $var.
//!
//! References:
//! - https://www.shellcheck.net/wiki/SC2116

use crate::linter::{Diagnostic, Fix, LintResult, Severity, Span};
use regex::Regex;

/// Check for useless echo in command substitutions (SC2116)
pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();

    // Pattern: $(echo SOMETHING) where SOMETHING doesn't need echo
    let pattern = Regex::new(r"\$\(\s*echo\s+(?P<flags>-[a-z]+\s+)?(?P<content>[^)]+)\)").unwrap();

    for (line_num, line) in source.lines().enumerate() {
        let line_num = line_num + 1;

        // Skip comments
        if line.trim_start().starts_with('#') {
            continue;
        }

        for cap in pattern.captures_iter(line) {
            let full_match = cap.get(0).unwrap();

            // If echo has flags (-n, -e, etc.), it's NOT useless
            if cap.name("flags").is_some() {
                continue;
            }

            let content = cap.name("content").unwrap().as_str().trim();

            // Skip if content has a pipe - this is a pipeline, not useless echo
            // Example: $(echo "$x" | cut -d. -f1) is NOT useless
            if content.contains('|') {
                continue;
            }

            let col = full_match.start() + 1; // 1-indexed
            let end_col = full_match.end() + 1; // 1-indexed (after last char)

            let span = Span::new(line_num, col, line_num, end_col);

            // Simple fix: just use the content directly
            let fix = Fix::new(content.to_string());

            let diag = Diagnostic::new(
                "SC2116",
                Severity::Info,
                format!("Useless echo; just use {} directly", content),
                span,
            )
            .with_fix(fix);

            result.add(diag);
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sc2116_basic_detection() {
        let bash_code = "result=$(echo $var)";
        let result = check(bash_code);

        assert_eq!(result.diagnostics.len(), 1);
        assert_eq!(result.diagnostics[0].code, "SC2116");
        assert!(result.diagnostics[0].message.contains("Useless echo"));
    }

    #[test]
    fn test_sc2116_autofix() {
        let bash_code = "result=$(echo $var)";
        let result = check(bash_code);

        assert!(result.diagnostics[0].fix.is_some());
        assert_eq!(
            result.diagnostics[0].fix.as_ref().unwrap().replacement,
            "$var"
        );
    }

    #[test]
    fn test_sc2116_false_positive_with_flags() {
        let bash_code = "result=$(echo -n $var)";
        let result = check(bash_code);

        // Should NOT trigger - echo has flags
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2116_with_literal() {
        let bash_code = "result=$(echo hello)";
        let result = check(bash_code);

        assert_eq!(result.diagnostics.len(), 1);
        assert_eq!(
            result.diagnostics[0].fix.as_ref().unwrap().replacement,
            "hello"
        );
    }

    #[test]
    fn test_sc2116_severity() {
        let bash_code = "result=$(echo $var)";
        let result = check(bash_code);

        assert_eq!(result.diagnostics[0].severity, Severity::Info);
    }

    #[test]
    fn test_sc2116_skip_comments() {
        let bash_code = "# result=$(echo $var)";
        let result = check(bash_code);

        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2116_skip_pipelines() {
        // Should NOT trigger when echo is part of a pipeline
        let bash_code = r#"val=$(echo "$x" | cut -d. -f1)"#;
        let result = check(bash_code);

        assert_eq!(
            result.diagnostics.len(),
            0,
            "Should not trigger on pipelines"
        );
    }
}
