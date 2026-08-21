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
#[path = "suppression_tests_parse_file.rs"]
mod tests_extracted;

/// A comment that looks like a bashrs suppression directive but is not one.
///
/// Issue #240: `parse_suppression` matches four exact prefixes. Anything else —
/// `# bashrs-disable: SC2086`, `# bashrs: disable=SC2086`, `# bashrs disable
/// SC2086` with a space instead of `=` — is silently ignored. The author sees a
/// directive in the source, the finding stays, and nothing anywhere says the two
/// facts are related. That is worse than rejecting it: a suppression that does
/// nothing looks exactly like a rule that fires anyway.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnrecognisedDirective {
    /// 1-indexed line the comment appears on.
    pub line: usize,
    /// The comment text, trimmed.
    pub text: String,
}

/// The four forms that actually work.
pub const SUPPORTED_DIRECTIVES: &[&str] = &[
    "# bashrs disable-file=RULE[,RULE...]",
    "# bashrs disable-next-line=RULE",
    "# bashrs disable-line=RULE",
    "# bashrs disable=RULE",
];

/// Does this comment mention bashrs and disabling, without being a valid directive?
///
/// Deliberately narrow: it must contain BOTH "bashrs" and "disable"/"ignore" in a
/// comment. A line merely mentioning bashrs (`# built by bashrs`) is not an
/// attempted directive and must not be reported.
fn looks_like_attempted_directive(line: &str) -> bool {
    let Some(hash) = line.find('#') else {
        return false;
    };
    let comment = line[hash..].to_lowercase();
    if !comment.contains("bashrs") {
        return false;
    }
    if !(comment.contains("disable") || comment.contains("ignore")) {
        return false;
    }
    // A recognised directive parses; anything else in this shape does not.
    parse_suppression(line, 1).is_none()
}

/// Find comments that were probably meant as suppressions but are inert.
///
/// Returned rather than reported here, so the caller decides the severity and
/// this module keeps no dependency on `Diagnostic`.
pub fn unrecognised_directives(source: &str) -> Vec<UnrecognisedDirective> {
    source
        .lines()
        .enumerate()
        .filter(|(_, l)| looks_like_attempted_directive(l))
        .map(|(i, l)| UnrecognisedDirective {
            line: i + 1,
            text: l.trim().to_string(),
        })
        .collect()
}

#[cfg(test)]
mod unrecognised_tests {
    use super::*;

    /// The reported case (#240) and its near neighbours: each LOOKS like a
    /// directive, and each does nothing.
    #[test]
    fn near_miss_directives_are_reported() {
        for line in [
            "# bashrs-disable: SC2086",
            "# bashrs: disable=SC2086",
            "# bashrs disable SC2086",
            "# bashrs ignore SC2086",
            "echo hi  # bashrs-disable-line: SC2086",
        ] {
            let found = unrecognised_directives(line);
            assert_eq!(found.len(), 1, "not reported: {line}");
            assert_eq!(found[0].line, 1);
        }
    }

    /// Guard the guard: every SUPPORTED form must stay silent, or the warning
    /// fires on correct code and becomes noise people learn to ignore.
    #[test]
    fn supported_directives_are_not_reported() {
        for line in [
            "# bashrs disable-file=SC2086,DET002",
            "# bashrs disable-next-line=SC2086",
            "echo $var  # bashrs disable-line=SC2086",
            "# bashrs disable=SEC010",
        ] {
            assert!(
                unrecognised_directives(line).is_empty(),
                "a SUPPORTED directive was reported as unrecognised: {line}"
            );
        }
    }

    /// A comment that merely mentions bashrs is not an attempted directive.
    #[test]
    fn ordinary_comments_are_not_reported() {
        for line in [
            "# built by bashrs",
            "# see bashrs docs for details",
            "# disable the cache here",
            "echo 'bashrs disable=SC2086'",
            "#!/usr/bin/env bash",
        ] {
            assert!(
                unrecognised_directives(line).is_empty(),
                "ordinary comment reported: {line}"
            );
        }
    }

    /// shellcheck's own directive is understood elsewhere and must not be
    /// reported here.
    #[test]
    fn shellcheck_directives_are_not_reported() {
        assert!(unrecognised_directives("# shellcheck disable=SC2086").is_empty());
    }
}

/// Append a `BASHRS001` warning for every comment that was meant as a
/// suppression but matches none of the supported forms.
///
/// Lives here, and is called from BOTH lint entry points, because the two have
/// already diverged once: `lint_shell` (mod_lint.rs) and `lint_shell_with_path`
/// (mod_lint_2.rs) each apply suppressions separately, so wiring only one meant
/// the CLI — which uses `lint_shell` — reported nothing while the unit tests
/// passed. Same shape as the quoting allowlist that did nothing on one of its
/// two paths (paiml/bashrs#247).
pub fn report_unrecognised_directives(
    source: &str,
    manager: &SuppressionManager,
    result: &mut crate::linter::LintResult,
) {
    for d in unrecognised_directives(source) {
        if manager.is_suppressed("BASHRS001", d.line) {
            continue;
        }
        result.add(crate::linter::Diagnostic::new(
            "BASHRS001",
            crate::linter::Severity::Warning,
            format!(
                "`{}` is not a recognised bashrs directive and does nothing. Supported: {}",
                d.text,
                SUPPORTED_DIRECTIVES.join(", ")
            ),
            crate::linter::Span::new(d.line, 1, d.line, d.text.len() + 1),
        ));
    }
}
