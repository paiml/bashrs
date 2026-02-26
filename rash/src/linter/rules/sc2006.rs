//! SC2006: Use $(...) instead of legacy backticks
//!
//! # Examples
//!
//! Bad:
//! ```bash
//! result=`date`
//! files=`ls *.txt`
//! ```
//!
//! Good:
//! ```bash
//! result=$(date)
//! files=$(ls *.txt)
//! ```
//!
//! # Rationale
//!
//! Backticks are the legacy syntax for command substitution. Modern `$(...)` syntax:
//! - Is easier to nest
//! - Is more readable
//! - Is POSIX compliant
//!
//! # Auto-fix
//!
//! Replace backticks with `$(...)`: `` `cmd` `` → `$(cmd)`

use crate::linter::{Diagnostic, Fix, LintResult, Severity, Span};
use regex::Regex;
use std::collections::HashSet;

/// Regex to detect single-quoted heredoc: << 'DELIM' or <<- 'DELIM'
#[allow(clippy::expect_used)] // Compile-time regex, panic on invalid pattern is acceptable
static HEREDOC_SINGLE_QUOTED: std::sync::LazyLock<Regex> = std::sync::LazyLock::new(|| {
    Regex::new(r"<<-?\s*'(\w+)'").expect("valid single-quoted heredoc regex")
});

/// Regex to detect double-quoted heredoc: << "DELIM" or <<- "DELIM"
#[allow(clippy::expect_used)] // Compile-time regex, panic on invalid pattern is acceptable
static HEREDOC_DOUBLE_QUOTED: std::sync::LazyLock<Regex> = std::sync::LazyLock::new(|| {
    Regex::new(r#"<<-?\s*"(\w+)""#).expect("valid double-quoted heredoc regex")
});

/// Regex to detect backtick command substitution: `command`
#[allow(clippy::expect_used)] // Compile-time regex, panic on invalid pattern is acceptable
static BACKTICK_PATTERN: std::sync::LazyLock<Regex> =
    std::sync::LazyLock::new(|| Regex::new(r"`([^`]+)`").expect("valid backtick regex"));

/// F080: Regex to detect assignment context: var=`cmd` or local/export/readonly var=`cmd`
#[allow(clippy::expect_used)] // Compile-time regex, panic on invalid pattern is acceptable
static ASSIGNMENT_BACKTICK: std::sync::LazyLock<Regex> = std::sync::LazyLock::new(|| {
    Regex::new(
        r"(?:^|;|\s)(?:local|export|readonly|declare|typeset)?\s*[A-Za-z_][A-Za-z0-9_]*=`[^`]+`",
    )
    .expect("valid assignment backtick regex")
});

/// Collect line numbers inside a heredoc body (from start_idx+1 until delimiter is found)
fn collect_heredoc_body_lines(
    lines: &[&str],
    start_idx: usize,
    delimiter: &str,
    quoted_lines: &mut HashSet<usize>,
) {
    for (inner_idx, inner_line) in lines.iter().enumerate().skip(start_idx + 1) {
        if inner_line.trim() == delimiter {
            break;
        }
        quoted_lines.insert(inner_idx + 1);
    }
}

/// Issue #96: Parse heredoc regions and return line numbers inside quoted heredocs
/// Quoted heredocs (single or double quoted delimiter) have literal content - no expansion
fn get_quoted_heredoc_lines(source: &str) -> HashSet<usize> {
    let mut quoted_lines = HashSet::new();
    let lines: Vec<&str> = source.lines().collect();

    for (idx, line) in lines.iter().enumerate() {
        for pattern in &[&*HEREDOC_SINGLE_QUOTED, &*HEREDOC_DOUBLE_QUOTED] {
            if let Some(caps) = pattern.captures(line) {
                if let Some(delim) = caps.get(1) {
                    collect_heredoc_body_lines(&lines, idx, delim.as_str(), &mut quoted_lines);
                }
            }
        }
    }

    quoted_lines
}

/// Check for deprecated backtick command substitution
pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();

    // Issue #96: Get lines inside quoted heredocs (content is literal, not expanded)
    let quoted_heredoc_lines = get_quoted_heredoc_lines(source);

    for (line_num, line) in source.lines().enumerate() {
        let line_num = line_num + 1;

        // Skip comment lines
        if line.trim_start().starts_with('#') {
            continue;
        }

        // Issue #96: Skip lines inside quoted heredocs
        if quoted_heredoc_lines.contains(&line_num) {
            continue;
        }

        // F080: Skip lines where backticks are in assignment context
        // Legacy backticks in assignments are intentional - user knows what they're doing
        if ASSIGNMENT_BACKTICK.is_match(line) {
            continue;
        }

        for cap in BACKTICK_PATTERN.captures_iter(line) {
            let full_match = cap.get(0).unwrap();
            let command = cap.get(1).unwrap().as_str();

            let start_col = full_match.start() + 1;
            let end_col = full_match.end() + 1;

            let fix_text = format!("$({})", command);

            let diagnostic = Diagnostic::new(
                "SC2006",
                Severity::Info,
                "Use $(...) instead of deprecated backticks",
                Span::new(line_num, start_col, line_num, end_col),
            )
            .with_fix(Fix::new(fix_text));

            result.add(diagnostic);
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    // F080: Backticks in assignments are intentional - not flagged
    #[test]
    fn test_sc2006_assignment_not_flagged() {
        let script = r#"result=`date`"#;
        let result = check(script);
        assert_eq!(
            result.diagnostics.len(),
            0,
            "F080: Backticks in assignments are intentional, not flagged"
        );
    }

    #[test]
    fn test_sc2006_autofix_non_assignment() {
        // Test autofix on non-assignment context (echo)
        let script = r#"echo `date`"#;
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 1);
        assert!(result.diagnostics[0].fix.is_some());
        assert_eq!(
            result.diagnostics[0].fix.as_ref().unwrap().replacement,
            "$(date)"
        );
    }

    #[test]
    fn test_sc2006_ls_assignment_not_flagged() {
        let script = r#"files=`ls *.txt`"#;
        let result = check(script);
        assert_eq!(
            result.diagnostics.len(),
            0,
            "F080: Backticks in assignments are intentional"
        );
    }

    #[test]
    fn test_sc2006_command_with_args_assignment_not_flagged() {
        let script = r#"output=`grep "pattern" file.txt`"#;
        let result = check(script);
        assert_eq!(
            result.diagnostics.len(),
            0,
            "F080: Backticks in assignments are intentional"
        );
    }

    #[test]
    fn test_sc2006_false_positive_modern_syntax() {
        let script = r#"result=$(date)"#;
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2006_false_positive_in_comment() {
        let script = r#"# This is a comment with `backticks`"#;
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2006_multiple_assignments_not_flagged() {
        // F080: Multiple assignments with backticks - all intentional
        let script = r#"
a=`cmd1`
b=`cmd2`
c=`cmd3`
"#;
        let result = check(script);
        assert_eq!(
            result.diagnostics.len(),
            0,
            "F080: Backticks in assignments are intentional"
        );
    }

    #[test]
    fn test_sc2006_multiple_non_assignments_flagged() {
        // Non-assignment backticks should still be flagged
        let script = r#"
echo `cmd1`
echo `cmd2`
echo `cmd3`
"#;
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 3);
    }

    #[test]
    fn test_sc2006_in_if_statement() {
        let script = r#"if [ "`whoami`" = "root" ]; then echo "root"; fi"#;
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2006_echo_statement() {
        let script = r#"echo "Current date: `date`""#;
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2006_assignment_in_function_not_flagged() {
        // F080: Assignment inside function is still intentional
        let script = r#"
function get_time() {
    time=`date +%H:%M:%S`
    echo "$time"
}
"#;
        let result = check(script);
        assert_eq!(
            result.diagnostics.len(),
            0,
            "F080: Backticks in function assignments are intentional"
        );
    }

    // ===== Issue #96: Heredoc Tests =====
    // Backticks inside quoted heredocs are literal, not command substitution

    #[test]
    fn test_FP_096_single_quoted_heredoc_not_flagged() {
        // Single-quoted delimiter means content is literal - no expansion
        let script = "cat << 'EOF'\n`date`\nEOF";
        let result = check(script);
        assert_eq!(
            result.diagnostics.len(),
            0,
            "SC2006 must NOT flag backticks inside single-quoted heredoc"
        );
    }

    #[test]
    fn test_FP_096_double_quoted_heredoc_not_flagged() {
        // Double-quoted delimiter also treats content as literal
        let script = "cat << \"EOF\"\n`whoami`\nEOF";
        let result = check(script);
        assert_eq!(
            result.diagnostics.len(),
            0,
            "SC2006 must NOT flag backticks inside double-quoted heredoc"
        );
    }

    #[test]
    fn test_FP_096_unquoted_heredoc_still_flagged() {
        // Unquoted delimiter means content CAN have expansion - should flag
        let script = "cat << EOF\n`date`\nEOF";
        let result = check(script);
        assert_eq!(
            result.diagnostics.len(),
            1,
            "SC2006 SHOULD flag backticks inside unquoted heredoc"
        );
    }

    #[test]
    fn test_FP_096_markdown_backticks_not_flagged() {
        // Markdown backticks in heredocs should not be flagged
        let script = "cat << 'EOF'\n| `config.json` | Configuration |\nEOF";
        let result = check(script);
        assert_eq!(
            result.diagnostics.len(),
            0,
            "SC2006 must NOT flag markdown backticks in quoted heredoc"
        );
    }

    // ===== F080: Backticks in assignments =====
    // Legacy backticks in assignments are intentional - user knows what they're doing
    // SC2006 should NOT flag assignments, only other contexts (echo, if, etc.)

    #[test]
    fn test_FP_080_simple_assignment_not_flagged() {
        let script = r#"x=`cmd`"#;
        let result = check(script);
        assert_eq!(
            result.diagnostics.len(),
            0,
            "F080: Simple assignment with backticks must NOT be flagged"
        );
    }

    #[test]
    fn test_FP_080_local_assignment_not_flagged() {
        let script = r#"local x=`cmd`"#;
        let result = check(script);
        assert_eq!(
            result.diagnostics.len(),
            0,
            "F080: local assignment with backticks must NOT be flagged"
        );
    }

    #[test]
    fn test_FP_080_export_assignment_not_flagged() {
        let script = r#"export x=`cmd`"#;
        let result = check(script);
        assert_eq!(
            result.diagnostics.len(),
            0,
            "F080: export assignment with backticks must NOT be flagged"
        );
    }

    #[test]
    fn test_FP_080_readonly_assignment_not_flagged() {
        let script = r#"readonly x=`cmd`"#;
        let result = check(script);
        assert_eq!(
            result.diagnostics.len(),
            0,
            "F080: readonly assignment with backticks must NOT be flagged"
        );
    }

    #[test]
    fn test_FP_080_non_assignment_still_flagged() {
        // Backticks in non-assignment context should still be flagged
        let script = r#"echo `date`"#;
        let result = check(script);
        assert_eq!(
            result.diagnostics.len(),
            1,
            "Backticks in non-assignment context SHOULD be flagged"
        );
    }

    #[test]
    fn test_FP_080_if_condition_still_flagged() {
        // Backticks in if condition should still be flagged
        let script = r#"if [ "`whoami`" = "root" ]; then echo hi; fi"#;
        let result = check(script);
        assert_eq!(
            result.diagnostics.len(),
            1,
            "Backticks in if condition SHOULD be flagged"
        );
    }
}
