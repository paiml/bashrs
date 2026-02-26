// SC2209: Use var=$(command) instead of var=command for command output
use crate::linter::{Diagnostic, LintResult, Severity, Span};
use regex::Regex;

static VAR_EQUALS_COMMAND: std::sync::LazyLock<Regex> = std::sync::LazyLock::new(|| {
    // Match var=command without $ or quotes
    // Must be a command name (letter start), not path or variable
    Regex::new(r"\b\w+\s*=\s*([a-z_][a-z0-9_-]*)\s*$").unwrap()
});

/// Check if a line is a comment
fn is_comment_line(line: &str) -> bool {
    line.trim_start().starts_with('#')
}

/// Check if line has command substitution
fn has_command_substitution(line: &str) -> bool {
    line.contains("$(") || line.contains('`')
}

/// Check if line has variable expansion
fn has_variable_expansion(line: &str) -> bool {
    line.contains('$')
}

/// Check if line has quoted string
fn has_quoted_string(line: &str) -> bool {
    line.contains("=\"") || line.contains("='")
}

/// Check if command is a known output command
fn is_output_command(cmd: &str) -> bool {
    let output_commands = ["date", "pwd", "whoami", "hostname", "id", "uname"];
    output_commands.contains(&cmd)
}

/// Create diagnostic for var=command without substitution
fn create_command_substitution_diagnostic(
    line_num: usize,
    line_len: usize,
    cmd: &str,
) -> Diagnostic {
    Diagnostic::new(
        "SC2209",
        Severity::Warning,
        format!(
            "Use var=$({}) to capture command output, not var={}",
            cmd, cmd
        ),
        Span::new(line_num, 1, line_num, line_len + 1),
    )
}

pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();
    for (line_num, line) in source.lines().enumerate() {
        let line_num = line_num + 1;

        if is_comment_line(line) {
            continue;
        }

        // Skip if already using command substitution
        if has_command_substitution(line) {
            continue;
        }

        // Skip variable assignments
        if has_variable_expansion(line) {
            continue;
        }

        // Skip string literals
        if has_quoted_string(line) {
            continue;
        }

        if let Some(cap) = VAR_EQUALS_COMMAND.captures(line) {
            let cmd = cap.get(1).map_or("", |m| m.as_str());

            // Only warn for common commands that produce output
            if is_output_command(cmd) {
                let diagnostic = create_command_substitution_diagnostic(line_num, line.len(), cmd);
                result.add(diagnostic);
            }
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    // ===== Manual Property Tests =====

    #[test]
    fn prop_sc2209_comments_never_diagnosed() {
        // Property: Comment lines should never produce diagnostics
        let test_cases = vec!["# timestamp=date", "  # dir=pwd", "\t# user=whoami"];

        for code in test_cases {
            let result = check(code);
            assert_eq!(result.diagnostics.len(), 0);
        }
    }

    #[test]
    fn prop_sc2209_command_substitution_never_diagnosed() {
        // Property: Command substitution $(cmd) never diagnosed
        let test_cases = vec![
            "timestamp=$(date)",
            "dir=$(pwd)",
            "user=$(whoami)",
            "host=$(hostname)",
        ];

        for code in test_cases {
            let result = check(code);
            assert_eq!(result.diagnostics.len(), 0);
        }
    }

    #[test]
    fn prop_sc2209_backtick_substitution_never_diagnosed() {
        // Property: Backtick substitution `cmd` never diagnosed
        let test_cases = vec!["timestamp=`date`", "dir=`pwd`", "user=`whoami`"];

        for code in test_cases {
            let result = check(code);
            assert_eq!(result.diagnostics.len(), 0);
        }
    }

    #[test]
    fn prop_sc2209_quoted_strings_never_diagnosed() {
        // Property: Quoted string literals never diagnosed
        let test_cases = vec![
            "var=\"value\"",
            "name='string'",
            "path=\"/usr/bin\"",
            "cmd='literal'",
        ];

        for code in test_cases {
            let result = check(code);
            assert_eq!(result.diagnostics.len(), 0);
        }
    }

    #[test]
    fn prop_sc2209_variable_expansion_never_diagnosed() {
        // Property: Variable expansions never diagnosed
        let test_cases = vec!["new=$old", "copy=$original", "ref=$value", "alias=$var"];

        for code in test_cases {
            let result = check(code);
            assert_eq!(result.diagnostics.len(), 0);
        }
    }

    #[test]
    fn prop_sc2209_output_commands_always_diagnosed() {
        // Property: Known output commands without $(.) always diagnosed
        let test_cases = vec![
            "timestamp=date",
            "dir=pwd",
            "user=whoami",
            "host=hostname",
            "id_info=id",
            "os=uname",
        ];

        for code in test_cases {
            let result = check(code);
            assert_eq!(result.diagnostics.len(), 1, "Should diagnose: {}", code);
            assert!(result.diagnostics[0].message.contains("$"));
        }
    }

    #[test]
    fn prop_sc2209_unknown_commands_never_diagnosed() {
        // Property: Unknown commands not in output_commands list never diagnosed
        let test_cases = vec![
            "cmd=myapp",
            "tool=customtool",
            "script=myscript",
            "binary=unknown",
        ];

        for code in test_cases {
            let result = check(code);
            assert_eq!(result.diagnostics.len(), 0);
        }
    }

    #[test]
    fn prop_sc2209_diagnostic_code_always_sc2209() {
        // Property: All diagnostics must have code "SC2209"
        let code = "timestamp=date\ndir=pwd";
        let result = check(code);

        for diagnostic in &result.diagnostics {
            assert_eq!(&diagnostic.code, "SC2209");
        }
    }

    #[test]
    fn prop_sc2209_diagnostic_severity_always_warning() {
        // Property: All diagnostics must be Warning severity
        let code = "timestamp=date";
        let result = check(code);

        for diagnostic in &result.diagnostics {
            assert_eq!(diagnostic.severity, Severity::Warning);
        }
    }

    #[test]
    fn prop_sc2209_empty_source_no_diagnostics() {
        // Property: Empty source should produce no diagnostics
        let result = check("");
        assert_eq!(result.diagnostics.len(), 0);
    }

    // ===== Original Unit Tests =====

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
