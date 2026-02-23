// SC2247: Multiplying strings doesn't work - use repetition
use crate::linter::{Diagnostic, LintResult, Severity, Span};
use once_cell::sync::Lazy;
use regex::Regex;

static STRING_MULTIPLY: Lazy<Regex> = Lazy::new(|| {
    // Match: "string" * number or $var * number in non-arithmetic context
    Regex::new(r#"(["'][\w\s]+['"]|\$\w+)\s*\*\s*\d+"#).unwrap()
});

/// Extract heredoc delimiter from a line containing "<<"
fn extract_heredoc_delimiter(line: &str) -> Option<String> {
    let pos = line.find("<<")?;
    let after_heredoc = &line[pos + 2..];
    let delim_start = after_heredoc.trim_start_matches('-');
    let delim: String = delim_start
        .trim_start()
        .trim_start_matches(['\'', '"'])
        .chars()
        .take_while(|c| c.is_alphanumeric() || *c == '_')
        .collect();
    if delim.is_empty() { None } else { Some(delim) }
}

/// Check if a line should be skipped for SC2247 analysis (arithmetic, awk, bc, etc.)
fn should_skip_sc2247(line: &str) -> bool {
    line.trim_start().starts_with('#')
        || line.contains("((")
        || line.contains("awk")
        || line.contains("| bc")
        || line.contains("|bc")
        || line.contains("expr")
        || line.contains("python")
        || line.contains("perl")
        || line.contains("ruby")
}

pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();
    let mut in_heredoc = false;
    let mut heredoc_delimiter: Option<String> = None;

    for (line_num, line) in source.lines().enumerate() {
        let line_num = line_num + 1;

        if in_heredoc {
            if let Some(ref delim) = heredoc_delimiter {
                if line.trim() == delim {
                    in_heredoc = false;
                    heredoc_delimiter = None;
                }
            }
            continue;
        }

        if line.contains("<<") {
            if let Some(delim) = extract_heredoc_delimiter(line) {
                in_heredoc = true;
                heredoc_delimiter = Some(delim);
            }
        }

        if should_skip_sc2247(line) {
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

    // Issue #120: Python in heredoc should NOT be flagged
    #[test]
    fn test_issue_120_python_heredoc_not_flagged() {
        let code = r#"
python3 <<EOF
result = "x" * 10
print(result)
EOF
"#;
        let result = check(code);
        assert_eq!(
            result.diagnostics.len(),
            0,
            "SC2247 must NOT flag Python code in heredoc"
        );
    }

    #[test]
    fn test_issue_120_ruby_heredoc_not_flagged() {
        let code = r#"
ruby <<RUBY
puts "hello" * 3
RUBY
"#;
        let result = check(code);
        assert_eq!(
            result.diagnostics.len(),
            0,
            "SC2247 must NOT flag Ruby code in heredoc"
        );
    }

    #[test]
    fn test_issue_120_perl_heredoc_not_flagged() {
        let code = r#"
perl <<'PERL'
print "x" * 5;
PERL
"#;
        let result = check(code);
        assert_eq!(
            result.diagnostics.len(),
            0,
            "SC2247 must NOT flag Perl code in heredoc"
        );
    }

    #[test]
    fn test_issue_120_python_command_not_flagged() {
        let code = r#"python3 -c 'print("x" * 10)'"#;
        let result = check(code);
        assert_eq!(
            result.diagnostics.len(),
            0,
            "SC2247 must NOT flag Python -c command"
        );
    }
}
