//! Semantic analysis for Makefile AST
//!
//! Validates AST and performs semantic checks.
//!
//! ## Purification Rules
//!
//! - **NO_TIMESTAMPS**: Detect $(shell date) patterns that produce non-deterministic timestamps
//! - **NO_RANDOM**: Detect $RANDOM or random shell commands
//! - **NO_WILDCARD**: Detect $(wildcard) that produces non-deterministic file ordering

use super::ast::*;

/// Issue severity levels for semantic analysis
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IssueSeverity {
    /// Critical - breaks determinism or idempotency
    Critical,
    /// High - reduces build reproducibility
    High,
    /// Medium - potential issue
    Medium,
    /// Low - style or best practice
    Low,
}

/// Semantic issue found in Makefile
#[derive(Debug, Clone, PartialEq)]
pub struct SemanticIssue {
    /// Issue description
    pub message: String,
    /// Severity level
    pub severity: IssueSeverity,
    /// Location in source
    pub span: Span,
    /// Purification rule that detected this
    pub rule: String,
    /// Suggested fix (if available)
    pub suggestion: Option<String>,
}

/// Detect non-deterministic $(shell date) patterns in a variable value
///
/// This function identifies timestamp-generating shell commands that make
/// builds non-reproducible.
///
/// # Arguments
///
/// * `value` - Variable value to analyze
///
/// # Returns
///
/// * `true` if $(shell date...) pattern is detected
/// * `false` otherwise
///
/// # Examples
///
/// ```
/// use bashrs::make_parser::semantic::detect_shell_date;
///
/// assert!(detect_shell_date("$(shell date +%s)"));
/// assert!(detect_shell_date("RELEASE := $(shell date +%Y%m%d)"));
/// assert!(!detect_shell_date("VERSION := 1.0.0"));
/// ```
pub fn detect_shell_date(value: &str) -> bool {
    value.contains("$(shell date")
}

/// Detect non-deterministic $(wildcard) patterns in a variable value
///
/// This function identifies wildcard function calls that produce
/// non-deterministic filesystem ordering.
///
/// # Arguments
///
/// * `value` - Variable value to analyze
///
/// # Returns
///
/// * `true` if $(wildcard ...) pattern is detected
/// * `false` otherwise
///
/// # Examples
///
/// ```
/// use bashrs::make_parser::semantic::detect_wildcard;
///
/// assert!(detect_wildcard("$(wildcard *.c)"));
/// assert!(detect_wildcard("SOURCES := $(wildcard src/*.c)"));
/// assert!(!detect_wildcard("SOURCES := main.c util.c"));
/// ```
pub fn detect_wildcard(value: &str) -> bool {
    value.contains("$(wildcard")
}

/// Analyze a Makefile AST for semantic issues
///
/// Scans the entire AST for non-deterministic patterns, style issues,
/// and purification opportunities.
///
/// # Arguments
///
/// * `ast` - Parsed Makefile AST
///
/// # Returns
///
/// * `Vec<SemanticIssue>` - List of issues found (empty if none)
///
/// # Examples
///
/// ```
/// use bashrs::make_parser::{parse_makefile, semantic::analyze_makefile};
///
/// let makefile = "RELEASE := $(shell date +%s)";
/// let ast = parse_makefile(makefile).unwrap();
/// let issues = analyze_makefile(&ast);
/// assert_eq!(issues.len(), 1);
/// assert_eq!(issues[0].rule, "NO_TIMESTAMPS");
/// ```
pub fn analyze_makefile(ast: &MakeAst) -> Vec<SemanticIssue> {
    let mut issues = Vec::new();

    for item in &ast.items {
        if let MakeItem::Variable { name, value, span, .. } = item {
            // Check for non-deterministic shell date
            if detect_shell_date(value) {
                issues.push(SemanticIssue {
                    message: format!(
                        "Variable '{}' uses non-deterministic $(shell date) - replace with explicit version",
                        name
                    ),
                    severity: IssueSeverity::Critical,
                    span: *span,
                    rule: "NO_TIMESTAMPS".to_string(),
                    suggestion: Some(format!("{} := 1.0.0", name)),
                });
            }

            // Check for non-deterministic wildcard
            if detect_wildcard(value) {
                issues.push(SemanticIssue {
                    message: format!(
                        "Variable '{}' uses non-deterministic $(wildcard) - replace with explicit sorted file list",
                        name
                    ),
                    severity: IssueSeverity::High,
                    span: *span,
                    rule: "NO_WILDCARD".to_string(),
                    suggestion: Some(format!("{} := file1.c file2.c file3.c", name)),
                });
            }
        }
    }

    issues
}

#[cfg(test)]
mod tests {
    use super::*;

    // Unit tests for shell date detection
    #[test]
    fn test_FUNC_SHELL_001_detect_shell_date_basic() {
        // Should detect $(shell date +%s)
        assert!(detect_shell_date("$(shell date +%s)"));
    }

    #[test]
    fn test_FUNC_SHELL_001_detect_shell_date_with_format() {
        // Should detect $(shell date +%Y%m%d-%H%M%S)
        assert!(detect_shell_date("$(shell date +%Y%m%d-%H%M%S)"));
    }

    #[test]
    fn test_FUNC_SHELL_001_no_false_positive() {
        // Should NOT detect when no shell date
        assert!(!detect_shell_date("VERSION := 1.0.0"));
    }

    #[test]
    fn test_FUNC_SHELL_001_detect_in_variable_context() {
        // Should detect in full variable assignment context
        let value = "RELEASE := $(shell date +%s)";
        assert!(detect_shell_date(value));
    }

    // Edge cases
    #[test]
    fn test_FUNC_SHELL_001_empty_string() {
        assert!(!detect_shell_date(""));
    }

    #[test]
    fn test_FUNC_SHELL_001_no_shell_command() {
        assert!(!detect_shell_date("$(CC) -o output"));
    }

    #[test]
    fn test_FUNC_SHELL_001_shell_but_not_date() {
        assert!(!detect_shell_date("$(shell pwd)"));
    }

    #[test]
    fn test_FUNC_SHELL_001_multiple_shell_commands() {
        // Should detect if ANY contain shell date
        assert!(detect_shell_date("A=$(shell pwd) B=$(shell date +%s)"));
    }

    #[test]
    fn test_FUNC_SHELL_001_date_without_shell() {
        // "date" alone is not a problem
        assert!(!detect_shell_date("# Update date: 2025-10-16"));
    }

    #[test]
    fn test_FUNC_SHELL_001_case_sensitive() {
        // Should be case-sensitive (shell commands are case-sensitive)
        assert!(!detect_shell_date("$(SHELL DATE)"));
    }

    // Mutation-killing tests
    #[test]
    fn test_FUNC_SHELL_001_mut_contains_must_check_substring() {
        // Ensures we use .contains() not .eq()
        assert!(detect_shell_date("prefix $(shell date +%s) suffix"));
    }

    #[test]
    fn test_FUNC_SHELL_001_mut_exact_pattern() {
        // Ensures we check for "$(shell date" not just "date"
        assert!(!detect_shell_date("datestamp"));
    }

    #[test]
    fn test_FUNC_SHELL_001_mut_non_empty_check() {
        // Ensures we don't crash on empty strings
        let result = detect_shell_date("");
        assert_eq!(result, false);
    }

    // Property-based tests
    #[cfg(test)]
    mod property_tests {
        use super::*;
        use proptest::prelude::*;

        proptest! {
            #[test]
            fn prop_FUNC_SHELL_001_any_string_no_panic(s in "\\PC*") {
                // Should never panic on any string
                let _ = detect_shell_date(&s);
            }

            #[test]
            fn prop_FUNC_SHELL_001_shell_date_always_detected(
                format in "[+%a-zA-Z0-9-]*"
            ) {
                let input = format!("$(shell date {})", format);
                prop_assert!(detect_shell_date(&input));
            }

            #[test]
            fn prop_FUNC_SHELL_001_no_shell_never_detected(
                s in "[^$]*"
            ) {
                // Strings without $ should never be detected
                prop_assert!(!detect_shell_date(&s));
            }

            #[test]
            fn prop_FUNC_SHELL_001_deterministic(s in "\\PC*") {
                // Same input always gives same output
                let result1 = detect_shell_date(&s);
                let result2 = detect_shell_date(&s);
                prop_assert_eq!(result1, result2);
            }

            #[test]
            fn prop_FUNC_SHELL_001_shell_without_date_not_detected(
                cmd in "[a-z]{3,10}"
            ) {
                // $(shell <non-date-command>) should not be detected
                if cmd != "date" {
                    let input = format!("$(shell {})", cmd);
                    prop_assert!(!detect_shell_date(&input));
                }
            }
        }
    }

    // Integration tests for analyze_makefile()
    #[test]
    fn test_FUNC_SHELL_001_analyze_detects_shell_date() {
        use crate::make_parser::parse_makefile;

        let makefile = "RELEASE := $(shell date +%s)";
        let ast = parse_makefile(makefile).unwrap();
        let issues = analyze_makefile(&ast);

        assert_eq!(issues.len(), 1);
        assert_eq!(issues[0].rule, "NO_TIMESTAMPS");
        assert_eq!(issues[0].severity, IssueSeverity::Critical);
        assert!(issues[0].message.contains("RELEASE"));
        assert!(issues[0].suggestion.is_some());
    }

    #[test]
    fn test_FUNC_SHELL_001_analyze_no_issues_clean_makefile() {
        use crate::make_parser::parse_makefile;

        let makefile = "VERSION := 1.0.0\nCC := gcc";
        let ast = parse_makefile(makefile).unwrap();
        let issues = analyze_makefile(&ast);

        assert_eq!(issues.len(), 0);
    }

    #[test]
    fn test_FUNC_SHELL_001_analyze_multiple_issues() {
        use crate::make_parser::parse_makefile;

        let makefile = r#"RELEASE := $(shell date +%s)
VERSION := 1.0.0
BUILD_TIME := $(shell date +%Y%m%d)"#;
        let ast = parse_makefile(makefile).unwrap();
        let issues = analyze_makefile(&ast);

        assert_eq!(issues.len(), 2);
        assert!(issues[0].message.contains("RELEASE"));
        assert!(issues[1].message.contains("BUILD_TIME"));
    }

    #[test]
    fn test_FUNC_SHELL_001_analyze_suggestion_format() {
        use crate::make_parser::parse_makefile;

        let makefile = "TIMESTAMP := $(shell date +%s)";
        let ast = parse_makefile(makefile).unwrap();
        let issues = analyze_makefile(&ast);

        assert_eq!(issues.len(), 1);
        let suggestion = issues[0].suggestion.as_ref().unwrap();
        assert!(suggestion.contains("TIMESTAMP"));
        assert!(suggestion.contains(":="));
    }

    #[test]
    fn test_FUNC_SHELL_001_analyze_ignores_targets() {
        use crate::make_parser::parse_makefile;

        // Should NOT detect shell date in recipe commands (only in variables)
        let makefile = "build:\n\techo $(shell date +%s)";
        let ast = parse_makefile(makefile).unwrap();
        let issues = analyze_makefile(&ast);

        // Currently only checks variables, not recipe commands
        assert_eq!(issues.len(), 0);
    }

    #[test]
    fn test_FUNC_SHELL_001_analyze_span_preserved() {
        use crate::make_parser::parse_makefile;

        let makefile = "RELEASE := $(shell date +%s)";
        let ast = parse_makefile(makefile).unwrap();
        let issues = analyze_makefile(&ast);

        assert_eq!(issues.len(), 1);
        // Span should be non-zero
        assert!(issues[0].span.end > issues[0].span.start);
        assert!(issues[0].span.line > 0);
    }

    // Unit tests for wildcard detection (FUNC-WILDCARD-001)
    #[test]
    fn test_FUNC_WILDCARD_001_detect_wildcard_basic() {
        // Should detect $(wildcard *.c)
        assert!(detect_wildcard("$(wildcard *.c)"));
    }

    #[test]
    fn test_FUNC_WILDCARD_001_detect_wildcard_with_path() {
        // Should detect $(wildcard src/*.c)
        assert!(detect_wildcard("$(wildcard src/*.c)"));
    }

    #[test]
    fn test_FUNC_WILDCARD_001_no_false_positive() {
        // Should NOT detect when no wildcard
        assert!(!detect_wildcard("SOURCES := main.c util.c"));
    }

    #[test]
    fn test_FUNC_WILDCARD_001_detect_in_variable_context() {
        // Should detect in full variable assignment context
        let value = "SOURCES := $(wildcard *.c)";
        assert!(detect_wildcard(value));
    }

    // Edge cases
    #[test]
    fn test_FUNC_WILDCARD_001_empty_string() {
        assert!(!detect_wildcard(""));
    }

    #[test]
    fn test_FUNC_WILDCARD_001_wildcard_text_not_function() {
        // Just the word "wildcard" is not a problem
        assert!(!detect_wildcard("# Use wildcard to find files"));
    }

    #[test]
    fn test_FUNC_WILDCARD_001_multiple_wildcards() {
        // Should detect if ANY contain wildcard
        assert!(detect_wildcard("A=*.c B=$(wildcard *.h)"));
    }

    #[test]
    fn test_FUNC_WILDCARD_001_case_sensitive() {
        // Should be case-sensitive (Make functions are case-sensitive)
        assert!(!detect_wildcard("$(WILDCARD *.c)"));
    }

    // Mutation-killing tests
    #[test]
    fn test_FUNC_WILDCARD_001_mut_contains_must_check_substring() {
        // Ensures we use .contains() not .eq()
        assert!(detect_wildcard("prefix $(wildcard *.c) suffix"));
    }

    #[test]
    fn test_FUNC_WILDCARD_001_mut_exact_pattern() {
        // Ensures we check for "$(wildcard" not just "wildcard"
        assert!(!detect_wildcard("wildcard_var"));
    }

    #[test]
    fn test_FUNC_WILDCARD_001_mut_non_empty_check() {
        // Ensures we don't crash on empty strings
        let result = detect_wildcard("");
        assert_eq!(result, false);
    }

    // Property-based tests for wildcard detection
    #[cfg(test)]
    mod wildcard_property_tests {
        use super::*;
        use proptest::prelude::*;

        proptest! {
            #[test]
            fn prop_FUNC_WILDCARD_001_any_string_no_panic(s in "\\PC*") {
                // Should never panic on any string
                let _ = detect_wildcard(&s);
            }

            #[test]
            fn prop_FUNC_WILDCARD_001_wildcard_always_detected(
                pattern in "[*.a-zA-Z0-9/_-]*"
            ) {
                let input = format!("$(wildcard {})", pattern);
                prop_assert!(detect_wildcard(&input));
            }

            #[test]
            fn prop_FUNC_WILDCARD_001_no_dollar_never_detected(
                s in "[^$]*"
            ) {
                // Strings without $ should never be detected
                prop_assert!(!detect_wildcard(&s));
            }

            #[test]
            fn prop_FUNC_WILDCARD_001_deterministic(s in "\\PC*") {
                // Same input always gives same output
                let result1 = detect_wildcard(&s);
                let result2 = detect_wildcard(&s);
                prop_assert_eq!(result1, result2);
            }

            #[test]
            fn prop_FUNC_WILDCARD_001_other_functions_not_detected(
                func in "(shell|subst|patsubst|filter|sort|dir|notdir|basename|suffix)"
            ) {
                // $(other_function ...) should not be detected as wildcard
                let input = format!("$({} test)", func);
                prop_assert!(!detect_wildcard(&input));
            }
        }
    }

    // Integration tests for analyze_makefile() with wildcard
    #[test]
    fn test_FUNC_WILDCARD_001_analyze_detects_wildcard() {
        use crate::make_parser::parse_makefile;

        let makefile = "SOURCES := $(wildcard *.c)";
        let ast = parse_makefile(makefile).unwrap();
        let issues = analyze_makefile(&ast);

        assert_eq!(issues.len(), 1);
        assert_eq!(issues[0].rule, "NO_WILDCARD");
        assert_eq!(issues[0].severity, IssueSeverity::High);
        assert!(issues[0].message.contains("SOURCES"));
        assert!(issues[0].suggestion.is_some());
    }

    #[test]
    fn test_FUNC_WILDCARD_001_analyze_wildcard_severity_high() {
        use crate::make_parser::parse_makefile;

        let makefile = "FILES := $(wildcard src/*.c)";
        let ast = parse_makefile(makefile).unwrap();
        let issues = analyze_makefile(&ast);

        assert_eq!(issues.len(), 1);
        // Wildcard is High severity (less critical than timestamps)
        assert_eq!(issues[0].severity, IssueSeverity::High);
    }

    #[test]
    fn test_FUNC_WILDCARD_001_analyze_multiple_issues() {
        use crate::make_parser::parse_makefile;

        let makefile = r#"SOURCES := $(wildcard *.c)
HEADERS := $(wildcard *.h)"#;
        let ast = parse_makefile(makefile).unwrap();
        let issues = analyze_makefile(&ast);

        assert_eq!(issues.len(), 2);
        assert!(issues[0].message.contains("SOURCES"));
        assert!(issues[1].message.contains("HEADERS"));
    }

    #[test]
    fn test_FUNC_WILDCARD_001_analyze_mixed_issues() {
        use crate::make_parser::parse_makefile;

        // Both shell date (Critical) and wildcard (High)
        let makefile = r#"RELEASE := $(shell date +%s)
SOURCES := $(wildcard *.c)"#;
        let ast = parse_makefile(makefile).unwrap();
        let issues = analyze_makefile(&ast);

        assert_eq!(issues.len(), 2);
        // First issue is shell date (Critical)
        assert_eq!(issues[0].rule, "NO_TIMESTAMPS");
        assert_eq!(issues[0].severity, IssueSeverity::Critical);
        // Second issue is wildcard (High)
        assert_eq!(issues[1].rule, "NO_WILDCARD");
        assert_eq!(issues[1].severity, IssueSeverity::High);
    }
}
