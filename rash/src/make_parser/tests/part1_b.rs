#![allow(clippy::unwrap_used)]
#![allow(unused_imports)]

use super::super::*;
use crate::make_parser::ast::{MakeCondition, Span, VarFlavor};

/// RED PHASE: Test for RULE-SYNTAX-001 - Basic rule syntax
///
/// This test validates the fundamental building block of Makefiles:
/// a target with prerequisites and a recipe.
///
/// Input Makefile:
/// ```makefile
/// target: prerequisites
///     recipe
/// ```
///
/// Expected AST:
/// - One MakeItem::Target
/// - name: "target"
/// - prerequisites: ["prerequisites"]
/// - recipe: ["recipe"]
/// - phony: false (will be detected/added in purification)
    #[test]
    fn test_VAR_BASIC_001_mut_operator_edge_cases() {
        // Test cases that stress operator detection logic
        let test_cases = vec![
            // Variable assignments (should all succeed)
            ("A=1", true),
            ("B:=2", true),
            ("C?=3", true),
            ("D+=4", true),
            ("E!=echo 5", true),
            // Edge case: variable name with colon (unusual but valid)
            ("X:Y:=value", true),
        ];

        for (input, should_succeed) in test_cases {
            let result = parse_makefile(input);

            if should_succeed {
                assert!(result.is_ok(), "Should parse: {}", input);

                let ast = result.unwrap();
                if let MakeItem::Variable { name, .. } = &ast.items[0] {
                    // Just verify it's a variable
                    assert!(!name.is_empty());
                }
            }
        }
    }
}

/// RED PHASE: Test for PHONY-001 - .PHONY declarations
///
/// This test validates .PHONY target declarations in Makefiles.
///
/// Input Makefile:
/// ```makefile
/// .PHONY: clean
/// clean:
///     rm -f *.o
/// ```
///
/// Expected AST:
/// - Two MakeItems: one Target for ".PHONY", one Target for "clean"
/// - The ".PHONY" target should have "clean" as a prerequisite
#[test]
fn test_PHONY_001_basic_phony_declaration() {
    // ARRANGE: .PHONY declaration with clean target
    let makefile = ".PHONY: clean\nclean:\n\trm -f *.o";

    // ACT: Parse makefile
    let result = parse_makefile(makefile);

    // ASSERT: Successfully parsed
    assert!(
        result.is_ok(),
        "Should parse .PHONY declaration, got error: {:?}",
        result.err()
    );

    let ast = result.unwrap();

    // ASSERT: Two items in AST (.PHONY and clean)
    assert_eq!(
        ast.items.len(),
        2,
        "Should have two items (.PHONY and clean), got {}",
        ast.items.len()
    );

    // ASSERT: First item is .PHONY target
    match &ast.items[0] {
        MakeItem::Target {
            name,
            prerequisites,
            recipe,
            ..
        } => {
            assert_eq!(name, ".PHONY", "First target should be .PHONY");
            assert_eq!(prerequisites.len(), 1, "Should have one prerequisite");
            assert_eq!(prerequisites[0], "clean", "Prerequisite should be 'clean'");
            assert_eq!(recipe.len(), 0, ".PHONY should have no recipe");
        }
        other => panic!("Expected Target item for .PHONY, got {:?}", other),
    }

    // ASSERT: Second item is clean target
    match &ast.items[1] {
        MakeItem::Target {
            name,
            recipe,
            phony,
            ..
        } => {
            assert_eq!(name, "clean", "Second target should be clean");
            assert_eq!(recipe.len(), 1, "Should have one recipe line");
            assert_eq!(recipe[0], "rm -f *.o");
            // Parser now detects .PHONY declarations and marks targets
            assert!(
                *phony,
                "clean should be marked as phony since .PHONY: clean was declared"
            );
        }
        other => panic!("Expected Target item for clean, got {:?}", other),
    }
}

/// RED PHASE: Test for PHONY-001 - Multiple phony targets
#[test]
fn test_PHONY_001_multiple_phony_targets() {
    // ARRANGE: .PHONY with multiple targets
    let makefile = ".PHONY: all clean test\nall:\n\techo done";

    // ACT: Parse makefile
    let result = parse_makefile(makefile);

    // ASSERT: Successfully parsed
    assert!(result.is_ok());

    let ast = result.unwrap();
    assert_eq!(ast.items.len(), 2, "Should have .PHONY and all targets");

    // ASSERT: .PHONY has 3 prerequisites
    match &ast.items[0] {
        MakeItem::Target {
            name,
            prerequisites,
            ..
        } => {
            assert_eq!(name, ".PHONY");
            assert_eq!(prerequisites.len(), 3, "Should have 3 phony targets");
            assert_eq!(prerequisites[0], "all");
            assert_eq!(prerequisites[1], "clean");
            assert_eq!(prerequisites[2], "test");
        }
        _ => panic!("Expected Target item for .PHONY"),
    }
}

/// RED PHASE: Test for PHONY-001 - .PHONY before and after targets
#[test]
fn test_PHONY_001_phony_declaration_position() {
    // ARRANGE: .PHONY can appear after target definition
    let makefile = "test:\n\tcargo test\n\n.PHONY: test";

    // ACT: Parse makefile
    let result = parse_makefile(makefile);

    // ASSERT: Successfully parsed
    assert!(result.is_ok());

    let ast = result.unwrap();
    assert_eq!(ast.items.len(), 2, "Should have test and .PHONY targets");

    // First item is test target
    match &ast.items[0] {
        MakeItem::Target { name, .. } => {
            assert_eq!(name, "test");
        }
        _ => panic!("Expected Target item for test"),
    }

    // Second item is .PHONY
    match &ast.items[1] {
        MakeItem::Target {
            name,
            prerequisites,
            ..
        } => {
            assert_eq!(name, ".PHONY");
            assert_eq!(prerequisites[0], "test");
        }
        _ => panic!("Expected Target item for .PHONY"),
    }
}

// PROPERTY TESTING PHASE: Tests for PHONY-001
//
// These property tests verify .PHONY declarations work across various inputs.
#[cfg(test)]
mod phony_property_tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        /// Property: .PHONY declarations always parse successfully
        #[test]
        fn test_PHONY_001_prop_phony_always_parses(
            target_name in "[a-z][a-z0-9_-]{0,15}"
        ) {
            // ARRANGE: Generate .PHONY declaration
            let makefile = format!(".PHONY: {}", target_name);

            // ACT: Parse makefile
            let result = parse_makefile(&makefile);

            // ASSERT: Parsing succeeds
            prop_assert!(result.is_ok(), "Failed to parse: {}", makefile);

            let ast = result.unwrap();
            prop_assert_eq!(ast.items.len(), 1);

            // ASSERT: .PHONY target properties
            if let MakeItem::Target { name, prerequisites, .. } = &ast.items[0] {
                prop_assert_eq!(name, ".PHONY");
                prop_assert_eq!(prerequisites.len(), 1);
                prop_assert_eq!(&prerequisites[0], &target_name);
            } else {
                return Err(TestCaseError::fail("Expected Target item"));
            }
        }

        /// Property: Multiple phony targets parse correctly
        #[test]
        fn test_PHONY_001_prop_multiple_phony_targets(
            targets in prop::collection::vec("[a-z]{1,10}", 1..5)
        ) {
            let targets_str = targets.join(" ");
            let makefile = format!(".PHONY: {}", targets_str);

            let result = parse_makefile(&makefile);
            prop_assert!(result.is_ok());

            let ast = result.unwrap();
            if let MakeItem::Target { name, prerequisites, .. } = &ast.items[0] {
                prop_assert_eq!(name, ".PHONY");
                prop_assert_eq!(prerequisites.len(), targets.len());
                for (i, target) in targets.iter().enumerate() {
                    prop_assert_eq!(&prerequisites[i], target);
                }
            } else {
                return Err(TestCaseError::fail("Expected Target item"));
            }
        }

        /// Property: .PHONY parsing is deterministic
        #[test]
        fn test_PHONY_001_prop_parsing_is_deterministic(
            target in "[a-z]{1,10}"
        ) {
            let makefile = format!(".PHONY: {}", target);

            // Parse twice
            let result1 = parse_makefile(&makefile);
            let result2 = parse_makefile(&makefile);

            // Both should succeed
            prop_assert!(result1.is_ok());
            prop_assert!(result2.is_ok());

            // Results should be identical
            let ast1 = result1.unwrap();
            let ast2 = result2.unwrap();
            prop_assert_eq!(ast1.items.len(), ast2.items.len());
            prop_assert_eq!(ast1.items, ast2.items);
        }
    }
}

/// RED PHASE: Test for VAR-BASIC-002 - Variable reference parsing
///
/// This test validates variable references in Makefiles using $(VAR) syntax.
///
/// Input Makefile:
/// ```makefile
/// CC = gcc
/// build:
///     $(CC) -o output main.c
/// ```
///
/// Expected: Parser should recognize $(CC) as a variable reference
#[test]
fn test_VAR_BASIC_002_variable_reference_in_recipe() {
    // ARRANGE: Variable definition and recipe using variable reference
    let makefile = "CC = gcc\nbuild:\n\t$(CC) -o output main.c";

    // ACT: Parse makefile
    let result = parse_makefile(makefile);

    // ASSERT: Successfully parsed
    assert!(
        result.is_ok(),
        "Should parse variable reference in recipe, got error: {:?}",
        result.err()
    );

    let ast = result.unwrap();

    // ASSERT: Two items (variable + target)
    assert_eq!(ast.items.len(), 2, "Should have variable and target");

    // ASSERT: First item is variable
    match &ast.items[0] {
        MakeItem::Variable { name, value, .. } => {
            assert_eq!(name, "CC");
            assert_eq!(value, "gcc");
        }
        _ => panic!("Expected Variable item"),
    }

    // ASSERT: Second item is target with recipe containing variable reference
    match &ast.items[1] {
        MakeItem::Target { name, recipe, .. } => {
            assert_eq!(name, "build");
            assert_eq!(recipe.len(), 1);
            // Recipe should preserve $(CC) reference
            assert!(
                recipe[0].contains("$(CC)"),
                "Recipe should contain $(CC) reference, got: {}",
                recipe[0]
            );
        }
        _ => panic!("Expected Target item"),
    }
}

/// RED PHASE: Test for VAR-BASIC-002 - Variable reference in variable value
#[test]
fn test_VAR_BASIC_002_variable_reference_in_value() {
    // ARRANGE: Variable using another variable's value
    let makefile = "CC = gcc\nCOMPILER = $(CC)";

    // ACT: Parse makefile
    let result = parse_makefile(makefile);

    // ASSERT: Successfully parsed
    assert!(result.is_ok());

    let ast = result.unwrap();
    assert_eq!(ast.items.len(), 2);

    // ASSERT: Second variable contains reference to first
    match &ast.items[1] {
        MakeItem::Variable { name, value, .. } => {
            assert_eq!(name, "COMPILER");
            // Value should preserve $(CC) reference
            assert!(
                value.contains("$(CC)"),
                "Variable value should contain $(CC) reference, got: {}",
                value
            );
        }
        _ => panic!("Expected Variable item"),
    }
}

/// RED PHASE: Test for VAR-BASIC-002 - Multiple variable references
#[test]
fn test_VAR_BASIC_002_multiple_variable_references() {
    // ARRANGE: Recipe with multiple variable references
    let makefile = "CC = gcc\nCFLAGS = -O2\nbuild:\n\t$(CC) $(CFLAGS) -o output main.c";

    // ACT: Parse makefile
    let result = parse_makefile(makefile);

    // ASSERT: Successfully parsed
    assert!(result.is_ok());

    let ast = result.unwrap();
    assert_eq!(ast.items.len(), 3);

    // ASSERT: Target recipe contains both references
    match &ast.items[2] {
        MakeItem::Target { recipe, .. } => {
            assert!(recipe[0].contains("$(CC)"));
            assert!(recipe[0].contains("$(CFLAGS)"));
        }
        _ => panic!("Expected Target item"),
    }
}

/// RED PHASE: Test for VAR-BASIC-002 - Curly brace syntax ${VAR}
#[test]
fn test_VAR_BASIC_002_curly_brace_syntax() {
    // ARRANGE: Variable reference using ${VAR} syntax
    let makefile = "PREFIX = /usr/local\ninstall:\n\tcp binary ${PREFIX}/bin/";

    // ACT: Parse makefile
    let result = parse_makefile(makefile);

    // ASSERT: Successfully parsed
    assert!(result.is_ok());

    let ast = result.unwrap();

    // ASSERT: Recipe preserves ${PREFIX} reference
    match &ast.items[1] {
        MakeItem::Target { recipe, .. } => {
            assert!(
                recipe[0].contains("${PREFIX}"),
                "Recipe should contain ${{PREFIX}} reference, got: {}",
                recipe[0]
            );
        }
        _ => panic!("Expected Target item"),
    }
}

/// RED PHASE: Test for VAR-BASIC-002 - Variable reference in prerequisites
#[test]
fn test_VAR_BASIC_002_variable_reference_in_prerequisites() {
    // ARRANGE: Target with variable reference in prerequisites
    let makefile = "DEPS = dep1.o dep2.o\ntarget: $(DEPS)\n\tld -o target $(DEPS)";

    // ACT: Parse makefile
    let result = parse_makefile(makefile);

    // ASSERT: Successfully parsed
    assert!(result.is_ok());

    let ast = result.unwrap();

    // ASSERT: Prerequisites contain variable reference
    match &ast.items[1] {
        MakeItem::Target {
            prerequisites,
            recipe,
            ..
        } => {
            assert!(
                prerequisites[0].contains("$(DEPS)"),
                "Prerequisites should contain $(DEPS) reference"
            );
            assert!(recipe[0].contains("$(DEPS)"));
        }
        _ => panic!("Expected Target item"),
    }
}

// PROPERTY TESTING PHASE: Tests for VAR-BASIC-002
//
// These property tests verify variable references work across various inputs.
#[cfg(test)]
mod var_reference_property_tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        /// Property: Variable references are always preserved in recipes
        ///
        /// This test verifies that $(VAR) syntax is preserved as-is in recipe lines.
