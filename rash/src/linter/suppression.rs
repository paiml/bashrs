//! Inline suppression support for bashrs warnings
//!
//! Allows users to suppress specific warnings using inline comments.
//!
//! # Syntax
//!
//! - File-level: `# bashrs disable-file=SC2086,DET002`
//! - Next-line: `# bashrs disable-next-line=SC2086`
//! - Inline: `command  # bashrs disable-line=SC2086`
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
//! timestamp=$(date +%s)  # bashrs disable-line=DET002
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
    pub fn from_source(source: &str) -> Self {
        let mut manager = Self::default();
        let lines: Vec<&str> = source.lines().collect();

        for (line_idx, line) in lines.iter().enumerate() {
            let line_num = line_idx + 1;

            // Check for suppression directives
            if let Some(suppression) = parse_suppression(line, line_num) {
                match suppression.suppression_type {
                    SuppressionType::File => {
                        // File-level suppression applies to all lines
                        manager.file_suppressions.extend(suppression.rules);
                    }
                    SuppressionType::NextLine => {
                        // Next-line suppression applies to line_num + 1
                        if line_idx + 1 < lines.len() {
                            manager
                                .line_suppressions
                                .entry(line_num + 1)
                                .or_default()
                                .extend(suppression.rules);
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

/// Parse a suppression directive from a line
fn parse_suppression(line: &str, line_num: usize) -> Option<Suppression> {
    let trimmed = line.trim();

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

    None
}

/// Parse comma-separated rule list
fn parse_rule_list(rules_str: &str) -> HashSet<String> {
    rules_str
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect()
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
        // Script with backticks (normally triggers SC2006)
        let script_without_suppression = "result=`date`";
        let result = lint_shell(script_without_suppression);

        // Should detect SC2006 without suppression
        assert!(
            result.diagnostics.iter().any(|d| d.code == "SC2006"),
            "SC2006 should be detected without suppression"
        );

        // Script with next-line suppression
        let script_with_suppression = "# bashrs disable-next-line=SC2006\nresult=`date`";
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
}
