// SC2094: Make sure not to read and write the same file in the same pipeline
//
// Reading and writing to the same file in a pipeline can lead to unexpected results.
// The output redirection truncates the file before the command reads it.
//
// Examples:
// Bad:
//   cat file.txt | grep pattern > file.txt
//   sort file.txt > file.txt
//   sed 's/foo/bar/' file.txt > file.txt
//
// Good:
//   grep pattern file.txt > temp && mv temp file.txt
//   sort file.txt > file.txt.tmp && mv file.txt.tmp file.txt
//   sed -i 's/foo/bar/' file.txt  # In-place editing

use crate::linter::{Diagnostic, LintResult, Severity, Span};

/// Check if a line is a comment
fn is_comment_line(line: &str) -> bool {
    line.trim_start().starts_with('#')
}

/// Check if line uses append (>>)
fn is_append_redirect(line: &str) -> bool {
    line.contains(">>")
}

/// Check if line has a redirect
fn has_redirect(line: &str) -> bool {
    line.contains(" > ") || line.contains(">")
}

/// Check if line is a simple redirect (exactly one > separator)
fn is_simple_redirect(parts: &[&str]) -> bool {
    parts.len() == 2
}

/// Extract output filename from redirect
fn extract_output_filename(after_redirect: &str) -> Option<&str> {
    let output_file = after_redirect.split_whitespace().next().unwrap_or("");
    if output_file.is_empty() || !output_file.contains('.') {
        None
    } else {
        Some(output_file)
    }
}

/// Check if filename appears in command before redirect
fn filename_in_command(before_redirect: &str, filename: &str) -> bool {
    let search_pattern = format!("\\b{}\\b", regex::escape(filename));
    if let Ok(re) = regex::Regex::new(&search_pattern) {
        re.is_match(before_redirect)
    } else {
        false
    }
}

/// Create diagnostic for same file read/write
fn create_same_file_diagnostic(filename: &str, line_num: usize, line_len: usize) -> Diagnostic {
    Diagnostic::new(
        "SC2094",
        Severity::Warning,
        format!(
            "Make sure not to read and write the same file '{}'. Use a temp file or in-place editing instead",
            filename
        ),
        Span::new(line_num, 1, line_num, line_len),
    )
}

pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();

    for (line_num, line) in source.lines().enumerate() {
        let line_num = line_num + 1;

        if is_comment_line(line) || is_append_redirect(line) || !has_redirect(line) {
            continue;
        }

        // Look for pattern: command file ... > file
        let parts: Vec<&str> = line.split('>').collect();
        if !is_simple_redirect(&parts) {
            continue;
        }

        let before_redirect = parts[0];
        let after_redirect = parts[1];

        // Extract potential filename after >
        if let Some(output_file) = extract_output_filename(after_redirect) {
            // Check if this filename appears in the command before redirect
            if filename_in_command(before_redirect, output_file) {
                let diagnostic = create_same_file_diagnostic(output_file, line_num, line.len());
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
    fn prop_sc2094_comments_never_diagnosed() {
        // Property: Comment lines should never produce diagnostics
        let test_cases = vec![
            "# cat file.txt > file.txt",
            "  # sort data.csv > data.csv",
            "\t# sed 's/a/b/' input.txt > input.txt",
        ];

        for code in test_cases {
            let result = check(code);
            assert_eq!(result.diagnostics.len(), 0);
        }
    }

    #[test]
    fn prop_sc2094_append_never_diagnosed() {
        // Property: Append (>>) should not be diagnosed
        let test_cases = vec![
            "cat file.txt >> file.txt",
            "echo line >> data.txt",
            "grep foo log.txt >> log.txt",
        ];

        for code in test_cases {
            let result = check(code);
            assert_eq!(result.diagnostics.len(), 0);
        }
    }

    #[test]
    fn prop_sc2094_different_files_never_diagnosed() {
        // Property: Different input/output files should not be diagnosed
        let test_cases = vec![
            "cat input.txt > output.txt",
            "sort data.csv > sorted.csv",
            "grep foo log.txt > results.txt",
        ];

        for code in test_cases {
            let result = check(code);
            assert_eq!(result.diagnostics.len(), 0);
        }
    }

    #[test]
    fn prop_sc2094_same_file_always_diagnosed() {
        // Property: Same input and output file should always be diagnosed
        let test_cases = vec![
            ("cat file.txt > file.txt", "file.txt"),
            ("sort data.csv > data.csv", "data.csv"),
            ("sed 's/a/b/' input.txt > input.txt", "input.txt"),
            ("awk '{print $1}' log.txt > log.txt", "log.txt"),
        ];

        for (code, filename) in test_cases {
            let result = check(code);
            assert_eq!(result.diagnostics.len(), 1, "Should diagnose: {}", code);
            assert!(result.diagnostics[0].message.contains(filename));
        }
    }

    #[test]
    fn prop_sc2094_temp_files_never_diagnosed() {
        // Property: Using temp files should not be diagnosed
        let test_cases = vec![
            "sort file.txt > file.txt.tmp",
            "cat data.csv > temp.csv",
            "sed 's/a/b/' input.txt > input.tmp",
        ];

        for code in test_cases {
            let result = check(code);
            assert_eq!(result.diagnostics.len(), 0);
        }
    }

    #[test]
    fn prop_sc2094_in_place_editing_never_diagnosed() {
        // Property: In-place editing should not be diagnosed
        let test_cases = vec![
            "sed -i 's/foo/bar/' file.txt",
            "perl -pi -e 's/a/b/' data.txt",
            "awk -i inplace '{print}' log.txt",
        ];

        for code in test_cases {
            let result = check(code);
            assert_eq!(result.diagnostics.len(), 0);
        }
    }

    #[test]
    fn prop_sc2094_diagnostic_code_always_sc2094() {
        // Property: All diagnostics must have code "SC2094"
        let code = "cat f1.txt > f1.txt\nsort f2.csv > f2.csv";
        let result = check(code);

        for diagnostic in &result.diagnostics {
            assert_eq!(&diagnostic.code, "SC2094");
        }
    }

    #[test]
    fn prop_sc2094_diagnostic_severity_always_warning() {
        // Property: All diagnostics must be Warning severity
        let code = "cat file.txt > file.txt";
        let result = check(code);

        for diagnostic in &result.diagnostics {
            assert_eq!(diagnostic.severity, Severity::Warning);
        }
    }

    #[test]
    fn prop_sc2094_empty_source_no_diagnostics() {
        // Property: Empty source should produce no diagnostics
        let result = check("");
        assert_eq!(result.diagnostics.len(), 0);
    }

    // ===== Original Unit Tests =====

    #[test]
    fn test_sc2094_cat_pipe_redirect() {
        let code = r#"cat file.txt | grep pattern > file.txt"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
        assert_eq!(result.diagnostics[0].code, "SC2094");
        assert_eq!(result.diagnostics[0].severity, Severity::Warning);
        assert!(result.diagnostics[0].message.contains("file.txt"));
    }

    #[test]
    fn test_sc2094_sort_redirect() {
        let code = r#"sort file.txt > file.txt"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
        assert!(result.diagnostics[0].message.contains("file.txt"));
    }

    #[test]
    fn test_sc2094_sed_redirect() {
        let code = r#"sed 's/foo/bar/' input.txt > input.txt"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2094_different_files_ok() {
        let code = r#"cat input.txt | grep pattern > output.txt"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2094_temp_file_ok() {
        let code = r#"sort file.txt > file.txt.tmp && mv file.txt.tmp file.txt"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2094_sed_inplace_ok() {
        let code = r#"sed -i 's/foo/bar/' file.txt"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2094_awk_redirect() {
        let code = r#"awk '{print $1}' data.csv > data.csv"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2094_grep_pipe() {
        let code = r#"grep foo log.txt | sort > log.txt"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2094_multiple_pipes_ok() {
        let code = r#"cat input.txt | grep foo | sort | uniq > output.txt"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2094_append_ok() {
        let code = r#"echo "new line" >> file.txt"#;
        let result = check(code);
        // >> (append) is generally safe
        assert_eq!(result.diagnostics.len(), 0);
    }
}
