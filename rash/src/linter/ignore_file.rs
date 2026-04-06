//! `.bashrsignore` file support for excluding files and rules from linting
//!
//! Implements gitignore-style pattern matching for file exclusion,
//! plus rule code ignoring (Issue #85) and line-specific ignoring (Issue #109).
//!
//! # Syntax
//!
//! - `pattern` - Glob pattern to match files (e.g., `vendor/**/*.sh`)
//! - `RULE_CODE` - Rule code to ignore globally (e.g., `SEC010`, `SC2031`, `DET001`)
//! - `RULE_CODE:path` - Ignore rule only in specific file (Issue #109)
//! - `RULE_CODE:path:line` - Ignore rule only on specific line (Issue #109)
//! - `# comment` - Comments (lines starting with #)
//! - Empty lines are ignored
//! - `!pattern` - Negation (re-include a previously excluded file pattern)
//!
//! # Rule Code Format
//!
//! Rule codes are automatically detected when a line matches:
//! - SEC followed by digits (e.g., `SEC001`, `SEC010`)
//! - SC followed by digits (e.g., `SC2031`, `SC2046`)
//! - DET followed by digits (e.g., `DET001`, `DET002`)
//! - IDEM followed by digits (e.g., `IDEM001`, `IDEM002`)
//!
//! # Examples
//!
//! ```text
//! # .bashrsignore example
//!
//! # Ignore specific lint rules globally (Issue #85)
//! SEC010
//! SC2031
//! SC2046
//!
//! # Ignore rule only in specific file (Issue #109)
//! SEC010:scripts/install.sh
//! DET001:scripts/metrics.sh
//!
//! # Ignore rule only on specific line (Issue #109)
//! SEC010:scripts/install.sh:42
//! DET002:scripts/record.sh:15
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
use std::collections::{HashMap, HashSet};
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
    /// Rule codes to ignore globally (Issue #85)
    /// Stored in uppercase for case-insensitive matching
    ignored_rule_codes: HashSet<String>,
    /// Issue #109: File-specific rule ignores
    /// Key: normalized file path, Value: set of rule codes to ignore
    file_specific_ignores: HashMap<String, HashSet<String>>,
    /// Issue #109: Line-specific rule ignores
    /// Key: (normalized file path, line number), Value: set of rule codes to ignore
    line_specific_ignores: HashMap<(String, usize), HashSet<String>>,
}

#[derive(Debug)]
struct CompiledPattern {
    /// Original pattern string (for error messages)
    original: String,
    /// Compiled glob pattern
    pattern: Pattern,
}

/// Check if a string looks like a rule code (Issue #85)
///
/// Rule codes follow patterns like:
/// - SEC001, SEC010 (security rules)
/// - SC2031, SC2046 (shellcheck rules)
/// - DET001, DET002 (determinism rules)
/// - IDEM001, IDEM002 (idempotency rules)
fn is_rule_code(s: &str) -> bool {
    let s = s.trim().to_uppercase();

    // Check for common rule code patterns
    // SEC followed by digits
    if s.starts_with("SEC") && s.len() >= 4 && s[3..].chars().all(|c| c.is_ascii_digit()) {
        return true;
    }

    // SC followed by digits (shellcheck)
    if s.starts_with("SC") && s.len() >= 3 && s[2..].chars().all(|c| c.is_ascii_digit()) {
        return true;
    }

    // DET followed by digits
    if s.starts_with("DET") && s.len() >= 4 && s[3..].chars().all(|c| c.is_ascii_digit()) {
        return true;
    }

    // IDEM followed by digits
    if s.starts_with("IDEM") && s.len() >= 5 && s[4..].chars().all(|c| c.is_ascii_digit()) {
        return true;
    }

    false
}

/// Issue #109: Parse a rule specifier with optional file and line
///
/// Format: RULE_CODE or RULE_CODE:path or RULE_CODE:path:line
///
/// Returns: Some((rule_code, file_path, line_num)) if valid, None otherwise
fn parse_rule_specifier(s: &str) -> Option<(String, Option<String>, Option<usize>)> {
    let trimmed = s.trim();

    // Split by ':'
    let parts: Vec<&str> = trimmed.splitn(3, ':').collect();

    // First part must be a rule code
    let rule_code = parts.first()?;
    if !is_rule_code(rule_code) {
        return None;
    }

    match parts.len() {
        // Just rule code: SEC010
        1 => Some((rule_code.to_string(), None, None)),
        // Rule + file: SEC010:path/to/file.sh
        2 => {
            let path = parts[1].trim();
            if path.is_empty() {
                return None;
            }
            Some((rule_code.to_string(), Some(path.to_string()), None))
        }
        // Rule + file + line: SEC010:path/to/file.sh:42
        3 => {
            let path = parts[1].trim();
            let line_str = parts[2].trim();
            if path.is_empty() {
                return None;
            }
            let line_num = line_str.parse::<usize>().ok()?;
            Some((
                rule_code.to_string(),
                Some(path.to_string()),
                Some(line_num),
            ))
        }
        _ => None,
    }
}

/// Normalize a file path for consistent matching
fn normalize_path(path: &str) -> String {
    // Remove leading ./ if present
    let path = path.strip_prefix("./").unwrap_or(path);
    // Convert backslashes to forward slashes (Windows compatibility)
    path.replace('\\', "/")
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
    /// Supports both file patterns (glob syntax) and rule codes (Issue #85).
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
    ///
    /// # Ignore specific rules (Issue #85)
    /// SEC010
    /// SC2031
    /// "#;
    ///
    /// let ignore = IgnoreFile::parse(content).expect("valid patterns");
    /// assert!(ignore.should_ignore_rule("SEC010"));
    /// ```
    pub fn parse(content: &str) -> Result<Self, String> {
        let mut ignore = Self::default();

        for (line_num, line) in content.lines().enumerate() {
            let trimmed = line.trim();

            // Skip empty lines and comments
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }

            // Check for negation pattern (only applies to file patterns)
            let (is_negation, pattern_str) = if let Some(stripped) = trimmed.strip_prefix('!') {
                (true, stripped.trim())
            } else {
                (false, trimmed)
            };

            // Issue #85/#109: Check if this is a rule code (with optional file:line specifier)
            // Format: RULE_CODE or RULE_CODE:path or RULE_CODE:path:line
            if !is_negation {
                if let Some((rule_code, file_path, line_num)) = parse_rule_specifier(pattern_str) {
                    let rule_upper = rule_code.to_uppercase();

                    match (file_path, line_num) {
                        // Issue #109: Line-specific ignore
                        (Some(path), Some(line)) => {
                            let key = (normalize_path(&path), line);
                            ignore
                                .line_specific_ignores
                                .entry(key)
                                .or_default()
                                .insert(rule_upper);
                        }
                        // Issue #109: File-specific ignore
                        (Some(path), None) => {
                            let key = normalize_path(&path);
                            ignore
                                .file_specific_ignores
                                .entry(key)
                                .or_default()
                                .insert(rule_upper);
                        }
                        // Issue #85: Global rule ignore
                        (None, None) => {
                            ignore.ignored_rule_codes.insert(rule_upper);
                        }
                        // Invalid: line without file
                        (None, Some(_)) => {
                            // This shouldn't happen with proper parsing
                        }
                    }
                    continue;
                }
            }

            // Otherwise, treat as a file pattern
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

    /// Check if a rule code should be ignored globally (Issue #85)
    ///
    /// Rule codes are matched case-insensitively.
    /// This only checks global ignores; use `should_ignore_rule_at` for
    /// file/line-specific checks.
    ///
    /// # Examples
    ///
    /// ```
    /// use bashrs::linter::IgnoreFile;
    ///
    /// let content = "SEC010\nSC2031";
    /// let ignore = IgnoreFile::parse(content).expect("valid patterns");
    ///
    /// assert!(ignore.should_ignore_rule("SEC010"));
    /// assert!(ignore.should_ignore_rule("sec010")); // Case-insensitive
    /// assert!(ignore.should_ignore_rule("SC2031"));
    /// assert!(!ignore.should_ignore_rule("SEC001")); // Not in file
    /// ```
    pub fn should_ignore_rule(&self, rule_code: &str) -> bool {
        self.ignored_rule_codes.contains(&rule_code.to_uppercase())
    }

    /// Issue #109: Check if a rule should be ignored at a specific location
    ///
    /// Checks in order:
    /// 1. Line-specific ignores (RULE:file:line)
    /// 2. File-specific ignores (RULE:file)
    /// 3. Global ignores (RULE)
    ///
    /// # Examples
    ///
    /// ```
    /// use bashrs::linter::IgnoreFile;
    /// use std::path::Path;
    ///
    /// let content = r#"
    /// SEC010
    /// SC2031:scripts/install.sh
    /// DET001:scripts/metrics.sh:42
    /// "#;
    /// let ignore = IgnoreFile::parse(content).expect("valid patterns");
    ///
    /// // Global ignore applies everywhere
    /// assert!(ignore.should_ignore_rule_at("SEC010", Path::new("any/file.sh"), 1));
    ///
    /// // File-specific ignore only applies in that file
    /// assert!(ignore.should_ignore_rule_at("SC2031", Path::new("scripts/install.sh"), 1));
    /// assert!(!ignore.should_ignore_rule_at("SC2031", Path::new("other/file.sh"), 1));
    ///
    /// // Line-specific ignore only applies on that line
    /// assert!(ignore.should_ignore_rule_at("DET001", Path::new("scripts/metrics.sh"), 42));
    /// assert!(!ignore.should_ignore_rule_at("DET001", Path::new("scripts/metrics.sh"), 43));
    /// ```
    pub fn should_ignore_rule_at(&self, rule_code: &str, file_path: &Path, line: usize) -> bool {
        let rule_upper = rule_code.to_uppercase();
        let path_str = normalize_path(&file_path.to_string_lossy());

        // Check line-specific ignore first
        let line_key = (path_str.clone(), line);
        if let Some(rules) = self.line_specific_ignores.get(&line_key) {
            if rules.contains(&rule_upper) {
                return true;
            }
        }

        // Check file-specific ignore
        if let Some(rules) = self.file_specific_ignores.get(&path_str) {
            if rules.contains(&rule_upper) {
                return true;
            }
        }

        // Check global ignore
        self.ignored_rule_codes.contains(&rule_upper)
    }

    /// Get all ignored rule codes (Issue #85)
    ///
    /// Returns a vector of all rule codes that should be ignored,
    /// in uppercase form.
    ///
    /// # Examples
    ///
    /// ```
    /// use bashrs::linter::IgnoreFile;
    ///
    /// let content = "SEC010\nSC2031\nDET001";
    /// let ignore = IgnoreFile::parse(content).expect("valid patterns");
    ///
    /// let rules = ignore.ignored_rules();
    /// assert_eq!(rules.len(), 3);
    /// assert!(rules.contains(&"SEC010".to_string()));
    /// ```
    pub fn ignored_rules(&self) -> Vec<String> {
        self.ignored_rule_codes.iter().cloned().collect()
    }

    /// Check if there are any ignored rule codes (Issue #85)
    pub fn has_ignored_rules(&self) -> bool {
        !self.ignored_rule_codes.is_empty()
    }
}

// FIXME(PMAT-238): #[cfg(test)]
// FIXME(PMAT-238): #[path = "ignore_file_tests_issue_85.rs"]
// FIXME(PMAT-238): mod tests_extracted;
