// SC2194: This word is constant - consider using the command name directly
//
// When a variable is assigned a constant command name and then immediately executed,
// it's clearer and more efficient to just run the command directly.
//
// Examples:
// Bad:
//   cmd="ls"
//   $cmd                   // Just use 'ls' directly
//
//   command="grep"
//   $command pattern file  // Just use 'grep pattern file'
//
// Good:
//   ls                     // Direct command
//   grep pattern file      // Direct command
//
// When variable command execution is OK:
//   # When command varies based on condition
//   if [ condition ]; then
//     cmd="ls"
//   else
//     cmd="find"
//   fi
//   $cmd                  // Variable command is meaningful here
//
// Note: This rule catches simple cases where a constant is assigned and
// immediately used, making the code unnecessarily indirect.

use crate::linter::{Diagnostic, LintResult, Severity, Span};
use once_cell::sync::Lazy;
use regex::Regex;

static COMMAND_VAR_ASSIGNMENT: Lazy<Regex> = Lazy::new(|| {
    // Match: cmd="command" or command='command' or cmd=command
    Regex::new(r##"^([a-zA-Z_][a-zA-Z0-9_]*)=["']?([a-z_][a-z0-9_-]*)["']?\s*$"##).unwrap()
});

static COMMAND_VAR_USAGE: Lazy<Regex> = Lazy::new(|| {
    // Match: $cmd or ${cmd} at start of command
    Regex::new(r"^\s*\$(\{)?([a-zA-Z_][a-zA-Z0-9_]*)(\})?").unwrap()
});

pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();
    let lines: Vec<&str> = source.lines().collect();

    #[allow(clippy::needless_range_loop)]
    for i in 0..lines.len() {
        let line = lines[i];
        let line_num = i + 1;

        if line.trim_start().starts_with('#') {
            continue;
        }

        // Check if this line assigns a constant to a variable
        if let Some(assign_cap) = COMMAND_VAR_ASSIGNMENT.captures(line.trim()) {
            let var_name = assign_cap.get(1).unwrap().as_str();
            let command_name = assign_cap.get(2).unwrap().as_str();

            // Look at the next non-empty, non-comment line
            #[allow(clippy::needless_range_loop)]
            for j in (i + 1)..lines.len() {
                let next_line = lines[j];
                let next_trimmed = next_line.trim();

                if next_trimmed.is_empty() || next_trimmed.starts_with('#') {
                    continue;
                }

                // Check if next line uses this variable as a command
                if let Some(usage_cap) = COMMAND_VAR_USAGE.captures(next_trimmed) {
                    let used_var = usage_cap.get(2).unwrap().as_str();

                    if used_var == var_name {
                        let start_col = next_line.find('$').unwrap_or(0) + 1;
                        let end_col = start_col + usage_cap.get(0).unwrap().as_str().len();

                        let diagnostic = Diagnostic::new(
                            "SC2194",
                            Severity::Info,
                            format!(
                                "This variable '{}' is constant (assigned '{}' on line {}). Consider using '{}' directly",
                                var_name, command_name, line_num, command_name
                            ),
                            Span::new(j + 1, start_col, j + 1, end_col),
                        );

                        result.add(diagnostic);
                    }
                }

                // Stop at first non-comment line
                break;
            }
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sc2194_constant_command() {
        let code = r#"
cmd="ls"
$cmd
"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
        assert_eq!(result.diagnostics[0].code, "SC2194");
        assert_eq!(result.diagnostics[0].severity, Severity::Info);
        assert!(result.diagnostics[0].message.contains("constant"));
    }

    #[test]
    fn test_sc2194_constant_grep() {
        let code = r#"
command="grep"
$command pattern file
"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2194_single_quoted() {
        let code = r#"
tool='find'
$tool . -name "*.txt"
"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2194_braces() {
        let code = r#"
cmd="echo"
${cmd} "hello"
"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2194_with_dash() {
        let code = r#"
cmd="git-status"
$cmd
"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2194_different_usage_ok() {
        let code = r#"
cmd="ls"
other_cmd="find"
$other_cmd
"#;
        let result = check(code);
        // other_cmd is still a constant usage (cmd is not flagged since it's not used)
        assert_eq!(result.diagnostics.len(), 1);
        assert!(result.diagnostics[0].message.contains("other_cmd"));
    }

    #[test]
    fn test_sc2194_non_constant_ok() {
        let code = r#"
cmd=$1
$cmd
"#;
        let result = check(code);
        // Not a constant assignment
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2194_command_with_path_ok() {
        let code = r#"
cmd="/usr/bin/ls"
$cmd
"#;
        let result = check(code);
        // Path, not simple command name
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2194_not_used_immediately() {
        let code = r#"
cmd="ls"
echo "Running command..."
$cmd
"#;
        let result = check(code);
        // Not immediate usage
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2194_multiple_cases() {
        let code = r#"
cmd1="ls"
$cmd1

cmd2="pwd"
$cmd2
"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 2);
    }
}
