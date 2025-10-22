// SC2129: Consider using { cmd1; cmd2; } >> file instead of individual redirects
//
// When multiple commands redirect to the same file, it's more efficient and clearer
// to group them. This also reduces the number of file open/close operations.
//
// Examples:
// Bad:
//   echo "line1" >> file
//   echo "line2" >> file
//   echo "line3" >> file
//
// Good:
//   {
//     echo "line1"
//     echo "line2"
//     echo "line3"
//   } >> file
//
//   # Or use a function
//   write_data() {
//     echo "line1"
//     echo "line2"
//   }
//   write_data >> file

use crate::linter::{Diagnostic, LintResult, Severity, Span};
use once_cell::sync::Lazy;
use regex::Regex;
use std::collections::HashMap;

static APPEND_REDIRECT: Lazy<Regex> = Lazy::new(|| {
    // Match: command >> file
    Regex::new(r">>\s*([^\s;|&<>]+)").unwrap()
});

pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();
    let lines: Vec<&str> = source.lines().collect();

    // Track consecutive redirects to the same file
    let file_redirects: HashMap<String, Vec<usize>> = HashMap::new();
    let mut consecutive_groups: Vec<(String, usize, usize)> = Vec::new(); // (file, start_line, count)

    let mut current_file: Option<String> = None;
    let mut current_start: usize = 0;
    let mut current_count: usize = 0;

    for (idx, line) in lines.iter().enumerate() {
        let line_num = idx + 1;
        let trimmed = line.trim();

        if trimmed.starts_with('#') || trimmed.is_empty() {
            // End current group on blank lines or comments
            if current_count >= 2 {
                consecutive_groups.push((
                    current_file.clone().unwrap(),
                    current_start,
                    current_count,
                ));
            }
            current_file = None;
            current_count = 0;
            continue;
        }

        if let Some(cap) = APPEND_REDIRECT.captures(trimmed) {
            let file = cap.get(1).unwrap().as_str().to_string();

            if let Some(ref curr_file) = current_file {
                if curr_file == &file {
                    // Continue current group
                    current_count += 1;
                } else {
                    // Different file, start new group
                    if current_count >= 2 {
                        consecutive_groups.push((curr_file.clone(), current_start, current_count));
                    }
                    current_file = Some(file);
                    current_start = line_num;
                    current_count = 1;
                }
            } else {
                // Start new group
                current_file = Some(file);
                current_start = line_num;
                current_count = 1;
            }
        } else {
            // Line doesn't have redirect, end current group
            if current_count >= 2 {
                consecutive_groups.push((
                    current_file.clone().unwrap(),
                    current_start,
                    current_count,
                ));
            }
            current_file = None;
            current_count = 0;
        }
    }

    // Check last group
    if current_count >= 2 {
        consecutive_groups.push((current_file.unwrap(), current_start, current_count));
    }

    // Generate warnings for groups of 2+ consecutive redirects
    for (file, start_line, count) in consecutive_groups {
        let diagnostic = Diagnostic::new(
            "SC2129",
            Severity::Info,
            format!(
                "Consider using {{ cmd1; cmd2; }} >> {} instead of {} individual redirects for better performance",
                file, count
            ),
            Span::new(start_line, 1, start_line + count - 1, 1),
        );

        result.add(diagnostic);
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sc2129_multiple_redirects() {
        let code = r#"
echo "line1" >> file.txt
echo "line2" >> file.txt
echo "line3" >> file.txt
"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
        assert_eq!(result.diagnostics[0].code, "SC2129");
        assert_eq!(result.diagnostics[0].severity, Severity::Info);
        assert!(result.diagnostics[0].message.contains("cmd1; cmd2"));
    }

    #[test]
    fn test_sc2129_two_redirects() {
        let code = r#"
echo "a" >> out.log
echo "b" >> out.log
"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2129_single_redirect_ok() {
        let code = r#"
echo "line" >> file.txt
"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2129_different_files_ok() {
        let code = r#"
echo "a" >> file1.txt
echo "b" >> file2.txt
echo "c" >> file3.txt
"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2129_non_consecutive_ok() {
        let code = r#"
echo "a" >> file.txt
echo "middle"
echo "b" >> file.txt
"#;
        let result = check(code);
        // Not consecutive, so no warning
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2129_blank_line_breaks_group() {
        let code = r#"
echo "a" >> file.txt
echo "b" >> file.txt

echo "c" >> file.txt
echo "d" >> file.txt
"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 2);
    }

    #[test]
    fn test_sc2129_mixed_redirects() {
        let code = r#"
echo "a" >> file.txt
cat data >> file.txt
printf "%s\n" "b" >> file.txt
"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2129_comment_breaks_group() {
        let code = r#"
echo "a" >> file.txt
# Comment
echo "b" >> file.txt
"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2129_write_redirect_ok() {
        let code = r#"
echo "a" > file.txt
echo "b" >> file.txt
"#;
        let result = check(code);
        // > is not >> (append), so no warning
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2129_long_sequence() {
        let code = r#"
echo "1" >> log.txt
echo "2" >> log.txt
echo "3" >> log.txt
echo "4" >> log.txt
echo "5" >> log.txt
"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
        assert!(result.diagnostics[0].message.contains("5 individual"));
    }
}
