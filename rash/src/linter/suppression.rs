//! Inline suppression support for bashrs warnings (Issue #70)
//!
//! Allows users to suppress specific warnings using inline comments.
//! Supports BOTH bashrs-native syntax AND shellcheck syntax for compatibility.
//!
//! # Bashrs Syntax
//!
//! - File-level: `# bashrs disable-file=SC2086,DET002`
//! - Next-line: `# bashrs disable-next-line=SC2086`
//! - Shorthand: `# bashrs disable=SEC010` (alias for disable-next-line)
//! - Inline: `command  # bashrs disable-line=SC2086`
//!
//! # Shellcheck Syntax (also supported)
//!
//! - File-level: `# shellcheck disable=SC2086,SC2046` (at top of file)
//! - Next-line: `# shellcheck disable=SC2086` (before the line)
//!
//! # Examples
//!
//! ```bash
//! # bashrs disable-file=DET002
//! # Entire file exempt from DET002
//!
//! # bashrs disable-next-line=SC2086
//! echo $var  # Won't trigger SC2086
//!
//! # bashrs disable=SEC010  # Shorthand syntax (Issue #70)
//! mkdir -p "${BASELINE_DIR}"
//!
//! timestamp=$(date +%s)  # bashrs disable-line=DET002
//!
//! # shellcheck disable=SC2086
//! echo $var  # Also suppressed (shellcheck compatibility)
//! ```

use std::collections::{HashMap, HashSet};

/// Suppression directive type
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SuppressionType {
    /// Suppress rules for entire file
    File,
    /// Suppress rules for next line only
    NextLine,
    /// Suppress rules for current line only
    Line,
}

/// A suppression directive parsed from source comments
#[derive(Debug, Clone)]
pub struct Suppression {
    /// Type of suppression (file/next-line/line)
    pub suppression_type: SuppressionType,
    /// Line number where directive appears (1-indexed)
    pub line: usize,
    /// Rule codes to suppress (e.g., ["SC2086", "DET002"])
    pub rules: HashSet<String>,
}

/// Manages suppressions for a source file
#[derive(Debug, Default)]
pub struct SuppressionManager {
    /// File-level suppressions (apply to entire file)
    file_suppressions: HashSet<String>,
    /// Line-specific suppressions (line number -> rule codes)
    line_suppressions: HashMap<usize, HashSet<String>>,
}

impl SuppressionManager {
    /// Create a new suppression manager by parsing source code
    ///
    /// # Examples
    ///
    /// ## File-level suppression
    ///
    /// ```
    /// use bashrs::linter::SuppressionManager;
    ///
    /// let script = "# bashrs disable-file=SC2086,DET002\necho $var\ntimestamp=$(date +%s)";
    ///
    /// let manager = SuppressionManager::from_source(script);
    /// assert!(manager.is_suppressed("SC2086", 1));
    /// assert!(manager.is_suppressed("SC2086", 2));
    /// assert!(manager.is_suppressed("DET002", 3));
    /// ```
    ///
    /// ## Next-line suppression
    ///
    /// ```
    /// use bashrs::linter::SuppressionManager;
    ///
    /// let script = "# bashrs disable-next-line=SC2086\necho $var";
    ///
    /// let manager = SuppressionManager::from_source(script);
    /// assert!(manager.is_suppressed("SC2086", 2));
    /// assert!(!manager.is_suppressed("SC2086", 1));
    /// ```
    ///
    /// ## Inline suppression
    ///
    /// ```
    /// use bashrs::linter::SuppressionManager;
    ///
    /// let script = "echo $var  # bashrs disable-line=SC2086";
    ///
    /// let manager = SuppressionManager::from_source(script);
    /// assert!(manager.is_suppressed("SC2086", 1));
    /// assert!(!manager.is_suppressed("SC2002", 1));
    /// ```
    ///
    /// ## Shellcheck file-level suppression (Issue #130)
    ///
    /// Shellcheck directives at the top of a file (before any code) apply to
    /// the entire file:
    ///
    /// ```
    /// use bashrs::linter::SuppressionManager;
    ///
    /// let script = "#!/bin/bash\n# shellcheck disable=SC2086\necho $var";
    ///
    /// let manager = SuppressionManager::from_source(script);
    /// // Directive at top of file applies to all lines
    /// assert!(manager.is_suppressed("SC2086", 3));
    /// ```
    pub fn from_source(source: &str) -> Self {
        let mut manager = Self::default();
        let lines: Vec<&str> = source.lines().collect();

        // Issue #130: Track whether we've seen any code yet
        // Shellcheck directives before any code are file-level
        let mut seen_code = false;

        for (line_idx, line) in lines.iter().enumerate() {
            let line_num = line_idx + 1;
            let trimmed = line.trim();

            // Check if this line is code (not a comment, shebang, or empty)
            if !trimmed.is_empty()
                && !trimmed.starts_with('#')
                && !trimmed.starts_with("set ")
                && !trimmed.starts_with("shopt ")
            {
                seen_code = true;
            }

            // Check for suppression directives
            if let Some(suppression) = parse_suppression(line, line_num) {
                match suppression.suppression_type {
                    SuppressionType::File => {
                        // File-level suppression applies to all lines
                        manager.file_suppressions.extend(suppression.rules);
                    }
                    SuppressionType::NextLine => {
                        // Issue #130: Shellcheck directives at top of file are file-level
                        // Check if we've seen code yet - if not, treat as file-level
                        if !seen_code && is_shellcheck_directive(line) {
                            manager.file_suppressions.extend(suppression.rules);
                        } else {
                            // Next-line suppression applies to line_num + 1
                            if line_idx + 1 < lines.len() {
                                manager
                                    .line_suppressions
                                    .entry(line_num + 1)
                                    .or_default()
                                    .extend(suppression.rules);
                            }
                        }
                    }
                    SuppressionType::Line => {
                        // Inline suppression applies to current line
                        manager
                            .line_suppressions
                            .entry(line_num)
                            .or_default()
                            .extend(suppression.rules);
                    }
                }
            }
        }

        manager
    }

    /// Check if a rule is suppressed at a given line
    pub fn is_suppressed(&self, rule_code: &str, line: usize) -> bool {
        // Check file-level suppressions first
        if self.file_suppressions.contains(rule_code) {
            return true;
        }

        // Check line-specific suppressions
        if let Some(rules) = self.line_suppressions.get(&line) {
            if rules.contains(rule_code) {
                return true;
            }
        }

        false
    }
}

/// Issue #130: Check if line contains a shellcheck directive
fn is_shellcheck_directive(line: &str) -> bool {
    line.contains("# shellcheck disable=")
}

/// Parse a suppression directive from a line
/// Supports BOTH bashrs-native syntax AND shellcheck syntax
fn parse_suppression(line: &str, line_num: usize) -> Option<Suppression> {
    let trimmed = line.trim();

    // =====================================================
    // Bashrs-native syntax
    // =====================================================
    // Match patterns: # bashrs disable-file=SC2086,DET002
    // Match patterns: # bashrs disable-next-line=SC2086
    // Match patterns: command  # bashrs disable-line=SC2086

    if let Some(pos) = trimmed.find("# bashrs disable-file=") {
        let rules_str = &trimmed[pos + "# bashrs disable-file=".len()..];
        let rules = parse_rule_list(rules_str);
        return Some(Suppression {
            suppression_type: SuppressionType::File,
            line: line_num,
            rules,
        });
    }

    if let Some(pos) = trimmed.find("# bashrs disable-next-line=") {
        let rules_str = &trimmed[pos + "# bashrs disable-next-line=".len()..];
        let rules = parse_rule_list(rules_str);
        return Some(Suppression {
            suppression_type: SuppressionType::NextLine,
            line: line_num,
            rules,
        });
    }

    if let Some(pos) = line.find("# bashrs disable-line=") {
        let rules_str = &line[pos + "# bashrs disable-line=".len()..];
        let rules = parse_rule_list(rules_str);
        return Some(Suppression {
            suppression_type: SuppressionType::Line,
            line: line_num,
            rules,
        });
    }

    // Shorthand syntax: # bashrs disable=RULE (alias for disable-next-line)
    // Must check AFTER the more specific patterns to avoid matching them
    if let Some(pos) = trimmed.find("# bashrs disable=") {
        // Make sure it's not one of the more specific patterns
        if !trimmed.contains("disable-file=")
            && !trimmed.contains("disable-next-line=")
            && !trimmed.contains("disable-line=")
        {
            let rules_str = &trimmed[pos + "# bashrs disable=".len()..];
            let rules = parse_rule_list(rules_str);
            return Some(Suppression {
                suppression_type: SuppressionType::NextLine,
                line: line_num,
                rules,
            });
        }
    }

    // =====================================================
    // Shellcheck syntax (for compatibility)
    // =====================================================
    // Match patterns: # shellcheck disable=SC2086,SC2046
    // Shellcheck directives apply to the NEXT line (like bashrs disable-next-line)

    if let Some(pos) = trimmed.find("# shellcheck disable=") {
        let rules_str = &trimmed[pos + "# shellcheck disable=".len()..];
        let rules = parse_rule_list(rules_str);
        return Some(Suppression {
            suppression_type: SuppressionType::NextLine,
            line: line_num,
            rules,
        });
    }

    // Also support shellcheck source directive being ignored (not a suppression)
    // # shellcheck source=./lib.sh - we don't need to handle this

    None
}

/// Parse comma-separated rule list
/// Stops at parentheses and validates rule codes
fn parse_rule_list(rules_str: &str) -> HashSet<String> {
    // Strip trailing explanation text:
    // - "(validated via case statement)" - parenthesized explanations
    // - "# $* is intentional" - hash comments
    // - whitespace after rule codes
    let rules_part = rules_str
        .split('(')
        .next()
        .unwrap_or(rules_str)
        .split('#')
        .next()
        .unwrap_or(rules_str);

    rules_part
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty() && is_valid_rule_code(s))
        .collect()
}

/// Check if string looks like a valid rule code (e.g., SC2086, DET002, SEC010)
fn is_valid_rule_code(code: &str) -> bool {
    // Valid codes are uppercase letters followed by digits (e.g., SC2086, DET002)
    let code = code.trim();
    if code.len() < 3 || code.len() > 10 {
        return false;
    }

    // Must start with 1-6 uppercase letters
    let letter_count = code.chars().take_while(|c| c.is_ascii_uppercase()).count();
    if letter_count == 0 || letter_count > 6 {
        return false;
    }

    // Must end with 1-5 digits
    let digit_part = &code[letter_count..];
    !digit_part.is_empty() && digit_part.chars().all(|c| c.is_ascii_digit())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_file_level_suppression() {
        let source = "# bashrs disable-file=SC2086,DET002\necho $var\n";
        let manager = SuppressionManager::from_source(source);

        assert!(manager.is_suppressed("SC2086", 2));
        assert!(manager.is_suppressed("DET002", 2));
        assert!(!manager.is_suppressed("SC2046", 2));
    }

    #[test]
    fn test_parse_next_line_suppression() {
        let source = "# bashrs disable-next-line=SC2086\necho $var\n";
        let manager = SuppressionManager::from_source(source);

        assert!(manager.is_suppressed("SC2086", 2));
        assert!(!manager.is_suppressed("SC2086", 1));
        assert!(!manager.is_suppressed("SC2086", 3));
    }

    #[test]
    fn test_parse_inline_suppression() {
        let source = "echo $var  # bashrs disable-line=SC2086\n";
        let manager = SuppressionManager::from_source(source);

        assert!(manager.is_suppressed("SC2086", 1));
        assert!(!manager.is_suppressed("SC2086", 2));
    }

    #[test]
    fn test_multiple_rules() {
        let source = "# bashrs disable-next-line=SC2086,SC2046,DET002\necho $var\n";
        let manager = SuppressionManager::from_source(source);

        assert!(manager.is_suppressed("SC2086", 2));
        assert!(manager.is_suppressed("SC2046", 2));
        assert!(manager.is_suppressed("DET002", 2));
    }

    #[test]
    fn test_no_suppression() {
        let source = "echo $var\n";
        let manager = SuppressionManager::from_source(source);

        assert!(!manager.is_suppressed("SC2086", 1));
    }

    // =====================================================
    // Shorthand syntax tests (Issue #70)
    // =====================================================

    #[test]
    fn test_shorthand_disable_syntax() {
        // Issue #70: Support shorthand # bashrs disable=RULE
        let source = "# bashrs disable=SEC010\nmkdir -p \"${BASELINE_DIR}\"\n";
        let manager = SuppressionManager::from_source(source);

        assert!(manager.is_suppressed("SEC010", 2));
        assert!(!manager.is_suppressed("SEC010", 1));
        assert!(!manager.is_suppressed("SEC010", 3));
    }

    #[test]
    fn test_shorthand_disable_multiple_rules() {
        let source = "# bashrs disable=SEC010,DET002\nmkdir -p \"${BASELINE_DIR}\"\n";
        let manager = SuppressionManager::from_source(source);

        assert!(manager.is_suppressed("SEC010", 2));
        assert!(manager.is_suppressed("DET002", 2));
    }

    #[test]
    fn test_shorthand_does_not_match_specific_patterns() {
        // Ensure shorthand doesn't interfere with specific patterns
        let source = "# bashrs disable-file=SEC010\nline2\nline3\n";
        let manager = SuppressionManager::from_source(source);

        // File-level should suppress all lines
        assert!(manager.is_suppressed("SEC010", 1));
        assert!(manager.is_suppressed("SEC010", 2));
        assert!(manager.is_suppressed("SEC010", 3));
    }

    // =====================================================
    // Shellcheck syntax compatibility tests (Issue #58)
    // =====================================================

    #[test]
    fn test_shellcheck_disable_next_line() {
        // Shellcheck disable directives AFTER code apply to the next line only
        // Issue #130: Directives at top of file are file-level, so we need code first
        let source = "echo start\n# shellcheck disable=SC2086\necho $var\n";
        let manager = SuppressionManager::from_source(source);

        assert!(manager.is_suppressed("SC2086", 3)); // next line after directive
        assert!(!manager.is_suppressed("SC2086", 1)); // line before directive
        assert!(!manager.is_suppressed("SC2086", 2)); // directive line itself
    }

    #[test]
    fn test_shellcheck_disable_multiple_rules() {
        // Issue #130: Directives at top of file (before any code) are file-level
        let source = "# shellcheck disable=SC2086,SC2046,DET002\necho $var\n";
        let manager = SuppressionManager::from_source(source);

        // File-level suppression applies to all lines
        assert!(manager.is_suppressed("SC2086", 2));
        assert!(manager.is_suppressed("SC2046", 2));
        assert!(manager.is_suppressed("DET002", 2));
    }

    #[test]
    fn test_shellcheck_disable_does_not_affect_other_lines() {
        // After code, shellcheck directive only affects next line
        let source = "echo start\n# shellcheck disable=SC2086\necho $var\necho $another\n";
        let manager = SuppressionManager::from_source(source);

        // Only line 3 should be suppressed (next line after directive)
        assert!(manager.is_suppressed("SC2086", 3));
        assert!(!manager.is_suppressed("SC2086", 4)); // Line after that is NOT suppressed
    }

    #[test]
    fn test_mixed_bashrs_and_shellcheck_syntax() {
        let source = r#"
# shellcheck disable=SC2086
echo $var
# bashrs disable-next-line=SC2046
echo $(cat file)
"#;
        let manager = SuppressionManager::from_source(source);

        // SC2086 suppressed on line 3 (after shellcheck directive on line 2)
        assert!(manager.is_suppressed("SC2086", 3));
        // SC2046 suppressed on line 5 (after bashrs directive on line 4)
        assert!(manager.is_suppressed("SC2046", 5));
    }

    // Issue #130: Shellcheck file-level suppression tests

    #[test]
    fn test_shellcheck_file_level_suppression_at_top() {
        // Issue #130: Shellcheck directives at top of file (before any code)
        // should apply to the entire file
        let source = r#"#!/bin/bash
# shellcheck disable=SC2086
# shellcheck disable=SEC010
set -euo pipefail
echo $var
mkdir -p "$PATH/dir"
"#;
        let manager = SuppressionManager::from_source(source);

        // Directives at top should apply to all lines
        assert!(manager.is_suppressed("SC2086", 5)); // echo $var
        assert!(manager.is_suppressed("SC2086", 6)); // mkdir
        assert!(manager.is_suppressed("SEC010", 6)); // mkdir
    }

    #[test]
    fn test_shellcheck_mid_file_is_next_line_only() {
        // Shellcheck directive in the middle of code should only apply to next line
        let source = r#"#!/bin/bash
echo "hello"
# shellcheck disable=SC2086
echo $var
echo $another
"#;
        let manager = SuppressionManager::from_source(source);

        // After code, shellcheck directive is next-line only
        assert!(manager.is_suppressed("SC2086", 4)); // next line
        assert!(!manager.is_suppressed("SC2086", 5)); // NOT the line after
    }

    #[test]
    fn test_shellcheck_file_level_with_shebang_and_comments() {
        // Issue #130: Real-world pattern from raid-targets.sh
        let source = r#"#!/usr/bin/env bash
# raid-targets.sh - Symlink Cargo target directories to NVMe RAID
#
# shellcheck disable=SC2145  # $* is intentional for log concatenation
# shellcheck disable=SEC010  # Paths validated via validate_path()
# shellcheck disable=IDEM003 # ln -sfn IS idempotent

set -euo pipefail

log_info() { echo -e "$*"; }
mkdir -p "$RAID_PATH"
ln -sfn "$target" "$link"
"#;
        let manager = SuppressionManager::from_source(source);

        // All directives should be file-level since they appear before any code
        assert!(manager.is_suppressed("SC2145", 10)); // log_info
        assert!(manager.is_suppressed("SEC010", 11)); // mkdir
        assert!(manager.is_suppressed("IDEM003", 12)); // ln
    }
}

#[cfg(test)]
mod integration_tests {
    use crate::linter::lint_shell;

    /// Integration test: Verify inline suppression works end-to-end with linter
    #[test]
    fn test_integration_inline_suppression_sc2086() {
        // Script with unquoted variable (normally triggers SC2086)
        let script_without_suppression = "echo $var";
        let result = lint_shell(script_without_suppression);

        // Should detect SC2086 without suppression
        assert!(
            result.diagnostics.iter().any(|d| d.code == "SC2086"),
            "SC2086 should be detected without suppression"
        );

        // Script with inline suppression
        let script_with_suppression = "echo $var  # bashrs disable-line=SC2086";
        let result = lint_shell(script_with_suppression);

        // Should NOT detect SC2086 with suppression
        assert!(
            !result.diagnostics.iter().any(|d| d.code == "SC2086"),
            "SC2086 should be suppressed with inline comment"
        );
    }

    /// Integration test: Verify next-line suppression works end-to-end
    #[test]
    fn test_integration_next_line_suppression_sc2006() {
        // Script with backticks in non-assignment context (triggers SC2006)
        // Note: F080 fix means assignments don't trigger SC2006, so use echo
        let script_without_suppression = "echo `date`";
        let result = lint_shell(script_without_suppression);

        // Should detect SC2006 without suppression
        assert!(
            result.diagnostics.iter().any(|d| d.code == "SC2006"),
            "SC2006 should be detected without suppression"
        );

        // Script with next-line suppression
        let script_with_suppression = "# bashrs disable-next-line=SC2006\necho `date`";
        let result = lint_shell(script_with_suppression);

        // Should NOT detect SC2006 with suppression
        assert!(
            !result.diagnostics.iter().any(|d| d.code == "SC2006"),
            "SC2006 should be suppressed with next-line comment"
        );
    }

    /// Integration test: Verify file-level suppression works end-to-end
    #[test]
    fn test_integration_file_level_suppression_det002() {
        // Script with timestamp (normally triggers DET002)
        let script_without_suppression = "timestamp=$(date +%s)";
        let result = lint_shell(script_without_suppression);

        // Should detect DET002 without suppression
        assert!(
            result.diagnostics.iter().any(|d| d.code == "DET002"),
            "DET002 should be detected without suppression"
        );

        // Script with file-level suppression
        let script_with_suppression = "# bashrs disable-file=DET002\ntimestamp=$(date +%s)";
        let result = lint_shell(script_with_suppression);

        // Should NOT detect DET002 with suppression
        assert!(
            !result.diagnostics.iter().any(|d| d.code == "DET002"),
            "DET002 should be suppressed with file-level comment"
        );
    }

    /// Integration test: Verify multiple rule suppression works
    #[test]
    fn test_integration_multiple_rule_suppression() {
        // Script with multiple issues
        let script = r#"
# bashrs disable-next-line=SC2006,SC2086
result=`echo $var`
"#;
        let result = lint_shell(script);

        // Both SC2006 and SC2086 should be suppressed
        assert!(
            !result.diagnostics.iter().any(|d| d.code == "SC2006"),
            "SC2006 should be suppressed"
        );
        assert!(
            !result.diagnostics.iter().any(|d| d.code == "SC2086"),
            "SC2086 should be suppressed"
        );
    }

    /// Integration test: Verify suppression is selective (only affects specified rules)
    #[test]
    fn test_integration_selective_suppression() {
        // Script with two issues: useless cat (SC2002) and backticks (SC2006)
        let script = r#"
# bashrs disable-next-line=SC2006
cat file.txt | grep pattern; result=`date`
"#;
        let result = lint_shell(script);

        // SC2006 should be suppressed (backticks)
        assert!(
            !result.diagnostics.iter().any(|d| d.code == "SC2006"),
            "SC2006 should be suppressed"
        );

        // SC2002 should NOT be suppressed (useless cat)
        assert!(
            result.diagnostics.iter().any(|d| d.code == "SC2002"),
            "SC2002 should NOT be suppressed (not in suppression list)"
        );
    }

    // =====================================================
    // Shellcheck syntax compatibility integration tests (Issue #58)
    // =====================================================

    /// Integration test: Verify shellcheck disable works end-to-end with linter
    #[test]
    fn test_integration_shellcheck_disable_sc2086() {
        // Script with unquoted variable (normally triggers SC2086)
        let script_without_suppression = "echo $var";
        let result = lint_shell(script_without_suppression);

        // Should detect SC2086 without suppression
        assert!(
            result.diagnostics.iter().any(|d| d.code == "SC2086"),
            "SC2086 should be detected without suppression"
        );

        // Script with shellcheck disable
        let script_with_shellcheck = "# shellcheck disable=SC2086\necho $var";
        let result = lint_shell(script_with_shellcheck);

        // Should NOT detect SC2086 with shellcheck suppression
        assert!(
            !result.diagnostics.iter().any(|d| d.code == "SC2086"),
            "SC2086 should be suppressed with shellcheck disable comment"
        );
    }

    /// Integration test: Verify shellcheck disable works for DET002 (Issue #58)
    #[test]
    fn test_integration_shellcheck_disable_det002() {
        // Script with timestamp (normally triggers DET002)
        let script_without_suppression = "timestamp=$(date +%s)";
        let result = lint_shell(script_without_suppression);

        // Should detect DET002 without suppression
        assert!(
            result.diagnostics.iter().any(|d| d.code == "DET002"),
            "DET002 should be detected without suppression"
        );

        // Script with shellcheck disable (should work for DET002 too)
        let script_with_shellcheck = "# shellcheck disable=DET002\ntimestamp=$(date +%s)";
        let result = lint_shell(script_with_shellcheck);

        // Should NOT detect DET002 with shellcheck suppression
        assert!(
            !result.diagnostics.iter().any(|d| d.code == "DET002"),
            "DET002 should be suppressed with shellcheck disable comment (Issue #58)"
        );
    }

    /// Integration test: Verify shellcheck disable works for multiple rules
    #[test]
    fn test_integration_shellcheck_disable_multiple_rules() {
        // Script with multiple issues
        let script = "# shellcheck disable=SC2006,SC2086\nresult=`echo $var`";
        let result = lint_shell(script);

        // Both SC2006 and SC2086 should be suppressed
        assert!(
            !result.diagnostics.iter().any(|d| d.code == "SC2006"),
            "SC2006 should be suppressed by shellcheck disable"
        );
        assert!(
            !result.diagnostics.iter().any(|d| d.code == "SC2086"),
            "SC2086 should be suppressed by shellcheck disable"
        );
    }

    // =====================================================
    // Shorthand syntax integration tests (Issue #70)
    // =====================================================

    /// Integration test: Verify shorthand # bashrs disable=RULE works (Issue #70)
    #[test]
    fn test_integration_shorthand_disable_sec010() {
        // Script with path expansion (normally triggers SEC010)
        let script_without_suppression = r#"mkdir -p "${BASELINE_DIR}""#;
        let result = lint_shell(script_without_suppression);

        // Should detect SEC010 without suppression
        assert!(
            result.diagnostics.iter().any(|d| d.code == "SEC010"),
            "SEC010 should be detected without suppression"
        );

        // Script with shorthand suppression (Issue #70 requested syntax)
        let script_with_suppression = "# bashrs disable=SEC010\nmkdir -p \"${BASELINE_DIR}\"";
        let result = lint_shell(script_with_suppression);

        // Should NOT detect SEC010 with shorthand suppression
        assert!(
            !result.diagnostics.iter().any(|d| d.code == "SEC010"),
            "SEC010 should be suppressed with shorthand # bashrs disable=RULE (Issue #70)"
        );
    }

    /// Integration test: Shorthand syntax with explanation comment (Issue #70 use case)
    #[test]
    fn test_integration_shorthand_with_explanation() {
        // Simplified test case from Issue #70 - explanation text in suppression comment
        let script = "# bashrs disable=SEC010 (validated via case statement above)\nmkdir -p \"${BASELINE_DIR}\"";
        let result = lint_shell(script);

        // SEC010 should be suppressed on line 2
        assert!(
            !result.diagnostics.iter().any(|d| d.code == "SEC010"),
            "SEC010 should be suppressed with explanation comment (Issue #70 use case)"
        );
    }
}
