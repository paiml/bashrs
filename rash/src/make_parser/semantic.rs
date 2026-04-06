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

/// A semantic check entry: (predicate, message, severity, rule_name, suggestion).
type SemanticCheckTable<'a> = &'a [(fn(&str) -> bool, &'a str, IssueSeverity, &'a str, &'a str)];

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
    // Check for $(shell date ...) with word boundary after "date"
    // Must match "date" as a complete command, not as part of another word
    if let Some(pos) = value.find("$(shell date") {
        let after_date = pos + "$(shell date".len();
        if after_date >= value.len() {
            return true; // "$(shell date" at end
        }
        // Check next character is whitespace, ), or other delimiter
        let next_char = value.as_bytes()[after_date] as char;
        next_char.is_whitespace() || next_char == ')' || next_char == '+' || next_char == '-'
    } else {
        false
    }
}

/// Detect non-deterministic $(wildcard) patterns in a variable value
///
/// This function identifies wildcard function calls that produce
/// non-deterministic filesystem ordering. It EXCLUDES already-purified
/// patterns like `$(sort $(wildcard ...))`.
///
/// # Arguments
///
/// * `value` - Variable value to analyze
///
/// # Returns
///
/// * `true` if $(wildcard ...) pattern is detected AND not already purified
/// * `false` otherwise (no wildcard OR already wrapped with sort)
///
/// # Examples
///
/// ```
/// use bashrs::make_parser::semantic::detect_wildcard;
///
/// // Non-purified wildcards are detected
/// assert!(detect_wildcard("$(wildcard *.c)"));
/// assert!(detect_wildcard("SOURCES := $(wildcard src/*.c)"));
///
/// // Already purified wildcards are NOT detected
/// assert!(!detect_wildcard("$(sort $(wildcard *.c))"));
/// assert!(!detect_wildcard("SOURCES := $(sort $(wildcard src/*.c))"));
///
/// // Safe patterns are NOT detected
/// assert!(!detect_wildcard("SOURCES := main.c util.c"));
/// ```
pub fn detect_wildcard(value: &str) -> bool {
    // Check if contains wildcard
    if !value.contains("$(wildcard") {
        return false;
    }

    // Check if already purified with $(sort $(wildcard ...))
    // Look for the pattern "$(sort $(wildcard"
    if value.contains("$(sort $(wildcard") {
        return false;
    }

    // Found unpurified wildcard
    true
}

/// Common non-file targets that should be marked as .PHONY
const COMMON_PHONY_TARGETS: &[&str] =
    &["test", "clean", "install", "deploy", "build", "all", "help"];

/// Detect non-deterministic $RANDOM or $(shell echo $$RANDOM) patterns
///
/// This function identifies random value generation that makes builds
/// non-reproducible.
///
/// # Arguments
///
/// * `value` - Variable value to analyze
///
/// # Returns
///
/// * `true` if $RANDOM or $(shell echo $$RANDOM) pattern is detected
/// * `false` otherwise
///
/// # Examples
///
/// ```
/// use bashrs::make_parser::semantic::detect_random;
///
/// assert!(detect_random("$(shell echo $$RANDOM)"));
/// assert!(detect_random("ID := $RANDOM"));
/// assert!(!detect_random("VERSION := 1.0.0"));
/// ```
pub fn detect_random(value: &str) -> bool {
    value.contains("$RANDOM") || value.contains("$$RANDOM")
}

/// Detect non-deterministic $(shell find) patterns in a variable value
///
/// This function identifies shell find commands that produce non-deterministic
/// filesystem ordering, making builds non-reproducible. It EXCLUDES already-purified
/// patterns like `$(sort $(shell find ...))`.
///
/// # Arguments
///
/// * `value` - Variable value to analyze
///
/// # Returns
///
/// * `true` if $(shell find...) pattern is detected AND not already purified
/// * `false` otherwise (no shell find OR already wrapped with sort)
///
/// # Examples
///
/// ```
/// use bashrs::make_parser::semantic::detect_shell_find;
///
/// // Non-purified shell find is detected
/// assert!(detect_shell_find("$(shell find . -name '*.c')"));
/// assert!(detect_shell_find("FILES := $(shell find src -type f)"));
///
/// // Already purified shell find is NOT detected
/// assert!(!detect_shell_find("$(sort $(shell find . -name '*.c'))"));
/// assert!(!detect_shell_find("FILES := $(sort $(shell find src -type f))"));
///
/// // Safe patterns are NOT detected
/// assert!(!detect_shell_find("FILES := main.c util.c"));
/// ```
pub fn detect_shell_find(value: &str) -> bool {
    // Check if contains shell find
    if !value.contains("$(shell find") {
        return false;
    }

    // Check if already purified with $(sort $(shell find ...))
    // Look for the pattern "$(sort $(shell find"
    if value.contains("$(sort $(shell find") {
        return false;
    }

    // Found unpurified shell find
    true
}

/// Check if a target name is a common non-file target that should be .PHONY
///
/// This function identifies targets that don't represent actual files
/// and should be declared as .PHONY for idempotent builds.
///
/// # Arguments
///
/// * `target_name` - The name of the Makefile target
///
/// # Returns
///
/// * `true` if target is a common non-file target (test, clean, install, etc.)
/// * `false` otherwise
///
/// # Examples
///
/// ```
/// use bashrs::make_parser::semantic::is_common_phony_target;
///
/// assert!(is_common_phony_target("test"));
/// assert!(is_common_phony_target("clean"));
/// assert!(!is_common_phony_target("main.o"));
/// ```
pub fn is_common_phony_target(target_name: &str) -> bool {
    COMMON_PHONY_TARGETS.contains(&target_name)
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
/// Check a variable for non-deterministic patterns
fn check_variable_determinism(
    name: &str,
    value: &str,
    span: Span,
    issues: &mut Vec<SemanticIssue>,
) {
    let checks: SemanticCheckTable<'_> = &[
        (
            detect_shell_date,
            "uses non-deterministic $(shell date) - replace with explicit version",
            IssueSeverity::Critical,
            "NO_TIMESTAMPS",
            "1.0.0",
        ),
        (
            detect_wildcard,
            "uses non-deterministic $(wildcard) - replace with explicit sorted file list",
            IssueSeverity::High,
            "NO_WILDCARD",
            "file1.c file2.c file3.c",
        ),
        (
            detect_shell_find,
            "uses non-deterministic $(shell find) - replace with explicit sorted file list",
            IssueSeverity::High,
            "NO_UNORDERED_FIND",
            "src/a.c src/b.c src/main.c",
        ),
        (
            detect_random,
            "uses non-deterministic $RANDOM - replace with fixed value or seed",
            IssueSeverity::Critical,
            "NO_RANDOM",
            "42",
        ),
    ];
    for (detect_fn, msg, severity, rule, suggestion) in checks {
        if detect_fn(value) {
            issues.push(SemanticIssue {
                message: format!("Variable '{}' {}", name, msg),
                severity: severity.clone(),
                span,
                rule: rule.to_string(),
                suggestion: Some(format!("{} := {}", name, suggestion)),
            });
        }
    }
}

pub fn analyze_makefile(ast: &MakeAst) -> Vec<SemanticIssue> {
    let mut issues = Vec::new();

    for item in &ast.items {
        match item {
            MakeItem::Variable {
                name, value, span, ..
            } => {
                check_variable_determinism(name, value, *span, &mut issues);
            }
            MakeItem::Target {
                name, phony, span, ..
            } => {
                if !phony && is_common_phony_target(name) {
                    issues.push(SemanticIssue {
                        message: format!(
                            "Target '{}' should be marked as .PHONY (common non-file target)",
                            name
                        ),
                        severity: IssueSeverity::High,
                        span: *span,
                        rule: "AUTO_PHONY".to_string(),
                        suggestion: Some(format!(".PHONY: {}", name)),
                    });
                }
            }
            _ => {}
        }
    }

    issues
}

// FIXME(PMAT-238): #[cfg(test)]
// FIXME(PMAT-238): #[path = "semantic_tests_func_shell.rs"]
// FIXME(PMAT-238): mod tests_extracted;
