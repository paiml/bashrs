// SC2308: This shebang is not used in remote scripts
use crate::linter::{Diagnostic, LintResult, Severity, Span};
use once_cell::sync::Lazy;
use regex::Regex;

static REMOTE_SCRIPT: Lazy<Regex> =
    Lazy::new(|| Regex::new(r#"(?:ssh|curl|wget).*(?:bash|sh)\s+-[cs]"#).unwrap());

pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();
    let lines: Vec<&str> = source.lines().collect();

    if lines.is_empty() {
        return result;
    }

    // Check if first line is a shebang
    let has_shebang = lines[0].starts_with("#!");

    if has_shebang {
        // Check subsequent lines for remote execution
        for (line_num, line) in lines.iter().enumerate().skip(1) {
            let line_num = line_num + 1;
            if line.trim_start().starts_with('#') {
                continue;
            }
            if REMOTE_SCRIPT.is_match(line) {
                let diagnostic = Diagnostic::new(
                    "SC2308",
                    Severity::Info,
                    "Shebang is ignored when script is executed remotely via ssh/curl/wget"
                        .to_string(),
                    Span::new(1, 1, 1, lines[0].len() + 1),
                );
                result.add(diagnostic);
                break; // Only warn once
            }
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sc2308_ssh_remote() {
        let code = r#"#!/bin/bash
ssh user@host 'bash -c "command"'
"#;
        assert_eq!(check(code).diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2308_curl_pipe_sh() {
        let code = r#"#!/bin/sh
curl https://example.com/script.sh | sh -s
"#;
        assert_eq!(check(code).diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2308_no_shebang_ok() {
        let code = r#"ssh user@host 'bash -c "command"'"#;
        assert_eq!(check(code).diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2308_local_script_ok() {
        let code = r#"#!/bin/bash
echo "Hello"
"#;
        assert_eq!(check(code).diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2308_empty() {
        assert_eq!(check("").diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2308_comment() {
        let code = r#"#!/bin/bash
# ssh user@host 'bash -c "command"'
"#;
        assert_eq!(check(code).diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2308_wget_remote() {
        let code = r#"#!/bin/bash
wget -O- https://example.com/script.sh | bash -s
"#;
        assert_eq!(check(code).diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2308_ssh_no_bash() {
        let code = r#"#!/bin/bash
ssh user@host 'echo test'
"#;
        assert_eq!(check(code).diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2308_only_warns_once() {
        let code = r#"#!/bin/bash
ssh host1 'bash -c "cmd1"'
ssh host2 'bash -c "cmd2"'
"#;
        assert_eq!(check(code).diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2308_no_remote_ok() {
        let code = r#"#!/bin/bash
echo "test"
cat file.txt
"#;
        assert_eq!(check(code).diagnostics.len(), 0);
    }
}
