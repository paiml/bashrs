// Purification Engine for Makefiles
//
// This module provides functionality to automatically fix non-deterministic
// patterns detected by semantic analysis.
//
// Transformations:
// - $(wildcard *.c) → $(sort $(wildcard *.c))
// - $(shell find ...) → $(sort $(shell find ...))
// - Nested patterns in filter, foreach, call
// - Comments for manual fixes (shell date, $RANDOM)

use crate::make_parser::ast::{MakeAst, MakeItem};
use crate::make_parser::semantic::{analyze_makefile, SemanticIssue};

mod error_handling;
mod parallel_safety;
mod performance;
mod portability;
mod report;
mod reproducible_builds;

#[cfg(test)]
mod tests;

#[cfg(test)]
#[path = "purify_tests.rs"]
mod purify_tests;

/// Result of purification process
#[derive(Debug, Clone, PartialEq)]
pub struct PurificationResult {
    /// Purified AST
    pub ast: MakeAst,
    /// Number of transformations applied
    pub transformations_applied: usize,
    /// Number of issues successfully fixed
    pub issues_fixed: usize,
    /// Number of issues requiring manual intervention
    pub manual_fixes_needed: usize,
    /// Report of transformations
    pub report: Vec<String>,
}

/// Type of transformation to apply
#[derive(Debug, Clone, PartialEq)]
pub enum Transformation {
    /// Wrap pattern with $(sort ...)
    WrapWithSort {
        variable_name: String,
        pattern: String,
        safe: bool,
    },
    /// Add comment suggesting manual fix
    AddComment {
        variable_name: String,
        rule: String,
        suggestion: String,
        safe: bool,
    },
    // Sprint 83 - Parallel Safety Transformations
    /// Recommend adding .NOTPARALLEL directive
    RecommendNotParallel { reason: String, safe: bool },
    /// Detect race condition in shared file write
    DetectRaceCondition {
        target_names: Vec<String>,
        conflicting_file: String,
        safe: bool,
    },
    /// Recommend adding order-only prerequisite
    RecommendOrderOnlyPrereq {
        target_name: String,
        prereq_name: String,
        reason: String,
        safe: bool,
    },
    /// Detect missing dependency
    DetectMissingDependency {
        target_name: String,
        missing_file: String,
        provider_target: String,
        safe: bool,
    },
    /// Detect multiple targets with same output
    DetectOutputConflict {
        target_names: Vec<String>,
        output_file: String,
        safe: bool,
    },
    /// Recommend handling recursive make calls
    RecommendRecursiveMakeHandling {
        target_name: String,
        subdirs: Vec<String>,
        safe: bool,
    },
    /// Detect shared directory creation race
    DetectDirectoryRace {
        target_names: Vec<String>,
        directory: String,
        safe: bool,
    },
    // Sprint 83 - Reproducible Builds Transformations (Day 4)
    /// Detect non-deterministic timestamp ($(shell date))
    DetectTimestamp {
        variable_name: String,
        pattern: String,
        safe: bool,
    },
    /// Detect $RANDOM usage
    DetectRandom { variable_name: String, safe: bool },
    /// Detect process ID $$ usage
    DetectProcessId { variable_name: String, safe: bool },
    /// Suggest SOURCE_DATE_EPOCH for reproducibility
    SuggestSourceDateEpoch {
        variable_name: String,
        original_pattern: String,
        safe: bool,
    },
    /// Detect non-deterministic command (hostname, git, mktemp, etc.)
    DetectNonDeterministicCommand {
        variable_name: String,
        command: String,
        reason: String,
        safe: bool,
    },
    // Sprint 83 - Performance Optimization Transformations (Day 5)
    /// Suggest combining multiple shell invocations
    SuggestCombineShellInvocations {
        target_name: String,
        recipe_count: usize,
        safe: bool,
    },
    /// Suggest using := instead of = for simple variables
    SuggestSimpleExpansion {
        variable_name: String,
        reason: String,
        safe: bool,
    },
    /// Recommend adding .SUFFIXES: to disable builtin rules
    RecommendSuffixes { reason: String, safe: bool },
    /// Detect multiple sequential recipe lines that could be combined
    DetectSequentialRecipes {
        target_name: String,
        recipe_count: usize,
        safe: bool,
    },
    /// Suggest pattern rule instead of explicit rules
    SuggestPatternRule {
        pattern: String,
        target_count: usize,
        safe: bool,
    },

    // Sprint 83 - Error Handling Transformations (Day 6)
    /// Detect missing error handling in recipes
    DetectMissingErrorHandling {
        target_name: String,
        command: String,
        safe: bool,
    },
    /// Detect silent failures with @ prefix
    DetectSilentFailure {
        target_name: String,
        command: String,
        safe: bool,
    },
    /// Recommend .DELETE_ON_ERROR
    RecommendDeleteOnError { reason: String, safe: bool },
    /// Recommend .ONESHELL for multiline recipes
    RecommendOneshell {
        target_name: String,
        reason: String,
        safe: bool,
    },
    /// Detect missing set -e in shell scripts
    DetectMissingSetE {
        target_name: String,
        command: String,
        safe: bool,
    },
    /// Detect missing error handling in loops
    DetectLoopWithoutErrorHandling {
        target_name: String,
        loop_command: String,
        safe: bool,
    },

    // Sprint 83 - Portability Transformations (Day 7)
    /// Detect bashisms (non-POSIX shell constructs)
    DetectBashism {
        target_name: String,
        construct: String,
        posix_alternative: String,
        safe: bool,
    },
    /// Detect platform-specific commands
    DetectPlatformSpecific {
        target_name: String,
        command: String,
        reason: String,
        safe: bool,
    },
    /// Detect shell-specific features (source, declare, etc.)
    DetectShellSpecific {
        target_name: String,
        feature: String,
        posix_alternative: String,
        safe: bool,
    },
    /// Detect non-portable command flags
    DetectNonPortableFlags {
        target_name: String,
        command: String,
        flag: String,
        reason: String,
        safe: bool,
    },
    /// Detect non-portable echo usage
    DetectNonPortableEcho {
        target_name: String,
        command: String,
        safe: bool,
    },
}

/// Purify a Makefile AST by fixing non-deterministic patterns
///
/// # Example
/// ```ignore
/// let ast = parse_makefile("FILES := $(wildcard *.c)").unwrap();
/// let result = purify_makefile(&ast);
/// // result.ast contains: FILES := $(sort $(wildcard *.c))
/// ```
pub fn purify_makefile(ast: &MakeAst) -> PurificationResult {
    // 1. Run semantic analysis to find issues
    let issues = analyze_makefile(ast);

    // 2. Plan transformations for each issue
    let mut transformations = plan_transformations(ast, &issues);

    // 3. Add parallel safety analysis (Sprint 83 - Days 2-3)
    transformations.extend(parallel_safety::analyze_parallel_safety(ast));

    // 4. Add reproducible builds analysis (Sprint 83 - Day 4)
    transformations.extend(reproducible_builds::analyze_reproducible_builds(ast));

    // 5. Add performance optimization analysis (Sprint 83 - Day 5)
    transformations.extend(performance::analyze_performance_optimization(ast));

    // 6. Add error handling analysis (Sprint 83 - Day 6)
    transformations.extend(error_handling::analyze_error_handling(ast));

    // 7. Add portability analysis (Sprint 83 - Day 7)
    transformations.extend(portability::analyze_portability(ast));

    // 8. Apply transformations
    let purified_ast = apply_transformations(ast, &transformations);

    // 8. Count results
    let issues_fixed = transformations
        .iter()
        .filter(|t| report::is_safe_transformation(t))
        .count();

    let manual_fixes_needed = transformations
        .iter()
        .filter(|t| !report::is_safe_transformation(t))
        .count();

    // 9. Generate report
    let report = report::generate_report(&transformations);

    PurificationResult {
        ast: purified_ast,
        transformations_applied: transformations.len(),
        issues_fixed,
        manual_fixes_needed,
        report,
    }
}

/// Plan which transformations to apply for detected issues
fn plan_transformations(_ast: &MakeAst, issues: &[SemanticIssue]) -> Vec<Transformation> {
    let mut transformations = Vec::new();

    for issue in issues {
        // Extract variable name from issue message
        let var_name = extract_variable_name(&issue.message);

        match issue.rule.as_str() {
            "NO_WILDCARD" => {
                transformations.push(Transformation::WrapWithSort {
                    variable_name: var_name,
                    pattern: "$(wildcard".to_string(),
                    safe: true,
                });
            }
            "NO_UNORDERED_FIND" => {
                transformations.push(Transformation::WrapWithSort {
                    variable_name: var_name,
                    pattern: "$(shell find".to_string(),
                    safe: true,
                });
            }
            "NO_TIMESTAMPS" => {
                transformations.push(Transformation::AddComment {
                    variable_name: var_name,
                    rule: issue.rule.clone(),
                    suggestion: issue.suggestion.clone().unwrap_or_default(),
                    safe: false, // Manual fix required
                });
            }
            "NO_RANDOM" => {
                transformations.push(Transformation::AddComment {
                    variable_name: var_name,
                    rule: issue.rule.clone(),
                    suggestion: issue.suggestion.clone().unwrap_or_default(),
                    safe: false, // Manual fix required
                });
            }
            _ => {}
        }
    }

    transformations
}

/// Apply transformations to AST
fn apply_transformations(ast: &MakeAst, transformations: &[Transformation]) -> MakeAst {
    let mut purified = ast.clone();

    for transformation in transformations {
        if let Transformation::WrapWithSort {
            variable_name,
            pattern,
            ..
        } = transformation
        {
            wrap_variable_with_sort(&mut purified, variable_name, pattern);
        }
        // All other transformation variants are detection/recommendation only —
        // they generate reports but don't modify the AST.
    }

    purified
}

/// Wrap pattern in specific variable with $(sort ...)
fn wrap_variable_with_sort(ast: &mut MakeAst, variable_name: &str, pattern: &str) {
    for item in &mut ast.items {
        if let MakeItem::Variable { name, value, .. } = item {
            if name == variable_name && value.contains(pattern) {
                *value = wrap_pattern_with_sort(value, pattern);
            }
        }
    }
}

/// Wrap specific pattern with $(sort ...)
///
/// Examples:
/// - "$(wildcard *.c)" → "$(sort $(wildcard *.c))"
/// - "$(filter %.o, $(wildcard *.c))" → "$(filter %.o, $(sort $(wildcard *.c)))"
fn wrap_pattern_with_sort(value: &str, pattern: &str) -> String {
    // Find pattern start
    if let Some(start) = value.find(pattern) {
        // Find matching closing paren
        if let Some(end) = find_matching_paren(value, start) {
            // Extract the complete pattern
            let complete_pattern = &value[start..=end];

            // Wrap with $(sort ...)
            let wrapped = format!("$(sort {})", complete_pattern);

            // Replace in original string
            value.replace(complete_pattern, &wrapped)
        } else {
            // No matching paren found, return unchanged
            value.to_string()
        }
    } else {
        // Pattern not found, return unchanged
        value.to_string()
    }
}

/// Find matching closing parenthesis for pattern starting at position
///
/// Handles nested parentheses correctly.
///
/// # Arguments
/// * `s` - The string to search
/// * `start` - Starting position (should point to the pattern start, e.g., "$(" )
///
/// # Returns
/// Position of the matching closing parenthesis
fn find_matching_paren(s: &str, start: usize) -> Option<usize> {
    let bytes = s.as_bytes();

    // Find the opening '(' for the pattern at start
    let mut depth = 0;
    let mut found_opening = false;
    let mut i = start;

    // Skip to the opening paren
    while i < bytes.len() {
        if bytes.get(i) == Some(&b'(') && i > 0 && bytes.get(i - 1) == Some(&b'$') {
            depth = 1;
            found_opening = true;
            i += 1;
            break;
        }
        i += 1;
    }

    if !found_opening {
        return None;
    }

    // Now find the matching closing paren
    while i < bytes.len() {
        match bytes.get(i) {
            Some(&b'$') if bytes.get(i + 1) == Some(&b'(') => {
                depth += 1;
                i += 2; // Skip past $(
                continue;
            }
            Some(&b')') => {
                depth -= 1;
                if depth == 0 {
                    return Some(i);
                }
            }
            _ => {}
        }
        i += 1;
    }

    None
}

/// Extract variable name from semantic issue message
///
/// Message format: "Variable 'NAME' uses non-deterministic ..."
fn extract_variable_name(message: &str) -> String {
    if let Some(start) = message.find('\'') {
        if let Some(end) = message[start + 1..].find('\'') {
            return message[start + 1..start + 1 + end].to_string();
        }
    }
    String::new()
}
