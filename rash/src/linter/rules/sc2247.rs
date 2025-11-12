// SC2247: Multiplying strings doesn't work - use repetition
use crate::linter::{Diagnostic, LintResult, Severity, Span};
use once_cell::sync::Lazy;
use regex::Regex;

static STRING_MULTIPLY: Lazy<Regex> = Lazy::new(|| {
    // Match: "string" * number or $var * number in non-arithmetic context
    Regex::new(r#"(["'][\w\s]+['"]|\$\w+)\s*\*\s*\d+"#).unwrap()
});

pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();
    for (line_num, line) in source.lines().enumerate() {
        let line_num = line_num + 1;
        if line.trim_start().starts_with('#') {
            continue;
        }

        // Skip if inside $(( )) or (( ))
        if line.contains("((") {
            continue;
        }

        // Skip if inside awk command or bc pipeline (issue #22)
        // awk and bc use * for legitimate mathematical operations
        if line.contains("awk") || line.contains("| bc") || line.contains("|bc") {
            continue;
        }

        // Skip expr command (already handled but making explicit)
        if line.contains("expr") {
            continue;
        }

        if STRING_MULTIPLY.is_match(line) {
            let diagnostic = Diagnostic::new(
                "SC2247",
                Severity::Error,
                "Multiplying strings doesn't work in shell. Use printf or a loop for repetition"
                    .to_string(),
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
    fn test_sc2247_string_multiply() {
        let code = r#"echo "x" * 5"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }
    #[test]
    fn test_sc2247_var_multiply() {
        let code = "result=$str * 3";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }
    #[test]
    fn test_sc2247_arithmetic_ok() {
        let code = "result=$(( num * 5 ))";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }
    #[test]
    fn test_sc2247_double_paren_ok() {
        let code = "(( count * 10 ))";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }
    #[test]
    fn test_sc2247_comment_skipped() {
        let code = r#"# echo "x" * 5"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }
    #[test]
    fn test_sc2247_multiplication_symbol() {
        let code = "echo test * file";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0); // Not string multiplication
    }
    #[test]
    fn test_sc2247_no_code() {
        let code = "";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }
    #[test]
    fn test_sc2247_normal_command() {
        let code = "echo test";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }
    #[test]
    fn test_sc2247_glob_pattern() {
        let code = "ls *.txt";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }
    #[test]
    fn test_sc2247_expr_command() {
        let code = "expr 3 * 4";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_issue_22_bc_multiplication() {
        // Reproduce issue #22: Math operations in bc should not trigger SC2247
        let code = r#"PERCENTAGE=$(echo "scale=1; $VALUE * 100" | bc)"#;
        let result = check(code);
        assert_eq!(
            result.diagnostics.len(),
            0,
            "bc math expressions should not trigger SC2247"
        );
    }

    #[test]
    fn test_issue_22_awk_multiplication() {
        // Reproduce issue #22: Math operations in awk should not trigger SC2247
        let code = r#"PERCENTAGE=$(awk "BEGIN {printf \"%.1f\", $VALUE * 100}")"#;
        let result = check(code);
        assert_eq!(
            result.diagnostics.len(),
            0,
            "awk math expressions should not trigger SC2247"
        );
    }

    #[test]
    fn test_issue_22_bc_in_pipeline() {
        let code = r#"result=$(echo "$num * 5" | bc)"#;
        let result = check(code);
        assert_eq!(
            result.diagnostics.len(),
            0,
            "bc pipeline math should not trigger SC2247"
        );
    }

    #[test]
    fn test_issue_22_awk_printf_multiplication() {
        let code = r#"awk '{print $1 * 100}' file.txt"#;
        let result = check(code);
        assert_eq!(
            result.diagnostics.len(),
            0,
            "awk multiplication should not trigger SC2247"
        );
    }

    #[test]
    fn test_sc2247_still_detects_real_errors() {
        // Ensure we still detect actual string multiplication errors
        let code = r#"result="hello" * 5"#;
        let result = check(code);
        assert_eq!(
            result.diagnostics.len(),
            1,
            "Should still detect string multiplication"
        );
    }

    #[test]
    fn test_sc2247_real_error_not_in_awk_or_bc() {
        // Real error outside awk/bc context
        let code = r#"
echo "test"
count=$num * 10
echo "done"
"#;
        let result = check(code);
        assert_eq!(
            result.diagnostics.len(),
            1,
            "Should detect string multiplication outside awk/bc"
        );
    }
}
