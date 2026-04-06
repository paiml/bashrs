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
fn test_RULE_SYNTAX_001_basic_rule_syntax() {
    // ARRANGE: Simple rule with target, prerequisites, and recipe
    let makefile = "target: prerequisites\n\trecipe";

    // ACT: Parse makefile
    let result = parse_makefile(makefile);

    // ASSERT: Successfully parsed
    assert!(
        result.is_ok(),
        "Should parse basic rule syntax, got error: {:?}",
        result.err()
    );

    let ast = result.unwrap();

    // ASSERT: One item in AST
    assert_eq!(
        ast.items.len(),
        1,
        "Should have exactly one item, got {}",
        ast.items.len()
    );

    // ASSERT: Item is a Target
    match &ast.items[0] {
        MakeItem::Target {
            name,
            prerequisites,
            recipe,
            phony,
            ..
        } => {
            assert_eq!(name, "target", "Target name should be 'target'");
            assert_eq!(prerequisites.len(), 1, "Should have one prerequisite");
            assert_eq!(
                prerequisites[0], "prerequisites",
                "Prerequisite should be 'prerequisites'"
            );
            assert_eq!(recipe.len(), 1, "Should have one recipe line");
            assert_eq!(recipe[0], "recipe", "Recipe should be 'recipe'");
            assert!(!(*phony), "Should not be marked as phony initially");
        }
        other => panic!("Expected Target item, got {:?}", other),
    }
}

/// RED PHASE: Test for RULE-SYNTAX-001 - Multiple prerequisites
#[test]
fn test_RULE_SYNTAX_001_multiple_prerequisites() {
    // ARRANGE: Rule with multiple prerequisites
    let makefile = "all: build test deploy";

    // ACT: Parse makefile
    let result = parse_makefile(makefile);

    // ASSERT: Successfully parsed
    assert!(result.is_ok());

    let ast = result.unwrap();
    assert_eq!(ast.items.len(), 1);

    // ASSERT: Check prerequisites
    match &ast.items[0] {
        MakeItem::Target {
            name,
            prerequisites,
            ..
        } => {
            assert_eq!(name, "all");
            assert_eq!(prerequisites.len(), 3);
            assert_eq!(prerequisites[0], "build");
            assert_eq!(prerequisites[1], "test");
            assert_eq!(prerequisites[2], "deploy");
        }
        _ => panic!("Expected Target item"),
    }
}

/// RED PHASE: Test for RULE-SYNTAX-001 - Empty recipe
#[test]
fn test_RULE_SYNTAX_001_empty_recipe() {
    // ARRANGE: Rule with no recipe
    let makefile = "target: prerequisites";

    // ACT: Parse makefile
    let result = parse_makefile(makefile);

    // ASSERT: Successfully parsed
    assert!(result.is_ok());

    let ast = result.unwrap();
    assert_eq!(ast.items.len(), 1);

    // ASSERT: Recipe is empty
    match &ast.items[0] {
        MakeItem::Target { recipe, .. } => {
            assert_eq!(recipe.len(), 0, "Recipe should be empty");
        }
        _ => panic!("Expected Target item"),
    }
}

/// RED PHASE: Test for RULE-SYNTAX-001 - Multi-line recipe
#[test]
fn test_RULE_SYNTAX_001_multiline_recipe() {
    // ARRANGE: Rule with multiple recipe lines
    let makefile =
        "deploy:\n\tcargo build --release\n\tcargo test\n\tscp target/release/app server:/opt/";

    // ACT: Parse makefile
    let result = parse_makefile(makefile);

    // ASSERT: Successfully parsed
    assert!(result.is_ok());

    let ast = result.unwrap();
    assert_eq!(ast.items.len(), 1);

    // ASSERT: Multiple recipe lines
    match &ast.items[0] {
        MakeItem::Target { recipe, .. } => {
            assert_eq!(recipe.len(), 3, "Should have 3 recipe lines");
            assert_eq!(recipe[0], "cargo build --release");
            assert_eq!(recipe[1], "cargo test");
            assert_eq!(recipe[2], "scp target/release/app server:/opt/");
        }
        _ => panic!("Expected Target item"),
    }
}

// FIXME(PMAT-238): #[cfg(test)]
// FIXME(PMAT-238): #[path = "part1_tests_rule_syntax.rs"]
// FIXME(PMAT-238): mod tests_extracted;
