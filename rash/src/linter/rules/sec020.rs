//! SEC020: Dangerous Command Execution Patterns
//!
//! **Rule**: Detect alternative command execution vectors that bypass standard
//! eval/curl|bash detection. These patterns execute user-controlled strings
//! as commands via different mechanisms.
//!
//! Catches: `bash -c "$var"`, `exec "$var"`, `xargs sh`, `ssh ... "$cmd"`,
//! `su -c "$cmd"`, `perl -e "$code"`, `awk '{system(...)}'`

use crate::linter::{Diagnostic, LintResult, Severity, Span};
use std::sync::LazyLock;

static RE_BASH_C: LazyLock<regex::Regex> = LazyLock::new(|| {
    regex::Regex::new(r#"(?:bash|sh|zsh|ksh|dash)\s+-c\s+"\$"#).expect("valid regex")
});

static RE_EXEC_VAR: LazyLock<regex::Regex> = LazyLock::new(|| {
    regex::Regex::new(r#"exec\s+"\$[^"]*""#).expect("valid regex")
});

static RE_XARGS_SH: LazyLock<regex::Regex> = LazyLock::new(|| {
    regex::Regex::new(r"xargs\s+(?:sh|bash|zsh)").expect("valid regex")
});

static RE_SSH_CMD: LazyLock<regex::Regex> = LazyLock::new(|| {
    regex::Regex::new(r#"ssh\s+\S+\s+"\$"#).expect("valid regex")
});

static RE_SU_C: LazyLock<regex::Regex> = LazyLock::new(|| {
    regex::Regex::new(r#"su\s+-c\s+"\$"#).expect("valid regex")
});

static RE_PERL_E: LazyLock<regex::Regex> = LazyLock::new(|| {
    regex::Regex::new(r#"perl\s+-e\s+"\$"#).expect("valid regex")
});

static RE_AWK_SYSTEM: LazyLock<regex::Regex> = LazyLock::new(|| {
    regex::Regex::new(r"awk\s+'[^']*system\s*\(").expect("valid regex")
});

static RE_ENV_INTERP: LazyLock<regex::Regex> = LazyLock::new(|| {
    regex::Regex::new(r#"(?:#!/usr/bin/env|env)\s+"\$"#).expect("valid regex")
});

/// Check for dangerous command execution patterns.
pub fn check(source: &str) -> LintResult {
    let mut diagnostics = Vec::new();

    for (line_num, line) in source.lines().enumerate() {
        let ln = line_num + 1;

        if RE_BASH_C.is_match(line) {
            diagnostics.push(Diagnostic::new(
                "SEC020",
                Severity::Warning,
                "Command execution via sh/bash -c with variable — injection risk",
                Span::new(ln, 1, ln, line.len()),
            ));
        }

        if RE_EXEC_VAR.is_match(line) {
            diagnostics.push(Diagnostic::new(
                "SEC020",
                Severity::Warning,
                "exec with user-controlled path — command injection risk",
                Span::new(ln, 1, ln, line.len()),
            ));
        }

        if RE_XARGS_SH.is_match(line) {
            diagnostics.push(Diagnostic::new(
                "SEC020",
                Severity::Warning,
                "Piping to xargs sh — command injection from input data",
                Span::new(ln, 1, ln, line.len()),
            ));
        }

        if RE_SSH_CMD.is_match(line) {
            diagnostics.push(Diagnostic::new(
                "SEC020",
                Severity::Warning,
                "ssh with variable command — remote command injection risk",
                Span::new(ln, 1, ln, line.len()),
            ));
        }

        if RE_SU_C.is_match(line) {
            diagnostics.push(Diagnostic::new(
                "SEC020",
                Severity::Warning,
                "su -c with variable — privileged command injection risk",
                Span::new(ln, 1, ln, line.len()),
            ));
        }

        if RE_PERL_E.is_match(line) {
            diagnostics.push(Diagnostic::new(
                "SEC020",
                Severity::Warning,
                "perl -e with variable — code injection via interpreter",
                Span::new(ln, 1, ln, line.len()),
            ));
        }

        if RE_AWK_SYSTEM.is_match(line) {
            diagnostics.push(Diagnostic::new(
                "SEC020",
                Severity::Warning,
                "awk system() call — command execution from input data",
                Span::new(ln, 1, ln, line.len()),
            ));
        }

        if RE_ENV_INTERP.is_match(line) {
            diagnostics.push(Diagnostic::new(
                "SEC020",
                Severity::Warning,
                "env/shebang with variable interpreter — command injection risk",
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
    fn test_bash_c_injection() {
        let diags = check("bash -c \"$untrusted\"").diagnostics;
        assert!(!diags.is_empty());
        assert_eq!(diags[0].code, "SEC020");
    }

    #[test]
    fn test_exec_variable() {
        let diags = check("exec \"$user_binary\"").diagnostics;
        assert!(!diags.is_empty());
    }

    #[test]
    fn test_xargs_sh() {
        let diags = check("find /tmp -name '*.sh' | xargs sh").diagnostics;
        assert!(!diags.is_empty());
    }

    #[test]
    fn test_ssh_command() {
        let diags = check("ssh user@host \"$remote_cmd\"").diagnostics;
        assert!(!diags.is_empty());
    }

    #[test]
    fn test_su_c_command() {
        let diags = check("su -c \"$admin_cmd\" root").diagnostics;
        assert!(!diags.is_empty());
    }

    #[test]
    fn test_perl_e_variable() {
        let diags = check("perl -e \"$perl_code\"").diagnostics;
        assert!(!diags.is_empty());
    }

    #[test]
    fn test_awk_system() {
        let diags = check("awk '{system($1)}' input.txt").diagnostics;
        assert!(!diags.is_empty());
    }

    #[test]
    fn test_safe_bash_c_literal() {
        let diags = check("bash -c 'echo hello'").diagnostics;
        assert!(diags.is_empty(), "Literal string in bash -c should be safe");
    }

    #[test]
    fn test_safe_exec_literal() {
        let diags = check("exec /usr/bin/app").diagnostics;
        assert!(diags.is_empty(), "Literal exec should be safe");
    }
}
