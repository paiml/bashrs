//! SEC022: Privilege Escalation Patterns
//!
//! **Rule**: Detect operations that escalate or abuse system privileges:
//! setuid installation, chmod +s, sudoers modification, setcap,
//! docker.sock mount, crontab injection, authorized_keys append,
//! PATH/LD_PRELOAD manipulation.

use crate::linter::{Diagnostic, LintResult, Severity, Span};
use std::sync::LazyLock;

static RE_SETUID_INSTALL: LazyLock<regex::Regex> =
    LazyLock::new(|| regex::Regex::new(r"install\s+-m\s*[24]7[0-7]{2}\s").expect("valid regex"));

static RE_CHMOD_SUID: LazyLock<regex::Regex> =
    LazyLock::new(|| regex::Regex::new(r"chmod\s+\+s\s").expect("valid regex"));

static RE_SUDOERS: LazyLock<regex::Regex> =
    LazyLock::new(|| regex::Regex::new(r"(?:>>|>)\s*/etc/sudoers").expect("valid regex"));

static RE_SETCAP: LazyLock<regex::Regex> =
    LazyLock::new(|| regex::Regex::new(r"setcap\s+cap_").expect("valid regex"));

static RE_DOCKER_SOCK: LazyLock<regex::Regex> = LazyLock::new(|| {
    regex::Regex::new(r"docker\s+run\s+.*-v\s+/var/run/docker\.sock").expect("valid regex")
});

static RE_CRONTAB_INJECT: LazyLock<regex::Regex> =
    LazyLock::new(|| regex::Regex::new(r"crontab\s+-").expect("valid regex"));

static RE_AUTHKEYS: LazyLock<regex::Regex> =
    LazyLock::new(|| regex::Regex::new(r">>\s*~?/?\.?ssh/authorized_keys").expect("valid regex"));

static RE_LD_PRELOAD: LazyLock<regex::Regex> =
    LazyLock::new(|| regex::Regex::new(r"LD_PRELOAD=\S+\s+\S+").expect("valid regex"));

static RE_PATH_PREPEND: LazyLock<regex::Regex> =
    LazyLock::new(|| regex::Regex::new(r"(?:export\s+)?PATH=/tmp[:/]").expect("valid regex"));

static RE_WORLD_WRITABLE: LazyLock<regex::Regex> =
    LazyLock::new(|| regex::Regex::new(r"chmod\s+(?:777|o\+w)\s+/etc/").expect("valid regex"));

/// Check for privilege escalation patterns.
pub fn check(source: &str) -> LintResult {
    let mut diagnostics = Vec::new();

    for (line_num, line) in source.lines().enumerate() {
        let ln = line_num + 1;

        if RE_SETUID_INSTALL.is_match(line) {
            diagnostics.push(Diagnostic::new(
                "SEC022",
                Severity::Error,
                "Setuid binary installation — privilege escalation risk",
                Span::new(ln, 1, ln, line.len()),
            ));
        }

        if RE_CHMOD_SUID.is_match(line) {
            diagnostics.push(Diagnostic::new(
                "SEC022",
                Severity::Error,
                "chmod +s — setuid/setgid bit enables privilege escalation",
                Span::new(ln, 1, ln, line.len()),
            ));
        }

        if RE_SUDOERS.is_match(line) {
            diagnostics.push(Diagnostic::new(
                "SEC022",
                Severity::Error,
                "Direct sudoers modification — privilege escalation",
                Span::new(ln, 1, ln, line.len()),
            ));
        }

        if RE_SETCAP.is_match(line) {
            diagnostics.push(Diagnostic::new(
                "SEC022",
                Severity::Warning,
                "setcap grants Linux capabilities — privilege escalation risk",
                Span::new(ln, 1, ln, line.len()),
            ));
        }

        if RE_DOCKER_SOCK.is_match(line) {
            diagnostics.push(Diagnostic::new(
                "SEC022",
                Severity::Error,
                "Docker socket mount — container escape / host access",
                Span::new(ln, 1, ln, line.len()),
            ));
        }

        if RE_CRONTAB_INJECT.is_match(line) {
            diagnostics.push(Diagnostic::new(
                "SEC022",
                Severity::Warning,
                "Crontab modification via pipe — persistence mechanism",
                Span::new(ln, 1, ln, line.len()),
            ));
        }

        if RE_AUTHKEYS.is_match(line) {
            diagnostics.push(Diagnostic::new(
                "SEC022",
                Severity::Error,
                "authorized_keys append — SSH backdoor installation",
                Span::new(ln, 1, ln, line.len()),
            ));
        }

        if RE_LD_PRELOAD.is_match(line) {
            diagnostics.push(Diagnostic::new(
                "SEC022",
                Severity::Error,
                "LD_PRELOAD injection — library hijacking",
                Span::new(ln, 1, ln, line.len()),
            ));
        }

        if RE_PATH_PREPEND.is_match(line) {
            diagnostics.push(Diagnostic::new(
                "SEC022",
                Severity::Warning,
                "PATH prepend with /tmp — command hijacking risk",
                Span::new(ln, 1, ln, line.len()),
            ));
        }

        if RE_WORLD_WRITABLE.is_match(line) {
            diagnostics.push(Diagnostic::new(
                "SEC022",
                Severity::Error,
                "World-writable system file — security configuration compromise",
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
    fn test_setuid_install() {
        let diags = check("install -m 4755 ./binary /usr/local/bin/").diagnostics;
        assert!(!diags.is_empty());
        assert_eq!(diags[0].code, "SEC022");
    }

    #[test]
    fn test_chmod_suid() {
        let diags = check("chown root:root /tmp/exploit && chmod +s /tmp/exploit").diagnostics;
        assert!(!diags.is_empty());
    }

    #[test]
    fn test_sudoers_modification() {
        let diags = check("echo 'user ALL=(ALL) NOPASSWD: ALL' >> /etc/sudoers").diagnostics;
        assert!(!diags.is_empty());
    }

    #[test]
    fn test_setcap() {
        let diags = check("setcap cap_net_raw+ep ./tool").diagnostics;
        assert!(!diags.is_empty());
    }

    #[test]
    fn test_docker_socket() {
        let diags =
            check("docker run -v /var/run/docker.sock:/var/run/docker.sock alpine").diagnostics;
        assert!(!diags.is_empty());
    }

    #[test]
    fn test_crontab_injection() {
        let diags = check("echo '* * * * * /tmp/backdoor.sh' | crontab -").diagnostics;
        assert!(!diags.is_empty());
    }

    #[test]
    fn test_authorized_keys() {
        let diags = check("echo \"$pubkey\" >> ~/.ssh/authorized_keys").diagnostics;
        assert!(!diags.is_empty());
    }

    #[test]
    fn test_ld_preload() {
        let diags = check("LD_PRELOAD=/tmp/evil.so /usr/bin/target").diagnostics;
        assert!(!diags.is_empty());
    }

    #[test]
    fn test_path_manipulation() {
        let diags = check("export PATH=/tmp/evil:$PATH").diagnostics;
        assert!(!diags.is_empty());
    }

    #[test]
    fn test_world_writable() {
        let diags = check("chmod 777 /etc/cron.d/backup").diagnostics;
        assert!(!diags.is_empty());
    }

    #[test]
    fn test_safe_install() {
        let diags = check("install -m 755 ./app /usr/local/bin/").diagnostics;
        assert!(diags.is_empty(), "Normal install should be safe");
    }

    #[test]
    fn test_safe_chmod() {
        let diags = check("chmod 644 /etc/config.conf").diagnostics;
        assert!(diags.is_empty(), "Normal chmod should be safe");
    }
}
