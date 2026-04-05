#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]
#![allow(unused_imports)]

use super::super::*;
use crate::make_parser::ast::{MakeCondition, Span, VarFlavor};

#[test]
fn test_PURIFY_001_wrap_simple_wildcard_with_sort() {
    use crate::make_parser::parse_makefile;
    use crate::make_parser::purify::purify_makefile;

    // ARRANGE: Simple wildcard (non-deterministic)
    let makefile = "FILES := $(wildcard *.c)";
    let ast = parse_makefile(makefile).unwrap();

    // ACT: Purify
    let result = purify_makefile(&ast);

    // ASSERT: Wildcard wrapped with sort
    // Sprint 83 Day 5: Performance optimization may detect additional issues (e.g., missing .SUFFIXES)
    assert!(
        result.transformations_applied >= 1,
        "Should apply at least 1 transformation"
    );
    assert!(result.issues_fixed >= 1, "Should fix at least 1 issue");
    assert_eq!(result.manual_fixes_needed, 0, "No manual fixes needed");

    // Check purified output
    let purified_var = &result.ast.items[0];
    if let crate::make_parser::ast::MakeItem::Variable { value, .. } = purified_var {
        assert!(
            value.contains("$(sort $(wildcard"),
            "Should contain $(sort $(wildcard"
        );
        assert_eq!(value, "$(sort $(wildcard *.c))", "Should be fully wrapped");
    } else {
        panic!("Expected Variable");
    }
}

#[test]
fn test_PURIFY_002_wrap_nested_wildcard_in_filter() {
    use crate::make_parser::parse_makefile;
    use crate::make_parser::purify::purify_makefile;

    // ARRANGE: Wildcard nested in filter
    let makefile = "OBJS := $(filter %.o, $(wildcard *.c))";
    let ast = parse_makefile(makefile).unwrap();

    // ACT: Purify
    let result = purify_makefile(&ast);

    // ASSERT: Inner wildcard wrapped with sort
    // Sprint 83 Day 5: Performance optimization may detect additional issues
    assert!(
        result.transformations_applied >= 1,
        "Should apply at least 1 transformation"
    );
    assert!(result.issues_fixed >= 1, "Should fix at least 1 issue");

    let purified_var = &result.ast.items[0];
    if let crate::make_parser::ast::MakeItem::Variable { value, .. } = purified_var {
        assert!(
            value.contains("$(sort $(wildcard"),
            "Should wrap inner wildcard"
        );
        assert_eq!(
            value, "$(filter %.o, $(sort $(wildcard *.c)))",
            "Should preserve filter"
        );
    } else {
        panic!("Expected Variable");
    }
}

#[test]
fn test_PURIFY_003_wrap_shell_find_with_sort() {
    use crate::make_parser::parse_makefile;
    use crate::make_parser::purify::purify_makefile;

    // ARRANGE: Shell find (non-deterministic)
    let makefile = "FILES := $(shell find src -name '*.c')";
    let ast = parse_makefile(makefile).unwrap();

    // ACT: Purify
    let result = purify_makefile(&ast);

    // ASSERT: Shell find wrapped with sort
    assert!(
        result.transformations_applied >= 1,
        "Should apply at least 1 transformation"
    );
    assert!(result.issues_fixed >= 1, "Should fix at least 1 issue");

    let purified_var = &result.ast.items[0];
    if let crate::make_parser::ast::MakeItem::Variable { value, .. } = purified_var {
        assert!(
            value.contains("$(sort $(shell find"),
            "Should wrap shell find"
        );
    } else {
        panic!("Expected Variable");
    }
}

#[test]
fn test_PURIFY_004_nested_wildcard_in_foreach() {
    use crate::make_parser::parse_makefile;
    use crate::make_parser::purify::purify_makefile;

    // ARRANGE: Wildcard in foreach list
    let makefile = "OBJS := $(foreach file, $(wildcard *.c), $(file:.c=.o))";
    let ast = parse_makefile(makefile).unwrap();

    // ACT: Purify
    let result = purify_makefile(&ast);

    // ASSERT: Wildcard in foreach list wrapped with sort
    assert!(
        result.transformations_applied >= 1,
        "Should apply at least 1 transformation"
    );
    assert!(result.issues_fixed >= 1, "Should fix at least 1 issue");

    let purified_var = &result.ast.items[0];
    if let crate::make_parser::ast::MakeItem::Variable { value, .. } = purified_var {
        assert!(value.contains("$(sort $(wildcard"), "Should wrap wildcard");
        assert!(
            value.contains("$(foreach file, $(sort $(wildcard"),
            "Should preserve foreach"
        );
    } else {
        panic!("Expected Variable");
    }
}

#[test]
fn test_PURIFY_005_nested_wildcard_in_call() {
    use crate::make_parser::parse_makefile;
    use crate::make_parser::purify::purify_makefile;

    // ARRANGE: Wildcard in call arguments
    let makefile = r#"
process = Processing $(1)
FILES := $(call process, $(wildcard *.c))
"#;
    let ast = parse_makefile(makefile).unwrap();

    // ACT: Purify
    let result = purify_makefile(&ast);

    // ASSERT: Wildcard in call args wrapped with sort
    // We should have 2 items: function definition + variable
    assert!(
        result.transformations_applied >= 1,
        "Should apply at least 1 transformation"
    );

    // Find the FILES variable
    let files_var = result
        .ast
        .items
        .iter()
        .find(|item| {
            if let crate::make_parser::ast::MakeItem::Variable { name, .. } = item {
                name == "FILES"
            } else {
                false
            }
        })
        .expect("FILES variable should exist");

    if let crate::make_parser::ast::MakeItem::Variable { value, .. } = files_var {
        assert!(value.contains("$(sort $(wildcard"), "Should wrap wildcard");
        assert!(
            value.contains("$(call process, $(sort $(wildcard"),
            "Should preserve call"
        );
    } else {
        panic!("Expected Variable");
    }
}

#[test]
fn test_PURIFY_006_shell_date_manual_fix() {
    use crate::make_parser::parse_makefile;
    use crate::make_parser::purify::purify_makefile;

    // ARRANGE: Shell date (cannot auto-fix)
    let makefile = "RELEASE := release-$(shell date +%s)";
    let ast = parse_makefile(makefile).unwrap();

    // ACT: Purify
    let result = purify_makefile(&ast);

    // ASSERT: Manual fix needed
    // Sprint 83 enhancement: Now detects multiple issues (semantic + Sprint 83 reproducible builds)
    // - Semantic analysis: NO_TIMESTAMPS (1)
    // - Sprint 83: DetectTimestamp (1)
    // - Sprint 83: SuggestSourceDateEpoch (1)
    // Total: 3 manual fixes
    assert!(
        result.manual_fixes_needed >= 1,
        "Should need at least 1 manual fix"
    );
    assert!(
        result.transformations_applied >= 1,
        "Should plan transformation"
    );

    // Check report mentions manual fix or timestamp
    assert!(!result.report.is_empty(), "Should have report");
    assert!(
        result.report.iter().any(|r| r.contains("Manual fix")
            || r.contains("timestamp")
            || r.contains("SOURCE_DATE_EPOCH")),
        "Report should mention manual fix or timestamp issue"
    );
}

#[test]
fn test_PURIFY_007_random_manual_fix() {
    use crate::make_parser::parse_makefile;
    use crate::make_parser::purify::purify_makefile;

    // ARRANGE: $RANDOM (cannot auto-fix)
    let makefile = "SESSION := session-$RANDOM";
    let ast = parse_makefile(makefile).unwrap();

    // ACT: Purify
    let result = purify_makefile(&ast);

    // ASSERT: Manual fix needed
    // Sprint 83 enhancement: Now detects multiple issues (semantic + Sprint 83 reproducible builds)
    // - Semantic analysis: NO_RANDOM (1)
    // - Sprint 83: DetectRandom (1)
    // Total: 2 manual fixes
    assert!(
        result.manual_fixes_needed >= 1,
        "Should need at least 1 manual fix"
    );
    assert!(
        result.transformations_applied >= 1,
        "Should plan transformation"
    );
}

#[test]
fn test_PURIFY_008_safe_patterns_unchanged() {
    use crate::make_parser::parse_makefile;
    use crate::make_parser::purify::purify_makefile;

    // ARRANGE: Safe deterministic patterns
    let makefile = "FILES := foo.c bar.c baz.c";
    let ast = parse_makefile(makefile).unwrap();

    // ACT: Purify
    let result = purify_makefile(&ast);

    // ASSERT: No transformations needed
    assert_eq!(
        result.transformations_applied, 0,
        "Should apply 0 transformations"
    );
    assert_eq!(result.issues_fixed, 0, "Should fix 0 issues");
    assert_eq!(result.manual_fixes_needed, 0, "Should need 0 manual fixes");

    // AST should be unchanged
    assert_eq!(
        result.ast.items.len(),
        ast.items.len(),
        "AST should be unchanged"
    );
}

#[test]
fn test_PURIFY_009_report_generation() {
    use crate::make_parser::parse_makefile;
    use crate::make_parser::purify::purify_makefile;

    // ARRANGE: Mix of auto-fix and manual fix
    let makefile = r#"
FILES := $(wildcard *.c)
RELEASE := release-$(shell date +%s)
"#;
    let ast = parse_makefile(makefile).unwrap();

    // ACT: Purify
    let result = purify_makefile(&ast);

    // ASSERT: Report generated
    assert!(!result.report.is_empty(), "Should generate report");
    assert!(
        result.report.len() >= 2,
        "Should have at least 2 report entries"
    );

    // Check report contains expected information
    let report_text = result.report.join("\n");
    assert!(
        report_text.contains("Wrapped") || report_text.contains("Manual fix"),
        "Report should describe transformations"
    );
}

// ============================================================================
// Property-Based Tests for Purification
// ============================================================================

#[cfg(test)]
mod purify_property_tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        /// Property: Purifying wildcard patterns always wraps with $(sort)
        #[test]
        fn prop_PURIFY_010_wildcard_always_wraps_with_sort(
            pattern in "[a-zA-Z0-9*._/-]{1,20}",
        ) {
            let makefile = format!("FILES := $(wildcard {})", pattern);
            let ast = parse_makefile(&makefile).unwrap();
            let result = purify_makefile(&ast);

            // Should apply transformation
            prop_assert!(result.transformations_applied >= 1,
                        "Should apply at least 1 transformation");

            // Should wrap with sort
            let purified_var = &result.ast.items[0];
            if let MakeItem::Variable { value, .. } = purified_var {
                prop_assert!(value.contains("$(sort $(wildcard"),
                           "Should wrap wildcard with sort");
            } else {
                prop_assert!(false, "Expected Variable item");
            }
        }

        /// Property: Purifying shell find patterns always wraps with $(sort)
        #[test]
        fn prop_PURIFY_011_shell_find_always_wraps_with_sort(
            dir in "[a-zA-Z0-9/_-]{1,15}",
            ext in "[a-z]{1,5}",
        ) {
            let makefile = format!("FILES := $(shell find {} -name '*.{}')", dir, ext);
            let ast = parse_makefile(&makefile).unwrap();
            let result = purify_makefile(&ast);

            // Should apply transformation
            prop_assert!(result.transformations_applied >= 1,
                        "Should apply at least 1 transformation");

            // Should wrap with sort
            let purified_var = &result.ast.items[0];
            if let MakeItem::Variable { value, .. } = purified_var {
                prop_assert!(value.contains("$(sort $(shell find"),
                           "Should wrap shell find with sort");
            } else {
                prop_assert!(false, "Expected Variable item");
            }
        }

        /// Property: Purification is idempotent - purifying twice gives same result
        #[test]
        fn prop_PURIFY_012_idempotent(
            pattern in "[a-zA-Z0-9*._/-]{1,15}",
        ) {
            let makefile = format!("FILES := $(wildcard {})", pattern);
            let ast = parse_makefile(&makefile).unwrap();

            // Purify once
            let result1 = purify_makefile(&ast);

            // Purify again
            let result2 = purify_makefile(&result1.ast);

            // Second purification should do nothing (already purified)
            prop_assert_eq!(result2.transformations_applied, 0,
                           "Second purification should apply 0 transformations");
            prop_assert_eq!(result2.issues_fixed, 0,
                           "Second purification should fix 0 issues");
        }

        /// Property: Purification preserves variable count
        #[test]
        fn prop_PURIFY_013_preserves_variable_count(
            var_name in "[A-Z][A-Z0-9_]{0,10}",
            pattern in "[a-zA-Z0-9*._/-]{1,15}",
        ) {
            let makefile = format!("{} := $(wildcard {})", var_name, pattern);
            let ast = parse_makefile(&makefile).unwrap();
            let original_count = ast.items.len();

            let result = purify_makefile(&ast);

            prop_assert_eq!(result.ast.items.len(), original_count,
                           "Purification should preserve variable count");
        }

        /// Property: Safe patterns require zero transformations
        #[test]
        fn prop_PURIFY_014_safe_patterns_unchanged(
            var_name in "[A-Z][A-Z0-9_]{0,10}",
            value in "[a-zA-Z0-9. _-]{1,30}",
        ) {
            // Only test values that don't contain special characters
            prop_assume!(!value.contains('$'));
            prop_assume!(!value.contains('('));

            let makefile = format!("{} := {}", var_name, value);
            let ast = parse_makefile(&makefile).unwrap();

            let result = purify_makefile(&ast);

            prop_assert_eq!(result.transformations_applied, 0,
                           "Safe patterns should apply 0 transformations");
            prop_assert_eq!(result.issues_fixed, 0,
                           "Safe patterns should fix 0 issues");
        }

        /// Property: Nested patterns are correctly handled
        #[test]
        fn prop_PURIFY_015_nested_in_filter(
            pattern in "[a-zA-Z0-9*._-]{1,15}",
            filter_pattern in "%\\.[a-z]{1,3}",
        ) {
            let makefile = format!("OBJS := $(filter {}, $(wildcard {}))", filter_pattern, pattern);
            let ast = parse_makefile(&makefile).unwrap();

            let result = purify_makefile(&ast);

            // Should apply transformation
            prop_assert!(result.transformations_applied >= 1,
                        "Should apply at least 1 transformation");

            // Inner wildcard should be wrapped
            let purified_var = &result.ast.items[0];
            if let MakeItem::Variable { value, .. } = purified_var {
                prop_assert!(value.contains("$(sort $(wildcard"),
                           "Inner wildcard should be wrapped");
                prop_assert!(value.contains("$(filter"),
                           "Outer filter should be preserved");
            } else {
                prop_assert!(false, "Expected Variable item");
            }
        }

        /// Property: Multiple variables are all purified
        #[test]
        fn prop_PURIFY_016_multiple_variables(
            pattern1 in "[a-zA-Z0-9*._-]{1,10}",
            pattern2 in "[a-zA-Z0-9*._-]{1,10}",
        ) {
            let makefile = format!(
                "FILES1 := $(wildcard {})\nFILES2 := $(wildcard {})",
                pattern1, pattern2
            );
            let ast = parse_makefile(&makefile).unwrap();

            let result = purify_makefile(&ast);

            // Should apply at least 2 transformations (one per variable)
            prop_assert!(result.transformations_applied >= 2,
                        "Should apply at least 2 transformations");

            // Both variables should be purified
            prop_assert_eq!(result.ast.items.len(), 2,
                           "Should have 2 variables");
        }
    }
}

// ============================================================================
// Edge Case Tests for Purification (Mutation Killers)
// ============================================================================

