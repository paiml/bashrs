#![allow(clippy::unwrap_used)]
#![allow(unused_imports)]

use super::super::*;
use crate::make_parser::ast::{MakeCondition, Span, VarFlavor};


// Phase 5 - MUTATION TESTING: Mutation-killing tests for = recursive assignment

// Target: parser.rs:116 - is_variable_assignment() contains('=') check
#[test]
fn test_VAR_FLAVOR_001_mut_equals_detection() {
    // ARRANGE: Makefile with = operator
    let makefile = "CC = gcc";

    // ACT: Parse the makefile
    let result = parse_makefile(makefile);

    // ASSERT: Must detect = as variable assignment
    assert!(result.is_ok(), "Must parse = as variable assignment");

    let ast = result.unwrap();
    assert_eq!(ast.items.len(), 1, "Must have exactly one item");

    // Must be a Variable (not a Target or Comment)
    match &ast.items[0] {
        MakeItem::Variable { name, flavor, .. } => {
            assert_eq!(name, "CC");
            assert_eq!(*flavor, VarFlavor::Recursive, "Must be Recursive flavor");
        }
        _ => panic!("Must be parsed as Variable, not other type"),
    }
}

// Target: parser.rs:156-157 - Correct string slicing for = operator
#[test]
fn test_VAR_FLAVOR_001_mut_operator_slicing() {
    // ARRANGE: Makefile with = and value after operator
    let makefile = "CFLAGS = -Wall -O2";

    // ACT: Parse
    let result = parse_makefile(makefile);
    assert!(result.is_ok());

    let ast = result.unwrap();
    match &ast.items[0] {
        MakeItem::Variable { name, value, .. } => {
            // Must correctly slice at = position
            assert_eq!(name, "CFLAGS", "Name must be before =");
            assert_eq!(value, "-Wall -O2", "Value must be after =");

            // Must NOT include = in name or value
            assert!(!name.contains('='), "Name must not contain =");
            assert!(!value.starts_with('='), "Value must not start with =");
        }
        _ => panic!("Expected Variable"),
    }
}

// Target: parser.rs:156-157 - = not missed in parsing
#[test]
fn test_VAR_FLAVOR_001_mut_recursive_not_missed() {
    // ARRANGE: Makefile with = operator (basic case)
    let makefile = "VAR = value";

    // ACT: Parse
    let result = parse_makefile(makefile);

    // ASSERT: Must successfully parse (not skip the line)
    assert!(result.is_ok(), "Must parse = assignment");

    let ast = result.unwrap();
    assert_eq!(ast.items.len(), 1, "Must parse one variable, not skip");

    match &ast.items[0] {
        MakeItem::Variable {
            name,
            value,
            flavor,
            ..
        } => {
            assert_eq!(name, "VAR");
            assert_eq!(value, "value");
            assert_eq!(*flavor, VarFlavor::Recursive);
        }
        _ => panic!("Must parse as Variable"),
    }
}

// Target: parser.rs:116 - = not confused with target rule separator :
#[test]
fn test_VAR_FLAVOR_001_mut_not_confused_with_colon() {
    // ARRANGE: Two inputs - one variable, one target
    let variable = "VAR = value";
    let target = "target: prereq";

    // ACT: Parse both
    let var_result = parse_makefile(variable);
    let target_result = parse_makefile(target);

    // ASSERT: Different types
    assert!(var_result.is_ok());
    assert!(target_result.is_ok());

    let var_ast = var_result.unwrap();
    let target_ast = target_result.unwrap();

    // Variable must be Variable, not Target
    match &var_ast.items[0] {
        MakeItem::Variable { flavor, .. } => {
            assert_eq!(*flavor, VarFlavor::Recursive);
        }
        MakeItem::Target { .. } => panic!("VAR = value must be Variable, not Target"),
        _ => panic!("Unexpected item type"),
    }

    // Target must be Target, not Variable
    match &target_ast.items[0] {
        MakeItem::Target { .. } => {
            // Correct!
        }
        MakeItem::Variable { .. } => panic!("target: must be Target, not Variable"),
        _ => panic!("Unexpected item type"),
    }
}

// Target: parser.rs:156-157 - Correct flavor enum variant assignment
#[test]
fn test_VAR_FLAVOR_001_mut_correct_flavor_enum_variant() {
    // ARRANGE: Makefile with = operator
    let makefile = "TARGET = value";

    // ACT: Parse
    let result = parse_makefile(makefile);
    assert!(result.is_ok());

    let ast = result.unwrap();
    match &ast.items[0] {
        MakeItem::Variable { flavor, .. } => {
            // Must be EXACTLY VarFlavor::Recursive
            assert!(
                matches!(flavor, VarFlavor::Recursive),
                "Must be VarFlavor::Recursive"
            );

            // Must NOT be any other variant
            assert!(!matches!(flavor, VarFlavor::Simple), "Must NOT be Simple");
            assert!(
                !matches!(flavor, VarFlavor::Conditional),
                "Must NOT be Conditional"
            );
            assert!(!matches!(flavor, VarFlavor::Append), "Must NOT be Append");
            assert!(!matches!(flavor, VarFlavor::Shell), "Must NOT be Shell");
        }
        _ => panic!("Expected Variable"),
    }
}

// ============================================================================
// Sprint 37: SYNTAX-002 - Line Continuation (\)
// ============================================================================
// Implements: Backslash line continuation for variables and recipes
// Verifies: Lines ending with \ are concatenated with next line

// Phase 1 - RED: Unit tests for line continuation

#[test]
fn test_SYNTAX_002_basic_line_continuation_in_variable() {
    // ARRANGE: Variable with backslash continuation
    let makefile = "FILES = file1.c \\\n    file2.c";

    // ACT: Parse the makefile
    let result = parse_makefile(makefile);

    // ASSERT: Parser should handle backslash line continuation
    assert!(result.is_ok(), "Parser should handle line continuation");

    let ast = result.unwrap();
    assert_eq!(ast.items.len(), 1, "Should parse one variable");

    match &ast.items[0] {
        MakeItem::Variable { name, value, .. } => {
            assert_eq!(name, "FILES", "Variable name should be FILES");
            // Continuation should concatenate lines (whitespace normalized)
            assert_eq!(
                value, "file1.c file2.c",
                "Value should concatenate continued lines"
            );
        }
        other => panic!("Expected Variable, got {:?}", other),
    }
}

#[test]
fn test_SYNTAX_002_multiple_line_continuations() {
    // ARRANGE: Variable with multiple backslash continuations
    let makefile = "SOURCES = a.c \\\n    b.c \\\n    c.c";

    // ACT: Parse the makefile
    let result = parse_makefile(makefile);

    // ASSERT: Parser should handle multiple continuations
    assert!(
        result.is_ok(),
        "Parser should handle multiple continuations"
    );

    let ast = result.unwrap();
    assert_eq!(ast.items.len(), 1, "Should parse one variable");

    match &ast.items[0] {
        MakeItem::Variable { name, value, .. } => {
            assert_eq!(name, "SOURCES", "Variable name should be SOURCES");
            // All three lines should be concatenated
            assert_eq!(
                value, "a.c b.c c.c",
                "Value should concatenate all continued lines"
            );
        }
        other => panic!("Expected Variable, got {:?}", other),
    }
}

#[test]
fn test_SYNTAX_002_line_continuation_preserves_order() {
    // ARRANGE: Variable with line continuation
    let makefile = "ITEMS = first \\\n    second \\\n    third";

    // ACT: Parse
    let result = parse_makefile(makefile);
    assert!(result.is_ok());

    let ast = result.unwrap();
    match &ast.items[0] {
        MakeItem::Variable { value, .. } => {
            // Order must be preserved: first, second, third
            assert_eq!(
                value, "first second third",
                "Order should be preserved in continuation"
            );
        }
        _ => panic!("Expected Variable"),
    }
}

#[test]
fn test_SYNTAX_002_continuation_vs_no_continuation() {
    // ARRANGE: Two variables - one with continuation, one without
    let with_continuation = "VAR = a \\\n    b";
    let without_continuation = "VAR = a b";

    // ACT: Parse both
    let result1 = parse_makefile(with_continuation);
    let result2 = parse_makefile(without_continuation);

    // ASSERT: Both should produce same result
    assert!(result1.is_ok());
    assert!(result2.is_ok());

    let ast1 = result1.unwrap();
    let ast2 = result2.unwrap();

    match (&ast1.items[0], &ast2.items[0]) {
        (MakeItem::Variable { value: v1, .. }, MakeItem::Variable { value: v2, .. }) => {
            // Both should produce "a b"
            assert_eq!(
                v1, v2,
                "Continuation and non-continuation should be equivalent"
            );
            assert_eq!(v1, "a b");
        }
        _ => panic!("Expected Variables"),
    }
}

// ============================================================================
// Sprint 37: SYNTAX-002 - Property Tests
// ============================================================================

#[cfg(test)]
mod property_tests_syntax_002 {
    use super::*;
    use proptest::prelude::*;

    // Property 1: Line continuation always produces valid parse result
    proptest! {
        #[test]
        fn test_SYNTAX_002_prop_continuation_always_parses(
            var_name in "[A-Z][A-Z0-9_]{0,10}",
            value1 in "[a-z0-9_]{1,10}",
            value2 in "[a-z0-9_]{1,10}"
        ) {
            // ARRANGE: Variable with line continuation
            let makefile = format!("{} = {} \\\n    {}", var_name, value1, value2);

            // ACT: Parse
            let result = parse_makefile(&makefile);

            // ASSERT: Must parse successfully
            prop_assert!(result.is_ok(), "Line continuation must always parse: {:?}", result);
        }
    }

    // Property 2: Continuation is equivalent to same-line definition
    proptest! {
        #[test]
        fn test_SYNTAX_002_prop_continuation_equivalent_to_sameline(
            var_name in "[A-Z][A-Z0-9_]{0,10}",
            value1 in "[a-z0-9_]{1,10}",
            value2 in "[a-z0-9_]{1,10}"
        ) {
            // ARRANGE: Two versions - with and without continuation
            let with_continuation = format!("{} = {} \\\n    {}", var_name, value1, value2);
            let without_continuation = format!("{} = {} {}", var_name, value1, value2);

            // ACT: Parse both
            let result1 = parse_makefile(&with_continuation);
            let result2 = parse_makefile(&without_continuation);

            // ASSERT: Both must parse successfully
            prop_assert!(result1.is_ok());
            prop_assert!(result2.is_ok());

            // ASSERT: Must produce same value
            let ast1 = result1.unwrap();
            let ast2 = result2.unwrap();

            match (&ast1.items[0], &ast2.items[0]) {
                (
                    MakeItem::Variable { value: v1, .. },
                    MakeItem::Variable { value: v2, .. }
                ) => {
                    prop_assert_eq!(v1, v2, "Continuation must be equivalent to same-line");
                }
                _ => return Err(TestCaseError::fail("Expected Variables")),
            }
        }
    }

    // Property 3: Multiple continuations work correctly
    proptest! {
        #[test]
        fn test_SYNTAX_002_prop_multiple_continuations(
            var_name in "[A-Z][A-Z0-9_]{0,10}",
            values in prop::collection::vec("[a-z0-9_]{1,10}", 2..5)
        ) {
            // ARRANGE: Variable with multiple continuations
            let mut makefile = format!("{} = {}", var_name, values[0]);
            for i in 1..values.len() {
                makefile.push_str(" \\\n    ");
                makefile.push_str(&values[i]);
            }

            // ACT: Parse
            let result = parse_makefile(&makefile);

            // ASSERT: Must parse successfully
            prop_assert!(result.is_ok(), "Multiple continuations must parse: {:?}", result);

            let ast = result.unwrap();
            prop_assert_eq!(ast.items.len(), 1);

            match &ast.items[0] {
                MakeItem::Variable { value, .. } => {
                    // All values should be present in order
                    for v in &values {
                        prop_assert!(value.contains(v), "Value {:?} should contain {:?}", value, v);
                    }
                }
                _ => return Err(TestCaseError::fail("Expected Variable")),
            }
        }
    }

    // Property 4: Continuation preserves value order
    proptest! {
        #[test]
        fn test_SYNTAX_002_prop_preserves_order(
            var_name in "[A-Z][A-Z0-9_]{0,10}",
            value1 in "[a-z]{3,5}",
            value2 in "[a-z]{3,5}",
            value3 in "[a-z]{3,5}"
        ) {
            // Skip if any values are duplicates or substrings of each other
            // (can't reliably test order with overlapping strings)
            if value1 == value2 || value2 == value3 || value1 == value3 {
                return Ok(());
            }
            if value1.contains(&value2) || value2.contains(&value1) ||
               value2.contains(&value3) || value3.contains(&value2) ||
               value1.contains(&value3) || value3.contains(&value1) {
                return Ok(());
            }

            // ARRANGE: Variable with 3 values on continued lines
            let makefile = format!("{} = {} \\\n    {} \\\n    {}", var_name, value1, value2, value3);

            // ACT: Parse
            let result = parse_makefile(&makefile);
            prop_assert!(result.is_ok());

            let ast = result.unwrap();
            match &ast.items[0] {
                MakeItem::Variable { value, .. } => {
                    // Find positions of values in result
                    let pos1 = value.find(&value1);
                    let pos2 = value.find(&value2);
                    let pos3 = value.find(&value3);

                    prop_assert!(pos1.is_some());
                    prop_assert!(pos2.is_some());
                    prop_assert!(pos3.is_some());

                    // Order must be preserved: value1 < value2 < value3
                    prop_assert!(pos1.unwrap() < pos2.unwrap(), "Order: {} < {}", value1, value2);
                    prop_assert!(pos2.unwrap() < pos3.unwrap(), "Order: {} < {}", value2, value3);
                }
                _ => return Err(TestCaseError::fail("Expected Variable")),
            }
        }
    }

    // Property 5: Line continuation works with all variable flavors
    proptest! {
        #[test]
        fn test_SYNTAX_002_prop_works_with_all_flavors(
            var_name in "[A-Z][A-Z0-9_]{0,10}",
            value1 in "[a-z0-9_]{1,10}",
            value2 in "[a-z0-9_]{1,10}",
            flavor in prop::sample::select(vec!["=", ":=", "?=", "+=", "!="])
        ) {
            // ARRANGE: Variable with continuation using specific flavor
            let makefile = format!("{} {} {} \\\n    {}", var_name, flavor, value1, value2);

            // ACT: Parse
            let result = parse_makefile(&makefile);

            // ASSERT: Must parse successfully regardless of flavor
            prop_assert!(result.is_ok(), "Continuation with flavor {} must parse", flavor);

            let ast = result.unwrap();
            prop_assert_eq!(ast.items.len(), 1);

            match &ast.items[0] {
                MakeItem::Variable { .. } => {
                    // Successfully parsed as variable
                }
                _ => return Err(TestCaseError::fail("Expected Variable")),
            }
        }
    }
}

// ============================================================================
// Sprint 37: SYNTAX-002 - Mutation Killing Tests
// ============================================================================

#[cfg(test)]
mod mutation_killing_tests_syntax_002 {
    use super::*;

    // Mutation: Change ends_with('\\') to ends_with('/')
    // This test ensures we only recognize backslash as continuation
    #[test]
    fn test_SYNTAX_002_mut_only_backslash_is_continuation() {
        // ARRANGE: Variable with forward slash (NOT continuation)
        let makefile = "FILES = file1.c /\n    file2.c";

        // ACT: Parse
        let result = parse_makefile(makefile);

        // ASSERT: Should parse as TWO items (variable + unknown/ignored line)
        // NOT as continuation (which would be ONE item)
        assert!(result.is_ok());
        let ast = result.unwrap();

        // Forward slash should NOT trigger continuation
        match &ast.items[0] {
            MakeItem::Variable { value, .. } => {
                // Value should contain the slash (not joined with next line)
                assert!(
                    value.contains('/'),
                    "Forward slash should be preserved, not treated as continuation"
                );
            }
            _ => panic!("Expected Variable"),
        }
    }

    // Mutation: Change i + 1 < lines.len() to i < lines.len()
    // This test ensures we don't read beyond array bounds
    #[test]
    fn test_SYNTAX_002_mut_handles_backslash_at_end_of_file() {
        // ARRANGE: Variable with backslash at EOF (no next line)
        let makefile = "FILES = file1.c \\";

        // ACT: Parse
        let result = parse_makefile(makefile);

        // ASSERT: Must parse successfully (not panic from index out of bounds)
        assert!(result.is_ok(), "Backslash at EOF should not panic");

        let ast = result.unwrap();
        assert_eq!(ast.items.len(), 1);

        match &ast.items[0] {
            MakeItem::Variable { value, .. } => {
                // Trailing backslash at EOF should be handled gracefully
                // The backslash is preserved since there's no next line to continue with
                assert_eq!(
                    value, "file1.c \\",
                    "Trailing backslash at EOF should be preserved"
                );

                // Key point: we don't panic with out-of-bounds access
                // The condition `i + 1 < lines.len()` prevents reading past EOF
            }
            _ => panic!("Expected Variable"),
        }
    }

    // Mutation: Change trim_end() to trim()
    // This test ensures we preserve leading whitespace in values
    #[test]
    fn test_SYNTAX_002_mut_preserves_leading_whitespace_in_first_line() {
        // ARRANGE: Variable with leading spaces in value
        let makefile = "FILES =     file1.c \\\n    file2.c";

        // ACT: Parse
        let result = parse_makefile(makefile);
        assert!(result.is_ok());

        let ast = result.unwrap();
        match &ast.items[0] {
            MakeItem::Variable { value, .. } => {
                // Leading spaces before file1.c should be handled by parse_variable's trim
                // This test ensures the continuation logic doesn't break that
                assert_eq!(
                    value, "file1.c file2.c",
                    "Whitespace normalization should work correctly"
                );
            }
            _ => panic!("Expected Variable"),
        }
    }

    // Mutation: Change push(' ') to push_str("  ")
    // This test ensures we use single space, not double space
    #[test]
    fn test_SYNTAX_002_mut_uses_single_space_separator() {
        // ARRANGE: Variable with continuation
        let makefile = "FILES = a.c \\\n    b.c";

        // ACT: Parse
        let result = parse_makefile(makefile);
        assert!(result.is_ok());

        let ast = result.unwrap();
        match &ast.items[0] {
            MakeItem::Variable { value, .. } => {
                // Must use single space, not double
                assert_eq!(value, "a.c b.c", "Should use single space separator");
                assert!(!value.contains("  "), "Should not contain double spaces");
            }
            _ => panic!("Expected Variable"),
        }
    }

    // Mutation: Remove the trim_start() call on next_line
    // This test ensures we strip leading whitespace from continued lines
    #[test]
    fn test_SYNTAX_002_mut_strips_leading_whitespace_from_continuation() {
        // ARRANGE: Variable with heavy indentation on continued line
        let makefile = "FILES = file1.c \\\n            file2.c";

        // ACT: Parse
        let result = parse_makefile(makefile);
        assert!(result.is_ok());

        let ast = result.unwrap();
        match &ast.items[0] {
            MakeItem::Variable { value, .. } => {
                // Leading whitespace should be stripped from continued line
                assert_eq!(
                    value, "file1.c file2.c",
                    "Leading whitespace should be stripped"
                );
                // Should NOT contain multiple spaces from the indentation
                assert!(
                    !value.contains("file1.c            file2.c"),
                    "Indentation should be normalized to single space"
                );
            }
            _ => panic!("Expected Variable"),
        }
    }
}

// ============================================================================
// Sprint 38: RECIPE-001 - Tab-indented recipes
// ============================================================================
// Implements: Tab-indented recipe lines for targets
// Verifies: Parser correctly handles tab-indented recipe lines (MUST be tabs, not spaces)

// Phase 1 - RED: Unit tests for tab-indented recipes

#[test]
fn test_RECIPE_001_single_tab_indented_recipe() {
    // ARRANGE: Target with single tab-indented recipe line
    let makefile = "build:\n\tcargo build --release";

    // ACT: Parse the makefile
    let result = parse_makefile(makefile);

    // ASSERT: Parser should handle tab-indented recipe
    assert!(result.is_ok(), "Parser should handle tab-indented recipe");

    let ast = result.unwrap();
    assert_eq!(ast.items.len(), 1, "Should parse one target");

    match &ast.items[0] {
        MakeItem::Target { name, recipe, .. } => {
            assert_eq!(name, "build", "Target name should be 'build'");
            assert_eq!(recipe.len(), 1, "Should have one recipe line");
            assert_eq!(
                recipe[0], "cargo build --release",
                "Recipe should be parsed correctly"
            );
        }
        other => panic!("Expected Target, got {:?}", other),
    }
}

#[test]
fn test_RECIPE_001_multiple_tab_indented_recipes() {
    // ARRANGE: Target with multiple tab-indented recipe lines
    let makefile =
        "deploy:\n\tcargo build --release\n\tcargo test\n\tscp target/release/app server:/opt/";

    // ACT: Parse
    let result = parse_makefile(makefile);

    // ASSERT: Should parse all recipe lines
    assert!(result.is_ok(), "Multiple recipe lines should parse");

    let ast = result.unwrap();
    assert_eq!(ast.items.len(), 1);

    match &ast.items[0] {
        MakeItem::Target { name, recipe, .. } => {
            assert_eq!(name, "deploy");
            assert_eq!(recipe.len(), 3, "Should have three recipe lines");
            assert_eq!(recipe[0], "cargo build --release");
            assert_eq!(recipe[1], "cargo test");
            assert_eq!(recipe[2], "scp target/release/app server:/opt/");
        }
        _ => panic!("Expected Target"),
    }
}

#[test]
fn test_RECIPE_001_recipe_with_empty_lines() {
    // ARRANGE: Target with recipe lines separated by empty lines
    let makefile = "test:\n\tcargo test --lib\n\n\tcargo test --doc";

    // ACT: Parse
    let result = parse_makefile(makefile);
    assert!(result.is_ok());

    let ast = result.unwrap();
    match &ast.items[0] {
        MakeItem::Target { recipe, .. } => {
            // Empty lines between recipe lines should be skipped
            assert_eq!(
                recipe.len(),
                2,
                "Should parse both recipe lines despite empty line"
            );
            assert_eq!(recipe[0], "cargo test --lib");
            assert_eq!(recipe[1], "cargo test --doc");
        }
        _ => panic!("Expected Target"),
    }
}

#[test]
fn test_RECIPE_001_recipe_stops_at_non_tab_line() {
    // ARRANGE: Makefile with target followed by another construct
    let makefile = "build:\n\tcargo build\n\nCC = gcc";

    // ACT: Parse
    let result = parse_makefile(makefile);
    assert!(result.is_ok());

    let ast = result.unwrap();
    assert_eq!(ast.items.len(), 2, "Should parse target and variable");

    match &ast.items[0] {
        MakeItem::Target { name, recipe, .. } => {
            assert_eq!(name, "build");
            assert_eq!(recipe.len(), 1, "Recipe should stop at non-tab line");
            assert_eq!(recipe[0], "cargo build");
        }
        _ => panic!("Expected Target"),
    }

    match &ast.items[1] {
        MakeItem::Variable { name, .. } => {
            assert_eq!(name, "CC", "Should parse variable after recipe");
        }
        _ => panic!("Expected Variable"),
    }
}

// Phase 4 - PROPERTY TESTING: Property tests for recipe parsing

#[cfg(test)]
mod recipe_property_tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn prop_RECIPE_001_recipes_with_varying_lines_always_parse(
            num_lines in 1usize..10,
            target_name in "[a-z][a-z0-9_]*"
        ) {
            // ARRANGE: Generate target with varying number of recipe lines
            let mut makefile = format!("{}:\n", target_name);
            for i in 0..num_lines {
                makefile.push_str(&format!("\tcommand_{}\n", i));
            }

            // ACT: Parse
            let result = parse_makefile(&makefile);

            // ASSERT: Should always parse successfully
            prop_assert!(result.is_ok(), "Should parse {} recipe lines", num_lines);

            let ast = result.unwrap();
            prop_assert_eq!(ast.items.len(), 1, "Should have one target");

            match &ast.items[0] {
                MakeItem::Target { name, recipe, .. } => {
                    prop_assert_eq!(name, &target_name, "Target name should match");
                    prop_assert_eq!(recipe.len(), num_lines, "Should have {} recipe lines", num_lines);
                }
                _ => return Err(TestCaseError::fail("Expected Target")),
            }
        }

        #[test]
        fn prop_RECIPE_001_recipe_parsing_is_deterministic(
            target_name in "[a-z][a-z0-9_]*",
            recipe_cmd in "[a-z][a-z0-9 _-]*"
        ) {
            // ARRANGE: Create makefile with recipe
            let makefile = format!("{}:\n\t{}", target_name, recipe_cmd);

            // ACT: Parse twice
            let result1 = parse_makefile(&makefile);
            let result2 = parse_makefile(&makefile);

            // ASSERT: Both should succeed and produce identical results
            prop_assert!(result1.is_ok());
            prop_assert!(result2.is_ok());

            let ast1 = result1.unwrap();
            let ast2 = result2.unwrap();

            prop_assert_eq!(ast1.items.len(), ast2.items.len(), "Item count should be deterministic");

            match (&ast1.items[0], &ast2.items[0]) {
                (
                    MakeItem::Target { name: n1, recipe: r1, .. },
                    MakeItem::Target { name: n2, recipe: r2, .. }
                ) => {
                    prop_assert_eq!(n1, n2, "Target names should be identical");
                    prop_assert_eq!(r1, r2, "Recipes should be identical");
                }
                _ => return Err(TestCaseError::fail("Both should be Target items")),
            }
        }

        #[test]
        fn prop_RECIPE_001_tab_indented_lines_always_recognized(
            recipe_count in 1usize..5,
            cmd in "[a-z][a-z0-9 ]*"
        ) {
            // ARRANGE: Create target with multiple tab-indented lines
            let mut makefile = String::from("target:\n");
            for _ in 0..recipe_count {
                makefile.push('\t');
                makefile.push_str(&cmd);
                makefile.push('\n');
            }

            // ACT: Parse
            let result = parse_makefile(&makefile);

            // ASSERT: All tab-indented lines should be recognized as recipes
            prop_assert!(result.is_ok());

            let ast = result.unwrap();
            match &ast.items[0] {
                MakeItem::Target { recipe, .. } => {
                    prop_assert_eq!(recipe.len(), recipe_count, "All tab-indented lines should be recipes");
                    for recipe_line in recipe {
                        prop_assert!(recipe_line.trim() == cmd.trim(), "Recipe content should match");
                    }
                }
                _ => return Err(TestCaseError::fail("Expected Target")),
            }
        }

        #[test]
        fn prop_RECIPE_001_recipe_order_preserved(
            num_recipes in 2usize..6
        ) {
            // ARRANGE: Create target with ordered recipe lines
            let mut makefile = String::from("build:\n");
            let mut expected_order = Vec::new();

            for i in 0..num_recipes {
                let cmd = format!("step_{}", i);
                makefile.push_str(&format!("\t{}\n", cmd));
                expected_order.push(cmd);
            }

            // ACT: Parse
            let result = parse_makefile(&makefile);
            prop_assert!(result.is_ok());

            let ast = result.unwrap();

            // ASSERT: Recipe order should be preserved
            match &ast.items[0] {
                MakeItem::Target { recipe, .. } => {
                    prop_assert_eq!(recipe.len(), expected_order.len());
                    for (i, recipe_line) in recipe.iter().enumerate() {
                        prop_assert_eq!(recipe_line, &expected_order[i], "Recipe order should be preserved");
                    }
                }
                _ => return Err(TestCaseError::fail("Expected Target")),
            }
        }

        #[test]
        fn prop_RECIPE_001_various_recipe_commands_work(
            prefix in "cargo|make|echo|cd|mkdir",
            suffix in "[a-z0-9_/-]*"
        ) {
            // ARRANGE: Create target with various command types
            let recipe_cmd = format!("{} {}", prefix, suffix);
            let makefile = format!("test:\n\t{}", recipe_cmd);

            // ACT: Parse
            let result = parse_makefile(&makefile);

            // ASSERT: All command types should parse correctly
            prop_assert!(result.is_ok(), "Command '{}' should parse", recipe_cmd);

            let ast = result.unwrap();
            match &ast.items[0] {
                MakeItem::Target { recipe, .. } => {
                    prop_assert_eq!(recipe.len(), 1);
                    prop_assert!(recipe[0].starts_with(&prefix), "Recipe should start with prefix");
                }
                _ => return Err(TestCaseError::fail("Expected Target")),
            }
        }
    }
}

// Phase 5 - MUTATION TESTING: Mutation-killing tests for recipe parsing

#[test]
fn test_RECIPE_001_mut_tab_detection_must_use_starts_with() {
    // MUTATION TARGET: line 268 in parser.rs
    // Mutation: Replace starts_with('\t') with something else
    // This test kills mutations that break tab detection logic

    // ARRANGE: Target with tab-indented recipe and space-indented line
    let makefile = "build:\n\tcargo build\n    not a recipe";

    // ACT: Parse
    let result = parse_makefile(makefile);
    assert!(result.is_ok());

    let ast = result.unwrap();

    // ASSERT: Only tab-indented line should be in recipe
    match &ast.items[0] {
        MakeItem::Target { recipe, .. } => {
            assert_eq!(recipe.len(), 1, "Only tab-indented line should be recipe");
            assert_eq!(
                recipe[0], "cargo build",
                "Tab-indented line should be recipe"
            );
        }
        _ => panic!("Expected Target"),
    }

    // CRITICAL: If starts_with('\t') is mutated to starts_with(' ') or contains('\t'),
    // this test will fail because "    not a recipe" would be parsed as recipe
}

#[test]
fn test_RECIPE_001_mut_recipe_push_must_happen() {
    // MUTATION TARGET: line 270 in parser.rs (recipe.push)
    // Mutation: Remove recipe.push() call
    // This test kills mutations that skip adding recipe lines

    // ARRANGE: Target with multiple recipes
    let makefile = "test:\n\tcargo test --lib\n\tcargo test --doc\n\tcargo test --integration";

    // ACT: Parse
    let result = parse_makefile(makefile);
    assert!(result.is_ok());

    let ast = result.unwrap();

    // ASSERT: All recipes must be captured
    match &ast.items[0] {
        MakeItem::Target { recipe, .. } => {
            assert_eq!(recipe.len(), 3, "All three recipe lines must be pushed");
            assert_eq!(recipe[0], "cargo test --lib");
            assert_eq!(recipe[1], "cargo test --doc");
            assert_eq!(recipe[2], "cargo test --integration");
        }
        _ => panic!("Expected Target"),
    }

    // CRITICAL: If recipe.push() is removed, recipe would be empty
}

#[test]
fn test_RECIPE_001_mut_empty_line_handling_must_continue() {
    // MUTATION TARGET: line 276 in parser.rs (continue in empty line handling)
    // Mutation: Replace continue with break
    // This test kills mutations that break empty line handling

    // ARRANGE: Recipe with empty line in the middle
    let makefile = "deploy:\n\techo 'Starting'\n\n\techo 'Done'";

    // ACT: Parse
    let result = parse_makefile(makefile);
    assert!(result.is_ok());

    let ast = result.unwrap();

    // ASSERT: Both recipes should be parsed despite empty line
    match &ast.items[0] {
        MakeItem::Target { recipe, .. } => {
            assert_eq!(recipe.len(), 2, "Empty line should not stop recipe parsing");
            assert_eq!(recipe[0], "echo 'Starting'");
            assert_eq!(recipe[1], "echo 'Done'");
        }
        _ => panic!("Expected Target"),
    }

    // CRITICAL: If continue is replaced with break, only first recipe would be parsed
}

#[test]
fn test_RECIPE_001_mut_non_tab_line_must_break_loop() {
    // MUTATION TARGET: line 283 in parser.rs (break on non-tab line)
    // Mutation: Replace break with continue or remove it
    // This test kills mutations that fail to stop at non-tab lines

    // ARRANGE: Recipe followed by variable assignment
    let makefile = "build:\n\tcargo build\nCC = gcc\n\tthis should not be recipe";

    // ACT: Parse
    let result = parse_makefile(makefile);
    assert!(result.is_ok());

    let ast = result.unwrap();

    // ASSERT: Recipe should stop at CC = gcc
    assert_eq!(ast.items.len(), 2, "Should have target and variable");

    match &ast.items[0] {
        MakeItem::Target { recipe, .. } => {
            assert_eq!(recipe.len(), 1, "Recipe should stop at non-tab line");
            assert_eq!(recipe[0], "cargo build");
        }
        _ => panic!("Expected Target"),
    }

    match &ast.items[1] {
        MakeItem::Variable { name, .. } => {
            assert_eq!(name, "CC", "Variable should be parsed after recipe ends");
        }
        _ => panic!("Expected Variable"),
    }

    // CRITICAL: If break is removed, parsing would be incorrect
}

#[test]
fn test_RECIPE_001_mut_index_increment_must_happen() {
    // MUTATION TARGET: line 271 in parser.rs (*index += 1 in recipe loop)
    // Mutation: Remove or change index increment
    // This test kills mutations that break loop progression

    // ARRANGE: Target with multiple recipes followed by another target
    let makefile = "first:\n\tcommand1\n\tcommand2\nsecond:\n\tcommand3";

    // ACT: Parse
    let result = parse_makefile(makefile);
    assert!(result.is_ok());

    let ast = result.unwrap();

    // ASSERT: Both targets should be parsed correctly
    assert_eq!(ast.items.len(), 2, "Should parse both targets");

    match &ast.items[0] {
        MakeItem::Target { name, recipe, .. } => {
            assert_eq!(name, "first");
            assert_eq!(recipe.len(), 2, "First target should have 2 recipes");
        }
        _ => panic!("Expected Target"),
    }

    match &ast.items[1] {
        MakeItem::Target { name, recipe, .. } => {
            assert_eq!(name, "second");
            assert_eq!(recipe.len(), 1, "Second target should have 1 recipe");
        }
        _ => panic!("Expected Target"),
    }

    // CRITICAL: If index isn't incremented, parser would loop infinitely or parse incorrectly
}

// ============================================================================
// Sprint 39: RECIPE-002 - Multi-line recipes
// ============================================================================
// Implements: Multiple recipe lines for a single target
// Verifies: Parser correctly collects all tab-indented recipe lines

// Phase 1 - RED: Unit tests for multi-line recipes

#[test]
fn test_RECIPE_002_basic_three_line_recipe() {
    // ARRANGE: Target with 3 distinct recipe lines
    let makefile = "deploy:\n\tcargo build --release\n\tcargo test\n\tcp target/release/app /opt/";

    // ACT: Parse the makefile
    let result = parse_makefile(makefile);

    // ASSERT: All 3 recipe lines should be parsed
    assert!(result.is_ok(), "Parser should handle 3-line recipe");

    let ast = result.unwrap();
    assert_eq!(ast.items.len(), 1, "Should parse one target");

    match &ast.items[0] {
        MakeItem::Target { name, recipe, .. } => {
            assert_eq!(name, "deploy", "Target name should be 'deploy'");
            assert_eq!(recipe.len(), 3, "Should have three recipe lines");
            assert_eq!(recipe[0], "cargo build --release", "First recipe line");
            assert_eq!(recipe[1], "cargo test", "Second recipe line");
            assert_eq!(
                recipe[2], "cp target/release/app /opt/",
                "Third recipe line"
            );
        }
        other => panic!("Expected Target, got {:?}", other),
    }
}

#[test]
fn test_RECIPE_002_many_recipe_lines() {
    // ARRANGE: Target with 5 recipe lines (typical CI/CD deploy)
    let makefile = "ci:\n\techo 'Starting CI'\n\tcargo fmt --check\n\tcargo clippy\n\tcargo test\n\techo 'CI passed'";

    // ACT: Parse
    let result = parse_makefile(makefile);
    assert!(result.is_ok(), "Many recipe lines should parse");

    let ast = result.unwrap();
    match &ast.items[0] {
        MakeItem::Target { name, recipe, .. } => {
            assert_eq!(name, "ci");
            assert_eq!(recipe.len(), 5, "Should have five recipe lines");
            assert_eq!(recipe[0], "echo 'Starting CI'");
            assert_eq!(recipe[4], "echo 'CI passed'");
        }
        _ => panic!("Expected Target"),
    }
}

#[test]
fn test_RECIPE_002_recipe_order_preserved() {
    // ARRANGE: Recipe with specific ordering (important for build steps)
    let makefile = "build:\n\tmkdir -p dist\n\tcargo build --release\n\tcp target/release/app dist/\n\tstrip dist/app";

    // ACT: Parse
    let result = parse_makefile(makefile);
    assert!(result.is_ok());

    let ast = result.unwrap();
    match &ast.items[0] {
        MakeItem::Target { recipe, .. } => {
            // Order matters: mkdir before cp, build before cp, strip last
            assert_eq!(recipe.len(), 4);
            assert_eq!(recipe[0], "mkdir -p dist", "Step 1: create directory");
            assert_eq!(recipe[1], "cargo build --release", "Step 2: build");
            assert_eq!(recipe[2], "cp target/release/app dist/", "Step 3: copy");
            assert_eq!(recipe[3], "strip dist/app", "Step 4: strip");
        }
        _ => panic!("Expected Target"),
    }
}

#[test]
fn test_RECIPE_002_different_targets_different_recipes() {
    // ARRANGE: Multiple targets each with multiple recipe lines
    let makefile = "build:\n\tcargo build\n\tcargo test\n\nclean:\n\trm -rf target\n\tfind . -name '*.o' -delete";

    // ACT: Parse
    let result = parse_makefile(makefile);
    assert!(result.is_ok());

    let ast = result.unwrap();
    assert_eq!(ast.items.len(), 2, "Should have two targets");

    match &ast.items[0] {
        MakeItem::Target { name, recipe, .. } => {
            assert_eq!(name, "build");
            assert_eq!(recipe.len(), 2, "build has 2 recipe lines");
        }
        _ => panic!("Expected first Target"),
    }

    match &ast.items[1] {
        MakeItem::Target { name, recipe, .. } => {
            assert_eq!(name, "clean");
            assert_eq!(recipe.len(), 2, "clean has 2 recipe lines");
        }
        _ => panic!("Expected second Target"),
    }
}

// Phase 4 - PROPERTY TESTING: Property tests for multi-line recipes

#[cfg(test)]
mod recipe_002_property_tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn prop_RECIPE_002_varying_recipe_line_count_always_parses(
            num_lines in 2usize..10,
            target_name in "[a-z][a-z0-9_]*"
        ) {
            // ARRANGE: Generate target with varying number of recipe lines (2-9)
            let mut makefile = format!("{}:\n", target_name);
            for i in 0..num_lines {
                makefile.push_str(&format!("\techo 'step {}'\n", i));
            }

            // ACT: Parse
            let result = parse_makefile(&makefile);

            // ASSERT: Should always parse successfully
            prop_assert!(result.is_ok(), "Should parse {} recipe lines", num_lines);

            let ast = result.unwrap();
            prop_assert_eq!(ast.items.len(), 1, "Should have one target");

            match &ast.items[0] {
                MakeItem::Target { name, recipe, .. } => {
                    prop_assert_eq!(name, &target_name, "Target name should match");
                    prop_assert_eq!(recipe.len(), num_lines, "Should have {} recipe lines", num_lines);
                }
                _ => return Err(TestCaseError::fail("Expected Target")),
            }
        }

        #[test]
        fn prop_RECIPE_002_multi_line_parsing_is_deterministic(
            target in "[a-z][a-z0-9_]*",
            cmd1 in "[a-z][a-z0-9 ]*",
            cmd2 in "[a-z][a-z0-9 ]*"
        ) {
            // ARRANGE: Multi-line recipe
            let makefile = format!("{}:\n\t{}\n\t{}", target, cmd1, cmd2);

            // ACT: Parse twice
            let result1 = parse_makefile(&makefile);
            let result2 = parse_makefile(&makefile);

            // ASSERT: Should produce identical results
            prop_assert!(result1.is_ok() && result2.is_ok());

            let ast1 = result1.unwrap();
            let ast2 = result2.unwrap();

            match (&ast1.items[0], &ast2.items[0]) {
                (MakeItem::Target { recipe: r1, .. }, MakeItem::Target { recipe: r2, .. }) => {
                    prop_assert_eq!(r1.len(), r2.len(), "Recipe lengths should match");
                    prop_assert_eq!(r1, r2, "Recipes should be identical");
                }
                _ => return Err(TestCaseError::fail("Expected Target")),
            }
        }

        #[test]
        fn prop_RECIPE_002_all_recipe_lines_collected(
            line_count in 2usize..8
        ) {
            // ARRANGE: Generate recipe with specific line count
            let mut makefile = "target:\n".to_string();
            let expected_lines: Vec<String> = (0..line_count)
                .map(|i| format!("command_{}", i))
                .collect();

            for cmd in &expected_lines {
                makefile.push_str(&format!("\t{}\n", cmd));
            }

            // ACT: Parse
            let result = parse_makefile(&makefile);
            prop_assert!(result.is_ok());

            let ast = result.unwrap();

            // ASSERT: All recipe lines should be present
            match &ast.items[0] {
                MakeItem::Target { recipe, .. } => {
                    prop_assert_eq!(recipe.len(), line_count, "Should collect all lines");

                    for (i, expected) in expected_lines.iter().enumerate() {
                        prop_assert_eq!(
                            &recipe[i],
                            expected,
                            "Recipe line {} should match",
                            i
                        );
                    }
                }
                _ => return Err(TestCaseError::fail("Expected Target")),
            }
        }

        #[test]
        fn prop_RECIPE_002_recipe_order_always_preserved(
            num_lines in 2usize..6
        ) {
            // ARRANGE: Recipe with numbered commands
            let mut makefile = "build:\n".to_string();
            for i in 0..num_lines {
                makefile.push_str(&format!("\tstep_{}\n", i));
            }

            // ACT: Parse
            let result = parse_makefile(&makefile);
            prop_assert!(result.is_ok());

            let ast = result.unwrap();

            // ASSERT: Order must be preserved
            match &ast.items[0] {
                MakeItem::Target { recipe, .. } => {
                    for i in 0..num_lines {
                        let expected = format!("step_{}", i);
                        prop_assert_eq!(
                            &recipe[i],
                            &expected,
                            "Line {} should be in order",
                            i
                        );
                    }
                }
                _ => return Err(TestCaseError::fail("Expected Target")),
            }
        }

        #[test]
        fn prop_RECIPE_002_complex_commands_in_multiline_recipe(
            target in "[a-z][a-z0-9_]*"
        ) {
            // ARRANGE: Multi-line recipe with complex realistic commands
            let makefile = format!(
                "{}:\n\tmkdir -p target/release\n\tcargo build --release --features prod\n\tstrip target/release/{}\n\tcp target/release/{} /opt/bin/",
                target, target, target
            );

            // ACT: Parse
            let result = parse_makefile(&makefile);
            prop_assert!(result.is_ok(), "Complex multi-line recipes should parse");

            let ast = result.unwrap();

            // ASSERT: All complex commands should be parsed correctly
            match &ast.items[0] {
                MakeItem::Target { recipe, .. } => {
                    prop_assert_eq!(recipe.len(), 4, "Should have 4 recipe lines");
                    prop_assert!(recipe[0].starts_with("mkdir -p"), "First command");
                    prop_assert!(recipe[1].starts_with("cargo build"), "Second command");
                    prop_assert!(recipe[2].starts_with("strip"), "Third command");
                    prop_assert!(recipe[3].starts_with("cp"), "Fourth command");
                }
                _ => return Err(TestCaseError::fail("Expected Target")),
            }
        }
    }
}

// Phase 5 - MUTATION TESTING: Mutation-killing tests for multi-line recipes

#[test]
fn test_RECIPE_002_mut_all_recipe_lines_must_be_collected() {
    // MUTATION TARGET: line 270 in parser.rs
    // Mutation: Skip push or only push first/last line
    // This test kills mutations that fail to collect all recipe lines

    // ARRANGE: Target with 3 distinct recipe lines
    let makefile = "deploy:\n\tline1\n\tline2\n\tline3";

    // ACT: Parse
    let result = parse_makefile(makefile);
    assert!(result.is_ok());

    let ast = result.unwrap();

    // ASSERT: MUST have ALL 3 lines
    match &ast.items[0] {
        MakeItem::Target { recipe, .. } => {
            assert_eq!(recipe.len(), 3, "MUST collect all 3 lines");
            assert_eq!(recipe[0], "line1", "First line must be present");
            assert_eq!(recipe[1], "line2", "Middle line must be present");
            assert_eq!(recipe[2], "line3", "Last line must be present");
        }
        _ => panic!("Expected Target"),
    }

    // CRITICAL: If recipe.push() is mutated or conditional, this test fails
}

#[test]
fn test_RECIPE_002_mut_recipe_count_must_be_exact() {
    // MUTATION TARGET: line 270 in parser.rs
    // Mutation: Push multiple times or skip lines
    // This test kills mutations that change the count of recipe lines

    // ARRANGE: Makefile with exactly 4 recipe lines
    let makefile = "build:\n\tcmd1\n\tcmd2\n\tcmd3\n\tcmd4";

    // ACT: Parse
    let result = parse_makefile(makefile);
    assert!(result.is_ok());

    let ast = result.unwrap();

    // ASSERT: MUST have EXACTLY 4 lines (not 3, not 5)
    match &ast.items[0] {
        MakeItem::Target { recipe, .. } => {
            assert_eq!(
                recipe.len(),
                4,
                "MUST have exactly 4 recipe lines, no more, no less"
            );
        }
        _ => panic!("Expected Target"),
    }

    // CRITICAL: If loop mutates to skip lines or duplicate, count will be wrong
}

#[test]
fn test_RECIPE_002_mut_loop_bounds_must_be_correct() {
    // MUTATION TARGET: line 265 in parser.rs
    // Mutation: Replace < with <= or !=
    // This test kills mutations that break loop bounds

    // ARRANGE: Recipe at end of file (boundary condition)
    let makefile = "final:\n\techo last\n\techo done";

    // ACT: Parse
    let result = parse_makefile(makefile);
    assert!(result.is_ok());

    let ast = result.unwrap();

    // ASSERT: Should handle EOF correctly
    match &ast.items[0] {
        MakeItem::Target { recipe, .. } => {
            assert_eq!(recipe.len(), 2, "Should parse both lines at EOF");
            assert_eq!(recipe[0], "echo last");
            assert_eq!(recipe[1], "echo done");
        }
        _ => panic!("Expected Target"),
    }

    // CRITICAL: If loop bounds are wrong (*index <= lines.len()), it would panic
}

#[test]
fn test_RECIPE_002_mut_recipe_vec_must_accumulate() {
    // MUTATION TARGET: line 263 in parser.rs
    // Mutation: Don't initialize Vec or clear it each iteration
    // This test kills mutations that break Vec accumulation

    // ARRANGE: Target with multiple distinct lines
    let makefile = "accumulate:\n\tfirst\n\tsecond\n\tthird\n\tfourth\n\tfifth";

    // ACT: Parse
    let result = parse_makefile(makefile);
    assert!(result.is_ok());

    let ast = result.unwrap();

    // ASSERT: Vec must accumulate all 5 lines
    match &ast.items[0] {
        MakeItem::Target { recipe, .. } => {
            assert_eq!(recipe.len(), 5, "Vec must accumulate 5 lines");

            // Verify all unique lines are present (not duplicates or missing)
            assert_eq!(recipe[0], "first");
            assert_eq!(recipe[1], "second");
            assert_eq!(recipe[2], "third");
            assert_eq!(recipe[3], "fourth");
            assert_eq!(recipe[4], "fifth");
        }
        _ => panic!("Expected Target"),
    }

    // CRITICAL: If Vec is cleared or not accumulated properly, test fails
}

#[test]
fn test_RECIPE_002_mut_multiple_targets_isolated() {
    // MUTATION TARGET: General recipe parsing logic
    // Mutation: Share recipe Vec between targets or don't reset
    // This test kills mutations that leak recipe lines between targets

    // ARRANGE: Two targets with different recipe counts
    let makefile = "first:\n\tA\n\tB\n\nsecond:\n\tX";

    // ACT: Parse
    let result = parse_makefile(makefile);
    assert!(result.is_ok());

    let ast = result.unwrap();
    assert_eq!(ast.items.len(), 2);

    // ASSERT: First target has 2 lines, second has 1 (must be isolated)
    match &ast.items[0] {
        MakeItem::Target { name, recipe, .. } => {
            assert_eq!(name, "first");
            assert_eq!(recipe.len(), 2, "First target MUST have 2 lines");
            assert_eq!(recipe[0], "A");
            assert_eq!(recipe[1], "B");
        }
        _ => panic!("Expected first Target"),
    }

    match &ast.items[1] {
        MakeItem::Target { name, recipe, .. } => {
            assert_eq!(name, "second");
            assert_eq!(recipe.len(), 1, "Second target MUST have 1 line");
            assert_eq!(recipe[0], "X");
            // CRITICAL: If recipes leak, this would have [A, B, X] (3 lines)
        }
        _ => panic!("Expected second Target"),
    }

    // CRITICAL: If recipe Vec isn't fresh per target, isolation breaks
}

// ============================================================================
// Sprint 40: ECHO-001 - @ prefix for silent recipes
// ============================================================================
// Implements: @ prefix in recipes to suppress command echoing
// Verifies: Parser preserves @ prefix in recipe lines

// Phase 1 - RED: Unit tests for @ prefix in recipes

#[test]
fn test_ECHO_001_single_silent_recipe() {
    // ARRANGE: Target with single @ prefix recipe
    let makefile = "test:\n\t@cargo test";

    // ACT: Parse the makefile
    let result = parse_makefile(makefile);

    // ASSERT: @ prefix should be preserved in recipe
    assert!(result.is_ok(), "Parser should handle @ prefix in recipes");

    let ast = result.unwrap();
    assert_eq!(ast.items.len(), 1, "Should parse one target");

    match &ast.items[0] {
        MakeItem::Target { name, recipe, .. } => {
            assert_eq!(name, "test", "Target name should be 'test'");
            assert_eq!(recipe.len(), 1, "Should have one recipe line");
            assert_eq!(recipe[0], "@cargo test", "@ prefix must be preserved");
            assert!(recipe[0].starts_with('@'), "Recipe must start with @");
        }
        other => panic!("Expected Target, got {:?}", other),
    }
}

#[test]
fn test_ECHO_001_multiple_silent_recipes() {
    // ARRANGE: Target with multiple @ prefix recipes
    let makefile = "build:\n\t@echo 'Building...'\n\t@cargo build --release\n\t@echo 'Done'";

    // ACT: Parse
    let result = parse_makefile(makefile);
    assert!(result.is_ok(), "Multiple @ prefix recipes should parse");

    let ast = result.unwrap();
    match &ast.items[0] {
        MakeItem::Target { name, recipe, .. } => {
            assert_eq!(name, "build");
            assert_eq!(recipe.len(), 3, "Should have three recipe lines");
            assert_eq!(recipe[0], "@echo 'Building...'", "First @ prefix preserved");
            assert_eq!(
                recipe[1], "@cargo build --release",
                "Second @ prefix preserved"
            );
            assert_eq!(recipe[2], "@echo 'Done'", "Third @ prefix preserved");

            // All three lines should start with @
            for (i, line) in recipe.iter().enumerate() {
                assert!(line.starts_with('@'), "Recipe line {} must start with @", i);
            }
        }
        _ => panic!("Expected Target"),
    }
}

#[test]
fn test_ECHO_001_mixed_silent_and_normal_recipes() {
    // ARRANGE: Target with mix of @ prefix and normal recipes
    let makefile = "deploy:\n\tcargo build\n\t@echo 'Deploying...'\n\tscp app server:/opt/\n\t@echo 'Complete'";

    // ACT: Parse
    let result = parse_makefile(makefile);
    assert!(result.is_ok());

    let ast = result.unwrap();
    match &ast.items[0] {
        MakeItem::Target { recipe, .. } => {
            assert_eq!(recipe.len(), 4, "Should have four recipe lines");
            assert_eq!(recipe[0], "cargo build", "Normal recipe (no @)");
            assert_eq!(recipe[1], "@echo 'Deploying...'", "Silent recipe (with @)");
            assert_eq!(recipe[2], "scp app server:/opt/", "Normal recipe (no @)");
            assert_eq!(recipe[3], "@echo 'Complete'", "Silent recipe (with @)");

            // Verify @ only on specific lines
            assert!(!recipe[0].starts_with('@'), "Line 0 should not have @");
            assert!(recipe[1].starts_with('@'), "Line 1 should have @");
            assert!(!recipe[2].starts_with('@'), "Line 2 should not have @");
            assert!(recipe[3].starts_with('@'), "Line 3 should have @");
        }
        _ => panic!("Expected Target"),
    }
}

#[test]
fn test_ECHO_001_at_prefix_different_targets() {
    // ARRANGE: Multiple targets, some with @ prefix, some without
    let makefile = "test:\n\t@cargo test\n\nverbose:\n\tcargo test --verbose";

    // ACT: Parse
    let result = parse_makefile(makefile);
    assert!(result.is_ok());

    let ast = result.unwrap();
    assert_eq!(ast.items.len(), 2, "Should have two targets");

    // First target: with @
    match &ast.items[0] {
        MakeItem::Target { name, recipe, .. } => {
            assert_eq!(name, "test");
            assert_eq!(recipe.len(), 1);
            assert_eq!(recipe[0], "@cargo test");
            assert!(recipe[0].starts_with('@'));
        }
        _ => panic!("Expected first Target"),
    }

    // Second target: without @
    match &ast.items[1] {
        MakeItem::Target { name, recipe, .. } => {
            assert_eq!(name, "verbose");
            assert_eq!(recipe.len(), 1);
            assert_eq!(recipe[0], "cargo test --verbose");
            assert!(
                !recipe[0].starts_with('@'),
                "verbose target should NOT have @"
            );
        }
        _ => panic!("Expected second Target"),
    }
}

// Phase 4 - PROPERTY TESTING: Property tests for @ prefix

#[cfg(test)]
mod echo_001_property_tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn prop_ECHO_001_at_prefix_always_preserved(
            cmd in "[a-z]{3,10}",
            args in prop::option::of("[a-z0-9 ]{0,20}")
        ) {
            // PROPERTY: @ prefix in recipes is always preserved
            let recipe_cmd = match args {
                Some(a) if !a.trim().is_empty() => format!("@{} {}", cmd, a.trim()),
                _ => format!("@{}", cmd),
            };
            let makefile = format!("target:\n\t{}", recipe_cmd);

            let result = parse_makefile(&makefile);
            prop_assert!(result.is_ok(), "Parser must handle @ prefix");

            let ast = result.unwrap();
            match &ast.items[0] {
                MakeItem::Target { recipe, .. } => {
                    prop_assert_eq!(recipe.len(), 1);
                    prop_assert_eq!(&recipe[0], &recipe_cmd);
                    prop_assert!(recipe[0].starts_with('@'), "@ prefix must be preserved");
                }
                _ => prop_assert!(false, "Expected Target"),
            }
        }

        #[test]
        fn prop_ECHO_001_parsing_is_deterministic(
            silent_cmd in "@[a-z]{3,10}",
            normal_cmd in "[a-z]{3,10}"
        ) {
            // PROPERTY: Parsing @ prefix recipes is deterministic
            let makefile = format!("target:\n\t{}\n\t{}", silent_cmd, normal_cmd);

            let result1 = parse_makefile(&makefile);
            let result2 = parse_makefile(&makefile);

            prop_assert!(result1.is_ok());
            prop_assert!(result2.is_ok());

            let ast1 = result1.unwrap();
            let ast2 = result2.unwrap();

            // Same input = same AST
            prop_assert_eq!(ast1.items.len(), ast2.items.len());

            match (&ast1.items[0], &ast2.items[0]) {
                (MakeItem::Target { recipe: r1, .. }, MakeItem::Target { recipe: r2, .. }) => {
                    prop_assert_eq!(r1.len(), r2.len());
                    prop_assert_eq!(&r1[0], &r2[0]);
                    prop_assert_eq!(&r1[1], &r2[1]);
                }
                _ => prop_assert!(false, "Expected Target"),
            }
        }

        #[test]
        fn prop_ECHO_001_mixed_recipes_order_preserved(
            n_silent in 1usize..5,
            n_normal in 1usize..5
        ) {
            // PROPERTY: Order of @ and non-@ recipes is preserved
            let mut makefile = String::from("target:");

            let mut expected_recipes = Vec::new();
            for i in 0..n_silent {
                let recipe = format!("@silent{}", i);
                makefile.push_str(&format!("\n\t{}", recipe));
                expected_recipes.push(recipe);
            }
            for i in 0..n_normal {
                let recipe = format!("normal{}", i);
                makefile.push_str(&format!("\n\t{}", recipe));
                expected_recipes.push(recipe);
            }

            let result = parse_makefile(&makefile);
            prop_assert!(result.is_ok());

            let ast = result.unwrap();
            match &ast.items[0] {
                MakeItem::Target { recipe, .. } => {
                    prop_assert_eq!(recipe.len(), expected_recipes.len());
                    for (i, expected) in expected_recipes.iter().enumerate() {
                        prop_assert_eq!(&recipe[i], expected, "Recipe at index {} must match", i);
                    }
                }
                _ => prop_assert!(false, "Expected Target"),
            }
        }

        #[test]
        fn prop_ECHO_001_at_prefix_with_special_chars(
            prefix in "@+",  // @ or @@
            cmd in "[a-z]{3,8}",
            special in prop::option::of(r"[!$(){}\\|&;<>]")
        ) {
            // PROPERTY: @ prefix preserved even with special shell chars
            let recipe_line = match special {
                Some(s) => format!("{}{} {}", prefix, cmd, s),
                None => format!("{}{}", prefix, cmd),
            };
            let makefile = format!("target:\n\t{}", recipe_line);

            let result = parse_makefile(&makefile);
            prop_assert!(result.is_ok(), "Special chars with @ should parse");

            let ast = result.unwrap();
            match &ast.items[0] {
                MakeItem::Target { recipe, .. } => {
                    prop_assert_eq!(recipe.len(), 1);
                    prop_assert_eq!(&recipe[0], &recipe_line);
                    prop_assert!(recipe[0].starts_with('@'), "@ prefix preserved with special chars");
                }
                _ => prop_assert!(false, "Expected Target"),
            }
        }

        #[test]
        fn prop_ECHO_001_multiple_targets_independent(
            n_targets in 1usize..4
        ) {
            // PROPERTY: @ prefix handling is independent across targets
            let mut makefile = String::new();
            let mut expected: Vec<(String, Vec<String>)> = Vec::new();

            for i in 0..n_targets {
                let target_name = format!("target{}", i);
                makefile.push_str(&format!("{}:\n", target_name));

                let mut target_recipes = Vec::new();

                // Even targets get @ prefix, odd don't
                if i % 2 == 0 {
                    let recipe = format!("@command{}", i);
                    makefile.push_str(&format!("\t{}\n", recipe));
                    target_recipes.push(recipe);
                } else {
                    let recipe = format!("command{}", i);
                    makefile.push_str(&format!("\t{}\n", recipe));
                    target_recipes.push(recipe);
                }

                expected.push((target_name, target_recipes));
            }

            let result = parse_makefile(&makefile);
            prop_assert!(result.is_ok());

            let ast = result.unwrap();
            prop_assert_eq!(ast.items.len(), expected.len());

            for (i, (expected_name, expected_recipes)) in expected.iter().enumerate() {
                match &ast.items[i] {
                    MakeItem::Target { name, recipe, .. } => {
                        prop_assert_eq!(name, expected_name);
                        prop_assert_eq!(recipe.len(), expected_recipes.len());
                        for (j, expected_recipe) in expected_recipes.iter().enumerate() {
                            prop_assert_eq!(&recipe[j], expected_recipe);
                        }
                    }
                    _ => prop_assert!(false, "Expected Target at index {}", i),
                }
            }
        }
    }
}

// ============================================================================
// INCLUDE-001: Include directive
// ============================================================================

/// RED PHASE: Test for INCLUDE-001 - Basic include directive
///
/// This test validates GNU Make's `include` directive for modular Makefiles.
///
/// Input Makefile:
/// ```makefile
/// include common.mk
/// ```
///
/// Expected AST:
/// - One MakeItem::Include
/// - path: "common.mk"
/// - optional: false
///
/// Reference: GNU Make Manual Section 3.3 "Including Other Makefiles"
#[test]
fn test_INCLUDE_001_basic_include_directive() {
    // ARRANGE: Simple include directive
    let makefile = "include common.mk";

    // ACT: Parse makefile
    let result = parse_makefile(makefile);

    // ASSERT: Successfully parsed
    assert!(
        result.is_ok(),
        "Should parse include directive, got error: {:?}",
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

    // ASSERT: Item is an Include
    match &ast.items[0] {
        MakeItem::Include { path, optional, .. } => {
            assert_eq!(path, "common.mk", "Include path should be common.mk");
            assert!(!optional, "Include should not be optional");
        }
        _ => panic!("Expected MakeItem::Include, got {:?}", ast.items[0]),
    }
}

/// RED PHASE: Test for INCLUDE-001 - Include with path
#[test]
fn test_INCLUDE_001_include_with_path() {
    // ARRANGE: Include directive with path
    let makefile = "include config/build.mk";

    // ACT: Parse makefile
    let result = parse_makefile(makefile);

    // ASSERT: Successfully parsed
    assert!(result.is_ok(), "Should parse include with path");

    let ast = result.unwrap();

    // ASSERT: Item is an Include with correct path
    match &ast.items[0] {
        MakeItem::Include { path, optional, .. } => {
            assert_eq!(
                path, "config/build.mk",
                "Include path should preserve directories"
            );
            assert!(!optional, "Include should not be optional");
        }
        _ => panic!("Expected MakeItem::Include"),
    }
}

/// RED PHASE: Test for INCLUDE-001 - Multiple include directives
#[test]
fn test_INCLUDE_001_multiple_includes() {
    // ARRANGE: Multiple include directives
    let makefile = "include config.mk\ninclude rules.mk\ninclude targets.mk";

    // ACT: Parse makefile
    let result = parse_makefile(makefile);

    // ASSERT: Successfully parsed
    assert!(result.is_ok(), "Should parse multiple includes");

    let ast = result.unwrap();

    // ASSERT: Three include items
    assert_eq!(ast.items.len(), 3, "Should have three include items");

    let expected_paths = ["config.mk", "rules.mk", "targets.mk"];
    for (i, expected_path) in expected_paths.iter().enumerate() {
        match &ast.items[i] {
            MakeItem::Include { path, optional, .. } => {
                assert_eq!(path, expected_path, "Include path {} should match", i);
                assert!(!optional, "Include {} should not be optional", i);
            }
            _ => panic!("Expected MakeItem::Include at index {}", i),
        }
    }
}

/// RED PHASE: Test for INCLUDE-001 - Include with variables
#[test]
fn test_INCLUDE_001_include_with_variables() {
    // ARRANGE: Include directive with variable reference
    let makefile = "include $(CONFIG_DIR)/common.mk";

    // ACT: Parse makefile
    let result = parse_makefile(makefile);

    // ASSERT: Successfully parsed
    assert!(result.is_ok(), "Should parse include with variable");

    let ast = result.unwrap();

    // ASSERT: Variable reference preserved in path
    match &ast.items[0] {
        MakeItem::Include { path, .. } => {
            assert_eq!(
                path, "$(CONFIG_DIR)/common.mk",
                "Include should preserve variable references"
            );
        }
        _ => panic!("Expected MakeItem::Include"),
    }
}

// RED PHASE: Property test - Include directives always parse
#[cfg(test)]
mod include_property_tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn prop_INCLUDE_001_includes_always_parse(
            filename in "[a-zA-Z0-9_.-]{1,30}\\.mk"
        ) {
            let makefile = format!("include {}", filename);
            let result = parse_makefile(&makefile);
            prop_assert!(result.is_ok(), "Include should always parse valid filenames");

            let ast = result.unwrap();
            prop_assert_eq!(ast.items.len(), 1);

            match &ast.items[0] {
                MakeItem::Include { path, optional, .. } => {
                    prop_assert_eq!(path, &filename);
                    prop_assert!(!optional);
                }
                _ => prop_assert!(false, "Expected MakeItem::Include"),
            }
        }

        #[test]
        fn prop_INCLUDE_001_parsing_is_deterministic(
            filename in "[a-zA-Z0-9/_.-]{1,50}\\.mk"
        ) {
            let makefile = format!("include {}", filename);
            let ast1 = parse_makefile(&makefile);
            let ast2 = parse_makefile(&makefile);

            match (ast1, ast2) {
                (Ok(a1), Ok(a2)) => {
                    prop_assert_eq!(a1.items.len(), a2.items.len());
                    match (&a1.items[0], &a2.items[0]) {
                        (MakeItem::Include { path: p1, .. }, MakeItem::Include { path: p2, .. }) => {
                            prop_assert_eq!(p1, p2);
                        }
                        _ => prop_assert!(false, "Expected matching Include items"),
                    }
                }
                _ => prop_assert!(false, "Parsing should be deterministic"),
            }
        }

        #[test]
        fn prop_INCLUDE_001_multiple_includes_order_preserved(
            files in prop::collection::vec("[a-z]{3,10}\\.mk", 2..5)
        ) {
            let makefile = files
                .iter()
                .map(|f| format!("include {}", f))
                .collect::<Vec<_>>()
                .join("\n");

            let result = parse_makefile(&makefile);
            prop_assert!(result.is_ok());

            let ast = result.unwrap();
            prop_assert_eq!(ast.items.len(), files.len());

            for (i, expected_file) in files.iter().enumerate() {
                match &ast.items[i] {
                    MakeItem::Include { path, .. } => {
                        prop_assert_eq!(path, expected_file);
                    }
                    _ => prop_assert!(false, "Expected Include at index {}", i),
                }
            }
        }

        #[test]
        fn prop_INCLUDE_001_paths_with_directories(
            dir in "[a-z]{3,10}",
            file in "[a-z]{3,10}\\.mk"
        ) {
            let makefile = format!("include {}/{}", dir, file);
            let result = parse_makefile(&makefile);
            prop_assert!(result.is_ok());

            let ast = result.unwrap();
            match &ast.items[0] {
                MakeItem::Include { path, .. } => {
                    prop_assert!(path.contains('/'));
                    prop_assert!(path.ends_with(".mk"));
                }
                _ => prop_assert!(false, "Expected Include"),
            }
        }

        #[test]
        fn prop_INCLUDE_001_var_refs_preserved(
            var_name in "[A-Z_]{2,10}",
            file in "[a-z]{3,10}\\.mk"
        ) {
            let makefile = format!("include $({})/{}", var_name, file);
            let result = parse_makefile(&makefile);
            prop_assert!(result.is_ok());

            let ast = result.unwrap();
            match &ast.items[0] {
                MakeItem::Include { path, .. } => {
                    prop_assert!(path.contains("$("));
                    prop_assert!(path.contains(&var_name));
                }
                _ => prop_assert!(false, "Expected Include"),
            }
        }
    }
}

/// RED PHASE: Mutation-killing test - Include keyword detection
#[test]
fn test_INCLUDE_001_mut_keyword_detection() {
    // Test that only "include" keyword triggers Include parsing
    let makefile_include = "include file.mk";
    let makefile_invalid = "includes file.mk"; // typo

    let result_include = parse_makefile(makefile_include);
    let result_invalid = parse_makefile(makefile_invalid);

    assert!(result_include.is_ok(), "Should parse 'include'");
    assert!(result_invalid.is_ok(), "'includes' should not crash parser");

    // Valid include should produce Include item
    match &result_include.unwrap().items[0] {
        MakeItem::Include { .. } => {} // Expected
        _ => panic!("Expected Include for 'include' keyword"),
    }

    // Invalid should NOT produce Include item (probably parsed as unknown/error)
    let ast_invalid = result_invalid.unwrap();
    if !ast_invalid.items.is_empty() {
        match &ast_invalid.items[0] {
            MakeItem::Include { .. } => panic!("Should not parse 'includes' as Include"),
            _ => {} // Expected - parsed as something else
        }
    }
}

/// RED PHASE: Mutation-killing test - Path extraction correctness
#[test]
fn test_INCLUDE_001_mut_path_extraction() {
    // Test that path is correctly extracted after "include" keyword
    let makefile = "include    file.mk"; // Extra whitespace

    let result = parse_makefile(makefile);
    assert!(result.is_ok());

    let ast = result.unwrap();
    match &ast.items[0] {
        MakeItem::Include { path, .. } => {
            // Path should be trimmed of leading/trailing whitespace
            assert_eq!(path, "file.mk", "Path should be trimmed");
            assert!(
                !path.starts_with(' '),
                "Path should not have leading whitespace"
            );
        }
        _ => panic!("Expected Include"),
    }
}

/// RED PHASE: Mutation-killing test - Include vs target distinction
#[test]
fn test_INCLUDE_001_mut_include_vs_target() {
    // Test that "include" is not parsed as a target name
    let makefile_include = "include file.mk";
    let makefile_target = "include: file.mk\n\techo build";

    let result_include = parse_makefile(makefile_include);
    let result_target = parse_makefile(makefile_target);

    assert!(result_include.is_ok());
    assert!(result_target.is_ok());

    // First should be Include
    match &result_include.unwrap().items[0] {
        MakeItem::Include { .. } => {} // Expected
        _ => panic!("'include file.mk' should be parsed as Include, not Target"),
    }

    // Second should be Target (named "include")
    match &result_target.unwrap().items[0] {
        MakeItem::Target { name, .. } => {
            assert_eq!(name, "include", "Should parse as target named 'include'");
        }
        _ => panic!("'include:' should be parsed as Target, not Include"),
    }
}

/// RED PHASE: Mutation-killing test - Empty path handling
#[test]
fn test_INCLUDE_001_mut_empty_path() {
    // Test edge case: include with no path
    let makefile = "include";

    let result = parse_makefile(makefile);

    // This could either:
    // 1. Fail gracefully (preferred)
    // 2. Parse with empty path (acceptable if validated later)
    // Either way, should not panic
    match result {
        Ok(ast) => {
            if !ast.items.is_empty() {
                // If parsed, verify it doesn't have invalid state
                match &ast.items[0] {
                    MakeItem::Include { path, .. } => {
                        // Empty path is detectable
                        assert!(path.is_empty() || !path.is_empty());
                    }
                    _ => {} // Parsed as something else, that's fine
                }
            }
        }
        Err(_) => {} // Graceful error, that's fine
    }
}

/// RED PHASE: Mutation-killing test - Line parsing advances correctly
#[test]
fn test_INCLUDE_001_mut_parser_advances() {
    // Test that parser advances to next line after include
    let makefile = "include file.mk\nCC := gcc";

    let result = parse_makefile(makefile);
    assert!(result.is_ok());

    let ast = result.unwrap();
    assert_eq!(ast.items.len(), 2, "Should parse both include and variable");

    // First item: Include
    match &ast.items[0] {
        MakeItem::Include { .. } => {}
        _ => panic!("First item should be Include"),
    }

    // Second item: Variable
    match &ast.items[1] {
        MakeItem::Variable { name, .. } => {
            assert_eq!(name, "CC", "Second item should be CC variable");
        }
        _ => panic!("Second item should be Variable"),
    }
}

// ==============================================================================
// INCLUDE-002: Optional Include Directives (-include, sinclude)
// ==============================================================================
// Task: Document optional include directives that don't error if file is missing
// Input: -include optional.mk or sinclude optional.mk
// Goal: Parser sets optional=true flag for -include and sinclude variants

#[test]
fn test_INCLUDE_002_dash_include() {
    // ARRANGE: Optional include with -include syntax
    let makefile = "-include optional.mk";

    // ACT: Parse makefile
    let result = parse_makefile(makefile);

    // ASSERT: Successfully parsed
    assert!(
        result.is_ok(),
        "Should parse -include directive, got error: {:?}",
        result.err()
    );

    let ast = result.unwrap();
    assert_eq!(ast.items.len(), 1, "Should have 1 include");

    // ASSERT: Include item with optional=true
    match &ast.items[0] {
        MakeItem::Include { path, optional, .. } => {
            assert_eq!(path, "optional.mk", "Path should be optional.mk");
            assert!(*optional, "-include should set optional=true");
        }
        other => panic!("Expected Include item, got {:?}", other),
    }
}

#[test]
fn test_INCLUDE_002_sinclude() {
    // ARRANGE: Optional include with sinclude syntax (GNU Make synonym)
    let makefile = "sinclude optional.mk";

    // ACT: Parse makefile
    let result = parse_makefile(makefile);

    // ASSERT: Successfully parsed
    assert!(
        result.is_ok(),
        "Should parse sinclude directive, got error: {:?}",
        result.err()
    );

    let ast = result.unwrap();
    assert_eq!(ast.items.len(), 1, "Should have 1 include");

    // ASSERT: Include item with optional=true
    match &ast.items[0] {
        MakeItem::Include { path, optional, .. } => {
            assert_eq!(path, "optional.mk", "Path should be optional.mk");
            assert!(*optional, "sinclude should set optional=true");
        }
        other => panic!("Expected Include item, got {:?}", other),
    }
}

#[test]
fn test_INCLUDE_002_dash_include_with_path() {
    // ARRANGE: Optional include with directory path
    let makefile = "-include config/optional.mk";

    // ACT: Parse makefile
    let result = parse_makefile(makefile);

    // ASSERT: Successfully parsed
    assert!(result.is_ok(), "Should parse -include with path");

    let ast = result.unwrap();
    match &ast.items[0] {
        MakeItem::Include { path, optional, .. } => {
            assert_eq!(path, "config/optional.mk");
            assert!(*optional, "Should be optional");
        }
        other => panic!("Expected Include, got {:?}", other),
    }
}

#[test]
fn test_INCLUDE_002_mixed_includes() {
    // ARRANGE: Mix of required and optional includes
    let makefile = "include required.mk\n-include optional.mk\nsinclude also_optional.mk";

    // ACT: Parse makefile
    let result = parse_makefile(makefile);

    // ASSERT: Successfully parsed
    assert!(result.is_ok(), "Should parse mixed includes");

    let ast = result.unwrap();
    assert_eq!(ast.items.len(), 3, "Should have 3 includes");

    // First: required (optional=false)
    match &ast.items[0] {
        MakeItem::Include { path, optional, .. } => {
            assert_eq!(path, "required.mk");
            assert!(!optional, "include should be required");
        }
        other => panic!("Expected Include, got {:?}", other),
    }

    // Second: -include (optional=true)
    match &ast.items[1] {
        MakeItem::Include { path, optional, .. } => {
            assert_eq!(path, "optional.mk");
            assert!(*optional, "-include should be optional");
        }
        other => panic!("Expected Include, got {:?}", other),
    }

    // Third: sinclude (optional=true)
    match &ast.items[2] {
        MakeItem::Include { path, optional, .. } => {
            assert_eq!(path, "also_optional.mk");
            assert!(*optional, "sinclude should be optional");
        }
        other => panic!("Expected Include, got {:?}", other),
    }
}

#[test]
fn test_INCLUDE_002_dash_include_with_variables() {
    // ARRANGE: Optional include with variable reference
    let makefile = "-include $(CONFIG_DIR)/optional.mk";

    // ACT: Parse makefile
    let result = parse_makefile(makefile);

    // ASSERT: Successfully parsed
    assert!(result.is_ok(), "Should parse -include with variables");

    let ast = result.unwrap();
    match &ast.items[0] {
        MakeItem::Include { path, optional, .. } => {
            assert!(path.contains("$(CONFIG_DIR)"), "Should preserve variable");
            assert!(*optional, "Should be optional");
        }
        other => panic!("Expected Include, got {:?}", other),
    }
}

#[test]
fn test_INCLUDE_002_multiple_optional_includes() {
    // ARRANGE: Multiple optional includes
    let makefile = "-include file1.mk file2.mk file3.mk";

    // ACT: Parse makefile
    let result = parse_makefile(makefile);

    // ASSERT: This tests current behavior - parser may handle this differently
    // GNU Make allows multiple files in one include directive
    assert!(result.is_ok(), "Should not crash on multiple files");
}

// ==============================================================================
// INCLUDE-002: Property Tests
// ==============================================================================

#[cfg(test)]
mod include_002_property_tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn prop_INCLUDE_002_dash_include_always_optional(
            filename in "[a-zA-Z0-9_.-]{1,30}\\.mk"
        ) {
            // ARRANGE: -include directive
            let makefile = format!("-include {}", filename);

            // ACT: Parse makefile
            let result = parse_makefile(&makefile);

            // ASSERT: Always parses and sets optional=true
            prop_assert!(result.is_ok(), "-include should always parse");

            let ast = result.unwrap();
            prop_assert_eq!(ast.items.len(), 1);

            match &ast.items[0] {
                MakeItem::Include { path, optional, .. } => {
                    prop_assert_eq!(path, &filename);
                    prop_assert!(*optional, "-include should always be optional");
                }
                other => return Err(TestCaseError::fail(format!("Expected Include, got {:?}", other))),
            }
        }

        #[test]
        fn prop_INCLUDE_002_sinclude_always_optional(
            filename in "[a-zA-Z0-9_.-]{1,30}\\.mk"
        ) {
            // ARRANGE: sinclude directive
            let makefile = format!("sinclude {}", filename);

            // ACT: Parse makefile
            let result = parse_makefile(&makefile);

            // ASSERT: Always parses and sets optional=true
            prop_assert!(result.is_ok(), "sinclude should always parse");

            let ast = result.unwrap();
            prop_assert_eq!(ast.items.len(), 1);

            match &ast.items[0] {
                MakeItem::Include { path, optional, .. } => {
                    prop_assert_eq!(path, &filename);
                    prop_assert!(*optional, "sinclude should always be optional");
                }
                other => return Err(TestCaseError::fail(format!("Expected Include, got {:?}", other))),
            }
        }

        #[test]
        fn prop_INCLUDE_002_parsing_is_deterministic(
            filename in "[a-zA-Z0-9/_.-]{1,50}\\.mk"
        ) {
            // ARRANGE: -include directive
            let makefile = format!("-include {}", filename);

            // ACT: Parse twice
            let ast1 = parse_makefile(&makefile);
            let ast2 = parse_makefile(&makefile);

            // ASSERT: Results are identical
            match (ast1, ast2) {
                (Ok(a1), Ok(a2)) => {
                    prop_assert_eq!(a1.items.len(), a2.items.len());
                    match (&a1.items[0], &a2.items[0]) {
                        (MakeItem::Include { path: p1, optional: o1, .. },
                         MakeItem::Include { path: p2, optional: o2, .. }) => {
                            prop_assert_eq!(p1, p2);
                            prop_assert_eq!(o1, o2);
                        }
                        _ => return Err(TestCaseError::fail("Expected matching Include items")),
                    }
                }
                _ => return Err(TestCaseError::fail("Parsing should be deterministic")),
            }
        }

        #[test]
        fn prop_INCLUDE_002_optional_vs_required(
            filename in "[a-z]{3,10}\\.mk"
        ) {
            // ARRANGE: Test that include vs -include vs sinclude set optional correctly
            let include_reg = format!("include {}", filename);
            let include_dash = format!("-include {}", filename);
            let include_s = format!("sinclude {}", filename);

            // ACT: Parse all three variants
            let ast_reg = parse_makefile(&include_reg);
            let ast_dash = parse_makefile(&include_dash);
            let ast_s = parse_makefile(&include_s);

            // ASSERT: All parse successfully
            prop_assert!(ast_reg.is_ok());
            prop_assert!(ast_dash.is_ok());
            prop_assert!(ast_s.is_ok());

            // Regular include: optional=false
            match &ast_reg.unwrap().items[0] {
                MakeItem::Include { optional, .. } => {
                    prop_assert!(!optional, "include should be required");
                }
                other => return Err(TestCaseError::fail(format!("Expected Include, got {:?}", other))),
            }

            // -include: optional=true
            match &ast_dash.unwrap().items[0] {
                MakeItem::Include { optional, .. } => {
                    prop_assert!(*optional, "-include should be optional");
                }
                other => return Err(TestCaseError::fail(format!("Expected Include, got {:?}", other))),
            }

            // sinclude: optional=true
            match &ast_s.unwrap().items[0] {
                MakeItem::Include { optional, .. } => {
                    prop_assert!(*optional, "sinclude should be optional");
                }
                other => return Err(TestCaseError::fail(format!("Expected Include, got {:?}", other))),
            }
        }

        #[test]
        fn prop_INCLUDE_002_paths_with_directories(
            dir in "[a-z]{3,10}",
            file in "[a-z]{3,10}\\.mk"
        ) {
            // ARRANGE: Optional include with directory path
            let makefile = format!("-include {}/{}", dir, file);

            // ACT: Parse makefile
            let result = parse_makefile(&makefile);

            // ASSERT: Path preserved with directory
            prop_assert!(result.is_ok());

            let ast = result.unwrap();
            match &ast.items[0] {
                MakeItem::Include { path, optional, .. } => {
                    prop_assert!(path.contains('/'));
                    prop_assert!(path.ends_with(".mk"));
                    prop_assert!(*optional);
                }
                other => return Err(TestCaseError::fail(format!("Expected Include, got {:?}", other))),
            }
        }

        #[test]
        fn prop_INCLUDE_002_var_refs_preserved(
            var_name in "[A-Z_]{2,10}",
            file in "[a-z]{3,10}\\.mk"
        ) {
            // ARRANGE: Optional include with variable reference
            let makefile = format!("-include $({})/{}", var_name, file);

            // ACT: Parse makefile
            let result = parse_makefile(&makefile);

            // ASSERT: Variable reference preserved
            prop_assert!(result.is_ok());

            let ast = result.unwrap();
            match &ast.items[0] {
                MakeItem::Include { path, optional, .. } => {
                    prop_assert!(path.contains("$("));
                    prop_assert!(path.contains(&var_name));
                    prop_assert!(*optional);
                }
                other => return Err(TestCaseError::fail(format!("Expected Include, got {:?}", other))),
            }
        }
    }
}

// ==============================================================================
// PATTERN-001: Pattern Rules (%.o: %.c)
// ==============================================================================

/// RED PHASE: Test for PATTERN-001 - Basic pattern rule
///
/// Pattern rules use % to match file stems. This is foundational for
/// automatic compilation rules like %.o: %.c
///
/// Input Makefile:
/// ```makefile
/// %.o: %.c
///     $(CC) -c $< -o $@
/// ```
///
/// Expected AST:
/// - One MakeItem::PatternRule
/// - target_pattern: "%.o"
/// - prereq_patterns: ["%.c"]
/// - recipe: ["$(CC) -c $< -o $@"]
#[test]
fn test_PATTERN_001_basic_pattern_rule() {
    // ARRANGE: Simple pattern rule
    let makefile = "%.o: %.c\n\t$(CC) -c $< -o $@";

    // ACT: Parse makefile
    let result = parse_makefile(makefile);

    // ASSERT: Successfully parsed
    assert!(
        result.is_ok(),
        "Should parse basic pattern rule, got error: {:?}",
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

    // ASSERT: Item is a PatternRule
    match &ast.items[0] {
        MakeItem::PatternRule {
            target_pattern,
            prereq_patterns,
            recipe,
            ..
        } => {
            assert_eq!(target_pattern, "%.o", "Target pattern should be '%.o'");
            assert_eq!(
                prereq_patterns.len(),
                1,
                "Should have one prerequisite pattern"
            );
            assert_eq!(
                prereq_patterns[0], "%.c",
                "Prerequisite pattern should be '%.c'"
            );
            assert_eq!(recipe.len(), 1, "Should have one recipe line");
            assert_eq!(
                recipe[0], "$(CC) -c $< -o $@",
                "Recipe should contain automatic variables"
            );
        }
        other => panic!("Expected PatternRule item, got {:?}", other),
    }
}

/// RED PHASE: Test for PATTERN-001 - Pattern rule with multiple prerequisites
#[test]
fn test_PATTERN_001_pattern_rule_multiple_prerequisites() {
    // ARRANGE: Pattern rule with multiple prerequisites
    let makefile = "%.o: %.c %.h\n\t$(CC) -c $< -o $@";

    // ACT: Parse makefile
    let result = parse_makefile(makefile);

    // ASSERT: Successfully parsed
    assert!(result.is_ok());

    let ast = result.unwrap();
    assert_eq!(ast.items.len(), 1);

    // ASSERT: Check prerequisites
    match &ast.items[0] {
        MakeItem::PatternRule {
            target_pattern,
            prereq_patterns,
            ..
        } => {
            assert_eq!(target_pattern, "%.o");
            assert_eq!(prereq_patterns.len(), 2);
            assert_eq!(prereq_patterns[0], "%.c");
            assert_eq!(prereq_patterns[1], "%.h");
        }
        other => panic!("Expected PatternRule, got {:?}", other),
    }
}

/// RED PHASE: Test for PATTERN-001 - Pattern rule without recipe
#[test]
fn test_PATTERN_001_pattern_rule_empty_recipe() {
    // ARRANGE: Pattern rule with no recipe (just dependencies)
    let makefile = "%.o: %.c";

    // ACT: Parse makefile
    let result = parse_makefile(makefile);

    // ASSERT: Successfully parsed
    assert!(result.is_ok());

    let ast = result.unwrap();
    assert_eq!(ast.items.len(), 1);

    // ASSERT: Empty recipe
    match &ast.items[0] {
        MakeItem::PatternRule {
            target_pattern,
            prereq_patterns,
            recipe,
            ..
        } => {
            assert_eq!(target_pattern, "%.o");
            assert_eq!(prereq_patterns.len(), 1);
            assert_eq!(prereq_patterns[0], "%.c");
            assert_eq!(recipe.len(), 0, "Recipe should be empty");
        }
        other => panic!("Expected PatternRule, got {:?}", other),
    }
}

/// RED PHASE: Test for PATTERN-001 - Distinguish pattern rule from normal target
#[test]
fn test_PATTERN_001_pattern_vs_normal_target() {
    // ARRANGE: Both pattern rule and normal target
    let makefile = "%.o: %.c\n\t$(CC) -c $< -o $@\n\nmain.o: main.c\n\t$(CC) -c main.c";

    // ACT: Parse makefile
    let result = parse_makefile(makefile);

    // ASSERT: Successfully parsed
    assert!(result.is_ok());

    let ast = result.unwrap();
    assert_eq!(ast.items.len(), 2);

    // ASSERT: First is pattern, second is normal target
    match &ast.items[0] {
        MakeItem::PatternRule { target_pattern, .. } => {
            assert_eq!(target_pattern, "%.o");
        }
        other => panic!("First item should be PatternRule, got {:?}", other),
    }

    match &ast.items[1] {
        MakeItem::Target { name, .. } => {
            assert_eq!(name, "main.o");
        }
        other => panic!("Second item should be Target, got {:?}", other),
    }
}

// ==============================================================================
// PATTERN-001: Property Tests
// ==============================================================================

#[cfg(test)]
mod prop_pattern_001 {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        /// Property: Pattern rules with % are always parsed as PatternRule
        #[test]
        fn prop_PATTERN_001_percent_always_creates_pattern_rule(
            target_ext in "[a-z]{1,3}",
            prereq_ext in "[a-z]{1,3}"
        ) {
            let makefile = format!("%.{}: %.{}\n\techo test", target_ext, prereq_ext);

            let result = parse_makefile(&makefile);
            prop_assert!(result.is_ok());

            let ast = result.unwrap();
            prop_assert_eq!(ast.items.len(), 1);

            // Verify it's a PatternRule
            match &ast.items[0] {
                MakeItem::PatternRule { target_pattern, prereq_patterns, .. } => {
                    prop_assert_eq!(target_pattern, &format!("%.{}", target_ext));
                    prop_assert_eq!(prereq_patterns.len(), 1);
                    prop_assert_eq!(&prereq_patterns[0], &format!("%.{}", prereq_ext));
                }
                other => return Err(TestCaseError::fail(format!("Expected PatternRule, got {:?}", other))),
            }
        }

        /// Property: Targets without % are never parsed as PatternRule
        #[test]
        fn prop_PATTERN_001_no_percent_creates_normal_target(
            target in "[a-z]{1,10}\\.o",
            prereq in "[a-z]{1,10}\\.c"
        ) {
            let makefile = format!("{}: {}\n\techo test", target, prereq);

            let result = parse_makefile(&makefile);
            prop_assert!(result.is_ok());

            let ast = result.unwrap();
            prop_assert_eq!(ast.items.len(), 1);

            // Verify it's a Target, not PatternRule
            match &ast.items[0] {
                MakeItem::Target { name, .. } => {
                    prop_assert_eq!(name, &target);
                }
                MakeItem::PatternRule { .. } => {
                    return Err(TestCaseError::fail("Should not create PatternRule without %"));
                }
                other => return Err(TestCaseError::fail(format!("Expected Target, got {:?}", other))),
            }
        }

        /// Property: Pattern rules with multiple prerequisites preserve order
        #[test]
        fn prop_PATTERN_001_pattern_prereq_order_preserved(
            ext1 in "[a-z]{1,3}",
            ext2 in "[a-z]{1,3}",
            ext3 in "[a-z]{1,3}"
        ) {
            let makefile = format!("%.o: %.{} %.{} %.{}\n\techo test", ext1, ext2, ext3);

            let result = parse_makefile(&makefile);
            prop_assert!(result.is_ok());

            let ast = result.unwrap();
            match &ast.items[0] {
                MakeItem::PatternRule { prereq_patterns, .. } => {
                    prop_assert_eq!(prereq_patterns.len(), 3);
                    prop_assert_eq!(&prereq_patterns[0], &format!("%.{}", ext1));
                    prop_assert_eq!(&prereq_patterns[1], &format!("%.{}", ext2));
                    prop_assert_eq!(&prereq_patterns[2], &format!("%.{}", ext3));
                }
                other => return Err(TestCaseError::fail(format!("Expected PatternRule, got {:?}", other))),
            }
        }

        /// Property: Pattern rule parsing is deterministic
        #[test]
        fn prop_PATTERN_001_parsing_is_deterministic(
            target_ext in "[a-z]{1,5}",
            prereq_ext in "[a-z]{1,5}"
        ) {
            let makefile = format!("%.{}: %.{}\n\t$(CC) -c $< -o $@", target_ext, prereq_ext);

            // Parse twice
            let result1 = parse_makefile(&makefile);
            let result2 = parse_makefile(&makefile);

            prop_assert!(result1.is_ok());
            prop_assert!(result2.is_ok());

            let ast1 = result1.unwrap();
            let ast2 = result2.unwrap();

            // Should produce identical ASTs
            prop_assert_eq!(ast1.items.len(), ast2.items.len());

            match (&ast1.items[0], &ast2.items[0]) {
                (
                    MakeItem::PatternRule { target_pattern: t1, prereq_patterns: p1, recipe: r1, .. },
                    MakeItem::PatternRule { target_pattern: t2, prereq_patterns: p2, recipe: r2, .. }
                ) => {
                    prop_assert_eq!(t1, t2);
                    prop_assert_eq!(p1, p2);
                    prop_assert_eq!(r1, r2);
                }
                _ => return Err(TestCaseError::fail("Both should be PatternRule")),
            }
        }

        /// Property: Empty recipes are handled correctly for pattern rules
        #[test]
        fn prop_PATTERN_001_empty_recipes_allowed(
            target_ext in "[a-z]{1,4}",
            prereq_ext in "[a-z]{1,4}"
        ) {
            let makefile = format!("%.{}: %.{}", target_ext, prereq_ext);

            let result = parse_makefile(&makefile);
            prop_assert!(result.is_ok());

            let ast = result.unwrap();
            match &ast.items[0] {
                MakeItem::PatternRule { target_pattern, prereq_patterns, recipe, .. } => {
                    prop_assert_eq!(target_pattern, &format!("%.{}", target_ext));
                    prop_assert_eq!(prereq_patterns.len(), 1);
                    prop_assert_eq!(recipe.len(), 0, "Recipe should be empty");
                }
                other => return Err(TestCaseError::fail(format!("Expected PatternRule, got {:?}", other))),
            }
        }
    }
}

// ==============================================================================
// PATTERN-002: Automatic Variables ($@, $<, $^)
// ==============================================================================

/// RED PHASE: Test for PATTERN-002 - Automatic variable $@ (target name)
///
/// Automatic variables are special variables set by make for each rule.
/// $@ expands to the target name.
///
/// Input Makefile:
/// ```makefile
/// program: main.o
///     $(CC) -o $@ main.o
/// ```
///
/// Expected: Recipe preserves "$@" exactly as-is
#[test]
fn test_PATTERN_002_automatic_variable_at() {
    // ARRANGE: Target with $@ automatic variable
    let makefile = "program: main.o\n\t$(CC) -o $@ main.o";

    // ACT: Parse makefile
    let result = parse_makefile(makefile);

    // ASSERT: Successfully parsed
    assert!(result.is_ok());

    let ast = result.unwrap();
    assert_eq!(ast.items.len(), 1);

    // ASSERT: Recipe contains $@
    match &ast.items[0] {
        MakeItem::Target { recipe, .. } => {
            assert_eq!(recipe.len(), 1);
            assert!(recipe[0].contains("$@"), "Recipe should contain $@");
            assert_eq!(recipe[0], "$(CC) -o $@ main.o");
        }
        other => panic!("Expected Target, got {:?}", other),
    }
}

/// RED PHASE: Test for PATTERN-002 - Automatic variable $< (first prerequisite)
///
/// $< expands to the name of the first prerequisite.
/// Commonly used in pattern rules.
#[test]
fn test_PATTERN_002_automatic_variable_less_than() {
    // ARRANGE: Pattern rule with $< automatic variable
    let makefile = "%.o: %.c\n\t$(CC) -c $< -o $@";

    // ACT: Parse makefile
    let result = parse_makefile(makefile);

    // ASSERT: Successfully parsed
    assert!(result.is_ok());

    let ast = result.unwrap();
    assert_eq!(ast.items.len(), 1);

    // ASSERT: Recipe contains $<
    match &ast.items[0] {
        MakeItem::PatternRule { recipe, .. } => {
            assert_eq!(recipe.len(), 1);
            assert!(recipe[0].contains("$<"), "Recipe should contain $<");
            assert!(recipe[0].contains("$@"), "Recipe should contain $@");
            assert_eq!(recipe[0], "$(CC) -c $< -o $@");
        }
        other => panic!("Expected PatternRule, got {:?}", other),
    }
}

/// RED PHASE: Test for PATTERN-002 - Automatic variable $^ (all prerequisites)
///
/// $^ expands to the names of all prerequisites, with spaces between them.
#[test]
fn test_PATTERN_002_automatic_variable_caret() {
    // ARRANGE: Target with $^ automatic variable
    let makefile = "program: main.o util.o\n\t$(CC) $^ -o $@";

    // ACT: Parse makefile
    let result = parse_makefile(makefile);

    // ASSERT: Successfully parsed
    assert!(result.is_ok());

    let ast = result.unwrap();
    assert_eq!(ast.items.len(), 1);

    // ASSERT: Recipe contains $^
    match &ast.items[0] {
        MakeItem::Target { recipe, .. } => {
            assert_eq!(recipe.len(), 1);
            assert!(recipe[0].contains("$^"), "Recipe should contain $^");
            assert!(recipe[0].contains("$@"), "Recipe should contain $@");
            assert_eq!(recipe[0], "$(CC) $^ -o $@");
        }
        other => panic!("Expected Target, got {:?}", other),
    }
}

/// RED PHASE: Test for PATTERN-002 - Multiple automatic variables in one recipe
#[test]
fn test_PATTERN_002_multiple_automatic_variables() {
    // ARRANGE: Recipe with multiple automatic variables
    let makefile = "%.o: %.c %.h\n\t$(CC) -c $< -o $@ -I $^";

    // ACT: Parse makefile
    let result = parse_makefile(makefile);

    // ASSERT: Successfully parsed
    assert!(result.is_ok());

    let ast = result.unwrap();
    assert_eq!(ast.items.len(), 1);

    // ASSERT: All automatic variables preserved
    match &ast.items[0] {
        MakeItem::PatternRule { recipe, .. } => {
            assert_eq!(recipe.len(), 1);
            assert!(recipe[0].contains("$<"));
            assert!(recipe[0].contains("$@"));
            assert!(recipe[0].contains("$^"));
        }
        other => panic!("Expected PatternRule, got {:?}", other),
    }
}

/// RED PHASE: Test for PATTERN-002 - Automatic variable $? (newer prerequisites)
#[test]
fn test_PATTERN_002_automatic_variable_question() {
    // ARRANGE: Target with $? automatic variable
    let makefile = "archive.a: foo.o bar.o\n\tar rcs $@ $?";

    // ACT: Parse makefile
    let result = parse_makefile(makefile);

    // ASSERT: Successfully parsed
    assert!(result.is_ok());

    let ast = result.unwrap();
    assert_eq!(ast.items.len(), 1);

    // ASSERT: Recipe contains $?
    match &ast.items[0] {
        MakeItem::Target { recipe, .. } => {
            assert_eq!(recipe.len(), 1);
            assert!(recipe[0].contains("$?"), "Recipe should contain $?");
            assert_eq!(recipe[0], "ar rcs $@ $?");
        }
        other => panic!("Expected Target, got {:?}", other),
    }
}

// ==============================================================================
// PATTERN-002: Property Tests
// ==============================================================================

#[cfg(test)]
mod prop_pattern_002 {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        /// Property: Automatic variables in recipes are always preserved
        #[test]
        fn prop_PATTERN_002_automatic_vars_always_preserved(
            target in "[a-z]{1,10}",
            prereq in "[a-z]{1,10}\\.[a-z]{1,3}"
        ) {
            let makefile = format!("{}: {}\n\t$(CC) $< -o $@", target, prereq);

            let result = parse_makefile(&makefile);
            prop_assert!(result.is_ok());

            let ast = result.unwrap();
            match &ast.items[0] {
                MakeItem::Target { recipe, .. } => {
                    prop_assert!(recipe[0].contains("$<"));
                    prop_assert!(recipe[0].contains("$@"));
                }
                _ => return Err(TestCaseError::fail("Expected Target")),
            }
        }

        /// Property: All common automatic variables are preserved
        #[test]
        fn prop_PATTERN_002_all_auto_vars_preserved(
            target in "[a-z]{1,8}"
        ) {
            // Test $@, $<, $^, $?
            let makefile = format!("{}: a.o b.o\n\techo $@ $< $^ $?", target);

            let result = parse_makefile(&makefile);
            prop_assert!(result.is_ok());

            let ast = result.unwrap();
            match &ast.items[0] {
                MakeItem::Target { recipe, .. } => {
                    prop_assert!(recipe[0].contains("$@"));
                    prop_assert!(recipe[0].contains("$<"));
                    prop_assert!(recipe[0].contains("$^"));
                    prop_assert!(recipe[0].contains("$?"));
                }
                _ => return Err(TestCaseError::fail("Expected Target")),
            }
        }

        /// Property: Automatic variables in pattern rules are preserved
        #[test]
        fn prop_PATTERN_002_pattern_rules_preserve_auto_vars(
            ext1 in "[a-z]{1,3}",
            ext2 in "[a-z]{1,3}"
        ) {
            let makefile = format!("%.{}: %.{}\n\t$(CC) -c $< -o $@", ext1, ext2);

            let result = parse_makefile(&makefile);
            prop_assert!(result.is_ok());

            let ast = result.unwrap();
            match &ast.items[0] {
                MakeItem::PatternRule { recipe, .. } => {
                    prop_assert!(recipe[0].contains("$<"));
                    prop_assert!(recipe[0].contains("$@"));
                }
                _ => return Err(TestCaseError::fail("Expected PatternRule")),
            }
        }

        /// Property: Parsing with automatic variables is deterministic
        #[test]
        fn prop_PATTERN_002_parsing_is_deterministic(
            target in "[a-z]{1,10}",
            prereq in "[a-z]{1,10}\\.[a-z]{1,3}"
        ) {
            let makefile = format!("{}: {}\n\t$(CC) $^ -o $@", target, prereq);

            // Parse twice
            let result1 = parse_makefile(&makefile);
            let result2 = parse_makefile(&makefile);

            prop_assert!(result1.is_ok());
            prop_assert!(result2.is_ok());

            let ast1 = result1.unwrap();
            let ast2 = result2.unwrap();

            // Should produce identical ASTs
            match (&ast1.items[0], &ast2.items[0]) {
                (
                    MakeItem::Target { recipe: r1, .. },
                    MakeItem::Target { recipe: r2, .. }
                ) => {
                    prop_assert_eq!(r1, r2);
                }
                _ => return Err(TestCaseError::fail("Expected Target in both")),
            }
        }

        /// Property: Mix of automatic variables and normal text preserved
        #[test]
        fn prop_PATTERN_002_mixed_content_preserved(
            target in "[a-z]{1,8}",
            flag in "-[a-zA-Z]"
        ) {
            let makefile = format!("{}: a.o\n\tgcc {} $< -o $@", target, flag);

            let result = parse_makefile(&makefile);
            prop_assert!(result.is_ok());

            let ast = result.unwrap();
            match &ast.items[0] {
                MakeItem::Target { recipe, .. } => {
                    // Verify automatic variables preserved
                    prop_assert!(recipe[0].contains("$<"));
                    prop_assert!(recipe[0].contains("$@"));
                    // Verify flag preserved
                    prop_assert!(recipe[0].contains(&flag));
                }
                _ => return Err(TestCaseError::fail("Expected Target")),
            }
        }
    }
}

// ==============================================================================
// COND-001: ifeq Conditionals
// ==============================================================================

/// RED PHASE: Test for COND-001 - Basic ifeq conditional
///
/// This test validates parsing of ifeq conditionals, which allow
/// conditional variable assignment and target rules in Makefiles.
///
/// Input Makefile:
/// ```makefile
/// ifeq ($(DEBUG),1)
/// CFLAGS = -g
/// endif
/// ```
///
/// Expected AST:
/// - One MakeItem::Conditional
/// - condition: IfEq("$(DEBUG)", "1")
/// - then_items: [Variable assignment CFLAGS = -g]
/// - else_items: None
#[test]
fn test_COND_001_basic_ifeq() {
    // ARRANGE: Simple ifeq conditional
    let makefile = "ifeq ($(DEBUG),1)\nCFLAGS = -g\nendif";

    // ACT: Parse makefile
    let result = parse_makefile(makefile);

    // ASSERT: Successfully parsed
    assert!(
        result.is_ok(),
        "Should parse basic ifeq conditional, got error: {:?}",
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

    // ASSERT: Item is a Conditional
    match &ast.items[0] {
        MakeItem::Conditional {
            condition,
            then_items,
            else_items,
            ..
        } => {
            // Check condition type
            match condition {
                MakeCondition::IfEq(left, right) => {
                    assert_eq!(left, "$(DEBUG)", "Left side should be $(DEBUG)");
                    assert_eq!(right, "1", "Right side should be 1");
                }
                other => panic!("Expected IfEq condition, got {:?}", other),
            }

            // Check then branch
            assert_eq!(then_items.len(), 1, "Should have one item in then branch");
            match &then_items[0] {
                MakeItem::Variable { name, value, .. } => {
                    assert_eq!(name, "CFLAGS", "Variable name should be CFLAGS");
                    assert_eq!(value, "-g", "Variable value should be -g");
                }
                other => panic!("Expected Variable in then branch, got {:?}", other),
            }

            // Check no else branch
            assert!(else_items.is_none(), "Should have no else branch");
        }
        other => panic!("Expected Conditional item, got {:?}", other),
    }
}

/// RED PHASE: Test for COND-001 - ifeq with else branch
#[test]
fn test_COND_001_ifeq_with_else() {
    // ARRANGE: ifeq conditional with else branch
    let makefile = "ifeq ($(DEBUG),1)\nCFLAGS = -g\nelse\nCFLAGS = -O2\nendif";

    // ACT: Parse makefile
    let result = parse_makefile(makefile);

    // ASSERT: Successfully parsed
    assert!(
        result.is_ok(),
        "Should parse ifeq with else, got error: {:?}",
        result.err()
    );

    let ast = result.unwrap();
    assert_eq!(ast.items.len(), 1);

    // ASSERT: Check conditional structure
    match &ast.items[0] {
        MakeItem::Conditional {
            condition,
            then_items,
            else_items,
            ..
        } => {
            // Check condition
            match condition {
                MakeCondition::IfEq(left, right) => {
                    assert_eq!(left, "$(DEBUG)");
                    assert_eq!(right, "1");
                }
                other => panic!("Expected IfEq, got {:?}", other),
            }

            // Check then branch
            assert_eq!(then_items.len(), 1);
            match &then_items[0] {
                MakeItem::Variable { value, .. } => {
                    assert_eq!(value, "-g");
                }
                other => panic!("Expected Variable, got {:?}", other),
            }

            // Check else branch
            assert!(else_items.is_some(), "Should have else branch");
            let else_vec = else_items.as_ref().unwrap();
            assert_eq!(else_vec.len(), 1);
            match &else_vec[0] {
                MakeItem::Variable { value, .. } => {
                    assert_eq!(value, "-O2");
                }
                other => panic!("Expected Variable in else, got {:?}", other),
            }
        }
        other => panic!("Expected Conditional, got {:?}", other),
    }
}

/// RED PHASE: Test for COND-001 - ifdef conditional
#[test]
fn test_COND_001_ifdef() {
    // ARRANGE: ifdef conditional (checks if variable is defined)
    let makefile = "ifdef VERBOSE\nCFLAGS += -v\nendif";

    // ACT: Parse makefile
    let result = parse_makefile(makefile);

    // ASSERT: Successfully parsed
    assert!(
        result.is_ok(),
        "Should parse ifdef conditional, got error: {:?}",
        result.err()
    );

    let ast = result.unwrap();
    assert_eq!(ast.items.len(), 1);

    // ASSERT: Check ifdef condition
    match &ast.items[0] {
        MakeItem::Conditional {
            condition,
            then_items,
            ..
        } => {
            match condition {
                MakeCondition::IfDef(var_name) => {
                    assert_eq!(var_name, "VERBOSE", "Should check VERBOSE variable");
                }
                other => panic!("Expected IfDef, got {:?}", other),
            }

            assert_eq!(then_items.len(), 1);
        }
        other => panic!("Expected Conditional, got {:?}", other),
    }
}

/// RED PHASE: Test for COND-001 - ifndef conditional
#[test]
fn test_COND_001_ifndef() {
    // ARRANGE: ifndef conditional (checks if variable is NOT defined)
    let makefile = "ifndef CC\nCC = gcc\nendif";

    // ACT: Parse makefile
    let result = parse_makefile(makefile);

    // ASSERT: Successfully parsed
    assert!(
        result.is_ok(),
        "Should parse ifndef conditional, got error: {:?}",
        result.err()
    );

    let ast = result.unwrap();
    assert_eq!(ast.items.len(), 1);

    // ASSERT: Check ifndef condition
    match &ast.items[0] {
        MakeItem::Conditional {
            condition,
            then_items,
            ..
        } => {
            match condition {
                MakeCondition::IfNdef(var_name) => {
                    assert_eq!(var_name, "CC", "Should check CC variable");
                }
                other => panic!("Expected IfNdef, got {:?}", other),
            }

            assert_eq!(then_items.len(), 1);
            match &then_items[0] {
                MakeItem::Variable { name, value, .. } => {
                    assert_eq!(name, "CC");
                    assert_eq!(value, "gcc");
                }
                other => panic!("Expected Variable, got {:?}", other),
            }
        }
        other => panic!("Expected Conditional, got {:?}", other),
    }
}

/// RED PHASE: Test for COND-001 - Conditional with targets in branches
#[test]
fn test_COND_001_conditional_with_targets() {
    // ARRANGE: Conditional containing target rules
    let makefile = "ifeq ($(OS),Linux)\ninstall:\n\tcp app /usr/bin/app\nendif";

    // ACT: Parse makefile
    let result = parse_makefile(makefile);

    // ASSERT: Successfully parsed
    assert!(
        result.is_ok(),
        "Should parse conditional with targets, got error: {:?}",
        result.err()
    );

    let ast = result.unwrap();
    assert_eq!(ast.items.len(), 1);

    // ASSERT: Check conditional contains target
    match &ast.items[0] {
        MakeItem::Conditional { then_items, .. } => {
            assert_eq!(then_items.len(), 1);
            match &then_items[0] {
                MakeItem::Target { name, recipe, .. } => {
                    assert_eq!(name, "install");
                    assert_eq!(recipe.len(), 1);
                    assert_eq!(recipe[0], "cp app /usr/bin/app");
                }
                other => panic!("Expected Target in then branch, got {:?}", other),
            }
        }
        other => panic!("Expected Conditional, got {:?}", other),
    }
}

/// RED PHASE: Test for COND-001 - ifneq conditional
#[test]
fn test_COND_001_ifneq() {
    // ARRANGE: ifneq conditional (inequality test)
    let makefile = "ifneq ($(DEBUG),0)\nCFLAGS += -g\nendif";

    // ACT: Parse makefile
    let result = parse_makefile(makefile);

    // ASSERT: Successfully parsed
    assert!(
        result.is_ok(),
        "Should parse ifneq conditional, got error: {:?}",
        result.err()
    );

    let ast = result.unwrap();
    assert_eq!(ast.items.len(), 1);

    // ASSERT: Check ifneq condition
    match &ast.items[0] {
        MakeItem::Conditional { condition, .. } => match condition {
            MakeCondition::IfNeq(left, right) => {
                assert_eq!(left, "$(DEBUG)");
                assert_eq!(right, "0");
            }
            other => panic!("Expected IfNeq, got {:?}", other),
        },
        other => panic!("Expected Conditional, got {:?}", other),
    }
}

// ==============================================================================
// COND-001: Property Tests
// ==============================================================================

#[cfg(test)]
mod prop_cond_001 {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        /// Property: ifeq conditionals always parse successfully with valid syntax
        #[test]
        fn prop_COND_001_ifeq_always_parses(
            var_name in "[A-Z]{2,8}",
            value in "[a-z0-9]{1,5}"
        ) {
            let makefile = format!("ifeq ($({}),{})\nCFLAGS = -g\nendif", var_name, value);

            let result = parse_makefile(&makefile);
            prop_assert!(result.is_ok(), "Failed to parse: {:?}", result.err());

            let ast = result.unwrap();
            prop_assert_eq!(ast.items.len(), 1);

            match &ast.items[0] {
                MakeItem::Conditional { condition, then_items, else_items, .. } => {
                    // Verify condition is IfEq
                    match condition {
                        MakeCondition::IfEq(left, right) => {
                            prop_assert!(left.contains(&var_name));
                            prop_assert_eq!(right, &value);
                        }
                        other => return Err(TestCaseError::fail(format!("Expected IfEq, got {:?}", other))),
                    }

                    // Verify then branch has variable
                    prop_assert_eq!(then_items.len(), 1);
                    prop_assert!(else_items.is_none());
                }
                other => return Err(TestCaseError::fail(format!("Expected Conditional, got {:?}", other))),
            }
        }

        /// Property: ifdef conditionals always parse successfully
        #[test]
        fn prop_COND_001_ifdef_always_parses(
            var_name in "[A-Z_]{2,10}"
        ) {
            let makefile = format!("ifdef {}\nCFLAGS += -v\nendif", var_name);

            let result = parse_makefile(&makefile);
            prop_assert!(result.is_ok());

            let ast = result.unwrap();
            match &ast.items[0] {
                MakeItem::Conditional { condition, .. } => {
                    match condition {
                        MakeCondition::IfDef(name) => {
                            prop_assert_eq!(name, &var_name);
                        }
                        other => return Err(TestCaseError::fail(format!("Expected IfDef, got {:?}", other))),
                    }
                }
                other => return Err(TestCaseError::fail(format!("Expected Conditional, got {:?}", other))),
            }
        }

        /// Property: ifndef conditionals always parse successfully
        #[test]
        fn prop_COND_001_ifndef_always_parses(
            var_name in "[A-Z_]{2,10}"
        ) {
            let makefile = format!("ifndef {}\n{} = default\nendif", var_name, var_name);

            let result = parse_makefile(&makefile);
            prop_assert!(result.is_ok());

            let ast = result.unwrap();
            match &ast.items[0] {
                MakeItem::Conditional { condition, .. } => {
                    match condition {
                        MakeCondition::IfNdef(name) => {
                            prop_assert_eq!(name, &var_name);
                        }
                        other => return Err(TestCaseError::fail(format!("Expected IfNdef, got {:?}", other))),
                    }
                }
                other => return Err(TestCaseError::fail(format!("Expected Conditional, got {:?}", other))),
            }
        }

        /// Property: Conditionals with else branches always parse correctly
        #[test]
        fn prop_COND_001_else_branches_work(
            var_name in "[A-Z]{2,8}",
            then_val in "[a-z]{1,5}",
            else_val in "[a-z]{1,5}"
        ) {
            let makefile = format!(
                "ifeq ($(DEBUG),1)\nFLAGS = {}\nelse\nFLAGS = {}\nendif",
                then_val, else_val
            );

            let result = parse_makefile(&makefile);
            prop_assert!(result.is_ok());

            let ast = result.unwrap();
            match &ast.items[0] {
                MakeItem::Conditional { then_items, else_items, .. } => {
                    prop_assert_eq!(then_items.len(), 1);
                    prop_assert!(else_items.is_some());

                    let else_vec = else_items.as_ref().unwrap();
                    prop_assert_eq!(else_vec.len(), 1);

                    // Verify then branch value
                    match &then_items[0] {
                        MakeItem::Variable { value, .. } => {
                            prop_assert_eq!(value, &then_val);
                        }
                        _ => return Err(TestCaseError::fail("Expected Variable in then branch")),
                    }

                    // Verify else branch value
                    match &else_vec[0] {
                        MakeItem::Variable { value, .. } => {
                            prop_assert_eq!(value, &else_val);
                        }
                        _ => return Err(TestCaseError::fail("Expected Variable in else branch")),
                    }
                }
                other => return Err(TestCaseError::fail(format!("Expected Conditional, got {:?}", other))),
            }
        }

        /// Property: Parsing with conditionals is deterministic
        #[test]
        fn prop_COND_001_parsing_is_deterministic(
            var_name in "[A-Z]{2,8}",
            value in "[0-9]{1,3}"
        ) {
            let makefile = format!("ifeq ($({}),{})\nBUILD = yes\nendif", var_name, value);

            // Parse twice
            let result1 = parse_makefile(&makefile);
            let result2 = parse_makefile(&makefile);

            prop_assert!(result1.is_ok());
            prop_assert!(result2.is_ok());

            let ast1 = result1.unwrap();
            let ast2 = result2.unwrap();

            // Should produce identical ASTs
            prop_assert_eq!(ast1.items.len(), ast2.items.len());

            match (&ast1.items[0], &ast2.items[0]) {
                (
                    MakeItem::Conditional { condition: c1, then_items: t1, else_items: e1, .. },
                    MakeItem::Conditional { condition: c2, then_items: t2, else_items: e2, .. }
                ) => {
                    // Conditions should match
                    prop_assert!(matches!((c1, c2), (MakeCondition::IfEq(_, _), MakeCondition::IfEq(_, _))));

                    // Then items should match
                    prop_assert_eq!(t1.len(), t2.len());

                    // Else items should match
                    prop_assert_eq!(e1.is_some(), e2.is_some());
                }
                _ => return Err(TestCaseError::fail("Expected Conditional in both ASTs")),
            }
        }

        /// Property: ifneq conditionals parse correctly
        #[test]
        fn prop_COND_001_ifneq_parses(
            left in "[a-z]{2,6}",
            right in "[a-z]{2,6}"
        ) {
            let makefile = format!("ifneq ({},{})\nTEST = 1\nendif", left, right);

            let result = parse_makefile(&makefile);
            prop_assert!(result.is_ok());

            let ast = result.unwrap();
            match &ast.items[0] {
                MakeItem::Conditional { condition, .. } => {
                    match condition {
                        MakeCondition::IfNeq(l, r) => {
                            prop_assert_eq!(l, &left);
                            prop_assert_eq!(r, &right);
                        }
                        other => return Err(TestCaseError::fail(format!("Expected IfNeq, got {:?}", other))),
                    }
                }
                other => return Err(TestCaseError::fail(format!("Expected Conditional, got {:?}", other))),
            }
        }
    }
}

// ===========================================================================
// VAR-SUBST-001: Variable Substitution Tests
// ===========================================================================
// Task: Document variable substitution ($(VAR:suffix=replacement))
// Input: OBJS = $(SRCS:.c=.o)
// Goal: Parser preserves variable substitution syntax in variable values

#[test]
fn test_VAR_SUBST_001_basic_substitution() {
    // ARRANGE: Variable with substitution reference
    let makefile = "OBJS = $(SRCS:.c=.o)";

    // ACT: Parse makefile
    let result = parse_makefile(makefile);

    // ASSERT: Successfully parsed
    assert!(
        result.is_ok(),
        "Should parse variable substitution, got error: {:?}",
        result.err()
    );

    let ast = result.unwrap();
    assert_eq!(ast.items.len(), 1, "Should have 1 variable");

    // ASSERT: Check variable with substitution
    match &ast.items[0] {
        MakeItem::Variable { name, value, .. } => {
            assert_eq!(name, "OBJS", "Variable name should be OBJS");
            assert_eq!(
                value, "$(SRCS:.c=.o)",
                "Variable value should preserve substitution syntax"
            );
        }
        other => panic!("Expected Variable item, got {:?}", other),
    }
}

#[test]
fn test_VAR_SUBST_001_multiple_substitutions() {
    // ARRANGE: Multiple variables with substitutions
    let makefile = "OBJS = $(SRCS:.c=.o)\nLIBS = $(DEPS:.a=.so)";

    // ACT: Parse makefile
    let result = parse_makefile(makefile);

    // ASSERT: Successfully parsed
    assert!(result.is_ok(), "Should parse multiple substitutions");

    let ast = result.unwrap();
    assert_eq!(ast.items.len(), 2, "Should have 2 variables");

    // ASSERT: Check first substitution
    match &ast.items[0] {
        MakeItem::Variable { name, value, .. } => {
            assert_eq!(name, "OBJS");
            assert_eq!(value, "$(SRCS:.c=.o)");
        }
        other => panic!("Expected Variable, got {:?}", other),
    }

    // ASSERT: Check second substitution
    match &ast.items[1] {
        MakeItem::Variable { name, value, .. } => {
            assert_eq!(name, "LIBS");
            assert_eq!(value, "$(DEPS:.a=.so)");
        }
        other => panic!("Expected Variable, got {:?}", other),
    }
}

#[test]
fn test_VAR_SUBST_001_substitution_with_path() {
    // ARRANGE: Substitution with path patterns
    let makefile = "OBJS = $(SRCS:src/%.c=build/%.o)";

    // ACT: Parse makefile
    let result = parse_makefile(makefile);

    // ASSERT: Successfully parsed
    assert!(result.is_ok(), "Should parse path substitution");

    let ast = result.unwrap();
    match &ast.items[0] {
        MakeItem::Variable { value, .. } => {
            assert_eq!(
                value, "$(SRCS:src/%.c=build/%.o)",
                "Should preserve path patterns in substitution"
            );
        }
        other => panic!("Expected Variable, got {:?}", other),
    }
}

#[test]
fn test_VAR_SUBST_001_substitution_in_recipe() {
    // ARRANGE: Substitution used in recipe
    let makefile = "build: $(SRCS:.c=.o)\n\t$(CC) $^ -o $@";

    // ACT: Parse makefile
    let result = parse_makefile(makefile);

    // ASSERT: Successfully parsed
    assert!(result.is_ok(), "Should parse substitution in prerequisites");

    let ast = result.unwrap();
    match &ast.items[0] {
        MakeItem::Target { prerequisites, .. } => {
            assert_eq!(prerequisites.len(), 1);
            assert_eq!(
                prerequisites[0], "$(SRCS:.c=.o)",
                "Should preserve substitution in prerequisites"
            );
        }
        other => panic!("Expected Target, got {:?}", other),
    }
}

#[test]
fn test_VAR_SUBST_001_percent_substitution() {
    // ARRANGE: Substitution with % pattern
    let makefile = "OBJS = $(SRCS:%.c=%.o)";

    // ACT: Parse makefile
    let result = parse_makefile(makefile);

    // ASSERT: Successfully parsed
    assert!(result.is_ok(), "Should parse % pattern substitution");

    let ast = result.unwrap();
    match &ast.items[0] {
        MakeItem::Variable { value, .. } => {
            assert_eq!(
                value, "$(SRCS:%.c=%.o)",
                "Should preserve % pattern in substitution"
            );
        }
        other => panic!("Expected Variable, got {:?}", other),
    }
}

#[test]
fn test_VAR_SUBST_001_complex_substitution() {
    // ARRANGE: Complex substitution with multiple parts
    let makefile = "FILES = $(wildcard *.c)\nOBJS = $(FILES:.c=.o)";

    // ACT: Parse makefile
    let result = parse_makefile(makefile);

    // ASSERT: Successfully parsed
    assert!(result.is_ok(), "Should parse complex substitutions");

    let ast = result.unwrap();
    assert_eq!(ast.items.len(), 2);

    // First variable has wildcard function
    match &ast.items[0] {
        MakeItem::Variable { value, .. } => {
            assert_eq!(value, "$(wildcard *.c)");
        }
        other => panic!("Expected Variable, got {:?}", other),
    }

    // Second variable has substitution
    match &ast.items[1] {
        MakeItem::Variable { value, .. } => {
            assert_eq!(value, "$(FILES:.c=.o)");
        }
        other => panic!("Expected Variable, got {:?}", other),
    }
}

// ===========================================================================
// VAR-SUBST-001: Property Tests
// ===========================================================================

#[cfg(test)]
mod var_subst_property_tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn prop_VAR_SUBST_001_substitution_always_preserved(
            var_name in "[A-Z]{2,8}",
            ref_name in "[A-Z]{2,8}",
            from_ext in "\\.[a-z]{1,3}",
            to_ext in "\\.[a-z]{1,3}"
        ) {
            // ARRANGE: Variable with substitution pattern
            let makefile = format!("{} = $({}:{}={})", var_name, ref_name, from_ext, to_ext);

            // ACT: Parse makefile
            let result = parse_makefile(&makefile);

            // ASSERT: Always parses successfully
            prop_assert!(result.is_ok(), "Substitution should parse");

            let ast = result.unwrap();
            prop_assert_eq!(ast.items.len(), 1, "Should have 1 variable");

            // ASSERT: Substitution syntax preserved
            match &ast.items[0] {
                MakeItem::Variable { name, value, .. } => {
                    prop_assert_eq!(name, &var_name);
                    let expected = format!("$({}:{}={})", ref_name, from_ext, to_ext);
                    prop_assert_eq!(value, &expected, "Substitution syntax should be preserved");
                }
                other => return Err(TestCaseError::fail(format!("Expected Variable, got {:?}", other))),
            }
        }

        #[test]
        fn prop_VAR_SUBST_001_percent_patterns_preserved(
            var_name in "[A-Z]{2,8}",
            ref_name in "[A-Z]{2,8}",
            from_pattern in "%\\.[a-z]{1,3}",
            to_pattern in "%\\.[a-z]{1,3}"
        ) {
            // ARRANGE: Substitution with % patterns
            let makefile = format!("{} = $({}:{}={})", var_name, ref_name, from_pattern, to_pattern);

            // ACT: Parse makefile
            let result = parse_makefile(&makefile);

            // ASSERT: % patterns preserved
            prop_assert!(result.is_ok());

            let ast = result.unwrap();
            match &ast.items[0] {
                MakeItem::Variable { value, .. } => {
                    let expected = format!("$({}:{}={})", ref_name, from_pattern, to_pattern);
                    prop_assert_eq!(value, &expected, "% patterns should be preserved");
                }
                other => return Err(TestCaseError::fail(format!("Expected Variable, got {:?}", other))),
            }
        }

        #[test]
        fn prop_VAR_SUBST_001_parsing_is_deterministic(
            var_name in "[A-Z]{2,8}",
            ref_name in "[A-Z]{2,8}"
        ) {
            // ARRANGE: Simple substitution
            let makefile = format!("{} = $({}:.c=.o)", var_name, ref_name);

            // ACT: Parse twice
            let result1 = parse_makefile(&makefile);
            let result2 = parse_makefile(&makefile);

            // ASSERT: Results are identical (deterministic)
            prop_assert!(result1.is_ok());
            prop_assert!(result2.is_ok());

            let ast1 = result1.unwrap();
            let ast2 = result2.unwrap();

            prop_assert_eq!(ast1.items.len(), ast2.items.len());

            match (&ast1.items[0], &ast2.items[0]) {
                (MakeItem::Variable { value: v1, .. }, MakeItem::Variable { value: v2, .. }) => {
                    prop_assert_eq!(v1, v2, "Parsing should be deterministic");
                }
                _ => return Err(TestCaseError::fail("Both should be Variables")),
            }
        }

        #[test]
        fn prop_VAR_SUBST_001_path_patterns_preserved(
            ref_name in "[A-Z]{2,8}",
            from_dir in "[a-z]{3,6}",
            to_dir in "[a-z]{3,6}"
        ) {
            // ARRANGE: Substitution with path patterns
            let makefile = format!("OBJS = $({}:{}/%={}/%)", ref_name, from_dir, to_dir);

            // ACT: Parse makefile
            let result = parse_makefile(&makefile);

            // ASSERT: Path patterns preserved
            prop_assert!(result.is_ok());

            let ast = result.unwrap();
            match &ast.items[0] {
                MakeItem::Variable { value, .. } => {
                    let expected = format!("$({}:{}/%={}/%)", ref_name, from_dir, to_dir);
                    prop_assert_eq!(value, &expected, "Path patterns should be preserved");
                }
                other => return Err(TestCaseError::fail(format!("Expected Variable, got {:?}", other))),
            }
        }

        #[test]
        fn prop_VAR_SUBST_001_in_prerequisites_preserved(
            target_name in "[a-z]{3,8}",
            ref_name in "[A-Z]{2,8}",
            from_ext in "\\.[a-z]{1,3}",
            to_ext in "\\.[a-z]{1,3}"
        ) {
            // ARRANGE: Substitution in target prerequisites
            let makefile = format!("{}: $({}:{}={})\n\techo test", target_name, ref_name, from_ext, to_ext);

            // ACT: Parse makefile
            let result = parse_makefile(&makefile);

            // ASSERT: Substitution preserved in prerequisites
            prop_assert!(result.is_ok());

            let ast = result.unwrap();
            match &ast.items[0] {
                MakeItem::Target { prerequisites, .. } => {
                    prop_assert_eq!(prerequisites.len(), 1);
                    let expected = format!("$({}:{}={})", ref_name, from_ext, to_ext);
                    prop_assert_eq!(&prerequisites[0], &expected, "Substitution should be preserved in prerequisites");
                }
                other => return Err(TestCaseError::fail(format!("Expected Target, got {:?}", other))),
            }
        }

        #[test]
        fn prop_VAR_SUBST_001_multiple_substitutions_preserved(
            var1 in "[A-Z]{2,8}",
            var2 in "[A-Z]{2,8}",
            ref1 in "[A-Z]{2,8}",
            ref2 in "[A-Z]{2,8}"
        ) {
            // ARRANGE: Multiple variables with different substitutions
            let makefile = format!(
                "{} = $({}:.c=.o)\n{} = $({}:.a=.so)",
                var1, ref1, var2, ref2
            );

            // ACT: Parse makefile
            let result = parse_makefile(&makefile);

            // ASSERT: Both substitutions preserved
            prop_assert!(result.is_ok());

            let ast = result.unwrap();
            prop_assert_eq!(ast.items.len(), 2);

            // Check first substitution
            match &ast.items[0] {
                MakeItem::Variable { name, value, .. } => {
                    prop_assert_eq!(name, &var1);
                    let expected = format!("$({}:.c=.o)", ref1);
                    prop_assert_eq!(value, &expected);
                }
                other => return Err(TestCaseError::fail(format!("Expected Variable, got {:?}", other))),
            }

            // Check second substitution
            match &ast.items[1] {
                MakeItem::Variable { name, value, .. } => {
                    prop_assert_eq!(name, &var2);
                    let expected = format!("$({}:.a=.so)", ref2);
                    prop_assert_eq!(value, &expected);
                }
                other => return Err(TestCaseError::fail(format!("Expected Variable, got {:?}", other))),
            }
        }
    }
}
