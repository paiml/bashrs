// SC1045: Not `foo &; bar` - remove the `;`
//
// `&` already acts as a command terminator (like `;`), so `&;` is a syntax
// error. Remove the `;`.
//
// Examples:
// Bad:
//   sleep 10 &; echo "started"
//   cmd1 &; cmd2
//
// Good:
//   sleep 10 & echo "started"
//   cmd1 & cmd2

use crate::linter::{Diagnostic, LintResult, Severity, Span};

pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();

    for (line_num, line) in source.lines().enumerate() {
        let line_num = line_num + 1;
        let trimmed = line.trim_start();

        if trimmed.starts_with('#') {
            continue;
        }

        // Simple scan for `&;` outside of quotes
        let bytes = line.as_bytes();
        let mut in_single_quote = false;
        let mut in_double_quote = false;

        for i in 0..bytes.len() {
            let ch = bytes[i] as char;

            if ch == '\'' && !in_double_quote {
                in_single_quote = !in_single_quote;
            } else if ch == '"' && !in_single_quote {
                in_double_quote = !in_double_quote;
            }

            if !in_single_quote && !in_double_quote && ch == '&' && i + 1 < bytes.len() {
                let next = bytes[i + 1] as char;
                // &; is the error, but skip && (logical and)
                if next == ';' {
                    let start_col = i + 1;
                    let end_col = i + 3;

                    let diagnostic = Diagnostic::new(
                        "SC1045",
                        Severity::Error,
                        "It's not `&;` - `&` already terminates the command. Remove the `;`."
                            .to_string(),
                        Span::new(line_num, start_col, line_num, end_col),
                    );

                    result.add(diagnostic);
                }
            }
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sc1045_ampersand_semicolon() {
        let code = "sleep 10 &; echo started";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
        assert_eq!(result.diagnostics[0].code, "SC1045");
        assert_eq!(result.diagnostics[0].severity, Severity::Error);
    }

    #[test]
    fn test_sc1045_multiple() {
        let code = "cmd1 &; cmd2 &; cmd3";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 2);
    }

    #[test]
    fn test_sc1045_ampersand_ok() {
        let code = "sleep 10 & echo started";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc1045_logical_and_ok() {
        let code = "cmd1 && cmd2";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc1045_in_quotes_ok() {
        let code = r#"echo "&;""#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc1045_comment_ok() {
        let code = "# cmd &; next";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }
}
