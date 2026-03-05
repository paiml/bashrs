//! SEC021: Destructive System Operations
//!
//! **Rule**: Detect operations that can cause irreversible system damage:
//! disk wiping (dd /dev/zero), fork bombs, sysrq triggers, firewall flush,
//! recursive permission removal, and similar destructive patterns.

use crate::linter::{Diagnostic, LintResult, Severity, Span};
use std::sync::LazyLock;

static RE_DD_WIPE: LazyLock<regex::Regex> = LazyLock::new(|| {
    regex::Regex::new(r"dd\s+if=/dev/(?:zero|urandom)\s+of=/dev/\w+").expect("valid regex")
});

static RE_FORK_BOMB: LazyLock<regex::Regex> = LazyLock::new(|| {
    regex::Regex::new(r":\(\)\{.*:\|:.*&.*\};:").expect("valid regex")
});

static RE_SYSRQ: LazyLock<regex::Regex> = LazyLock::new(|| {
    regex::Regex::new(r"(?:echo|printf)\s+\w+\s*>\s*/proc/sysrq").expect("valid regex")
});

static RE_IPTABLES_FLUSH: LazyLock<regex::Regex> = LazyLock::new(|| {
    regex::Regex::new(r"iptables\s+-[FX]").expect("valid regex")
});

static RE_CHMOD_REMOVE: LazyLock<regex::Regex> = LazyLock::new(|| {
    regex::Regex::new(r"chmod\s+-R\s+0{3}\s+/").expect("valid regex")
});

static RE_RM_RF_ROOT: LazyLock<regex::Regex> = LazyLock::new(|| {
    regex::Regex::new(r"rm\s+-rf\s+/(?:\s|$)").expect("valid regex")
});

/// Check for destructive system operations.
pub fn check(source: &str) -> LintResult {
    let mut diagnostics = Vec::new();

    for (line_num, line) in source.lines().enumerate() {
        let ln = line_num + 1;

        if RE_DD_WIPE.is_match(line) {
            diagnostics.push(Diagnostic::new(
                "SEC021",
                Severity::Error,
                "Disk wipe via dd — irreversible data destruction",
                Span::new(ln, 1, ln, line.len()),
            ));
        }

        if RE_FORK_BOMB.is_match(line) {
            diagnostics.push(Diagnostic::new(
                "SEC021",
                Severity::Error,
                "Fork bomb detected — system denial of service",
                Span::new(ln, 1, ln, line.len()),
            ));
        }

        if RE_SYSRQ.is_match(line) {
            diagnostics.push(Diagnostic::new(
                "SEC021",
                Severity::Error,
                "sysrq trigger — kernel-level system manipulation",
                Span::new(ln, 1, ln, line.len()),
            ));
        }

        if RE_IPTABLES_FLUSH.is_match(line) {
            diagnostics.push(Diagnostic::new(
                "SEC021",
                Severity::Warning,
                "Firewall rules flush — network security disruption",
                Span::new(ln, 1, ln, line.len()),
            ));
        }

        if RE_CHMOD_REMOVE.is_match(line) {
            diagnostics.push(Diagnostic::new(
                "SEC021",
                Severity::Error,
                "Recursive permission removal — system access destruction",
                Span::new(ln, 1, ln, line.len()),
            ));
        }

        if RE_RM_RF_ROOT.is_match(line) {
            diagnostics.push(Diagnostic::new(
                "SEC021",
                Severity::Error,
                "rm -rf / — complete filesystem destruction",
                Span::new(ln, 1, ln, line.len()),
            ));
        }
    }

    LintResult { diagnostics }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dd_wipe() {
        let diags = check("dd if=/dev/zero of=/dev/sda bs=1M").diagnostics;
        assert!(!diags.is_empty());
        assert_eq!(diags[0].code, "SEC021");
    }

    #[test]
    fn test_fork_bomb() {
        let diags = check(":(){ :|:& };:").diagnostics;
        assert!(!diags.is_empty());
    }

    #[test]
    fn test_sysrq() {
        let diags = check("echo c > /proc/sysrq-trigger").diagnostics;
        assert!(!diags.is_empty());
    }

    #[test]
    fn test_iptables_flush() {
        let diags = check("iptables -F && iptables -X && iptables -P INPUT DROP").diagnostics;
        assert!(!diags.is_empty());
    }

    #[test]
    fn test_chmod_recursive_remove() {
        let diags = check("chmod -R 000 /var/lib/").diagnostics;
        assert!(!diags.is_empty());
    }

    #[test]
    fn test_safe_dd() {
        let diags = check("dd if=image.iso of=/dev/sdb bs=4M").diagnostics;
        assert!(diags.is_empty(), "dd from file to device is normal usage");
    }

    #[test]
    fn test_safe_chmod() {
        let diags = check("chmod 755 /usr/local/bin/app").diagnostics;
        assert!(diags.is_empty(), "Normal chmod should be safe");
    }
}
