// SC1131: Use 'elif' instead of 'else if'
//
// In shell, 'else if' creates a nested if that requires an additional 'fi'.
// Use 'elif' for cleaner else-if chains that only need one closing 'fi'.
//
// Examples:
// Bad:
//   if [ $x -eq 1 ]; then
//       echo one
//   else
//       if [ $x -eq 2 ]; then     # Needs extra fi
//           echo two
//       fi
//   fi
//
// Good:
//   if [ $x -eq 1 ]; then
//       echo one
//   elif [ $x -eq 2 ]; then       # Cleaner, one fi
//       echo two
//   fi

use crate::linter::{Diagnostic, LintResult, Severity, Span};

pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();
    let lines: Vec<&str> = source.lines().collect();

    let mut saw_else = false;
    let mut else_line_num = 0;

    for (line_num, line) in lines.iter().enumerate() {
        let line_num = line_num + 1;
        let trimmed = line.trim();

        if trimmed.starts_with('#') {
            continue;
        }

        // Skip empty lines when tracking state
        if trimmed.is_empty() {
            continue;
        }

        if saw_else {
            // Check if this non-empty, non-comment line starts with 'if'
            if trimmed.starts_with("if ") || trimmed == "if" {
                result.add(Diagnostic::new(
                    "SC1131",
                    Severity::Info,
                    "Use 'elif' instead of 'else' followed by 'if' (avoids extra 'fi').".to_string(),
                    Span::new(else_line_num, 1, line_num, trimmed.len() + 1),
                ));
            }
            saw_else = false;
        }

        // Check if this line ends with or is 'else'
        // Handle: "else", "else  ", or lines ending with "else" (after ; or similar)
        if trimmed == "else" || trimmed.ends_with("else") {
            // Make sure "else" is a standalone keyword, not part of a word
            if trimmed == "else"
                || trimmed.ends_with(" else")
                || trimmed.ends_with(";else")
                || trimmed.ends_with("\telse")
            {
                saw_else = true;
                else_line_num = line_num;
            }
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sc1131_else_then_if() {
        let code = "if [ $x -eq 1 ]; then\n    echo one\nelse\n    if [ $x -eq 2 ]; then\n        echo two\n    fi\nfi";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
        assert_eq!(result.diagnostics[0].code, "SC1131");
        assert_eq!(result.diagnostics[0].severity, Severity::Info);
        assert!(result.diagnostics[0].message.contains("elif"));
    }

    #[test]
    fn test_sc1131_elif_ok() {
        let code = "if [ $x -eq 1 ]; then\n    echo one\nelif [ $x -eq 2 ]; then\n    echo two\nfi";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc1131_else_with_other_command_ok() {
        let code = "if [ $x -eq 1 ]; then\n    echo one\nelse\n    echo other\nfi";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc1131_else_if_with_blank_lines() {
        let code = "if [ $x -eq 1 ]; then\n    echo one\nelse\n\n    if [ $x -eq 2 ]; then\n        echo two\n    fi\nfi";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc1131_comment_ok() {
        let code = "# else\n# if [ $x -eq 2 ]; then";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc1131_else_echo_if_ok() {
        let code = "if true; then\n    echo a\nelse\n    echo if something\nfi";
        let result = check(code);
        // "echo if something" does not start with "if " as a standalone word at line start
        // Actually "echo if something" trimmed starts with "echo" not "if"
        assert_eq!(result.diagnostics.len(), 0);
    }
}
