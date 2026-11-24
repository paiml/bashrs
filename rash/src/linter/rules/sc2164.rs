//! SC2164: Use cd ... || exit in case cd fails
//!
//! # Examples
//!
//! Bad:
//! ```bash
//! cd /some/directory
//! ./script.sh  # Runs in wrong directory if cd fails!
//! ```
//!
//! Good:
//! ```bash
//! cd /some/directory || exit
//! ./script.sh
//! ```
//!
//! # Rationale
//!
//! If cd fails, subsequent commands run in the wrong directory:
//! - Can cause data loss
//! - Can execute wrong scripts
//! - Hard to debug failures
//!
//! Always check cd exit status or use `set -e`.
//!
//! # Auto-fix
//!
//! Add || exit to cd command

use crate::linter::{Diagnostic, Fix, LintResult, Severity, Span};
use regex::Regex;

/// Check for cd without error handling
static PATTERN: std::sync::LazyLock<Regex> =
    std::sync::LazyLock::new(|| Regex::new(r"\bcd\s+([^\s;&|]+)").unwrap());

/// Byte offset of the cd that is the condition of if/elif/while/until or is
/// negated with `!` (its exit status is tested, so shellcheck does not emit
/// SC2164). Only that cd is exempt; later cds on the line are still checked.
fn condition_cd_offset(line: &str) -> Option<usize> {
    let t = line.trim_start();
    ["if ", "elif ", "while ", "until ", "! "]
        .iter()
        .find_map(|kw| t.strip_prefix(kw))
        .map(|r| r.trim_start().trim_start_matches("! "))
        .filter(|r| r.starts_with("cd "))
        .map(|r| line.len() - r.len())
}

/// Whole-line skips: comments or inline handling.
fn is_skipped_line(line: &str) -> bool {
    line.trim_start().starts_with('#')
        || line.contains("|| exit")
        || line.contains("|| return")
        || line.contains("&& ")
}

/// Any `||` handler after the cd counts as handled (shellcheck parity, #587).
fn has_or_handler(rest: &str) -> bool {
    rest.contains("||")
}

fn build_diagnostic(line_num: usize, cap: &regex::Captures<'_>) -> Diagnostic {
    let full_match = cap.get(0).unwrap();
    let directory = cap.get(1).unwrap().as_str();
    let start_col = full_match.start() + 1;
    let end_col = full_match.end() + 1;

    Diagnostic::new(
        "SC2164",
        Severity::Warning,
        "Use 'cd ... || exit' in case cd fails",
        Span::new(line_num, start_col, line_num, end_col),
    )
    .with_fix(Fix::new(format!("cd {} || exit", directory)))
}

fn check_line(line_num: usize, line: &str, result: &mut LintResult) {
    if is_skipped_line(line) {
        return;
    }
    let cond = condition_cd_offset(line);
    for cap in PATTERN.captures_iter(line) {
        let full = cap.get(0).unwrap();
        if Some(full.start()) == cond {
            continue;
        }
        let end = full.end();
        if !has_or_handler(&line[end..]) {
            result.add(build_diagnostic(line_num, &cap));
        }
    }
}

pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();
    // Simple detection; negative-lookahead improvement tracked by PMAT-249.
    for (idx, line) in source.lines().enumerate() {
        check_line(idx + 1, line, &mut result);
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sc2164_basic_detection() {
        let script = "cd /tmp";
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 1);
        assert_eq!(result.diagnostics[0].code, "SC2164");
    }

    #[test]
    fn test_sc2164_autofix() {
        let script = "cd /tmp";
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 1);
        assert!(result.diagnostics[0].fix.is_some());
        assert_eq!(
            result.diagnostics[0].fix.as_ref().unwrap().replacement,
            "cd /tmp || exit"
        );
    }

    #[test]
    fn test_sc2164_with_variable() {
        let script = "cd \"$HOME/projects\"";
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2164_relative_path() {
        let script = "cd ../..";
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2164_false_positive_with_exit() {
        let script = "cd /tmp || exit";
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2164_false_positive_with_return() {
        let script = "cd /tmp || return";
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2164_false_positive_in_comment() {
        let script = "# cd /tmp";
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2164_in_script() {
        let script = "cd /var/log\ngrep ERROR syslog";
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2164_with_tilde() {
        let script = "cd ~/documents";
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2164_false_positive_with_and() {
        let script = "cd /tmp && ls";
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2164_condition_contexts_not_flagged() {
        // shellcheck 0.8.0 is silent: cd's status is tested by the construct
        for script in [
            "if cd /tmp; then echo a; fi",
            "while cd /z; do break; done",
            "until cd /z; do sleep 1; done",
            "if ! cd /q; then exit; fi",
            "elif cd /q; then :",
            "! cd /q",
        ] {
            let result = check(script);
            assert_eq!(result.diagnostics.len(), 0, "false positive: {script}");
        }
    }

    #[test]
    fn test_sc2164_plain_cd_still_flagged_after_condition_fix() {
        assert_eq!(check("cd /w; echo c").diagnostics.len(), 1);
        assert_eq!(check("if true; then cd /w; fi").diagnostics.len(), 1);
    }

    // Issue #587: any `||` handler after cd counts (shellcheck parity)
    #[test]
    fn test_sc2164_or_true_not_flagged() {
        assert_eq!(check("cd /tmp || true").diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2164_or_brace_group_not_flagged() {
        let script = "cd \"$d\" || { echo no; exit 1; }";
        assert_eq!(check(script).diagnostics.len(), 0);
    }

    // PR #590 review: only the condition cd is exempt, not the whole line
    #[test]
    fn test_sc2164_if_condition_then_body_cd_flagged() {
        let r = check("if cd /a; then cd /b; fi");
        assert_eq!(r.diagnostics.len(), 1);
        assert_eq!(
            r.diagnostics[0].fix.as_ref().unwrap().replacement,
            "cd /b || exit"
        );
    }

    #[test]
    fn test_sc2164_while_condition_do_body_cd_flagged() {
        let r = check("while cd /a; do cd /b; done");
        assert_eq!(r.diagnostics.len(), 1);
        assert_eq!(
            r.diagnostics[0].fix.as_ref().unwrap().replacement,
            "cd /b || exit"
        );
    }

    #[test]
    fn test_sc2164_negated_condition_then_cd_flagged() {
        let r = check("! cd /a; cd /b");
        assert_eq!(r.diagnostics.len(), 1);
        assert_eq!(
            r.diagnostics[0].fix.as_ref().unwrap().replacement,
            "cd /b || exit"
        );
    }

    #[test]
    fn test_sc2164_condition_only_still_not_flagged() {
        assert_eq!(check("if cd /a; then echo ok; fi").diagnostics.len(), 0);
    }
}
