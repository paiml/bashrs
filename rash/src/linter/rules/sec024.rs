//! SEC024: Race Condition and TOCTOU Patterns
//!
//! **Rule**: Detect time-of-check-time-of-use (TOCTOU) races, predictable
//! temp files, PID file races, and symlink attack surfaces.

use crate::linter::{Diagnostic, LintResult, Severity, Span};
use std::sync::LazyLock;

static RE_TOCTOU_FILE: LazyLock<regex::Regex> = LazyLock::new(|| {
    regex::Regex::new(r"\[\s+-[fd]\s+.*\]\s*(?:\|\||&&)\s*(?:touch|mkdir|rm|cd)\s")
        .expect("valid regex")
});

static RE_TOCTOU_TEST: LazyLock<regex::Regex> =
    LazyLock::new(|| regex::Regex::new(r"test\s+-[fd]\s+.*&&\s*(?:cd|rm)\s").expect("valid regex"));

static RE_PID_FILE_RACE: LazyLock<regex::Regex> = LazyLock::new(|| {
    regex::Regex::new(r"\[\s+!\s+-f\s+.*\.pid\s*\].*echo\s+\$\$\s*>").expect("valid regex")
});

static RE_PREDICTABLE_TEMP: LazyLock<regex::Regex> =
    LazyLock::new(|| regex::Regex::new(r"/tmp/\w+_\$\$").expect("valid regex"));

static RE_SYMLINK_ATTACK: LazyLock<regex::Regex> = LazyLock::new(|| {
    regex::Regex::new(r"ln\s+-s\s+/etc/(?:shadow|passwd|sudoers)\s+/tmp/").expect("valid regex")
});

/// Check for race condition and TOCTOU patterns.
pub fn check(source: &str) -> LintResult {
    let mut diagnostics = Vec::new();

    for (line_num, line) in source.lines().enumerate() {
        let ln = line_num + 1;

        if RE_TOCTOU_FILE.is_match(line) {
            diagnostics.push(Diagnostic::new(
                "SEC024",
                Severity::Warning,
                "TOCTOU race — check-then-act on file is not atomic",
                Span::new(ln, 1, ln, line.len()),
            ));
        }

        if RE_TOCTOU_TEST.is_match(line) {
            diagnostics.push(Diagnostic::new(
                "SEC024",
                Severity::Warning,
                "TOCTOU race — test then operate is not atomic",
                Span::new(ln, 1, ln, line.len()),
            ));
        }

        if RE_PID_FILE_RACE.is_match(line) {
            diagnostics.push(Diagnostic::new(
                "SEC024",
                Severity::Warning,
                "PID file race — check-then-write is not atomic",
                Span::new(ln, 1, ln, line.len()),
            ));
        }

        if RE_PREDICTABLE_TEMP.is_match(line) {
            diagnostics.push(Diagnostic::new(
                "SEC024",
                Severity::Warning,
                "Predictable temp file name with $$ — symlink race risk",
                Span::new(ln, 1, ln, line.len()),
            ));
        }

        if RE_SYMLINK_ATTACK.is_match(line) {
            diagnostics.push(Diagnostic::new(
                "SEC024",
                Severity::Error,
                "Symlink to sensitive file in /tmp — symlink attack",
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
    fn test_toctou_file() {
        let diags = check("[ -f \"$lock\" ] || touch \"$lock\"").diagnostics;
        assert!(!diags.is_empty());
        assert_eq!(diags[0].code, "SEC024");
    }

    #[test]
    fn test_toctou_dir() {
        let diags = check("test -d \"$dir\" && cd \"$dir\"").diagnostics;
        assert!(!diags.is_empty());
    }

    #[test]
    fn test_pid_file_race() {
        let diags =
            check("if [ ! -f /var/run/app.pid ]; then echo $$ > /var/run/app.pid; fi").diagnostics;
        assert!(!diags.is_empty());
    }

    #[test]
    fn test_predictable_temp() {
        let diags =
            check("tmpfile=/tmp/work_$$; echo data > $tmpfile; process $tmpfile; rm $tmpfile")
                .diagnostics;
        assert!(!diags.is_empty());
    }

    #[test]
    fn test_symlink_attack() {
        let diags = check("ln -s /etc/shadow /tmp/output; cat /tmp/output").diagnostics;
        assert!(!diags.is_empty());
    }

    #[test]
    fn test_safe_mktemp() {
        let diags = check("tmpfile=$(mktemp); echo data > \"$tmpfile\"").diagnostics;
        assert!(diags.is_empty(), "mktemp is the safe pattern");
    }
}
