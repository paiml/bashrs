//! `.bashrsignore` file support for excluding files from linting
//!
//! Implements gitignore-style pattern matching for file exclusion.
//!
//! # Syntax
//!
//! - `pattern` - Glob pattern to match files
//! - `# comment` - Comments (lines starting with #)
//! - Empty lines are ignored
//! - `!pattern` - Negation (re-include a previously excluded pattern)
//!
//! # Examples
//!
//! ```text
//! # .bashrsignore example
//!
//! # Exclude vendor scripts
//! vendor/**/*.sh
//!
//! # Exclude specific file with documented rationale
//! # Rationale: DET002 (timestamps) is intentional for metrics recording
//! scripts/record-metric.sh
//!
//! # Exclude by pattern
//! **/generated/*.sh
//!
//! # Re-include important.sh even if in vendor
//! !vendor/important.sh
//! ```

use glob::Pattern;
use std::fs;
use std::path::Path;

/// Result of checking if a file should be ignored
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IgnoreResult {
    /// File should be ignored (matched a pattern)
    Ignored(String), // The pattern that matched
    /// File should NOT be ignored
    NotIgnored,
}

/// Parser for `.bashrsignore` files
#[derive(Debug, Default)]
pub struct IgnoreFile {
    /// Include patterns (files to ignore)
    include_patterns: Vec<CompiledPattern>,
    /// Exclude patterns (files to NOT ignore, even if matched by include)
    exclude_patterns: Vec<CompiledPattern>,
}

#[derive(Debug)]
struct CompiledPattern {
    /// Original pattern string (for error messages)
    original: String,
    /// Compiled glob pattern
    pattern: Pattern,
}

impl IgnoreFile {
    /// Create an empty ignore file (no patterns)
    pub fn empty() -> Self {
        Self::default()
    }

    /// Load ignore patterns from a file
    ///
    /// Returns `Ok(None)` if the file doesn't exist.
    /// Returns `Ok(Some(IgnoreFile))` if the file exists and was parsed.
    /// Returns `Err` if the file exists but couldn't be read.
    ///
    /// # Examples
    ///
    /// ```
    /// use bashrs::linter::IgnoreFile;
    /// use std::path::Path;
    ///
    /// // Load from project root
    /// let ignore = IgnoreFile::load(Path::new(".bashrsignore"));
    /// ```
    pub fn load(path: &Path) -> Result<Option<Self>, String> {
        if !path.exists() {
            return Ok(None);
        }

        let content = fs::read_to_string(path)
            .map_err(|e| format!("Failed to read {}: {}", path.display(), e))?;

        Ok(Some(Self::parse(&content)?))
    }

    /// Parse ignore patterns from string content
    ///
    /// # Examples
    ///
    /// ```
    /// use bashrs::linter::IgnoreFile;
    ///
    /// let content = r#"
    /// # Vendor scripts
    /// vendor/*.sh
    ///
    /// # Generated files
    /// **/generated/*.sh
    /// "#;
    ///
    /// let ignore = IgnoreFile::parse(content).expect("valid patterns");
    /// ```
    pub fn parse(content: &str) -> Result<Self, String> {
        let mut ignore = Self::default();

        for (line_num, line) in content.lines().enumerate() {
            let trimmed = line.trim();

            // Skip empty lines and comments
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }

            // Check for negation pattern
            let (is_negation, pattern_str) = if let Some(stripped) = trimmed.strip_prefix('!') {
                (true, stripped.trim())
            } else {
                (false, trimmed)
            };

            // Compile the pattern
            let pattern = Pattern::new(pattern_str).map_err(|e| {
                format!(
                    "Invalid pattern on line {}: '{}' - {}",
                    line_num + 1,
                    pattern_str,
                    e
                )
            })?;

            let compiled = CompiledPattern {
                original: trimmed.to_string(),
                pattern,
            };

            if is_negation {
                ignore.exclude_patterns.push(compiled);
            } else {
                ignore.include_patterns.push(compiled);
            }
        }

        Ok(ignore)
    }

    /// Check if a file path should be ignored
    ///
    /// # Examples
    ///
    /// ```
    /// use bashrs::linter::{IgnoreFile, IgnoreResult};
    /// use std::path::Path;
    ///
    /// let content = "vendor/*.sh\n!vendor/important.sh";
    /// let ignore = IgnoreFile::parse(content).expect("valid patterns");
    ///
    /// // Matches include pattern
    /// assert!(matches!(
    ///     ignore.should_ignore(Path::new("vendor/foo.sh")),
    ///     IgnoreResult::Ignored(_)
    /// ));
    ///
    /// // Excluded by negation pattern
    /// assert_eq!(
    ///     ignore.should_ignore(Path::new("vendor/important.sh")),
    ///     IgnoreResult::NotIgnored
    /// );
    ///
    /// // Not matched at all
    /// assert_eq!(
    ///     ignore.should_ignore(Path::new("src/main.sh")),
    ///     IgnoreResult::NotIgnored
    /// );
    /// ```
    pub fn should_ignore(&self, path: &Path) -> IgnoreResult {
        let path_str = path.to_string_lossy();

        // Check exclude patterns first (negation wins)
        for pattern in &self.exclude_patterns {
            if pattern.pattern.matches(&path_str) {
                return IgnoreResult::NotIgnored;
            }
        }

        // Check include patterns
        for pattern in &self.include_patterns {
            if pattern.pattern.matches(&path_str) {
                return IgnoreResult::Ignored(pattern.original.clone());
            }
        }

        IgnoreResult::NotIgnored
    }

    /// Check if there are any patterns loaded
    pub fn has_patterns(&self) -> bool {
        !self.include_patterns.is_empty() || !self.exclude_patterns.is_empty()
    }

    /// Get the number of patterns
    pub fn pattern_count(&self) -> usize {
        self.include_patterns.len() + self.exclude_patterns.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_ignore_file() {
        let ignore = IgnoreFile::empty();
        assert!(!ignore.has_patterns());
        assert_eq!(
            ignore.should_ignore(Path::new("any/file.sh")),
            IgnoreResult::NotIgnored
        );
    }

    #[test]
    fn test_parse_simple_pattern() {
        let content = "vendor/*.sh";
        let ignore = IgnoreFile::parse(content).expect("valid pattern");

        assert!(ignore.has_patterns());
        assert_eq!(ignore.pattern_count(), 1);

        assert!(matches!(
            ignore.should_ignore(Path::new("vendor/foo.sh")),
            IgnoreResult::Ignored(_)
        ));
        assert_eq!(
            ignore.should_ignore(Path::new("src/main.sh")),
            IgnoreResult::NotIgnored
        );
    }

    #[test]
    fn test_parse_comments_and_empty_lines() {
        let content = r#"
# This is a comment
vendor/*.sh

# Another comment

**/generated/*.sh
"#;
        let ignore = IgnoreFile::parse(content).expect("valid patterns");

        // Should have 2 patterns (comments and empty lines ignored)
        assert_eq!(ignore.pattern_count(), 2);
    }

    #[test]
    fn test_negation_pattern() {
        let content = r#"
vendor/*.sh
!vendor/important.sh
"#;
        let ignore = IgnoreFile::parse(content).expect("valid patterns");

        // vendor/foo.sh should be ignored
        assert!(matches!(
            ignore.should_ignore(Path::new("vendor/foo.sh")),
            IgnoreResult::Ignored(_)
        ));

        // vendor/important.sh should NOT be ignored (negation)
        assert_eq!(
            ignore.should_ignore(Path::new("vendor/important.sh")),
            IgnoreResult::NotIgnored
        );
    }

    #[test]
    fn test_double_star_pattern() {
        let content = "**/generated/*.sh";
        let ignore = IgnoreFile::parse(content).expect("valid pattern");

        assert!(matches!(
            ignore.should_ignore(Path::new("src/generated/foo.sh")),
            IgnoreResult::Ignored(_)
        ));
        assert!(matches!(
            ignore.should_ignore(Path::new("deep/path/generated/bar.sh")),
            IgnoreResult::Ignored(_)
        ));
        assert_eq!(
            ignore.should_ignore(Path::new("src/main.sh")),
            IgnoreResult::NotIgnored
        );
    }

    #[test]
    fn test_exact_file_match() {
        let content = "scripts/record-metric.sh";
        let ignore = IgnoreFile::parse(content).expect("valid pattern");

        assert!(matches!(
            ignore.should_ignore(Path::new("scripts/record-metric.sh")),
            IgnoreResult::Ignored(_)
        ));
        assert_eq!(
            ignore.should_ignore(Path::new("scripts/other.sh")),
            IgnoreResult::NotIgnored
        );
    }

    #[test]
    fn test_issue_58_record_metric_script() {
        // Issue #58: .bashrsignore should allow excluding record-metric.sh
        let content = r#"
# Metrics recording script from paiml-mcp-agent-toolkit
# Rationale: DET002 (timestamps) and SEC010 (paths) are false positives
scripts/record-metric.sh
"#;
        let ignore = IgnoreFile::parse(content).expect("valid patterns");

        assert!(matches!(
            ignore.should_ignore(Path::new("scripts/record-metric.sh")),
            IgnoreResult::Ignored(_)
        ));
    }

    #[test]
    fn test_invalid_pattern_error() {
        // Invalid glob pattern (unclosed bracket)
        let content = "vendor/[invalid";
        let result = IgnoreFile::parse(content);

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.contains("Invalid pattern"));
    }

    #[test]
    fn test_load_nonexistent_file() {
        let result = IgnoreFile::load(Path::new("/nonexistent/.bashrsignore"));
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    #[test]
    fn test_multiple_patterns() {
        let content = r#"
# Exclude vendor and generated
vendor/**/*.sh
generated/**/*.sh

# But keep important ones
!vendor/critical.sh
"#;
        let ignore = IgnoreFile::parse(content).expect("valid patterns");

        assert_eq!(ignore.pattern_count(), 3); // 2 include + 1 exclude

        // vendor/foo.sh - ignored
        assert!(matches!(
            ignore.should_ignore(Path::new("vendor/foo.sh")),
            IgnoreResult::Ignored(_)
        ));

        // vendor/critical.sh - NOT ignored (negation)
        assert_eq!(
            ignore.should_ignore(Path::new("vendor/critical.sh")),
            IgnoreResult::NotIgnored
        );

        // generated/output.sh - ignored
        assert!(matches!(
            ignore.should_ignore(Path::new("generated/output.sh")),
            IgnoreResult::Ignored(_)
        ));

        // src/main.sh - not matched
        assert_eq!(
            ignore.should_ignore(Path::new("src/main.sh")),
            IgnoreResult::NotIgnored
        );
    }
}
