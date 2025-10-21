// SC2041: Use while read, not read in for loop, to read lines from files.
//
// Using `read` inside a for loop doesn't work as expected. The `read` command
// reads ONE line from stdin each iteration, not from the loop's data source.
//
// Examples:
// Bad:
//   for line in $(cat file.txt); do
//     read -r data  // Reads from stdin, not from cat output!
//   done
//
//   for i in 1 2 3; do
//     read var      // Reads from stdin, unrelated to loop
//   done
//
// Good:
//   while IFS= read -r line; do
//     echo "$line"  // Correctly reads each line from file
//   done < file.txt
//
//   while read -r var; do
//     process "$var"
//   done < data.txt
//
// Note: This is a common mistake. `read` always reads from stdin (fd 0),
// not from the for loop's iteration variable.

use crate::linter::{Diagnostic, LintResult, Severity, Span};
use once_cell::sync::Lazy;
use regex::Regex;

static READ_IN_FOR: Lazy<Regex> = Lazy::new(|| {
    // Match: for ... do ... read ...
    // We'll use a simpler heuristic: look for `read` command inside for loops
    Regex::new(r"\bread\s+").unwrap()
});

pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();
    let mut in_for_loop = false;
    let mut for_loop_start_line = 0;

    for (line_num, line) in source.lines().enumerate() {
        let line_num = line_num + 1;

        if line.trim_start().starts_with('#') {
            continue;
        }

        // Track for loop context
        if line.contains("for ") && line.contains(" in ") {
            in_for_loop = true;
            for_loop_start_line = line_num;

            // Handle single-line for loops (for x in y; do cmd; done)
            if line.contains("; do ") && line.contains("done") {
                // Check for read in this single line
                if line.contains("read ") {
                    let read_pos = line.find("read ").unwrap();
                    let do_pos = line.find("; do ").map(|p| p + 5).unwrap_or(0);
                    let done_pos = line.find("done").unwrap();

                    // Ensure read is between do and done
                    if read_pos >= do_pos && read_pos < done_pos {
                        let start_col = read_pos + 1;
                        let end_col = start_col + 5; // "read "

                        let diagnostic = Diagnostic::new(
                            "SC2041",
                            Severity::Warning,
                            format!(
                                "'read' in for loop reads from stdin, not loop data. Use 'while read' instead (for loop started at line {})",
                                for_loop_start_line
                            ),
                            Span::new(line_num, start_col, line_num, end_col),
                        );
                        result.add(diagnostic);
                    }
                }
                in_for_loop = false; // Single-line loop complete
                continue;
            }
        }

        // Reset context on `done` (end of loop)
        if in_for_loop && line.contains("done") {
            in_for_loop = false;
        }

        // Check for `read` inside for loop
        if in_for_loop && line.contains("read ") {
            if let Some(mat) = READ_IN_FOR.find(line) {
                // Skip if inside quotes
                let pos = mat.start();
                let before = &line[..pos];
                let quote_count = before.matches('"').count() + before.matches('\'').count();
                if quote_count % 2 == 1 {
                    continue;
                }

                // Skip if it's part of `while read` (that's correct)
                if line.contains("while") && line.find("while").unwrap_or(usize::MAX) < pos {
                    continue;
                }

                let start_col = pos + 1;
                let end_col = start_col + mat.as_str().len();

                let diagnostic = Diagnostic::new(
                    "SC2041",
                    Severity::Warning,
                    format!(
                        "'read' in for loop reads from stdin, not loop data. Use 'while read' instead (for loop started at line {})",
                        for_loop_start_line
                    ),
                    Span::new(line_num, start_col, line_num, end_col),
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
    fn test_sc2041_read_in_for() {
        let code = r#"
for line in $(cat file.txt); do
  read -r data
done
"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
        assert_eq!(result.diagnostics[0].code, "SC2041");
        assert!(result.diagnostics[0].message.contains("while read"));
    }

    #[test]
    fn test_sc2041_read_simple_for() {
        let code = r#"
for i in 1 2 3; do
  read var
  echo "$var"
done
"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2041_while_read_ok() {
        let code = r#"
while IFS= read -r line; do
  echo "$line"
done < file.txt
"#;
        let result = check(code);
        // while read is correct
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2041_read_outside_for_ok() {
        let code = r#"
read -r input
echo "$input"
"#;
        let result = check(code);
        // read outside for loop is OK
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2041_in_quotes_ok() {
        let code = r#"
for i in 1 2 3; do
  echo "read -r var"
done
"#;
        let result = check(code);
        // Inside quotes, not actual command
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2041_comment_ok() {
        let code = r#"
for i in 1 2 3; do
  # read -r var
  echo "$i"
done
"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2041_nested_while_read_ok() {
        let code = r#"
for dir in /var/log/*; do
  while read -r line; do
    echo "$line"
  done < "$dir/access.log"
done
"#;
        let result = check(code);
        // while read inside for is OK (different pattern)
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2041_single_line_for() {
        let code = r#"for x in a b c; do read y; echo "$y"; done"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2041_multiple_reads() {
        let code = r#"
for item in list; do
  read first
  read second
done
"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 2);
    }

    #[test]
    fn test_sc2041_read_with_options() {
        let code = r#"
for i in 1 2 3; do
  read -r -p "Enter: " value
done
"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }
}
