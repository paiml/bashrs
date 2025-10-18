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
    let transformations = plan_transformations(ast, &issues);

    // 3. Apply transformations
    let purified_ast = apply_transformations(ast, &transformations);

    // 4. Count results
    let issues_fixed = transformations.iter()
        .filter(|t| is_safe_transformation(t))
        .count();

    let manual_fixes_needed = transformations.iter()
        .filter(|t| !is_safe_transformation(t))
        .count();

    // 5. Generate report
    let report = generate_report(&transformations);

    PurificationResult {
        ast: purified_ast,
        transformations_applied: transformations.len(),
        issues_fixed,
        manual_fixes_needed,
        report,
    }
}

/// Plan which transformations to apply for detected issues
fn plan_transformations(ast: &MakeAst, issues: &[SemanticIssue]) -> Vec<Transformation> {
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
                    safe: false,  // Manual fix required
                });
            }
            "NO_RANDOM" => {
                transformations.push(Transformation::AddComment {
                    variable_name: var_name,
                    rule: issue.rule.clone(),
                    suggestion: issue.suggestion.clone().unwrap_or_default(),
                    safe: false,  // Manual fix required
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
        match transformation {
            Transformation::WrapWithSort { variable_name, pattern, .. } => {
                wrap_variable_with_sort(&mut purified, variable_name, pattern);
            }
            Transformation::AddComment { .. } => {
                // TODO: Implement comment addition
                // For now, we don't modify AST for manual fixes
            }
        }
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
        if bytes[i] == b'(' && i > 0 && bytes[i - 1] == b'$' {
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
        match bytes[i] {
            b'$' if i + 1 < bytes.len() && bytes[i + 1] == b'(' => {
                depth += 1;
                i += 2; // Skip past $(
                continue;
            }
            b')' => {
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
    if let Some(start) = message.find("'") {
        if let Some(end) = message[start + 1..].find("'") {
            return message[start + 1..start + 1 + end].to_string();
        }
    }
    String::new()
}

/// Check if transformation can be applied safely
fn is_safe_transformation(transformation: &Transformation) -> bool {
    match transformation {
        Transformation::WrapWithSort { safe, .. } => *safe,
        Transformation::AddComment { safe, .. } => *safe,
    }
}

/// Generate human-readable report of transformations
fn generate_report(transformations: &[Transformation]) -> Vec<String> {
    transformations.iter().map(|t| match t {
        Transformation::WrapWithSort { variable_name, pattern, .. } => {
            format!("✅ Wrapped {} in variable '{}' with $(sort ...)", pattern, variable_name)
        }
        Transformation::AddComment { variable_name, rule, .. } => {
            format!("⚠️  Manual fix needed for variable '{}': {}", variable_name, rule)
        }
    }).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find_matching_paren_simple() {
        let s = "$(wildcard *.c)";
        assert_eq!(find_matching_paren(s, 0), Some(14));
    }

    #[test]
    fn test_find_matching_paren_nested() {
        let s = "$(filter %.o, $(wildcard *.c))";
        // Start at "$(wildcard", find its closing paren
        // Position 15 is the start of "$(wildcard"
        // The wildcard closing paren is at position 28
        assert_eq!(find_matching_paren(s, 15), Some(28));
    }

    #[test]
    fn test_wrap_pattern_with_sort_simple() {
        let value = "$(wildcard *.c)";
        let result = wrap_pattern_with_sort(value, "$(wildcard");
        assert_eq!(result, "$(sort $(wildcard *.c))");
    }

    #[test]
    fn test_wrap_pattern_with_sort_nested() {
        let value = "$(filter %.o, $(wildcard *.c))";
        let result = wrap_pattern_with_sort(value, "$(wildcard");
        assert_eq!(result, "$(filter %.o, $(sort $(wildcard *.c)))");
    }

    #[test]
    fn test_extract_variable_name() {
        let message = "Variable 'FILES' uses non-deterministic $(wildcard)";
        assert_eq!(extract_variable_name(message), "FILES");
    }
}
