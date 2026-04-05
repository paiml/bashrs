#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]
#![allow(unused_imports)]

use super::super::*;
use crate::make_parser::ast::{MakeCondition, Span, VarFlavor};

#[test]
fn test_PURIFY_017_edge_case_variable_name_match() {
    use crate::make_parser::parse_makefile;
    use crate::make_parser::purify::purify_makefile;

    // ARRANGE: Test that && logic in wrap_variable_with_sort works correctly
    // If replaced with ||, would wrap wrong variables
    let makefile = r#"
FILES := $(wildcard *.c)
OTHER := foo.c bar.c
"#;
    let ast = parse_makefile(makefile).unwrap();

    // ACT: Purify
    let result = purify_makefile(&ast);

    // ASSERT: Only FILES should be transformed, not OTHER
    assert_eq!(
        result.transformations_applied, 1,
        "Should apply 1 transformation"
    );

    // Check FILES is wrapped
    let files_var = &result.ast.items[0];
    if let MakeItem::Variable { value, .. } = files_var {
        assert!(
            value.contains("$(sort $(wildcard"),
            "FILES should be wrapped with sort"
        );
    }

    // Check OTHER is unchanged
    let other_var = &result.ast.items[1];
    if let MakeItem::Variable { value, .. } = other_var {
        assert_eq!(value, "foo.c bar.c", "OTHER should be unchanged");
    }
}

#[test]
fn test_PURIFY_018_edge_case_parenthesis_matching_boundary() {
    use crate::make_parser::parse_makefile;
    use crate::make_parser::purify::purify_makefile;

    // ARRANGE: Test boundary condition for parenthesis matching
    // Tests the < vs <= condition in find_matching_paren
    let makefile = "X := $(wildcard a)";
    let ast = parse_makefile(makefile).unwrap();

    // ACT: Purify
    let result = purify_makefile(&ast);

    // ASSERT: Should successfully wrap even single-char pattern
    assert!(
        result.transformations_applied >= 1,
        "Should apply at least 1 transformation"
    );
    let var = &result.ast.items[0];
    if let MakeItem::Variable { value, .. } = var {
        assert_eq!(value, "$(sort $(wildcard a))");
    }
}

#[test]
fn test_PURIFY_019_edge_case_nested_dollar_paren() {
    use crate::make_parser::parse_makefile;
    use crate::make_parser::purify::purify_makefile;

    // ARRANGE: Test $( detection logic in find_matching_paren
    // If && replaced with ||, would fail to detect nested patterns
    let makefile = "FILES := $(filter %.c, $(wildcard *.c))";
    let ast = parse_makefile(makefile).unwrap();

    // ACT: Purify
    let result = purify_makefile(&ast);

    // ASSERT: Should wrap inner wildcard
    assert!(
        result.transformations_applied >= 1,
        "Should apply at least 1 transformation"
    );
    let var = &result.ast.items[0];
    if let MakeItem::Variable { value, .. } = var {
        assert!(value.contains("$(sort $(wildcard"), "Should wrap wildcard");
        assert!(value.contains("$(filter"), "Should preserve filter");
    }
}

#[test]
fn test_PURIFY_020_edge_case_empty_pattern() {
    use crate::make_parser::parse_makefile;
    use crate::make_parser::purify::purify_makefile;

    // ARRANGE: Empty wildcard pattern
    let makefile = "EMPTY := $(wildcard )";
    let ast = parse_makefile(makefile).unwrap();

    // ACT: Purify
    let result = purify_makefile(&ast);

    // ASSERT: Should still wrap even empty pattern
    assert!(
        result.transformations_applied >= 1,
        "Should apply at least 1 transformation"
    );
}

#[test]
fn test_PURIFY_021_edge_case_multiple_wildcards_same_variable() {
    use crate::make_parser::parse_makefile;
    use crate::make_parser::purify::purify_makefile;

    // ARRANGE: Multiple wildcard calls in same variable
    let makefile = "FILES := $(wildcard *.c) $(wildcard *.h)";
    let ast = parse_makefile(makefile).unwrap();

    // ACT: Purify
    let result = purify_makefile(&ast);

    // ASSERT: Currently wraps first occurrence
    // Future enhancement: wrap all occurrences
    assert!(result.transformations_applied >= 1);
}

#[test]
fn test_PURIFY_022_edge_case_already_purified_no_double_wrap() {
    use crate::make_parser::parse_makefile;
    use crate::make_parser::purify::purify_makefile;

    // ARRANGE: Already purified with $(sort $(wildcard))
    let makefile = "FILES := $(sort $(wildcard *.c))";
    let ast = parse_makefile(makefile).unwrap();

    // ACT: Purify twice
    let result1 = purify_makefile(&ast);
    let result2 = purify_makefile(&result1.ast);

    // ASSERT: Second purification should do nothing
    assert_eq!(
        result2.transformations_applied, 0,
        "Already purified should not be re-wrapped"
    );
    assert_eq!(result2.issues_fixed, 0);
}

#[test]
fn test_PURIFY_023_edge_case_shell_find_with_complex_args() {
    use crate::make_parser::parse_makefile;
    use crate::make_parser::purify::purify_makefile;

    // ARRANGE: Shell find with complex arguments
    let makefile = "FILES := $(shell find src -type f -name '*.c' -not -path '*/test/*')";
    let ast = parse_makefile(makefile).unwrap();

    // ACT: Purify
    let result = purify_makefile(&ast);

    // ASSERT: Should wrap complex shell find
    assert!(
        result.transformations_applied >= 1,
        "Should apply at least 1 transformation"
    );
    let var = &result.ast.items[0];
    if let MakeItem::Variable { value, .. } = var {
        assert!(
            value.contains("$(sort $(shell find"),
            "Should wrap shell find with sort"
        );
    }
}

// ============================================================================
// CODE GENERATOR TESTS - Sprint 68
// ============================================================================

/// RED PHASE: Test for GENERATE-001 - Simple variable generation
///
/// Tests that a simple variable assignment can be generated from AST.
///
/// Input AST:
/// ```
/// Variable { name: "CC", value: "gcc", flavor: Simple }
/// ```
///
/// Expected Output:
/// ```makefile
/// CC := gcc
/// ```
#[test]
fn test_GENERATE_001_simple_variable() {
    // ARRANGE: Create AST with simple variable
    let ast = MakeAst {
        items: vec![MakeItem::Variable {
            name: "CC".to_string(),
            value: "gcc".to_string(),
            flavor: VarFlavor::Simple,
            span: Span::dummy(),
        }],
        metadata: MakeMetadata::new(),
    };

    // ACT: Generate Makefile text
    let output = generate_purified_makefile(&ast);

    // ASSERT: Should generate variable assignment
    assert_eq!(output.trim(), "CC := gcc");
}

/// RED PHASE: Test for GENERATE-002 - All variable flavors
///
/// Tests that all 5 variable flavors can be generated correctly.
#[test]
fn test_GENERATE_002_all_variable_flavors() {
    // ARRANGE: Create AST with all variable flavors
    let ast = MakeAst {
        items: vec![
            MakeItem::Variable {
                name: "SIMPLE".to_string(),
                value: "value1".to_string(),
                flavor: VarFlavor::Simple,
                span: Span::dummy(),
            },
            MakeItem::Variable {
                name: "RECURSIVE".to_string(),
                value: "value2".to_string(),
                flavor: VarFlavor::Recursive,
                span: Span::dummy(),
            },
            MakeItem::Variable {
                name: "CONDITIONAL".to_string(),
                value: "value3".to_string(),
                flavor: VarFlavor::Conditional,
                span: Span::dummy(),
            },
            MakeItem::Variable {
                name: "APPEND".to_string(),
                value: "value4".to_string(),
                flavor: VarFlavor::Append,
                span: Span::dummy(),
            },
            MakeItem::Variable {
                name: "SHELL".to_string(),
                value: "command".to_string(),
                flavor: VarFlavor::Shell,
                span: Span::dummy(),
            },
        ],
        metadata: MakeMetadata::new(),
    };

    // ACT: Generate Makefile text
    let output = generate_purified_makefile(&ast);

    // ASSERT: Should generate all variable types with correct operators
    assert!(output.contains("SIMPLE := value1"));
    assert!(output.contains("RECURSIVE = value2"));
    assert!(output.contains("CONDITIONAL ?= value3"));
    assert!(output.contains("APPEND += value4"));
    assert!(output.contains("SHELL != command"));
}

/// RED PHASE: Test for GENERATE-003 - Target with recipe
///
/// Tests that a target with prerequisites and recipe can be generated.
///
/// Expected Output:
/// ```makefile
/// build: main.c
///     gcc -o build main.c
/// ```
#[test]
fn test_GENERATE_003_target_with_recipe() {
    // ARRANGE: Create AST with target
    let ast = MakeAst {
        items: vec![MakeItem::Target {
            name: "build".to_string(),
            prerequisites: vec!["main.c".to_string()],
            recipe: vec!["gcc -o build main.c".to_string()],
            phony: false,
            recipe_metadata: None,
            span: Span::dummy(),
        }],
        metadata: MakeMetadata::new(),
    };

    // ACT: Generate Makefile text
    let output = generate_purified_makefile(&ast);

    // ASSERT: Should generate target with tab-indented recipe
    let lines: Vec<&str> = output.trim().lines().collect();
    assert_eq!(lines.len(), 2);
    assert_eq!(lines[0], "build: main.c");
    assert_eq!(lines[1], "\tgcc -o build main.c");
}

/// RED PHASE: Test for GENERATE-004 - Comment preservation
///
/// Tests that comments are preserved in generated output.
#[test]
fn test_GENERATE_004_comment_preservation() {
    // ARRANGE: Create AST with comment
    let ast = MakeAst {
        items: vec![MakeItem::Comment {
            text: "This is a comment".to_string(),
            span: Span::dummy(),
        }],
        metadata: MakeMetadata::new(),
    };

    // ACT: Generate Makefile text
    let output = generate_purified_makefile(&ast);

    // ASSERT: Should generate comment with # prefix
    assert_eq!(output.trim(), "# This is a comment");
}

/// RED PHASE: Test for GENERATE-005 - PHONY target
///
/// Tests that .PHONY targets are generated correctly.
#[test]
fn test_GENERATE_005_phony_target() {
    // ARRANGE: Create AST with phony target
    let ast = MakeAst {
        items: vec![MakeItem::Target {
            name: "clean".to_string(),
            prerequisites: vec![],
            recipe: vec!["rm -f *.o".to_string()],
            phony: true,
            recipe_metadata: None,
            span: Span::dummy(),
        }],
        metadata: MakeMetadata::new(),
    };

    // ACT: Generate Makefile text
    let output = generate_purified_makefile(&ast);

    // ASSERT: Should generate .PHONY declaration before target
    assert!(output.contains(".PHONY: clean"));
    assert!(output.contains("clean:"));
    assert!(output.contains("\trm -f *.o"));
}

/// RED PHASE: Test for GENERATE-006 - Complex Makefile
///
/// Tests generation of a complex Makefile with multiple items.
#[test]
fn test_GENERATE_006_complex_makefile() {
    // ARRANGE: Create AST with multiple items
    let ast = MakeAst {
        items: vec![
            MakeItem::Comment {
                text: "Build configuration".to_string(),
                span: Span::dummy(),
            },
            MakeItem::Variable {
                name: "CC".to_string(),
                value: "gcc".to_string(),
                flavor: VarFlavor::Simple,
                span: Span::dummy(),
            },
            MakeItem::Variable {
                name: "CFLAGS".to_string(),
                value: "-O2 -Wall".to_string(),
                flavor: VarFlavor::Simple,
                span: Span::dummy(),
            },
            MakeItem::Target {
                name: "all".to_string(),
                prerequisites: vec!["build".to_string()],
                recipe: vec![],
                phony: true,
                recipe_metadata: None,
                span: Span::dummy(),
            },
            MakeItem::Target {
                name: "build".to_string(),
                prerequisites: vec!["main.c".to_string()],
                recipe: vec!["$(CC) $(CFLAGS) -o build main.c".to_string()],
                phony: false,
                recipe_metadata: None,
                span: Span::dummy(),
            },
        ],
        metadata: MakeMetadata::new(),
    };

    // ACT: Generate Makefile text
    let output = generate_purified_makefile(&ast);

    // ASSERT: Should contain all items in order
    assert!(output.contains("# Build configuration"));
    assert!(output.contains("CC := gcc"));
    assert!(output.contains("CFLAGS := -O2 -Wall"));
    assert!(output.contains(".PHONY: all"));
    assert!(output.contains("all: build"));
    assert!(output.contains("build: main.c"));
    assert!(output.contains("\t$(CC) $(CFLAGS) -o build main.c"));
}

// ============================================================================
// GENERATOR PROPERTY TESTS - Sprint 68 Phase 2
// ============================================================================

#[cfg(test)]
mod generator_property_tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        /// PROPERTY TEST: Round-trip variable generation
        ///
        /// Property: parse(generate(variable)) should preserve variable semantics
        ///
        /// This test generates 100+ random variable names and values, then verifies
        /// that generating Makefile text and parsing it back produces equivalent AST.
        #[test]
        fn prop_GENERATE_007_roundtrip_variables(
            var_name in "[A-Z_][A-Z0-9_]{0,15}",
            var_value in "[a-zA-Z0-9 ./_-]{1,30}",
        ) {
            // ARRANGE: Create variable AST
            let ast = MakeAst {
                items: vec![MakeItem::Variable {
                    name: var_name.clone(),
                    value: var_value.clone(),
                    flavor: VarFlavor::Simple,
                    span: Span::dummy(),
                }],
                metadata: MakeMetadata::new(),
            };

            // ACT: Generate and re-parse
            let generated = generate_purified_makefile(&ast);
            let reparsed = parse_makefile(&generated);

            // ASSERT: Should parse successfully
            prop_assert!(reparsed.is_ok(), "Failed to parse generated Makefile: {}", generated);

            let reparsed_ast = reparsed.unwrap();

            // ASSERT: Should have same number of items
            prop_assert_eq!(reparsed_ast.items.len(), 1);

            // ASSERT: Should preserve variable name and value
            if let MakeItem::Variable { name, value, flavor, .. } = &reparsed_ast.items[0] {
                prop_assert_eq!(name, &var_name);
                prop_assert_eq!(value.trim(), var_value.trim());
                prop_assert_eq!(flavor, &VarFlavor::Simple);
            } else {
                prop_assert!(false, "Expected Variable item, got {:?}", reparsed_ast.items[0]);
            }
        }

        /// PROPERTY TEST: Round-trip target generation
        ///
        /// Property: parse(generate(target)) should preserve target structure
