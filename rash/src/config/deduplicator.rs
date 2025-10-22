//! CONFIG-001: Deduplicate PATH entries
//!
//! This module detects and removes duplicate PATH entries in shell configuration
//! files while preserving the order of first occurrence.

use super::{ConfigIssue, PathEntry, Severity};
use std::collections::HashMap;

/// Analyze PATH entries in shell script source
pub fn analyze_path_entries(source: &str) -> Vec<PathEntry> {
    let mut entries = Vec::new();
    let mut seen = HashMap::new();
    let mut line_num = 0;

    for line in source.lines() {
        line_num += 1;

        // Match: export PATH="/some/path:$PATH"
        // Match: export PATH="/some/path:${PATH}"
        // Match: PATH="/some/path:$PATH"
        if let Some(path) = extract_path_addition(line) {
            let is_duplicate = seen.contains_key(&path);
            if is_duplicate {
                // Track which line had the original
                let _original_line = seen.get(&path).copied();
            } else {
                seen.insert(path.clone(), line_num);
            }

            entries.push(PathEntry {
                line: line_num,
                path,
                is_duplicate,
            });
        }
    }

    entries
}

/// Extract the path being added from a PATH export line
fn extract_path_addition(line: &str) -> Option<String> {
    let line = line.trim();

    // Skip comments
    if line.starts_with('#') {
        return None;
    }

    // Match patterns like:
    // export PATH="/usr/local/bin:$PATH"
    // export PATH="/usr/local/bin:${PATH}"
    // PATH="/usr/local/bin:$PATH"

    if !line.contains("PATH") || !line.contains('=') {
        return None;
    }

    // Extract the part after = and before :$PATH or :${PATH}
    let parts: Vec<&str> = line.split('=').collect();
    if parts.len() < 2 {
        return None;
    }

    let value = parts[1].trim();

    // Remove quotes
    let value = value.trim_matches('"').trim_matches('\'');

    // Extract the path being added (before :$PATH or :${PATH})
    if let Some(colon_pos) = value.find(':') {
        let path = &value[..colon_pos];
        Some(path.to_string())
    } else {
        None
    }
}

/// Generate issues for duplicate PATH entries
pub fn detect_duplicate_paths(entries: &[PathEntry]) -> Vec<ConfigIssue> {
    let mut issues = Vec::new();

    for entry in entries {
        if entry.is_duplicate {
            issues.push(ConfigIssue {
                rule_id: "CONFIG-001".to_string(),
                severity: Severity::Warning,
                message: format!(
                    "Duplicate PATH entry: '{}' (already added earlier)",
                    entry.path
                ),
                line: entry.line,
                column: 0,
                suggestion: Some(format!(
                    "Remove this line - '{}' is already in PATH",
                    entry.path
                )),
            });
        }
    }

    issues
}

/// Deduplicate PATH entries in source, preserving first occurrence
pub fn deduplicate_path_entries(source: &str) -> String {
    let entries = analyze_path_entries(source);
    let mut seen_paths = HashMap::new();
    let mut result = Vec::new();
    let mut line_num = 0;

    // First pass: identify which lines to keep
    for entry in &entries {
        if !entry.is_duplicate {
            seen_paths.insert(entry.line, true);
        }
    }

    // Second pass: rebuild source
    for line in source.lines() {
        line_num += 1;

        // Check if this is a PATH line
        let is_path_line = extract_path_addition(line).is_some();

        if is_path_line {
            // Only include if not a duplicate
            if seen_paths.contains_key(&line_num) {
                result.push(line.to_string());
            }
            // Duplicates are skipped
        } else {
            // Keep non-PATH lines as-is
            result.push(line.to_string());
        }
    }

    result.join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_001_extract_path_addition_basic() {
        // ARRANGE
        let line = r#"export PATH="/usr/local/bin:$PATH""#;

        // ACT
        let result = extract_path_addition(line);

        // ASSERT
        assert_eq!(result, Some("/usr/local/bin".to_string()));
    }

    #[test]
    fn test_config_001_extract_path_addition_with_braces() {
        // ARRANGE
        let line = r#"export PATH="/opt/homebrew/bin:${PATH}""#;

        // ACT
        let result = extract_path_addition(line);

        // ASSERT
        assert_eq!(result, Some("/opt/homebrew/bin".to_string()));
    }

    #[test]
    fn test_config_001_extract_path_addition_without_export() {
        // ARRANGE
        let line = r#"PATH="/home/user/.cargo/bin:$PATH""#;

        // ACT
        let result = extract_path_addition(line);

        // ASSERT
        assert_eq!(result, Some("/home/user/.cargo/bin".to_string()));
    }

    #[test]
    fn test_config_001_ignore_comments() {
        // ARRANGE
        let line = r#"# export PATH="/usr/local/bin:$PATH""#;

        // ACT
        let result = extract_path_addition(line);

        // ASSERT
        assert_eq!(result, None);
    }

    #[test]
    fn test_config_001_analyze_no_duplicates() {
        // ARRANGE
        let source = r#"
export PATH="/usr/local/bin:$PATH"
export PATH="/opt/homebrew/bin:$PATH"
export PATH="$HOME/.cargo/bin:$PATH"
        "#;

        // ACT
        let entries = analyze_path_entries(source);

        // ASSERT
        assert_eq!(entries.len(), 3);
        assert!(!entries[0].is_duplicate);
        assert!(!entries[1].is_duplicate);
        assert!(!entries[2].is_duplicate);
    }

    #[test]
    fn test_config_001_analyze_with_duplicates() {
        // ARRANGE
        let source = r#"
export PATH="/usr/local/bin:$PATH"
export PATH="/opt/homebrew/bin:$PATH"
export PATH="/usr/local/bin:$PATH"
        "#;

        // ACT
        let entries = analyze_path_entries(source);

        // ASSERT
        assert_eq!(entries.len(), 3);
        assert!(!entries[0].is_duplicate);  // First occurrence
        assert!(!entries[1].is_duplicate);  // Different path
        assert!(entries[2].is_duplicate);   // Duplicate of line 1
    }

    #[test]
    fn test_config_001_detect_duplicate_paths() {
        // ARRANGE
        let source = r#"
export PATH="/usr/local/bin:$PATH"
export PATH="/opt/homebrew/bin:$PATH"
export PATH="/usr/local/bin:$PATH"
export PATH="$HOME/.cargo/bin:$PATH"
export PATH="/usr/local/bin:$PATH"
        "#;

        let entries = analyze_path_entries(source);

        // ACT
        let issues = detect_duplicate_paths(&entries);

        // ASSERT
        assert_eq!(issues.len(), 2);  // Two duplicates
        assert_eq!(issues[0].rule_id, "CONFIG-001");
        assert_eq!(issues[0].severity, Severity::Warning);
        assert!(issues[0].message.contains("/usr/local/bin"));
        assert_eq!(issues[0].line, 4);  // Third line (first duplicate)
        assert_eq!(issues[1].line, 6);  // Fifth line (second duplicate)
    }

    #[test]
    fn test_config_001_deduplicate_removes_duplicates() {
        // ARRANGE
        let source = r#"export PATH="/usr/local/bin:$PATH"
export PATH="/opt/homebrew/bin:$PATH"
export PATH="/usr/local/bin:$PATH"
export PATH="$HOME/.cargo/bin:$PATH""#;

        let expected = r#"export PATH="/usr/local/bin:$PATH"
export PATH="/opt/homebrew/bin:$PATH"
export PATH="$HOME/.cargo/bin:$PATH""#;

        // ACT
        let result = deduplicate_path_entries(source);

        // ASSERT
        assert_eq!(result, expected);
    }

    #[test]
    fn test_config_001_deduplicate_preserves_non_path_lines() {
        // ARRANGE
        let source = r#"# My .bashrc
export EDITOR="vim"
export PATH="/usr/local/bin:$PATH"
alias ll='ls -la'
export PATH="/usr/local/bin:$PATH"
echo "Welcome!""#;

        let expected = r#"# My .bashrc
export EDITOR="vim"
export PATH="/usr/local/bin:$PATH"
alias ll='ls -la'
echo "Welcome!""#;

        // ACT
        let result = deduplicate_path_entries(source);

        // ASSERT
        assert_eq!(result, expected);
    }

    #[test]
    fn test_config_001_deduplicate_preserves_order() {
        // ARRANGE
        let source = r#"export PATH="/first:$PATH"
export PATH="/second:$PATH"
export PATH="/third:$PATH"
export PATH="/second:$PATH"
export PATH="/first:$PATH""#;

        let expected = r#"export PATH="/first:$PATH"
export PATH="/second:$PATH"
export PATH="/third:$PATH""#;

        // ACT
        let result = deduplicate_path_entries(source);

        // ASSERT
        assert_eq!(result, expected);
    }

    #[test]
    fn test_config_001_empty_input() {
        // ARRANGE
        let source = "";

        // ACT
        let entries = analyze_path_entries(source);
        let result = deduplicate_path_entries(source);

        // ASSERT
        assert_eq!(entries.len(), 0);
        assert_eq!(result, "");
    }

    #[test]
    fn test_config_001_no_path_entries() {
        // ARRANGE
        let source = r#"export EDITOR="vim"
alias ll='ls -la'
echo "Hello""#;

        // ACT
        let entries = analyze_path_entries(source);
        let result = deduplicate_path_entries(source);

        // ASSERT
        assert_eq!(entries.len(), 0);
        assert_eq!(result, source);
    }
}
