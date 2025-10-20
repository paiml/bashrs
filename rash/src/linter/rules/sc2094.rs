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

pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();

    for (line_num, line) in source.lines().enumerate() {
        let line_num = line_num + 1;

        if line.trim_start().starts_with('#') {
            continue;
        }

        // Skip lines with >> (append is generally safer)
        if line.contains(">>") {
            continue;
        }

        // Skip lines without > redirect
        if !line.contains(" > ") && !line.contains(">") {
            continue;
        }

        // Look for pattern: command file ... > file
        // Simple heuristic: extract words that look like filenames (have extensions)
        // and check if same filename appears before and after >
        let parts: Vec<&str> = line.split('>').collect();
        if parts.len() != 2 {
            continue; // Not a simple redirect
        }

        let before_redirect = parts[0];
        let after_redirect = parts[1];

        // Extract potential filename after >
        let output_file = after_redirect
            .split_whitespace()
            .next()
            .unwrap_or("");

        if output_file.is_empty() || !output_file.contains('.') {
            continue;
        }

        // Check if this filename appears in the command before redirect
        // Match word boundaries to avoid false positives
        let search_pattern = format!("\\b{}\\b", regex::escape(output_file));
        if let Ok(re) = regex::Regex::new(&search_pattern) {
            if re.is_match(before_redirect) {
                let diagnostic = Diagnostic::new(
                    "SC2094",
                    Severity::Warning,
                    format!(
                        "Make sure not to read and write the same file '{}'. Use a temp file or in-place editing instead",
                        output_file
                    ),
                    Span::new(line_num, 1, line_num, line.len()),
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
