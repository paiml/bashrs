// SC2249: Consider adding default case to case statement
use crate::linter::{Diagnostic, LintResult, Severity, Span};

pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();
    let full_source = source.to_string();

    // Find all "case " statements in source
    let mut pos = 0;
    while let Some(case_pos) = full_source[pos..].find("case ") {
        let abs_pos = pos + case_pos;
        let line_num = full_source[..abs_pos].matches('\n').count() + 1;

        // Skip if in a comment (check if line starts with #)
        let line_start = full_source[..abs_pos].rfind('\n').map_or(0, |p| p + 1);
        let line_prefix = &full_source[line_start..abs_pos];
        if line_prefix.trim().starts_with('#') {
            pos = abs_pos + 5;
            continue;
        }

        // Find corresponding esac
        if let Some(esac_pos) = full_source[abs_pos..].find("esac") {
            let case_block = &full_source[abs_pos..abs_pos + esac_pos + 4];

            // Check if it has a default pattern (*)
            let has_default = case_block.contains("*)");

            if !has_default {
                let diagnostic = Diagnostic::new(
                    "SC2249",
                    Severity::Info,
                    "Consider adding a *) default case to handle unexpected values".to_string(),
                    Span::new(line_num, 1, line_num, 10),
                );
                result.add(diagnostic);
            }

            // Move past this case statement
            pos = abs_pos + esac_pos + 4;
        } else {
            break;
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_sc2249_no_default() {
        let code = "case $x in\n  a) echo 1;;\n  b) echo 2;;\nesac";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }
    #[test]
    fn test_sc2249_has_default_ok() {
        let code = "case $x in\n  a) echo 1;;\n  *) echo default;;\nesac";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }
    #[test]
    fn test_sc2249_default_with_action() {
        let code = "case $x in\n  a) echo 1;;\n  *) ;;\nesac";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }
    #[test]
    fn test_sc2249_comment_skipped() {
        let code = "# case $x in a) ;; esac";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }
    #[test]
    fn test_sc2249_single_pattern() {
        let code = "case $x in\n  *) echo all;;\nesac";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }
    #[test]
    fn test_sc2249_multiple_cases() {
        let code = "case $a in\n  x) ;;\nesac\ncase $b in\n  y) ;;\n  *) ;;\nesac";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1); // First case missing default
    }
    #[test]
    fn test_sc2249_no_code() {
        let code = "";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }
    #[test]
    fn test_sc2249_normal_command() {
        let code = "echo test";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }
    #[test]
    fn test_sc2249_inline_case() {
        let code = "case $x in a) echo 1;; b) echo 2;; esac";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }
    #[test]
    fn test_sc2249_nested_case() {
        let code = "case $x in\n  a) case $y in z) ;; esac;;\n  *) ;;\nesac";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1); // Inner case missing default
    }
}
