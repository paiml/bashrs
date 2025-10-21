// SC2209: Use var=$(command) instead of var=command for command output
use crate::linter::{Diagnostic, LintResult, Severity, Span};
use once_cell::sync::Lazy;
use regex::Regex;

static VAR_EQUALS_COMMAND: Lazy<Regex> = Lazy::new(|| {
    // Match var=command without $ or quotes
    // Must be a command name (letter start), not path or variable
    Regex::new(r"\b\w+\s*=\s*([a-z_][a-z0-9_-]*)\s*$").unwrap()
});

pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();
    for (line_num, line) in source.lines().enumerate() {
        let line_num = line_num + 1;
        if line.trim_start().starts_with('#') {
            continue;
        }

        // Skip if already using command substitution
        if line.contains("$(") || line.contains("`") {
            continue;
        }
        // Skip variable assignments (contains $)
        if line.contains('$') {
            continue;
        }
        // Skip string literals (contains quotes after =)
        if line.contains("=\"") || line.contains("='") {
            continue;
        }

        if let Some(cap) = VAR_EQUALS_COMMAND.captures(line) {
            let cmd = cap.get(1).map(|m| m.as_str()).unwrap_or("");
            // Only warn for common commands that produce output
            let output_commands = ["date", "pwd", "whoami", "hostname", "id", "uname"];
            if output_commands.contains(&cmd) {
                let diagnostic = Diagnostic::new(
                    "SC2209",
                    Severity::Warning,
                    format!(
                        "Use var=$({}) to capture command output, not var={}",
                        cmd, cmd
                    ),
                    Span::new(line_num, 1, line_num, line.len() + 1),
                );
                result.add(diagnostic);
            }
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_sc2209_date_command() {
        let code = r#"timestamp=date"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }
    #[test]
    fn test_sc2209_with_cmdsub_ok() {
        let code = r#"timestamp=$(date)"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }
    #[test]
    fn test_sc2209_pwd_command() {
        let code = r#"dir=pwd"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }
    #[test]
    fn test_sc2209_string_ok() {
        let code = r#"var="value""#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }
    #[test]
    fn test_sc2209_variable_ok() {
        let code = r#"new=$old"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }
    #[test]
    fn test_sc2209_hostname() {
        let code = r#"host=hostname"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }
    #[test]
    fn test_sc2209_unknown_command_ok() {
        let code = r#"cmd=myapp"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0); // Not in output_commands list
    }
    #[test]
    fn test_sc2209_whoami() {
        let code = r#"user=whoami"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }
    #[test]
    fn test_sc2209_comment_skipped() {
        let code = r#"# time=date"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }
    #[test]
    fn test_sc2209_uname() {
        let code = r#"os=uname"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }
}
