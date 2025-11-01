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

/// Check if a line is a comment or empty
fn is_comment_or_empty(line: &str) -> bool {
    let trimmed = line.trim();
    trimmed.starts_with('#') || trimmed.is_empty()
}

/// Extract redirect filename from line
fn extract_redirect_file(line: &str) -> Option<String> {
    APPEND_REDIRECT
        .captures(line.trim())
        .map(|cap| cap.get(1).unwrap().as_str().to_string())
}

/// Check if group should be added (count >= 2)
fn should_add_group(count: usize) -> bool {
    count >= 2
}

/// Add group to list if count threshold met
fn add_group_if_needed(
    groups: &mut Vec<(String, usize, usize)>,
    file: Option<String>,
    start: usize,
    count: usize,
) {
    if should_add_group(count) {
        groups.push((file.unwrap(), start, count));
    }
}

/// Create diagnostic for redirect group
fn create_redirect_group_diagnostic(file: &str, start_line: usize, count: usize) -> Diagnostic {
    Diagnostic::new(
        "SC2129",
        Severity::Info,
        format!(
            "Consider using {{ cmd1; cmd2; }} >> {} instead of {} individual redirects for better performance",
            file, count
        ),
        Span::new(start_line, 1, start_line + count - 1, 1),
    )
}

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

        if is_comment_or_empty(line) {
            // End current group on blank lines or comments
            add_group_if_needed(
                &mut consecutive_groups,
                current_file.clone(),
                current_start,
                current_count,
            );
            current_file = None;
            current_count = 0;
            continue;
        }

        if let Some(file) = extract_redirect_file(line) {
            if let Some(ref curr_file) = current_file {
                if curr_file == &file {
                    // Continue current group
                    current_count += 1;
                } else {
                    // Different file, start new group
                    add_group_if_needed(
                        &mut consecutive_groups,
                        Some(curr_file.clone()),
                        current_start,
                        current_count,
                    );
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
            add_group_if_needed(
                &mut consecutive_groups,
                current_file.clone(),
                current_start,
                current_count,
            );
            current_file = None;
            current_count = 0;
        }
    }

    // Check last group
    add_group_if_needed(
        &mut consecutive_groups,
        current_file,
        current_start,
        current_count,
    );

    // Generate warnings for groups of 2+ consecutive redirects
    for (file, start_line, count) in consecutive_groups {
        let diagnostic = create_redirect_group_diagnostic(&file, start_line, count);
        result.add(diagnostic);
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    // ===== Manual Property Tests =====

    #[test]
    fn prop_sc2129_comments_break_groups() {
        // Property: Comments should break consecutive groups
        let test_cases = vec![
            "echo a >> f\n# comment\necho b >> f",
            "cat d >> f\n  # note\ncat e >> f",
        ];

        for code in test_cases {
            let result = check(code);
            assert_eq!(result.diagnostics.len(), 0);
        }
    }

    #[test]
    fn prop_sc2129_blank_lines_break_groups() {
        // Property: Blank lines should break consecutive groups
        let code = "echo a >> f\necho b >> f\n\necho c >> f\necho d >> f";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 2);
    }

    #[test]
    fn prop_sc2129_single_redirect_never_diagnosed() {
        // Property: Single redirects should not be diagnosed
        let test_cases = vec![
            "echo line >> file.txt",
            "cat data >> output.log",
            "printf test >> result.txt",
        ];

        for code in test_cases {
            let result = check(code);
            assert_eq!(result.diagnostics.len(), 0);
        }
    }

    #[test]
    fn prop_sc2129_consecutive_same_file_always_diagnosed() {
        // Property: 2+ consecutive redirects to same file should be diagnosed
        let test_cases = vec![
            ("echo a >> f\necho b >> f", "f", 2),
            ("cat x >> log\ncat y >> log\ncat z >> log", "log", 3),
            (
                "echo 1 >> out\necho 2 >> out\necho 3 >> out\necho 4 >> out",
                "out",
                4,
            ),
        ];

        for (code, filename, count) in test_cases {
            let result = check(code);
            assert_eq!(result.diagnostics.len(), 1, "Should diagnose: {}", code);
            assert!(result.diagnostics[0].message.contains(filename));
            assert!(result.diagnostics[0].message.contains(&count.to_string()));
        }
    }

    #[test]
    fn prop_sc2129_different_files_never_diagnosed() {
        // Property: Redirects to different files should not be diagnosed
        let test_cases = vec![
            "echo a >> f1\necho b >> f2",
            "cat x >> log1\ncat y >> log2\ncat z >> log3",
        ];

        for code in test_cases {
            let result = check(code);
            assert_eq!(result.diagnostics.len(), 0);
        }
    }

    #[test]
    fn prop_sc2129_non_consecutive_never_diagnosed() {
        // Property: Non-consecutive redirects should not be diagnosed
        let code = "echo a >> f\necho middle\necho b >> f";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn prop_sc2129_write_redirect_never_diagnosed() {
        // Property: Write redirects (>) should not be diagnosed
        let code = "echo a > f\necho b >> f";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn prop_sc2129_diagnostic_code_always_sc2129() {
        // Property: All diagnostics must have code "SC2129"
        let code = "echo a >> f\necho b >> f\n\ncat x >> g\ncat y >> g";
        let result = check(code);

        for diagnostic in &result.diagnostics {
            assert_eq!(&diagnostic.code, "SC2129");
        }
    }

    #[test]
    fn prop_sc2129_diagnostic_severity_always_info() {
        // Property: All diagnostics must be Info severity
        let code = "echo a >> f\necho b >> f";
        let result = check(code);

        for diagnostic in &result.diagnostics {
            assert_eq!(diagnostic.severity, Severity::Info);
        }
    }

    #[test]
    fn prop_sc2129_empty_source_no_diagnostics() {
        // Property: Empty source should produce no diagnostics
        let result = check("");
        assert_eq!(result.diagnostics.len(), 0);
    }

    // ===== Original Unit Tests =====

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
