//! DET003: Unordered wildcard usage
//!
//! **Rule**: Detect wildcards without sorting for deterministic results
//!
//! **Why this matters**:
//! File glob results vary by filesystem and can change between runs,
//! breaking determinism.
//!
//! **Auto-fix**: Wrap command substitution with sort (only for $(ls ...) patterns)
//!
//! ## Examples
//!
//! ❌ **BAD** (non-deterministic):
//! ```bash
//! FILES=$(ls *.txt)
//! for f in *.c; do echo $f; done
//! ```
//!
//! ✅ **GOOD** (deterministic):
//! ```bash
//! FILES=$(ls *.txt | sort)
//! for f in $(printf '%s\n' *.c | sort); do echo "$f"; done
//! ```
//!
//! **Note**: For `for f in *.c`, no auto-fix is provided since the correct
//! transformation is complex. Users should manually review.

use crate::linter::{Diagnostic, Fix, LintResult, Severity, Span};

/// Check a `$(ls ...)` pattern for unordered wildcards and emit diagnostic with auto-fix
fn check_ls_wildcard(line: &str, line_num: usize, ls_start: usize, result: &mut LintResult) {
    let after_ls = &line[ls_start..];
    if let Some(close_paren) = find_matching_paren(after_ls) {
        let cmd_sub = &after_ls[..=close_paren];
        if cmd_sub.contains('*') {
            let span = Span::new(
                line_num + 1,
                ls_start + 1,
                line_num + 1,
                ls_start + close_paren + 2,
            );
            let inner = &cmd_sub[2..cmd_sub.len() - 1];
            let fixed = format!("$({} | sort)", inner);
            let diag = Diagnostic::new(
                "DET003",
                Severity::Warning,
                "Unordered wildcard in command substitution - results may vary",
                span,
            )
            .with_fix(Fix::new(fixed));
            result.add(diag);
        }
    }
}

/// Check a `for ... in *` pattern for unordered wildcards (no auto-fix)
fn check_for_loop_wildcard(line: &str, line_num: usize, result: &mut LintResult) {
    if line.contains("for ") && line.contains(" in ") {
        if let Some(col) = line.find('*') {
            let span = Span::new(line_num + 1, col + 1, line_num + 1, col + 2);
            let diag = Diagnostic::new(
                "DET003",
                Severity::Info,
                "Unordered wildcard in for-loop - consider sorting for determinism",
                span,
            );
            result.add(diag);
        }
    }
}

/// Commands where bare wildcard arguments produce non-deterministic ordering.
const WILDCARD_COMMANDS: &[&str] = &[
    "cat",
    "head",
    "tail",
    "wc",
    "grep",
    "sort",
    "diff",
    "echo",
    "ls",
    "md5sum",
    "sha256sum",
    "file",
    "du",
    "stat",
];

/// Check for bare wildcards in command arguments (e.g., `cat *.log`, `wc -l *.txt`).
/// These expand in filesystem order, which is non-deterministic.
fn check_command_wildcard(line: &str, line_num: usize, result: &mut LintResult) {
    let trimmed = line.trim();

    // Skip if wildcard is inside quotes
    if is_wildcard_quoted(trimmed) {
        return;
    }

    // Skip if wildcard is inside command substitution $(...) — handled by check_ls_wildcard
    if is_wildcard_in_cmd_sub(trimmed) {
        return;
    }

    // Check if a known command appears at line start or after a pipe
    let segments: Vec<&str> = trimmed.split('|').collect();
    for segment in &segments {
        let seg = segment.trim();
        for cmd in WILDCARD_COMMANDS {
            if seg.starts_with(cmd) && seg[cmd.len()..].starts_with([' ', '\t']) {
                if let Some(col) = line.find('*') {
                    let span = Span::new(line_num + 1, col + 1, line_num + 1, col + 2);
                    let diag = Diagnostic::new(
                        "DET003",
                        Severity::Info,
                        "Unordered wildcard in command arguments - glob expansion order is non-deterministic",
                        span,
                    );
                    result.add(diag);
                    return;
                }
            }
        }
    }
}

/// Check if the wildcard `*` is inside single or double quotes.
fn is_wildcard_quoted(line: &str) -> bool {
    let mut in_single = false;
    let mut in_double = false;
    for c in line.chars() {
        match c {
            '\'' if !in_double => in_single = !in_single,
            '"' if !in_single => in_double = !in_double,
            '*' if in_single || in_double => return true,
            _ => {}
        }
    }
    false
}

/// Check if the wildcard `*` appears only inside a `$(...)` command substitution.
fn is_wildcard_in_cmd_sub(line: &str) -> bool {
    let mut depth = 0i32;
    let bytes = line.as_bytes();
    for i in 0..bytes.len() {
        if i > 0 && bytes[i - 1] == b'$' && bytes[i] == b'(' {
            depth += 1;
        } else if bytes[i] == b')' && depth > 0 {
            depth -= 1;
        } else if bytes[i] == b'*' && depth == 0 {
            return false;
        }
    }
    // If we never saw a * outside cmd sub, and there IS a * somewhere, it's all inside
    line.contains('*')
}

/// Check for unordered wildcard usage
pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();

    for (line_num, line) in source.lines().enumerate() {
        if line.contains('*') && !line.contains("| sort") {
            if let Some(ls_start) = line.find("$(ls ") {
                check_ls_wildcard(line, line_num, ls_start, &mut result);
            } else if line.contains("for ") && line.contains(" in ") {
                check_for_loop_wildcard(line, line_num, &mut result);
            } else {
                check_command_wildcard(line, line_num, &mut result);
            }
        }
    }

    result
}

/// Find the matching closing parenthesis for a command substitution
fn find_matching_paren(s: &str) -> Option<usize> {
    let mut depth = 0;
    for (i, c) in s.chars().enumerate() {
        match c {
            '(' => depth += 1,
            ')' => {
                depth -= 1;
                if depth == 0 {
                    return Some(i);
                }
            }
            _ => {}
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_DET003_detects_ls_wildcard() {
        let script = "FILES=$(ls *.txt)";
        let result = check(script);

        assert_eq!(result.diagnostics.len(), 1);
        let diag = &result.diagnostics[0];
        assert_eq!(diag.code, "DET003");
        assert_eq!(diag.severity, Severity::Warning);
    }

    #[test]
    fn test_DET003_detects_for_loop_wildcard() {
        let script = "for f in *.c; do echo $f; done";
        let result = check(script);

        assert_eq!(result.diagnostics.len(), 1);
        // For-loop wildcards get Info severity (no auto-fix)
        assert_eq!(result.diagnostics[0].severity, Severity::Info);
    }

    #[test]
    fn test_DET003_no_warning_with_sort() {
        let script = "FILES=$(ls *.txt | sort)";
        let result = check(script);

        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_DET003_provides_correct_fix_for_ls() {
        let script = "FILES=$(ls *.txt)";
        let result = check(script);

        assert!(result.diagnostics[0].fix.is_some());
        let fix = result.diagnostics[0].fix.as_ref().unwrap();
        // Fix should wrap the entire command substitution correctly
        assert_eq!(fix.replacement, "$(ls *.txt | sort)");
    }

    #[test]
    fn test_DET003_no_fix_for_for_loop() {
        // For-loop wildcards are too complex to auto-fix safely
        let script = "for f in *.c; do echo $f; done";
        let result = check(script);

        assert_eq!(result.diagnostics.len(), 1);
        // Should NOT have a fix (user must manually review)
        assert!(result.diagnostics[0].fix.is_none());
    }

    #[test]
    fn test_DET003_fix_span_covers_full_command_sub() {
        let script = "FILES=$(ls *.txt)";
        let result = check(script);

        let diag = &result.diagnostics[0];
        // Span should cover $(ls *.txt) which is columns 7-17 (1-indexed)
        assert_eq!(diag.span.start_col, 7);
        assert_eq!(diag.span.end_col, 18); // Exclusive end
    }

    #[test]
    fn test_DET003_nested_parens() {
        // Test with nested parentheses inside command substitution
        let script = "FILES=$(ls $(echo *.txt))";
        let result = check(script);

        // Should still detect the pattern
        assert!(!result.diagnostics.is_empty());
    }

    #[test]
    fn test_DET003_detects_cat_wildcard() {
        let script = "cat /var/log/*.log";
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 1);
        assert_eq!(result.diagnostics[0].code, "DET003");
        assert_eq!(result.diagnostics[0].severity, Severity::Info);
    }

    #[test]
    fn test_DET003_detects_wc_wildcard() {
        let script = "wc -l /opt/data/*.txt";
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 1);
        assert_eq!(result.diagnostics[0].code, "DET003");
    }

    #[test]
    fn test_DET003_detects_head_wildcard() {
        let script = "head -n 5 /var/log/*.csv";
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 1);
        assert_eq!(result.diagnostics[0].code, "DET003");
    }

    #[test]
    fn test_DET003_no_warning_wildcard_in_quotes() {
        let script = "echo \"*.log\"";
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_DET003_detects_wildcard_after_pipe() {
        let script = "find . -name foo | cat *.log";
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_DET003_no_warning_sort_pipe() {
        let script = "cat *.log | sort";
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_find_matching_paren() {
        assert_eq!(find_matching_paren("()"), Some(1));
        assert_eq!(find_matching_paren("(abc)"), Some(4));
        assert_eq!(find_matching_paren("((nested))"), Some(9));
        assert_eq!(find_matching_paren("(a(b)c)"), Some(6));
        assert_eq!(find_matching_paren("(unclosed"), None);
    }

    // ── check_command_wildcard coverage ──────────────────────────────────

    #[test]
    fn test_DET003_check_command_wildcard_tail() {
        let script = "tail -n 10 /var/log/*.log";
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 1);
        assert_eq!(result.diagnostics[0].code, "DET003");
        assert_eq!(result.diagnostics[0].severity, Severity::Info);
    }

    #[test]
    fn test_DET003_check_command_wildcard_grep() {
        let script = "grep -r pattern /etc/*.conf";
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 1);
        assert_eq!(result.diagnostics[0].code, "DET003");
    }

    #[test]
    fn test_DET003_check_command_wildcard_diff() {
        let script = "diff /backup/*.bak /current/*.conf";
        let result = check(script);
        assert!(!result.diagnostics.is_empty());
        assert_eq!(result.diagnostics[0].code, "DET003");
    }

    #[test]
    fn test_DET003_check_command_wildcard_md5sum() {
        let script = "md5sum /tmp/dist/*.tar.gz";
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 1);
        assert_eq!(result.diagnostics[0].code, "DET003");
    }

    #[test]
    fn test_DET003_check_command_wildcard_sha256sum() {
        let script = "sha256sum /releases/*.zip";
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_DET003_check_command_wildcard_du() {
        let script = "du -sh /var/log/*.log";
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_DET003_check_command_wildcard_stat() {
        let script = "stat /etc/conf/*.d";
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_DET003_check_command_wildcard_file() {
        let script = "file /tmp/uploads/*";
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_DET003_check_command_wildcard_unknown_command_no_warn() {
        // Unknown command with wildcard should NOT warn
        let script = "mycommand /tmp/*.txt";
        let result = check(script);
        assert_eq!(
            result.diagnostics.len(),
            0,
            "unknown commands should not trigger DET003"
        );
    }

    // ── is_wildcard_quoted coverage ───────────────────────────────────────

    #[test]
    fn test_DET003_wildcard_in_single_quotes_no_warn() {
        let script = "echo '*.log'";
        let result = check(script);
        assert_eq!(
            result.diagnostics.len(),
            0,
            "wildcard in single quotes should not warn"
        );
    }

    #[test]
    fn test_DET003_wildcard_in_double_quotes_no_warn() {
        let script = "echo \"*.log\"";
        let result = check(script);
        assert_eq!(
            result.diagnostics.len(),
            0,
            "wildcard in double quotes should not warn"
        );
    }

    #[test]
    fn test_DET003_wildcard_after_closing_quote_warns() {
        // After the closing quote, the wildcard is unquoted
        let script = "cat 'prefix' *.log";
        let result = check(script);
        assert_eq!(
            result.diagnostics.len(),
            1,
            "unquoted wildcard after quoted string should warn"
        );
    }

    // ── is_wildcard_in_cmd_sub coverage ──────────────────────────────────

    #[test]
    fn test_DET003_wildcard_in_cmd_sub_no_extra_warn() {
        // $(ls *.txt) is handled by check_ls_wildcard, not check_command_wildcard
        let script = "FILES=$(ls *.txt)";
        let result = check(script);
        // Should have exactly 1 warning from check_ls_wildcard
        assert_eq!(
            result.diagnostics.len(),
            1,
            "only ls-wildcard warning expected"
        );
        assert_eq!(result.diagnostics[0].severity, Severity::Warning);
    }

    #[test]
    fn test_DET003_wildcard_in_non_ls_cmd_sub_no_warn() {
        // $(find . -name '*.txt') - wildcard inside cmd sub, not at top level
        let script = "FILES=$(find . -name '*.txt')";
        let result = check(script);
        // The wildcard is quoted inside the cmd sub - no DET003 warning expected
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_DET003_no_wildcard_no_warn() {
        let script = "cat /var/log/syslog";
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_DET003_empty_script_no_warn() {
        let result = check("");
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_DET003_multiline_multiple_detections() {
        let script = "cat *.log\nwc -l *.txt\nsort output";
        let result = check(script);
        // cat *.log and wc -l *.txt should both warn, but sort is already fine
        assert_eq!(result.diagnostics.len(), 2);
    }

    #[test]
    fn test_DET003_wildcard_commands_list_covers_all() {
        // Verify every command in WILDCARD_COMMANDS triggers detection
        let commands = [
            "cat", "head", "tail", "wc", "grep", "diff", "echo", "du", "stat",
        ];
        for cmd in &commands {
            let script = format!("{cmd} /tmp/*.txt");
            let result = check(&script);
            assert!(
                !result.diagnostics.is_empty(),
                "command '{cmd}' should trigger DET003"
            );
        }
    }
}
