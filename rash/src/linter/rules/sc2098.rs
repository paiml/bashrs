// SC2098: This expansion will not see the assignment made in the command
//
// When assignments are made in piped commands or subshells, they only affect
// that subprocess, not the parent shell or subsequent commands.
//
// Examples:
// Bad:
//   echo "value" | read var
//   echo "$var"  # var is empty in parent shell
//
//   (count=5)
//   echo "$count"  # count is not set in parent shell
//
// Good:
//   read var < <(echo "value")
//   echo "$var"  # var is set
//
//   count=5
//   echo "$count"  # count is set

use crate::linter::{Diagnostic, LintResult, Severity, Span};
use regex::Regex;

static PIPE_TO_READ: std::sync::LazyLock<Regex> = std::sync::LazyLock::new(|| {
    // Match: command | read var
    Regex::new(r"\|\s*read\s+([A-Za-z_][A-Za-z0-9_]*)").unwrap()
});

static SUBSHELL_ASSIGNMENT: std::sync::LazyLock<Regex> = std::sync::LazyLock::new(|| {
    // Match: (var=value)  or  $(var=value)
    Regex::new(r"\(([A-Za-z_][A-Za-z0-9_]*)=").unwrap()
});

static WHILE_PIPE_READ: std::sync::LazyLock<Regex> = std::sync::LazyLock::new(|| {
    // Match: command | while read var
    Regex::new(r"\|\s*while\s+read\s+([A-Za-z_][A-Za-z0-9_]*)").unwrap()
});

pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();

    for (line_num, line) in source.lines().enumerate() {
        let line_num = line_num + 1;

        if line.trim_start().starts_with('#') {
            continue;
        }

        // Check for pipe to read
        if let Some(cap) = PIPE_TO_READ.captures(line) {
            let var_name = cap.get(1).unwrap().as_str();
            let start_col = cap.get(0).unwrap().start() + 1;
            let end_col = cap.get(0).unwrap().end() + 1;

            let diagnostic = Diagnostic::new(
                "SC2098",
                Severity::Warning,
                format!(
                    "Variable '{}' is set in a subshell due to pipe. Use process substitution 'read {} < <(...)' or read from file instead",
                    var_name, var_name
                ),
                Span::new(line_num, start_col, line_num, end_col),
            );

            result.add(diagnostic);
        }

        // Check for while read in pipe
        if let Some(cap) = WHILE_PIPE_READ.captures(line) {
            let var_name = cap.get(1).unwrap().as_str();
            let start_col = cap.get(0).unwrap().start() + 1;
            let end_col = cap.get(0).unwrap().end() + 1;

            let diagnostic = Diagnostic::new(
                "SC2098",
                Severity::Warning,
                format!(
                    "Variable '{}' and loop body execute in subshell due to pipe. Variables set in loop won't be visible outside. Use process substitution instead",
                    var_name
                ),
                Span::new(line_num, start_col, line_num, end_col),
            );

            result.add(diagnostic);
        }

        // Check for assignments in subshells (parentheses)
        if let Some(cap) = SUBSHELL_ASSIGNMENT.captures(line) {
            let var_name = cap.get(1).unwrap().as_str();
            let start_col = cap.get(0).unwrap().start() + 1;
            let end_col = cap.get(0).unwrap().end() + 1;

            // Skip if it's clearly a function definition
            if line.contains("function ") {
                continue;
            }

            let diagnostic = Diagnostic::new(
                "SC2098",
                Severity::Info,
                format!(
                    "Variable '{}' is assigned in a subshell and won't be visible in the parent shell",
                    var_name
                ),
                Span::new(line_num, start_col, line_num, end_col),
            );

            result.add(diagnostic);
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sc2098_pipe_to_read() {
        let code = r#"echo "value" | read var"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
        assert_eq!(result.diagnostics[0].code, "SC2098");
        assert_eq!(result.diagnostics[0].severity, Severity::Warning);
        assert!(result.diagnostics[0].message.contains("var"));
    }

    #[test]
    fn test_sc2098_while_pipe_read() {
        let code = r#"cat file.txt | while read line; do echo "$line"; done"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
        assert!(result.diagnostics[0].message.contains("line"));
    }

    #[test]
    fn test_sc2098_subshell_assignment() {
        let code = r#"(count=5)"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
        assert!(result.diagnostics[0].message.contains("count"));
    }

    #[test]
    fn test_sc2098_process_substitution_ok() {
        let code = r#"read var < <(echo "value")"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2098_regular_assignment_ok() {
        let code = r#"var="value""#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2098_while_read_file_ok() {
        let code = r#"while read line; do echo "$line"; done < file.txt"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2098_command_substitution_ok() {
        let code = r#"result=$(command)"#;
        let result = check(code);
        // Command substitution for output is OK
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2098_multiple_pipe_reads() {
        let code = r#"
echo "a" | read var1
echo "b" | read var2
"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 2);
    }

    #[test]
    fn test_sc2098_grep_pipe_read() {
        let code = r#"grep pattern file.txt | read match"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2098_heredoc_ok() {
        let code = r#"read var <<EOF
value
EOF"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }
}
