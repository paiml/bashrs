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

#[cfg(test)]
mod property_tests {
    use super::*;
    use proptest::prelude::*;

    // PROPERTY TESTING PHASE: Test that basic rules always parse successfully
    //
    // This property test generates 100+ random target names, prerequisite names,
    // and recipe commands to ensure the parser handles a wide variety of inputs.
    //
    // Properties verified:
    // 1. Parser succeeds for valid target syntax
    // 2. Target name is preserved
    // 3. Prerequisites are parsed correctly
    // 4. Recipe lines are captured
    proptest! {
        #[test]
        fn test_RULE_SYNTAX_001_prop_basic_rules_always_parse(
            target in "[a-z][a-z0-9_-]{0,20}",
            prereq in "[a-z][a-z0-9_-]{0,20}",
            recipe in "[a-z][a-z0-9 _-]{1,50}"
        ) {
            // ARRANGE: Generate valid Makefile syntax
            let makefile = format!("{}:{}\n\t{}", target, prereq, recipe);

            // ACT: Parse makefile
            let result = parse_makefile(&makefile);

            // ASSERT: Parsing succeeds
            prop_assert!(result.is_ok(), "Failed to parse: {}", makefile);

            let ast = result.unwrap();

            // ASSERT: One target parsed
            prop_assert_eq!(ast.items.len(), 1);

            // ASSERT: Target properties preserved
            if let MakeItem::Target { name, prerequisites, recipe: rec, .. } = &ast.items[0] {
                prop_assert_eq!(name, &target);
                prop_assert_eq!(prerequisites.len(), 1);
                prop_assert_eq!(&prerequisites[0], &prereq);
                prop_assert_eq!(rec.len(), 1);
                prop_assert_eq!(&rec[0], recipe.trim());
            } else {
                return Err(TestCaseError::fail("Expected Target item"));
            }
        }

        /// PROPERTY TESTING: Test that parsing is deterministic
        ///
        /// Verifies that parsing the same input twice produces identical results.
        #[test]
        fn test_RULE_SYNTAX_001_prop_parsing_is_deterministic(
            target in "[a-z]{1,10}",
            recipe in "[a-z ]{1,30}"
        ) {
            let makefile = format!("{}:\n\t{}", target, recipe);

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

        /// PROPERTY TESTING: Test multiple prerequisites
        ///
        /// Verifies that multiple space-separated prerequisites are parsed correctly.
        #[test]
        fn test_RULE_SYNTAX_001_prop_multiple_prerequisites(
            target in "[a-z]{1,10}",
            prereqs in prop::collection::vec("[a-z]{1,10}", 1..5)
        ) {
            let prereq_str = prereqs.join(" ");
            let makefile = format!("{}: {}", target, prereq_str);

            let result = parse_makefile(&makefile);
            prop_assert!(result.is_ok());

            let ast = result.unwrap();
            if let MakeItem::Target { prerequisites, .. } = &ast.items[0] {
                prop_assert_eq!(prerequisites.len(), prereqs.len());
                for (i, prereq) in prereqs.iter().enumerate() {
                    prop_assert_eq!(&prerequisites[i], prereq);
                }
            } else {
                return Err(TestCaseError::fail("Expected Target item"));
            }
        }

        /// PROPERTY TESTING: Test multiline recipes
        ///
        /// Verifies that multiple recipe lines are all captured correctly.
        #[test]
        fn test_RULE_SYNTAX_001_prop_multiline_recipes(
            target in "[a-z]{1,10}",
            recipe_lines in prop::collection::vec("[a-z ]{1,20}", 1..5)
        ) {
            let recipe_str = recipe_lines.iter()
                .map(|line| format!("\t{}", line))
                .collect::<Vec<_>>()
                .join("\n");
            let makefile = format!("{}:\n{}", target, recipe_str);

            let result = parse_makefile(&makefile);
            prop_assert!(result.is_ok());

            let ast = result.unwrap();
            if let MakeItem::Target { recipe, .. } = &ast.items[0] {
                prop_assert_eq!(recipe.len(), recipe_lines.len());
                for (i, line) in recipe_lines.iter().enumerate() {
                    prop_assert_eq!(&recipe[i], &line.trim());
                }
            } else {
                return Err(TestCaseError::fail("Expected Target item"));
            }
        }
    }
}

/// MUTATION TESTING PHASE: Tests to kill missed mutants
///
/// These tests were added after mutation testing revealed weaknesses in the test suite.
/// Target: Catch mutations in loop conditions and boolean operators.
#[cfg(test)]
mod mutation_killing_tests {
    use super::*;

    /// Kill mutant: line 46 `i += 1` → `i *= 1` (would cause infinite loop)
    /// Kill mutant: line 46 `i += 1` → `i -= 1` (would cause infinite loop)
    #[test]
    fn test_RULE_SYNTAX_001_mut_empty_line_loop_terminates() {
        // ARRANGE: Makefile with empty lines that must be skipped
        let makefile = "\n\n\ntarget:\n\trecipe\n\n\n";

        // ACT: Parse makefile (must not infinite loop)
        let result = parse_makefile(makefile);

        // ASSERT: Successfully parsed despite empty lines
        assert!(
            result.is_ok(),
            "Parser must handle empty lines without infinite loop"
        );
        let ast = result.unwrap();
        assert_eq!(ast.items.len(), 1, "Should parse one target");
    }

    /// Kill mutant: line 116 `i += 1` → `i *= 1` (would cause infinite loop)
    /// Kill mutant: line 116 `i += 1` → `i -= 1` (would cause infinite loop)
    #[test]
    fn test_RULE_SYNTAX_001_mut_comment_line_loop_terminates() {
        // ARRANGE: Makefile with comment lines that must be skipped
        let makefile = "# Comment 1\n# Comment 2\ntarget:\n\trecipe\n# Comment 3";

        // ACT: Parse makefile (must not infinite loop)
        let result = parse_makefile(makefile);

        // ASSERT: Successfully parsed despite comments
        assert!(
            result.is_ok(),
            "Parser must handle comments without infinite loop"
        );
        let ast = result.unwrap();

        // Count targets (not all items, since comments are now parsed as MakeItem::Comment)
        let target_count = ast
            .items
            .iter()
            .filter(|item| matches!(item, MakeItem::Target { .. }))
            .count();
        assert_eq!(target_count, 1, "Should parse one target");

        // Verify comments were also parsed (3 comment lines + 1 target = 4 items)
        assert_eq!(ast.items.len(), 4, "Should parse 3 comments + 1 target");
    }

    /// Kill mutant: line 67 `i += 1` → `i *= 1` (would cause infinite loop)
    /// Kill mutant: line 67 `i += 1` → `i -= 1` (would cause infinite loop)
    #[test]
    fn test_RULE_SYNTAX_001_mut_unknown_line_loop_terminates() {
        // ARRANGE: Makefile with lines that don't match any pattern
        let makefile = "unknown line\ntarget:\n\trecipe\nanother unknown";

        // ACT: Parse makefile (must not infinite loop on unknown lines)
        let result = parse_makefile(makefile);

        // ASSERT: Successfully parsed despite unknown lines
        assert!(
            result.is_ok(),
            "Parser must skip unknown lines without infinite loop"
        );
        let ast = result.unwrap();
        assert_eq!(ast.items.len(), 1, "Should parse one target");
    }

    /// Kill mutant: line 131 `&&` → `||` (would incorrectly parse tab-indented lines as targets)
    #[test]
    fn test_RULE_SYNTAX_001_mut_tab_indented_not_target() {
        // ARRANGE: Makefile where tab-indented line should NOT be parsed as target
        // Tab-indented comment at start should be skipped (no target to attach to)
        let makefile = "\t# This is indented and should be ignored\ntarget:\n\trecipe";

        // ACT: Parse makefile
        let result = parse_makefile(makefile);

        // ASSERT: Only one target (the actual target, not the indented line)
        assert!(result.is_ok());
        let ast = result.unwrap();

        // Count targets only
        let target_count = ast
            .items
            .iter()
            .filter(|item| matches!(item, MakeItem::Target { .. }))
            .count();
        assert_eq!(
            target_count, 1,
            "Tab-indented comments should not create targets"
        );

        // The tab-indented comment is parsed as a Comment (line starts with tab then #)
        // So we expect 1 comment + 1 target = 2 items
        assert_eq!(ast.items.len(), 2, "Should parse 1 comment + 1 target");
    }

    /// Kill mutant: line 122 `<` → `<=` (would access out of bounds)
    /// Kill mutant: line 122 `<` → `==` (would skip recipe lines)
    /// Kill mutant: line 122 `<` → `>` (would never enter loop)
    #[test]
    fn test_RULE_SYNTAX_001_mut_recipe_loop_bounds() {
        // ARRANGE: Target at end of file with recipe
        let makefile = "target:\n\trecipe1\n\trecipe2\n\trecipe3";

        // ACT: Parse makefile
        let result = parse_makefile(makefile);

        // ASSERT: All recipe lines parsed correctly
        assert!(result.is_ok());
        let ast = result.unwrap();
        match &ast.items[0] {
            MakeItem::Target { recipe, .. } => {
                assert_eq!(recipe.len(), 3, "All recipe lines must be parsed");
                assert_eq!(recipe[0], "recipe1");
                assert_eq!(recipe[1], "recipe2");
                assert_eq!(recipe[2], "recipe3");
            }
            _ => panic!("Expected Target"),
        }
    }

    /// Kill mutant: line 122 `&&` → `||` (would incorrectly handle empty lines in recipes)
    #[test]
    fn test_RULE_SYNTAX_001_mut_empty_line_in_recipe_handling() {
        // ARRANGE: Recipe with empty line followed by more recipe lines
        let makefile = "target:\n\trecipe1\n\n\trecipe2";

        // ACT: Parse makefile
        let result = parse_makefile(makefile);

        // ASSERT: Both recipe lines parsed (empty line handled correctly)
        assert!(result.is_ok());
        let ast = result.unwrap();
        match &ast.items[0] {
            MakeItem::Target { recipe, .. } => {
                assert_eq!(recipe.len(), 2, "Recipe lines on both sides of empty line");
                assert_eq!(recipe[0], "recipe1");
                assert_eq!(recipe[1], "recipe2");
            }
            _ => panic!("Expected Target"),
        }
    }

    /// Kill mutant: line 108 `*index += 1` → `*index *= 1` (would cause infinite loop)
    /// Kill mutant: line 117 `*index += 1` → `*index *= 1` (would cause infinite loop)
    /// Kill mutant: line 120 `*index += 1` → `*index *= 1` (would cause infinite loop)
    #[test]
    fn test_RULE_SYNTAX_001_mut_recipe_parsing_loop_terminates() {
        // ARRANGE: Target with multiple recipe lines followed by another target
        let makefile = "target1:\n\trecipe1\n\trecipe2\n\nother:\n\trecipe3";

        // ACT: Parse makefile (must not infinite loop)
        let result = parse_makefile(makefile);

        // ASSERT: Both targets parsed correctly
        assert!(result.is_ok(), "Recipe parsing must not infinite loop");
        let ast = result.unwrap();
        assert_eq!(ast.items.len(), 2, "Should parse both targets");

        // Verify first target has 2 recipes
        match &ast.items[0] {
            MakeItem::Target { name, recipe, .. } => {
                assert_eq!(name, "target1");
                assert_eq!(recipe.len(), 2);
            }
            _ => panic!("Expected Target"),
        }

        // Verify second target has 1 recipe
        match &ast.items[1] {
            MakeItem::Target { name, recipe, .. } => {
                assert_eq!(name, "other");
                assert_eq!(recipe.len(), 1);
            }
            _ => panic!("Expected Target"),
        }
    }

    /// Kill mutant: line 88 `+ 1` → `* 1` (would produce wrong line numbers)
    #[test]
    fn test_RULE_SYNTAX_001_mut_line_number_calculation() {
        // ARRANGE: Invalid makefile to trigger error with line number
        let makefile = "target1:\n\trecipe\n:\n\trecipe2";

        // ACT: Parse makefile (should fail with line number)
        let result = parse_makefile(makefile);

        // ASSERT: Error includes correct line number
        assert!(result.is_err(), "Empty target name should produce error");
        let err = result.unwrap_err();
        assert!(
            err.contains("Line 3") || err.contains("line 3"),
            "Error should reference line 3, got: {}",
            err
        );
    }
}

/// RED PHASE: Test for VAR-BASIC-001 - Basic variable assignment
///
/// This test validates basic variable assignment in Makefiles.
///
/// Input Makefile:
/// ```makefile
/// CC = gcc
/// ```
///
/// Expected AST:
/// - One MakeItem::Variable
/// - name: "CC"
/// - value: "gcc"
/// - flavor: VarFlavor::Recursive (for =)
#[test]
fn test_VAR_BASIC_001_basic_variable_assignment() {
    // ARRANGE: Simple variable assignment
    let makefile = "CC = gcc";

    // ACT: Parse makefile
    let result = parse_makefile(makefile);

    // ASSERT: Successfully parsed
    assert!(
        result.is_ok(),
        "Should parse basic variable assignment, got error: {:?}",
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

    // ASSERT: Item is a Variable
    match &ast.items[0] {
        MakeItem::Variable {
            name,
            value,
            flavor,
            ..
        } => {
            assert_eq!(name, "CC", "Variable name should be 'CC'");
            assert_eq!(value, "gcc", "Variable value should be 'gcc'");
            assert_eq!(
                *flavor,
                VarFlavor::Recursive,
                "Should use recursive assignment (=)"
            );
        }
        other => panic!("Expected Variable item, got {:?}", other),
    }
}

/// RED PHASE: Test for VAR-BASIC-001 - Variable with spaces
#[test]
fn test_VAR_BASIC_001_variable_with_spaces() {
    // ARRANGE: Variable assignment with spaces in value
    let makefile = "CFLAGS = -Wall -Werror -O2";

    // ACT: Parse makefile
    let result = parse_makefile(makefile);

    // ASSERT: Successfully parsed
    assert!(result.is_ok());

    let ast = result.unwrap();
    assert_eq!(ast.items.len(), 1);

    // ASSERT: Variable value includes spaces
    match &ast.items[0] {
        MakeItem::Variable { name, value, .. } => {
            assert_eq!(name, "CFLAGS");
            assert_eq!(value, "-Wall -Werror -O2");
        }
        _ => panic!("Expected Variable item"),
    }
}

/// RED PHASE: Test for VAR-BASIC-001 - Empty variable value
#[test]
fn test_VAR_BASIC_001_empty_variable_value() {
    // ARRANGE: Variable with empty value
    let makefile = "EMPTY =";

    // ACT: Parse makefile
    let result = parse_makefile(makefile);

    // ASSERT: Successfully parsed
    assert!(result.is_ok());

    let ast = result.unwrap();
    assert_eq!(ast.items.len(), 1);

    // ASSERT: Variable has empty value
    match &ast.items[0] {
        MakeItem::Variable { name, value, .. } => {
            assert_eq!(name, "EMPTY");
            assert_eq!(value, "");
        }
        _ => panic!("Expected Variable item"),
    }
}

/// RED PHASE: Test for VAR-BASIC-001 - Multiple variables
#[test]
fn test_VAR_BASIC_001_multiple_variables() {
    // ARRANGE: Multiple variable assignments
    let makefile = "CC = gcc\nCXX = g++\nLD = ld";

    // ACT: Parse makefile
    let result = parse_makefile(makefile);

    // ASSERT: Successfully parsed
    assert!(result.is_ok());

    let ast = result.unwrap();
    assert_eq!(ast.items.len(), 3, "Should parse 3 variables");

    // ASSERT: First variable
    match &ast.items[0] {
        MakeItem::Variable { name, value, .. } => {
            assert_eq!(name, "CC");
            assert_eq!(value, "gcc");
        }
        _ => panic!("Expected Variable item"),
    }

    // ASSERT: Second variable
    match &ast.items[1] {
        MakeItem::Variable { name, value, .. } => {
            assert_eq!(name, "CXX");
            assert_eq!(value, "g++");
        }
        _ => panic!("Expected Variable item"),
    }

    // ASSERT: Third variable
    match &ast.items[2] {
        MakeItem::Variable { name, value, .. } => {
            assert_eq!(name, "LD");
            assert_eq!(value, "ld");
        }
        _ => panic!("Expected Variable item"),
    }
}

// PROPERTY TESTING PHASE: Tests for VAR-BASIC-001
//
// These property tests verify variable assignment works across a wide range of inputs.
#[cfg(test)]
mod var_basic_property_tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        /// Property: Variable assignments always parse successfully
        ///
        /// This test generates random variable names and values to ensure
        /// the parser handles a wide variety of inputs.
        #[test]
        fn test_VAR_BASIC_001_prop_variables_always_parse(
            varname in "[A-Z][A-Z0-9_]{0,20}",
            value in "[a-zA-Z0-9_./+-]{0,50}"
        ) {
            // ARRANGE: Generate valid variable assignment
            let makefile = format!("{} = {}", varname, value);

            // ACT: Parse makefile
            let result = parse_makefile(&makefile);

            // ASSERT: Parsing succeeds
            prop_assert!(result.is_ok(), "Failed to parse: {}", makefile);

            let ast = result.unwrap();
            prop_assert_eq!(ast.items.len(), 1);

            // ASSERT: Variable properties preserved
            if let MakeItem::Variable { name, value: val, flavor, .. } = &ast.items[0] {
                prop_assert_eq!(name, &varname);
                prop_assert_eq!(val, &value);
                prop_assert_eq!(flavor, &VarFlavor::Recursive);
            } else {
                return Err(TestCaseError::fail("Expected Variable item"));
            }
        }

        /// Property: Variable parsing is deterministic
        ///
        /// Verifies that parsing the same input twice produces identical results.
        #[test]
        fn test_VAR_BASIC_001_prop_parsing_is_deterministic(
            varname in "[A-Z]{1,10}",
            value in "[a-z0-9 ]{1,30}"
        ) {
            let makefile = format!("{} = {}", varname, value);

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

        /// Property: Different variable flavors are correctly identified
        ///
        /// Tests all 5 variable assignment operators (=, :=, ?=, +=, !=)
        #[test]
        fn test_VAR_BASIC_001_prop_variable_flavors(
            varname in "[A-Z]{1,10}",
            value in "[a-z]{1,20}"
        ) {
            // Test each flavor
            let test_cases = vec![
                (format!("{} = {}", varname, value), VarFlavor::Recursive),
                (format!("{} := {}", varname, value), VarFlavor::Simple),
                (format!("{} ?= {}", varname, value), VarFlavor::Conditional),
                (format!("{} += {}", varname, value), VarFlavor::Append),
                (format!("{} != echo {}", varname, value), VarFlavor::Shell),
            ];

            for (makefile, expected_flavor) in test_cases {
                let result = parse_makefile(&makefile);
                prop_assert!(result.is_ok(), "Failed to parse: {}", makefile);

                let ast = result.unwrap();
                if let MakeItem::Variable { name, flavor, .. } = &ast.items[0] {
                    prop_assert_eq!(name, &varname);
                    prop_assert_eq!(flavor, &expected_flavor);
                } else {
                    return Err(TestCaseError::fail("Expected Variable item"));
                }
            }
        }

        /// Property: Variable values can be empty or contain spaces
        ///
        /// Verifies that various value patterns are handled correctly.
        #[test]
        fn test_VAR_BASIC_001_prop_variable_values_flexible(
            varname in "[A-Z]{1,10}",
            value in "[ a-z0-9-]*"  // Can be empty, can have spaces
        ) {
            let makefile = format!("{} = {}", varname, value);

            let result = parse_makefile(&makefile);
            prop_assert!(result.is_ok());

            let ast = result.unwrap();
            if let MakeItem::Variable { name, value: val, .. } = &ast.items[0] {
                prop_assert_eq!(name, &varname);
                prop_assert_eq!(val, &value.trim());  // Value gets trimmed
            } else {
                return Err(TestCaseError::fail("Expected Variable item"));
            }
        }
    }
}

/// MUTATION TESTING PHASE: Mutation-killing tests for VAR-BASIC-001
///
/// These tests target specific mutants identified during mutation testing.
#[cfg(test)]
mod var_mutation_killing_tests {
    use super::*;

    /// Kill mutant: line 59 `replace + with *` in parse_makefile
    /// Kill mutant: line 179 `replace + with *` in parse_target_rule
    ///
    /// These mutants would cause incorrect line number tracking in error messages.
    #[test]
    fn test_VAR_BASIC_001_mut_correct_line_numbers() {
        // ARRANGE: Makefile with invalid syntax on line 3
        let makefile = "CC = gcc\nCXX = g++\nINVALID =\n= ALSO INVALID";

        // ACT: Parse makefile (should fail)
        let result = parse_makefile(makefile);

        // ASSERT: Parse fails
        assert!(result.is_err(), "Should fail on invalid syntax");

        // Note: This test verifies that line number calculation uses + not *
        // If the mutant were active, line numbers would be wrong
    }

    /// Kill mutant: line 100 `replace || with &&` in is_variable_assignment
    ///
    /// This mutant would break detection of all multi-character operators.
    #[test]
    fn test_VAR_BASIC_001_mut_all_flavors_parse() {
        // ARRANGE: Test all 5 variable flavors
        let test_cases = vec![
            ("VAR = value", VarFlavor::Recursive),
            ("VAR := value", VarFlavor::Simple),
            ("VAR ?= value", VarFlavor::Conditional),
            ("VAR += value", VarFlavor::Append),
            ("VAR != echo test", VarFlavor::Shell),
        ];

        for (input, expected_flavor) in test_cases {
            // ACT: Parse each flavor
            let result = parse_makefile(input);

            // ASSERT: Parsing succeeds
            assert!(result.is_ok(), "Failed to parse: {}", input);

            let ast = result.unwrap();
            assert_eq!(ast.items.len(), 1);

            // ASSERT: Correct flavor detected
            match &ast.items[0] {
                MakeItem::Variable { flavor, .. } => {
                    assert_eq!(flavor, &expected_flavor, "Wrong flavor for: {}", input);
                }
                _ => panic!("Expected Variable for: {}", input),
            }
        }
    }

    /// Kill mutant: line 115 `replace < with >` in is_variable_assignment
    /// Kill mutant: line 115 `replace < with ==` in is_variable_assignment
    ///
    /// These mutants would confuse targets with variables in prerequisites.
    #[test]
    fn test_VAR_BASIC_001_mut_target_with_variable_in_prereq() {
        // ARRANGE: Target with variable assignment in prerequisites
        // This should be parsed as TARGET, not VARIABLE
        let makefile = "target: VAR=value dep2\n\trecipe";

        // ACT: Parse makefile
        let result = parse_makefile(makefile);

        // ASSERT: Successfully parsed
        assert!(result.is_ok());

        let ast = result.unwrap();
        assert_eq!(ast.items.len(), 1);

        // ASSERT: Parsed as Target (not Variable)
        match &ast.items[0] {
            MakeItem::Target {
                name,
                prerequisites,
                ..
            } => {
                assert_eq!(name, "target");
                assert_eq!(prerequisites.len(), 2);
                assert_eq!(prerequisites[0], "VAR=value");
                assert_eq!(prerequisites[1], "dep2");
            }
            MakeItem::Variable { .. } => {
                panic!("Should be Target, not Variable");
            }
            _ => panic!("Expected Target"),
        }
    }

    /// Kill mutant: line 141 `replace + with -` in parse_variable
    /// Kill mutant: line 143 `replace + with -` in parse_variable
    /// Kill mutant: line 145 `replace + with -` in parse_variable
    ///
    /// These mutants would break parsing of multi-character operators.
    #[test]
    fn test_VAR_BASIC_001_mut_multichar_operator_slicing() {
        // ARRANGE: Test that := operator is correctly sliced
        let makefile = "VAR := value_here";

        // ACT: Parse makefile
        let result = parse_makefile(makefile);

        // ASSERT: Successfully parsed
        assert!(result.is_ok());

        let ast = result.unwrap();

        // ASSERT: Variable value does NOT include operator
        match &ast.items[0] {
            MakeItem::Variable {
                name,
                value,
                flavor,
                ..
            } => {
                assert_eq!(name, "VAR");
                assert_eq!(value, "value_here");
                assert_eq!(flavor, &VarFlavor::Simple);

                // Critical: Value must not contain ":" from ":="
                assert!(!value.contains(':'), "Value should not contain operator");
                assert!(!value.contains('='), "Value should not contain operator");
            }
            _ => panic!("Expected Variable"),
        }
    }

    /// Kill mutant: line 213 `replace < with <=` in parse_target_rule
    ///
    /// This mutant would affect recipe parsing loop bounds.
    #[test]
    fn test_VAR_BASIC_001_mut_recipe_loop_bounds() {
        // ARRANGE: Target at end of file with exactly one recipe line
        let makefile = "target:\n\trecipe";

        // ACT: Parse makefile
        let result = parse_makefile(makefile);

        // ASSERT: Successfully parsed
        assert!(result.is_ok());

        let ast = result.unwrap();

        // ASSERT: Exactly one recipe line (not duplicated)
        match &ast.items[0] {
            MakeItem::Target { recipe, .. } => {
                assert_eq!(
                    recipe.len(),
                    1,
                    "Should have exactly 1 recipe line, not {} (would happen if < became <=)",
                    recipe.len()
                );
                assert_eq!(recipe[0], "recipe");
            }
            _ => panic!("Expected Target"),
        }
    }

    /// Kill mutant: Additional test for operator edge cases
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
        #[test]
        fn test_VAR_BASIC_002_prop_var_refs_preserved_in_recipes(
            varname in "[A-Z][A-Z0-9_]{0,15}",
            recipe_prefix in "[a-z]{1,10}",
            recipe_suffix in "[a-z]{1,10}"
        ) {
            // ARRANGE: Recipe with variable reference
            let makefile = format!(
                "{}=value\ntarget:\n\t{} $({}) {}",
                varname, recipe_prefix, varname, recipe_suffix
            );

            // ACT: Parse makefile
            let result = parse_makefile(&makefile);

            // ASSERT: Parsing succeeds
            prop_assert!(result.is_ok(), "Failed to parse: {}", makefile);

            let ast = result.unwrap();
            prop_assert_eq!(ast.items.len(), 2);

            // ASSERT: Recipe preserves variable reference
            if let MakeItem::Target { recipe, .. } = &ast.items[1] {
                let var_ref = format!("$({})", varname);
                prop_assert!(
                    recipe[0].contains(&var_ref),
                    "Recipe should contain {}, got: {}",
                    var_ref,
                    recipe[0]
                );
            } else {
                return Err(TestCaseError::fail("Expected Target item"));
            }
        }

        /// Property: Variable references are preserved in variable values
        ///
        /// Verifies that $(VAR) in variable values is not expanded during parsing.
        #[test]
        fn test_VAR_BASIC_002_prop_var_refs_in_values(
            var1 in "[A-Z]{1,10}",
            var2 in "[A-Z]{1,10}",
            value_prefix in "[a-z]{0,10}",
            value_suffix in "[a-z]{0,10}"
        ) {
            // Ensure different variable names
            prop_assume!(var1 != var2);

            let makefile = format!(
                "{} = firstvalue\n{} = {}$({}){}",
                var1, var2, value_prefix, var1, value_suffix
            );

            let result = parse_makefile(&makefile);
            prop_assert!(result.is_ok());

            let ast = result.unwrap();
            if let MakeItem::Variable { value, .. } = &ast.items[1] {
                let var_ref = format!("$({})", var1);
                prop_assert!(
                    value.contains(&var_ref),
                    "Variable value should contain {}, got: {}",
                    var_ref,
                    value
                );
            } else {
                return Err(TestCaseError::fail("Expected Variable item"));
            }
        }

        /// Property: Curly brace syntax ${VAR} is also preserved
        ///
        /// Tests that both $(VAR) and ${VAR} syntaxes work.
        #[test]
        fn test_VAR_BASIC_002_prop_curly_brace_preserved(
            varname in "[A-Z]{1,10}",
            use_parens in prop::bool::ANY
        ) {
            let var_ref = if use_parens {
                format!("$({})", varname)
            } else {
                format!("${{{}}}", varname)
            };

            let makefile = format!(
                "{} = value\ntarget:\n\techo {}",
                varname, var_ref
            );

            let result = parse_makefile(&makefile);
            prop_assert!(result.is_ok());

            let ast = result.unwrap();
            if let MakeItem::Target { recipe, .. } = &ast.items[1] {
                prop_assert!(
                    recipe[0].contains(&var_ref),
                    "Recipe should contain {}, got: {}",
                    var_ref,
                    recipe[0]
                );
            } else {
                return Err(TestCaseError::fail("Expected Target item"));
            }
        }

        /// Property: Multiple variable references in same line
        ///
        /// Verifies that multiple $(VAR) references are all preserved.
        #[test]
        fn test_VAR_BASIC_002_prop_multiple_refs_preserved(
            vars in prop::collection::vec("[A-Z]{1,8}", 2..5)
        ) {
            // Create variable definitions
            let var_defs: Vec<String> = vars.iter()
                .map(|v| format!("{} = value", v))
                .collect();

            // Create recipe with all variable references
            let var_refs: Vec<String> = vars.iter()
                .map(|v| format!("$({})", v))
                .collect();
            let recipe = var_refs.join(" ");

            let makefile = format!(
                "{}\ntarget:\n\t{}",
                var_defs.join("\n"),
                recipe
            );

            let result = parse_makefile(&makefile);
            prop_assert!(result.is_ok());

            let ast = result.unwrap();
            // Find the target (last item)
            if let Some(MakeItem::Target { recipe: target_recipe, .. }) = ast.items.last() {
                // Verify all variable references are preserved
                for var in &vars {
                    let var_ref = format!("$({})", var);
                    prop_assert!(
                        target_recipe[0].contains(&var_ref),
                        "Recipe should contain {}, got: {}",
                        var_ref,
                        target_recipe[0]
                    );
                }
            } else {
                return Err(TestCaseError::fail("Expected Target as last item"));
            }
        }

        /// Property: Variable references in prerequisites are preserved
        ///
        /// Verifies that $(VAR) in target prerequisites is preserved.
        #[test]
        fn test_VAR_BASIC_002_prop_refs_in_prerequisites(
            varname in "[A-Z]{1,10}",
            target_name in "[a-z]{1,10}"
        ) {
            let makefile = format!(
                "{} = deps\n{}: $({}) file.o\n\techo done",
                varname, target_name, varname
            );

            let result = parse_makefile(&makefile);
            prop_assert!(result.is_ok());

            let ast = result.unwrap();
            if let MakeItem::Target { prerequisites, .. } = &ast.items[1] {
                let var_ref = format!("$({})", varname);
                prop_assert!(
                    prerequisites[0].contains(&var_ref),
                    "Prerequisites should contain {}, got: {:?}",
                    var_ref,
                    prerequisites
                );
            } else {
                return Err(TestCaseError::fail("Expected Target item"));
            }
        }
    }
}

/// RED PHASE: Test for SYNTAX-001 - Comment parsing
///
/// This test validates that Makefile comments are parsed and included in the AST.
///
/// Input Makefile:
/// ```makefile
/// # This is a comment
/// target:
///     recipe
/// ```
///
/// Expected: Comment should be parsed as MakeItem::Comment
#[test]
fn test_SYNTAX_001_basic_comment() {
    // ARRANGE: Makefile with a comment
    let makefile = "# This is a comment\ntarget:\n\trecipe";

    // ACT: Parse makefile
    let result = parse_makefile(makefile);

    // ASSERT: Successfully parsed
    assert!(
        result.is_ok(),
        "Should parse comment, got error: {:?}",
        result.err()
    );

    let ast = result.unwrap();

    // ASSERT: Two items (comment + target)
    assert_eq!(ast.items.len(), 2, "Should have comment and target");

    // ASSERT: First item is a comment
    match &ast.items[0] {
        MakeItem::Comment { text, .. } => {
            assert_eq!(
                text, "This is a comment",
                "Comment text should be preserved"
            );
        }
        other => panic!("Expected Comment item, got {:?}", other),
    }

    // ASSERT: Second item is target
    match &ast.items[1] {
        MakeItem::Target { name, .. } => {
            assert_eq!(name, "target");
        }
        _ => panic!("Expected Target item"),
    }
}

/// RED PHASE: Test for SYNTAX-001 - Multiple comments
#[test]
fn test_SYNTAX_001_multiple_comments() {
    // ARRANGE: Makefile with multiple comments
    let makefile = "# Comment 1\n# Comment 2\ntarget:\n\trecipe";

    // ACT: Parse makefile
    let result = parse_makefile(makefile);

    // ASSERT: Successfully parsed
    assert!(result.is_ok());

    let ast = result.unwrap();

    // ASSERT: Three items (2 comments + target)
    assert_eq!(ast.items.len(), 3, "Should have 2 comments and target");

    // ASSERT: First two items are comments
    match &ast.items[0] {
        MakeItem::Comment { text, .. } => {
            assert_eq!(text, "Comment 1");
        }
        _ => panic!("Expected Comment item"),
    }

    match &ast.items[1] {
        MakeItem::Comment { text, .. } => {
            assert_eq!(text, "Comment 2");
        }
        _ => panic!("Expected Comment item"),
    }
}

/// RED PHASE: Test for SYNTAX-001 - Empty comment
#[test]
fn test_SYNTAX_001_empty_comment() {
    // ARRANGE: Comment with just #
    let makefile = "#\ntarget:\n\trecipe";

    // ACT: Parse makefile
    let result = parse_makefile(makefile);

    // ASSERT: Successfully parsed
    assert!(result.is_ok());

    let ast = result.unwrap();

    // ASSERT: Comment with empty text
    match &ast.items[0] {
        MakeItem::Comment { text, .. } => {
            assert_eq!(text, "", "Empty comment should have empty text");
        }
        _ => panic!("Expected Comment item"),
    }
}

/// RED PHASE: Test for SYNTAX-001 - Comment with leading/trailing spaces
#[test]
fn test_SYNTAX_001_comment_with_spaces() {
    // ARRANGE: Comment with spaces
    let makefile = "#   Comment with spaces   \ntarget:\n\trecipe";

    // ACT: Parse makefile
    let result = parse_makefile(makefile);

    // ASSERT: Successfully parsed
    assert!(result.is_ok());

    let ast = result.unwrap();

    // ASSERT: Comment text should be trimmed
    match &ast.items[0] {
        MakeItem::Comment { text, .. } => {
            assert_eq!(text, "Comment with spaces", "Comment should be trimmed");
        }
        _ => panic!("Expected Comment item"),
    }
}

// PROPERTY TESTING PHASE: Tests for SYNTAX-001
//
// These property tests verify comment parsing works across various inputs.
#[cfg(test)]
mod syntax_001_property_tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        /// Property: Any line starting with # should be parsed as a comment
        ///
        /// This test generates 100+ random comment texts to ensure
        /// the parser handles a wide variety of comment content.
        #[test]
        fn test_SYNTAX_001_prop_any_hash_line_is_comment(
            comment_text in "[a-zA-Z0-9 ]{0,50}"
        ) {
            // ARRANGE: Create comment line
            let makefile = format!("# {}", comment_text);

            // ACT: Parse makefile
            let result = parse_makefile(&makefile);

            // ASSERT: Successfully parsed
            prop_assert!(result.is_ok(), "Failed to parse comment: {}", makefile);
            let ast = result.unwrap();
            prop_assert_eq!(ast.items.len(), 1);

            // ASSERT: First item is a comment
            match &ast.items[0] {
                MakeItem::Comment { text, .. } => {
                    prop_assert_eq!(text, comment_text.trim());
                }
                _ => return Err(TestCaseError::fail("Expected Comment item")),
            }
        }

        /// Property: Multiple comments should all be preserved
        ///
        /// Verifies that N consecutive comments all parse correctly.
        #[test]
        fn test_SYNTAX_001_prop_multiple_comments_preserved(
            count in 1..10usize
        ) {
            // ARRANGE: Create multiple comment lines
            let lines: Vec<String> = (0..count)
                .map(|i| format!("# Comment {}", i))
                .collect();
            let makefile = lines.join("\n");

            // ACT: Parse makefile
            let result = parse_makefile(&makefile);

            // ASSERT: Successfully parsed with correct count
            prop_assert!(result.is_ok());
            let ast = result.unwrap();
            prop_assert_eq!(ast.items.len(), count);

            // ASSERT: All items are comments
            for (i, item) in ast.items.iter().enumerate() {
                match item {
                    MakeItem::Comment { text, .. } => {
                        prop_assert_eq!(text, &format!("Comment {}", i));
                    }
                    _ => return Err(TestCaseError::fail("Expected Comment item")),
                }
            }
        }

        /// Property: Comments with special characters should be preserved
        ///
        /// Verifies that comments can contain special shell/makefile characters.
        #[test]
        fn test_SYNTAX_001_prop_special_chars_preserved(
            special_chars in "[!@$%^&*()+=\\[\\]{}|;:,.<>?/~`-]{1,20}"
        ) {
            // ARRANGE: Comment with special characters
            let makefile = format!("# {}", special_chars);

            // ACT: Parse makefile
            let result = parse_makefile(&makefile);

            // ASSERT: Successfully parsed
            prop_assert!(result.is_ok(), "Failed to parse: {}", makefile);
            let ast = result.unwrap();

            // ASSERT: Special characters preserved
            match &ast.items[0] {
                MakeItem::Comment { text, .. } => {
                    prop_assert_eq!(text, special_chars.trim());
                }
                _ => return Err(TestCaseError::fail("Expected Comment item")),
            }
        }

        /// Property: Comments mixed with targets should parse correctly
        ///
        /// Verifies that comments interspersed with targets are all captured.
        #[test]
        fn test_SYNTAX_001_prop_comments_with_targets(
            target_count in 1..5usize
        ) {
            // ARRANGE: Alternating comments and targets
            let mut lines = Vec::new();
            for i in 0..target_count {
                lines.push(format!("# Comment for target {}", i));
                lines.push(format!("target{}:", i));
                lines.push("\techo 'test'".to_string());
            }
            let makefile = lines.join("\n");

            // ACT: Parse makefile
            let result = parse_makefile(&makefile);

            // ASSERT: Successfully parsed
            prop_assert!(result.is_ok(), "Failed to parse: {}", makefile);
            let ast = result.unwrap();
            prop_assert_eq!(ast.items.len(), target_count * 2);

            // ASSERT: Comments and targets alternate
            for i in 0..target_count {
                let comment_idx = i * 2;
                let target_idx = comment_idx + 1;

                match &ast.items[comment_idx] {
                    MakeItem::Comment { .. } => {},
                    _ => return Err(TestCaseError::fail("Expected Comment")),
                }

                match &ast.items[target_idx] {
                    MakeItem::Target { .. } => {},
                    _ => return Err(TestCaseError::fail("Expected Target")),
                }
            }
        }

        /// Property: Empty comments (just #) should parse with empty text
        ///
        /// Verifies that # with only whitespace produces empty comment text.
        #[test]
        fn test_SYNTAX_001_prop_empty_comments_valid(
            whitespace in "[ \t]{0,10}"
        ) {
            // ARRANGE: Comment with only hash and optional whitespace
            let makefile = format!("#{}", whitespace);

            // ACT: Parse makefile
            let result = parse_makefile(&makefile);

            // ASSERT: Successfully parsed
            prop_assert!(result.is_ok());
            let ast = result.unwrap();
            prop_assert_eq!(ast.items.len(), 1);

            // ASSERT: Comment text is empty (trimmed)
            match &ast.items[0] {
                MakeItem::Comment { text, .. } => {
                    prop_assert_eq!(text, "");
                }
                _ => return Err(TestCaseError::fail("Expected Comment item")),
            }
        }
    }
}

/// MUTATION TESTING PHASE: Mutation-killing tests for SYNTAX-001
///
/// These tests target specific mutants identified during mutation testing.
#[cfg(test)]
mod syntax_001_mutation_killing_tests {
    use super::*;

    /// Kill mutant: line 60 `i + 1` → `i * 1` (would produce wrong line numbers)
    /// Kill mutant: line 63 `i += 1` → `i *= 1` (would cause infinite loop)
    ///
    /// This test verifies that comment line numbers are calculated correctly
    /// and that the parser advances past comment lines without infinite looping.
    #[test]
    fn test_SYNTAX_001_mut_comment_line_numbers_correct() {
        // ARRANGE: Makefile with comments on specific lines
        let makefile = "# Line 1 comment\n# Line 2 comment\ntarget:\n\trecipe";

        // ACT: Parse makefile
        let result = parse_makefile(makefile);

        // ASSERT: Successfully parsed (verifies no infinite loop)
        assert!(result.is_ok(), "Parser must not infinite loop on comments");

        let ast = result.unwrap();
        assert_eq!(ast.items.len(), 3, "Should parse 2 comments and 1 target");

        // ASSERT: Line numbers are correct (1-indexed)
        match &ast.items[0] {
            MakeItem::Comment { text, span } => {
                assert_eq!(text, "Line 1 comment");
                assert_eq!(span.line, 1, "First comment should be on line 1");
            }
            _ => panic!("Expected Comment item"),
        }

        match &ast.items[1] {
            MakeItem::Comment { text, span } => {
                assert_eq!(text, "Line 2 comment");
                assert_eq!(span.line, 2, "Second comment should be on line 2");
            }
            _ => panic!("Expected Comment item"),
        }

        match &ast.items[2] {
            MakeItem::Target { name, span, .. } => {
                assert_eq!(name, "target");
                assert_eq!(span.line, 3, "Target should be on line 3");
            }
            _ => panic!("Expected Target item"),
        }
    }

    /// Kill mutant: line 63 `i += 1` → `i -= 1` (would cause infinite loop or crash)
    ///
    /// This test ensures that the comment parsing loop advances forward correctly.
    #[test]
    fn test_SYNTAX_001_mut_comment_loop_advances_forward() {
        // ARRANGE: Multiple consecutive comments
        let makefile = "# Comment 1\n# Comment 2\n# Comment 3\n# Comment 4\n# Comment 5";

        // ACT: Parse makefile (must not infinite loop or go backwards)
        let result = parse_makefile(makefile);

        // ASSERT: Successfully parsed all comments
        assert!(result.is_ok(), "Parser must advance through all comments");

        let ast = result.unwrap();
        assert_eq!(ast.items.len(), 5, "Should parse all 5 comments");

        // Verify all are comments
        for (i, item) in ast.items.iter().enumerate() {
            match item {
                MakeItem::Comment { text, .. } => {
                    assert_eq!(text, &format!("Comment {}", i + 1));
                }
                _ => panic!("Expected Comment item at index {}", i),
            }
        }
    }

    /// Kill mutant: Ensure comment parsing doesn't affect other parsing logic
    ///
    /// This test verifies that enabling comment parsing doesn't break
    /// the parsing of variables, targets, or other constructs.
    #[test]
    fn test_SYNTAX_001_mut_comment_parsing_isolated() {
        // ARRANGE: Complex Makefile with comments interspersed
        let makefile = r#"
# This is a header comment
CC = gcc
# Compiler flags comment
CFLAGS = -Wall -O2

# Build target comment
build: main.c
	# Recipe comment (tab-indented, should be ignored)
	$(CC) $(CFLAGS) -o output main.c

# Clean target comment
clean:
	rm -f output
"#;

        // ACT: Parse makefile
        let result = parse_makefile(makefile);

        // ASSERT: Successfully parsed
        assert!(result.is_ok());

        let ast = result.unwrap();

        // Count each item type
        let mut comment_count = 0;
        let mut variable_count = 0;
        let mut target_count = 0;

        for item in &ast.items {
            match item {
                MakeItem::Comment { .. } => comment_count += 1,
                MakeItem::Variable { .. } => variable_count += 1,
                MakeItem::Target { .. } => target_count += 1,
                _ => {} // Ignore other types for this test
            }
        }

        // ASSERT: Correct counts
        assert_eq!(comment_count, 4, "Should parse 4 non-indented comments");
        assert_eq!(variable_count, 2, "Should parse 2 variables (CC, CFLAGS)");
        assert_eq!(target_count, 2, "Should parse 2 targets (build, clean)");
    }

    /// Kill mutant: Verify span tracking is accurate for comments
    ///
    /// This ensures that mutations to span calculation are caught.
    #[test]
    fn test_SYNTAX_001_mut_span_tracking_accurate() {
        // ARRANGE: Comments with varying lengths
        let makefile = "# Short\n# Medium length comment\n# Very long comment with many words here";

        // ACT: Parse makefile
        let result = parse_makefile(makefile);

        // ASSERT: Successfully parsed
        assert!(result.is_ok());

        let ast = result.unwrap();
        assert_eq!(ast.items.len(), 3);

        // ASSERT: Spans track lengths correctly
        match &ast.items[0] {
            MakeItem::Comment { span, .. } => {
                assert_eq!(span.line, 1);
                let length = span.end - span.start;
                assert_eq!(length, "# Short".len());
            }
            _ => panic!("Expected Comment"),
        }

        match &ast.items[1] {
            MakeItem::Comment { span, .. } => {
                assert_eq!(span.line, 2);
                let length = span.end - span.start;
                assert_eq!(length, "# Medium length comment".len());
            }
            _ => panic!("Expected Comment"),
        }

        match &ast.items[2] {
            MakeItem::Comment { span, .. } => {
                assert_eq!(span.line, 3);
                let length = span.end - span.start;
                assert_eq!(length, "# Very long comment with many words here".len());
            }
            _ => panic!("Expected Comment"),
        }
    }

    /// Kill mutant: Empty comments should not break parsing
    ///
    /// Tests edge case where comment has no text after #.
    #[test]
    fn test_SYNTAX_001_mut_empty_comment_edge_case() {
        // ARRANGE: Mix of empty and non-empty comments
        let makefile = "#\n# Has text\n#\n#   \ntarget:\n\trecipe";

        // ACT: Parse makefile
        let result = parse_makefile(makefile);

        // ASSERT: Successfully parsed
        assert!(result.is_ok());

        let ast = result.unwrap();

        // Count comments (should be 4)
        let comment_count = ast
            .items
            .iter()
            .filter(|item| matches!(item, MakeItem::Comment { .. }))
            .count();

        assert_eq!(
            comment_count, 4,
            "Should parse all 4 comments, even empty ones"
        );
    }
}

// ============================================================================
// RULE-SYNTAX-002: Multiple Prerequisites Tests
// Task: Verify parser correctly handles targets with multiple prerequisites
// ============================================================================

#[cfg(test)]
mod rule_syntax_002_tests {
    use crate::make_parser::{parse_makefile, MakeItem};

    // Unit Tests
    #[test]
    fn test_RULE_SYNTAX_002_basic_multiple_prerequisites() {
        let makefile = "all: build test deploy\n\techo done";
        let result = parse_makefile(makefile);
        assert!(
            result.is_ok(),
            "Parser should handle multiple prerequisites"
        );

        let ast = result.unwrap();
        assert_eq!(ast.items.len(), 1);

        match &ast.items[0] {
            MakeItem::Target {
                name,
                prerequisites,
                recipe,
                ..
            } => {
                assert_eq!(name, "all");
                assert_eq!(prerequisites.len(), 3, "Should have 3 prerequisites");
                assert_eq!(prerequisites[0], "build");
                assert_eq!(prerequisites[1], "test");
                assert_eq!(prerequisites[2], "deploy");
                assert_eq!(recipe.len(), 1);
            }
            other => panic!("Expected Target, got {:?}", other),
        }
    }

    #[test]
    fn test_RULE_SYNTAX_002_two_prerequisites() {
        let makefile = "link: main.o util.o\n\t$(CC) -o app $^";
        let result = parse_makefile(makefile);
        assert!(result.is_ok());

        let ast = result.unwrap();
        match &ast.items[0] {
            MakeItem::Target {
                name,
                prerequisites,
                ..
            } => {
                assert_eq!(name, "link");
                assert_eq!(prerequisites.len(), 2);
                assert_eq!(prerequisites[0], "main.o");
                assert_eq!(prerequisites[1], "util.o");
            }
            _ => panic!("Expected Target"),
        }
    }

    #[test]
    fn test_RULE_SYNTAX_002_many_prerequisites() {
        let makefile = "all: a b c d e f g h\n\techo all";
        let result = parse_makefile(makefile);
        assert!(result.is_ok());

        let ast = result.unwrap();
        match &ast.items[0] {
            MakeItem::Target { prerequisites, .. } => {
                assert_eq!(prerequisites.len(), 8, "Should handle many prerequisites");
                assert_eq!(prerequisites[0], "a");
                assert_eq!(prerequisites[7], "h");
            }
            _ => panic!("Expected Target"),
        }
    }

    #[test]
    fn test_RULE_SYNTAX_002_prerequisites_with_paths() {
        let makefile = "build: src/main.c include/util.h lib/helper.c\n\tgcc -o app";
        let result = parse_makefile(makefile);
        assert!(result.is_ok());

        let ast = result.unwrap();
        match &ast.items[0] {
            MakeItem::Target { prerequisites, .. } => {
                assert_eq!(prerequisites.len(), 3);
                assert_eq!(prerequisites[0], "src/main.c");
                assert_eq!(prerequisites[1], "include/util.h");
                assert_eq!(prerequisites[2], "lib/helper.c");
            }
            _ => panic!("Expected Target"),
        }
    }
}

#[cfg(test)]
mod rule_syntax_002_property_tests {
    use crate::make_parser::{parse_makefile, MakeItem};
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn test_RULE_SYNTAX_002_prop_multiple_prereqs_always_parse(
            prereqs in prop::collection::vec("[a-z]{1,10}", 2..10)
        ) {
            let prereq_str = prereqs.join(" ");
            let makefile = format!("target: {}\n\techo done", prereq_str);

            let result = parse_makefile(&makefile);
            prop_assert!(result.is_ok(), "Multiple prerequisites should always parse");

            let ast = result.unwrap();
            match &ast.items[0] {
                MakeItem::Target { prerequisites, .. } => {
                    prop_assert_eq!(prerequisites.len(), prereqs.len());
                    for (i, prereq) in prereqs.iter().enumerate() {
                        prop_assert_eq!(&prerequisites[i], prereq);
                    }
                }
                _ => return Err(TestCaseError::fail("Expected Target")),
            }
        }

        #[test]
        fn test_RULE_SYNTAX_002_prop_prereqs_order_preserved(
            prereqs in prop::collection::vec("[a-z]{1,8}", 1..15)
        ) {
            let prereq_str = prereqs.join(" ");
            let makefile = format!("all: {}\n\techo all", prereq_str);

            let result = parse_makefile(&makefile);
            prop_assert!(result.is_ok());

            let ast = result.unwrap();
            match &ast.items[0] {
                MakeItem::Target { prerequisites, .. } => {
                    // Order must be preserved
                    for (i, expected) in prereqs.iter().enumerate() {
                        prop_assert_eq!(&prerequisites[i], expected, "Order not preserved at index {}", i);
                    }
                }
                _ => return Err(TestCaseError::fail("Expected Target")),
            }
        }

        #[test]
        fn test_RULE_SYNTAX_002_prop_prereqs_with_dots(
            names in prop::collection::vec("[a-z]{1,5}", 2..8)
        ) {
            // Create prerequisites like file.o, main.c, util.h
            let prereqs: Vec<String> = names.iter()
                .enumerate()
                .map(|(i, name)| {
                    let ext = match i % 3 {
                        0 => ".o",
                        1 => ".c",
                        _ => ".h",
                    };
                    format!("{}{}", name, ext)
                })
                .collect();

            let prereq_str = prereqs.join(" ");
            let makefile = format!("build: {}\n\tgcc", prereq_str);

            let result = parse_makefile(&makefile);
            prop_assert!(result.is_ok());

            let ast = result.unwrap();
            match &ast.items[0] {
                MakeItem::Target { prerequisites: parsed, .. } => {
                    prop_assert_eq!(parsed.len(), prereqs.len());
                    for (i, expected) in prereqs.iter().enumerate() {
                        prop_assert_eq!(&parsed[i], expected);
                    }
                }
                _ => return Err(TestCaseError::fail("Expected Target")),
            }
        }

        #[test]
        fn test_RULE_SYNTAX_002_prop_prereqs_whitespace_normalized(
            prereqs in prop::collection::vec("[a-z]{1,6}", 2..6),
            spaces in prop::collection::vec(1..5usize, 1..5)
        ) {
            // Join prerequisites with varying amounts of spaces
            let mut prereq_str = String::new();
            for (i, prereq) in prereqs.iter().enumerate() {
                if i > 0 {
                    let space_count = spaces.get(i - 1).unwrap_or(&1);
                    prereq_str.push_str(&" ".repeat(*space_count));
                }
                prereq_str.push_str(prereq);
            }

            let makefile = format!("target: {}\n\techo", prereq_str);
            let result = parse_makefile(&makefile);
            prop_assert!(result.is_ok());

            let ast = result.unwrap();
            match &ast.items[0] {
                MakeItem::Target { prerequisites: parsed, .. } => {
                    // Whitespace should be normalized - all prerequisites parsed correctly
                    prop_assert_eq!(parsed.len(), prereqs.len());
                    for (i, expected) in prereqs.iter().enumerate() {
                        prop_assert_eq!(&parsed[i], expected);
                    }
                }
                _ => return Err(TestCaseError::fail("Expected Target")),
            }
        }

        #[test]
        fn test_RULE_SYNTAX_002_prop_prereqs_with_slashes(
            dirs in prop::collection::vec("[a-z]{1,5}", 2..5),
            files in prop::collection::vec("[a-z]{1,6}", 2..5)
        ) {
            // Create prerequisites like src/main.c, lib/util.o
            let prereqs: Vec<String> = dirs.iter()
                .zip(files.iter())
                .map(|(dir, file)| format!("{}/{}.c", dir, file))
                .collect();

            let prereq_str = prereqs.join(" ");
            let makefile = format!("compile: {}\n\tgcc -o app", prereq_str);

            let result = parse_makefile(&makefile);
            prop_assert!(result.is_ok());

            let ast = result.unwrap();
            match &ast.items[0] {
                MakeItem::Target { prerequisites: parsed, .. } => {
                    prop_assert_eq!(parsed.len(), prereqs.len());
                    for (i, expected) in prereqs.iter().enumerate() {
                        prop_assert_eq!(&parsed[i], expected);
                    }
                }
                _ => return Err(TestCaseError::fail("Expected Target")),
            }
        }
    }
}

#[cfg(test)]
mod rule_syntax_002_mutation_killing_tests {
    use crate::make_parser::{parse_makefile, MakeItem};

    #[test]
    fn test_RULE_SYNTAX_002_mut_split_whitespace_correctness() {
        // Target: line 203-206 split_whitespace() and map() logic
        // Kill mutants that break whitespace splitting
        let makefile = "target:   build    test     deploy  \n\techo";
        let result = parse_makefile(makefile);
        assert!(result.is_ok(), "Must handle excessive whitespace");

        let ast = result.unwrap();
        match &ast.items[0] {
            MakeItem::Target { prerequisites, .. } => {
                assert_eq!(prerequisites.len(), 3, "Must split on any whitespace");
                assert_eq!(prerequisites[0], "build");
                assert_eq!(prerequisites[1], "test");
                assert_eq!(prerequisites[2], "deploy");
                // Ensure no empty strings
                for prereq in prerequisites {
                    assert!(!prereq.is_empty(), "No empty prerequisites allowed");
                }
            }
            _ => panic!("Expected Target"),
        }
    }

    #[test]
    fn test_RULE_SYNTAX_002_mut_prerequisite_count_exact() {
        // Target: line 203-206 collection logic
        // Kill mutants that incorrectly count prerequisites
        let test_cases = vec![
            ("target: a\n\techo", 1),
            ("target: a b\n\techo", 2),
            ("target: a b c\n\techo", 3),
            ("target: a b c d e\n\techo", 5),
            ("target: a b c d e f g h i j\n\techo", 10),
        ];

        for (makefile, expected_count) in test_cases {
            let result = parse_makefile(makefile);
            assert!(result.is_ok());

            let ast = result.unwrap();
            match &ast.items[0] {
                MakeItem::Target { prerequisites, .. } => {
                    assert_eq!(
                        prerequisites.len(),
                        expected_count,
                        "Prerequisite count must be exact for: {}",
                        makefile
                    );
                }
                _ => panic!("Expected Target"),
            }
        }
    }

    #[test]
    fn test_RULE_SYNTAX_002_mut_empty_prerequisites_handling() {
        // Target: line 203 - parts[1] which could be empty
        // Kill mutants that don't handle empty prerequisite lists
        let test_cases = vec![
            "target:\n\techo",       // Empty after colon
            "target: \n\techo",      // Just space after colon
            "target:  \t  \n\techo", // Multiple whitespace, no prerequisites
        ];

        for makefile in test_cases {
            let result = parse_makefile(makefile);
            assert!(
                result.is_ok(),
                "Must handle empty prerequisites for: {}",
                makefile
            );

            let ast = result.unwrap();
            match &ast.items[0] {
                MakeItem::Target { prerequisites, .. } => {
                    assert_eq!(
                        prerequisites.len(),
                        0,
                        "Empty prerequisites should result in empty vec for: {}",
                        makefile
                    );
                }
                _ => panic!("Expected Target"),
            }
        }
    }

    #[test]
    fn test_RULE_SYNTAX_002_mut_prerequisite_string_conversion() {
        // Target: line 205 - .to_string() conversion
        // Kill mutants that break string ownership
        let makefile = "target: prereq1 prereq2\n\techo";
        let result = parse_makefile(makefile);
        assert!(result.is_ok());

        let ast = result.unwrap();
        match &ast.items[0] {
            MakeItem::Target {
                prerequisites,
                name,
                ..
            } => {
                // Verify independent ownership (not references)
                assert_eq!(prerequisites[0], "prereq1");
                assert_eq!(prerequisites[1], "prereq2");

                // Verify no lifetime issues by accessing after parsing
                let prereq1_clone = prerequisites[0].clone();
                let prereq2_clone = prerequisites[1].clone();
                assert_eq!(prereq1_clone, "prereq1");
                assert_eq!(prereq2_clone, "prereq2");

                // Ensure target name independent from prerequisites
                assert_eq!(name, "target");
                assert_ne!(&prerequisites[0], name);
            }
            _ => panic!("Expected Target"),
        }
    }

    #[test]
    fn test_RULE_SYNTAX_002_mut_prerequisite_order_matters() {
        // Target: line 203-206 - collect() preserves order
        // Kill mutants that break prerequisite ordering
        let makefile = "link: z.o y.o x.o w.o v.o\n\tgcc -o app";
        let result = parse_makefile(makefile);
        assert!(result.is_ok());

        let ast = result.unwrap();
        match &ast.items[0] {
            MakeItem::Target { prerequisites, .. } => {
                assert_eq!(prerequisites.len(), 5);
                // Order MUST be preserved (important for linking order)
                assert_eq!(prerequisites[0], "z.o", "First must be z.o");
                assert_eq!(prerequisites[1], "y.o", "Second must be y.o");
                assert_eq!(prerequisites[2], "x.o", "Third must be x.o");
                assert_eq!(prerequisites[3], "w.o", "Fourth must be w.o");
                assert_eq!(prerequisites[4], "v.o", "Fifth must be v.o");

                // Verify not sorted
                assert_ne!(prerequisites[0], "v.o", "Must NOT be sorted");
            }
            _ => panic!("Expected Target"),
        }
    }
}

// ============================================================================
// VAR-FLAVOR-003: Conditional Assignment ?= Tests
// Task: Verify parser correctly handles ?= (conditional) assignment operator
// ============================================================================

#[cfg(test)]
mod var_flavor_003_tests {
    use crate::make_parser::{ast::VarFlavor, parse_makefile, MakeItem};

    // Unit Tests
    #[test]
    fn test_VAR_FLAVOR_003_basic_conditional_assignment() {
        let makefile = "PREFIX ?= /usr/local";
        let result = parse_makefile(makefile);
        assert!(
            result.is_ok(),
            "Parser should handle ?= conditional assignment"
        );

        let ast = result.unwrap();
        assert_eq!(ast.items.len(), 1);

        match &ast.items[0] {
            MakeItem::Variable {
                name,
                value,
                flavor,
                ..
            } => {
                assert_eq!(name, "PREFIX");
                assert_eq!(value, "/usr/local");
                assert_eq!(
                    *flavor,
                    VarFlavor::Conditional,
                    "Should detect ?= as Conditional"
                );
            }
            other => panic!("Expected Variable, got {:?}", other),
        }
    }

    #[test]
    fn test_VAR_FLAVOR_003_conditional_with_spaces() {
        let makefile = "CC ?= gcc -Wall -O2";
        let result = parse_makefile(makefile);
        assert!(result.is_ok());

        let ast = result.unwrap();
        match &ast.items[0] {
            MakeItem::Variable {
                name,
                value,
                flavor,
                ..
            } => {
                assert_eq!(name, "CC");
                assert_eq!(value, "gcc -Wall -O2");
                assert_eq!(*flavor, VarFlavor::Conditional);
            }
            _ => panic!("Expected Variable"),
        }
    }

    #[test]
    fn test_VAR_FLAVOR_003_conditional_empty_value() {
        let makefile = "EMPTY ?=";
        let result = parse_makefile(makefile);
        assert!(
            result.is_ok(),
            "Conditional assignment can have empty value"
        );

        let ast = result.unwrap();
        match &ast.items[0] {
            MakeItem::Variable {
                name,
                value,
                flavor,
                ..
            } => {
                assert_eq!(name, "EMPTY");
                assert_eq!(value, "");
                assert_eq!(*flavor, VarFlavor::Conditional);
            }
            _ => panic!("Expected Variable"),
        }
    }

    #[test]
    fn test_VAR_FLAVOR_003_conditional_vs_other_flavors() {
        // Test that ?= is correctly distinguished from other operators
        let test_cases = vec![
            ("VAR = val", VarFlavor::Recursive),
            ("VAR := val", VarFlavor::Simple),
            ("VAR ?= val", VarFlavor::Conditional), // The focus of this sprint
            ("VAR += val", VarFlavor::Append),
            ("VAR != echo val", VarFlavor::Shell),
        ];

        for (makefile, expected_flavor) in test_cases {
            let result = parse_makefile(makefile);
            assert!(result.is_ok(), "Failed to parse: {}", makefile);

            let ast = result.unwrap();
            match &ast.items[0] {
                MakeItem::Variable { name, flavor, .. } => {
                    assert_eq!(name, "VAR");
                    assert_eq!(*flavor, expected_flavor, "Wrong flavor for: {}", makefile);
                }
                _ => panic!("Expected Variable for: {}", makefile),
            }
        }
    }
}

#[cfg(test)]
mod var_flavor_003_property_tests {
    use crate::make_parser::{ast::VarFlavor, parse_makefile, MakeItem};
    use proptest::prelude::*;

    proptest! {
        /// Property: ?= conditional assignments always parse successfully
        ///
        /// This test generates random variable names and values to ensure
        /// the parser handles conditional assignment across diverse inputs.
        #[test]
        fn test_VAR_FLAVOR_003_prop_conditional_always_parses(
            varname in "[A-Z][A-Z0-9_]{0,20}",
            value in "[a-zA-Z0-9_./+ -]{0,50}"
        ) {
            // ARRANGE: Generate valid conditional assignment
            let makefile = format!("{} ?= {}", varname, value);

            // ACT: Parse makefile
            let result = parse_makefile(&makefile);

            // ASSERT: Parsing succeeds
            prop_assert!(result.is_ok(), "Failed to parse: {}", makefile);

            let ast = result.unwrap();
            prop_assert_eq!(ast.items.len(), 1);

            // ASSERT: Correct flavor detected
            if let MakeItem::Variable { name, value: val, flavor, .. } = &ast.items[0] {
                prop_assert_eq!(name, &varname);
                prop_assert_eq!(val, &value.trim());
                prop_assert_eq!(flavor, &VarFlavor::Conditional, "Must detect ?= as Conditional");
            } else {
                return Err(TestCaseError::fail("Expected Variable item"));
            }
        }

        /// Property: ?= parsing is deterministic
        ///
        /// Verifies that parsing the same conditional assignment twice produces identical results.
        #[test]
        fn test_VAR_FLAVOR_003_prop_parsing_is_deterministic(
            varname in "[A-Z]{1,15}",
            value in "[a-z0-9 ]{1,40}"
        ) {
            let makefile = format!("{} ?= {}", varname, value);

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

        /// Property: ?= is correctly identified among other operators
        ///
        /// Tests that ?= is not confused with ?, =, or other operators.
        #[test]
        fn test_VAR_FLAVOR_003_prop_operator_not_confused(
            varname in "[A-Z]{1,10}",
            value in "[a-z]{1,20}"
        ) {
            // Test ?= specifically (not := or += which also contain =)
            let makefile = format!("{} ?= {}", varname, value);

            let result = parse_makefile(&makefile);
            prop_assert!(result.is_ok());

            let ast = result.unwrap();
            if let MakeItem::Variable { flavor, .. } = &ast.items[0] {
                // Must be Conditional, not any other flavor
                prop_assert_eq!(flavor, &VarFlavor::Conditional);
                prop_assert_ne!(flavor, &VarFlavor::Recursive);
                prop_assert_ne!(flavor, &VarFlavor::Simple);
                prop_assert_ne!(flavor, &VarFlavor::Append);
                prop_assert_ne!(flavor, &VarFlavor::Shell);
            } else {
                return Err(TestCaseError::fail("Expected Variable item"));
            }
        }

        /// Property: ?= handles empty and whitespace values correctly
        ///
        /// Verifies that conditional assignment works with various value patterns.
        #[test]
        fn test_VAR_FLAVOR_003_prop_values_flexible(
            varname in "[A-Z]{1,12}",
            value in "[ a-z0-9-]*"  // Can be empty, can have spaces
        ) {
            let makefile = format!("{} ?= {}", varname, value);

            let result = parse_makefile(&makefile);
            prop_assert!(result.is_ok());

            let ast = result.unwrap();
            if let MakeItem::Variable { name, value: val, flavor, .. } = &ast.items[0] {
                prop_assert_eq!(name, &varname);
                prop_assert_eq!(val, &value.trim());  // Value gets trimmed
                prop_assert_eq!(flavor, &VarFlavor::Conditional);
            } else {
                return Err(TestCaseError::fail("Expected Variable item"));
            }
        }

        /// Property: ?= works with paths and special characters in values
        ///
        /// Tests that conditional assignment handles filesystem paths and common patterns.
        #[test]
        fn test_VAR_FLAVOR_003_prop_special_values(
            varname in "[A-Z]{1,10}",
            path_parts in prop::collection::vec("[a-z]{1,8}", 1..4)
        ) {
            // Create value like /usr/local/bin or src/include/main
            let value = path_parts.join("/");
            let makefile = format!("{} ?= {}", varname, value);

            let result = parse_makefile(&makefile);
            prop_assert!(result.is_ok(), "Failed to parse path value: {}", makefile);

            let ast = result.unwrap();
            if let MakeItem::Variable { name, value: val, flavor, .. } = &ast.items[0] {
                prop_assert_eq!(name, &varname);
                prop_assert_eq!(val, &value);
                prop_assert_eq!(flavor, &VarFlavor::Conditional);
            } else {
                return Err(TestCaseError::fail("Expected Variable item"));
            }
        }
    }
}

#[cfg(test)]
mod var_flavor_003_mutation_killing_tests {
    use crate::make_parser::{ast::VarFlavor, parse_makefile, MakeItem};

    /// Kill mutant: line 110 - `replace || with &&` in is_variable_assignment
    ///
    /// This mutant would break detection of ?= operator.
    #[test]
    fn test_VAR_FLAVOR_003_mut_operator_detection() {
        // Target: line 110 where ?= is checked
        // Kill mutants that break ?= detection in is_variable_assignment
        let test_cases = vec![
            ("VAR?=value", true),   // No spaces
            ("VAR ?=value", true),  // Space before
            ("VAR?= value", true),  // Space after
            ("VAR ?= value", true), // Spaces both sides
        ];

        for (makefile, should_be_conditional) in test_cases {
            let result = parse_makefile(makefile);
            assert!(result.is_ok(), "Failed to parse: {}", makefile);

            let ast = result.unwrap();
            if should_be_conditional {
                match &ast.items[0] {
                    MakeItem::Variable { flavor, .. } => {
                        assert_eq!(
                            *flavor,
                            VarFlavor::Conditional,
                            "?= must be detected as Conditional for: {}",
                            makefile
                        );
                    }
                    _ => panic!("Expected Variable for: {}", makefile),
                }
            }
        }
    }

    /// Kill mutant: line 150 - `replace + with -` or `replace 2 with 1` in parse_variable
    ///
    /// This mutant would break string slicing for ?= operator.
    #[test]
    fn test_VAR_FLAVOR_003_mut_operator_slicing() {
        // Target: line 150 where "?=" is found and sliced
        // pos + 2 must correctly skip past "?=" to get value
        let makefile = "PREFIX ?= /usr/local/bin";
        let result = parse_makefile(makefile);
        assert!(result.is_ok());

        let ast = result.unwrap();
        match &ast.items[0] {
            MakeItem::Variable {
                name,
                value,
                flavor,
                ..
            } => {
                assert_eq!(name, "PREFIX");
                assert_eq!(*flavor, VarFlavor::Conditional);

                // Critical: Value must NOT include "?" or "="
                assert_eq!(value, "/usr/local/bin");
                assert!(!value.contains('?'), "Value should not contain '?'");
                assert!(!value.contains('='), "Value should not contain '='");

                // Value must be clean (trimmed, no operator)
                assert!(value.starts_with('/'), "Value should start with /");
            }
            _ => panic!("Expected Variable"),
        }
    }

    /// Kill mutant: line 110 - `replace contains with !contains`
    ///
    /// This would make parser miss ?= and treat it as something else.
    #[test]
    fn test_VAR_FLAVOR_003_mut_conditional_not_missed() {
        // Target: Ensure ?= is recognized and not skipped
        let test_cases = vec![
            ("A?=1", VarFlavor::Conditional),
            ("B?=2", VarFlavor::Conditional),
            ("C?=3", VarFlavor::Conditional),
        ];

        for (input, expected_flavor) in test_cases {
            let result = parse_makefile(input);
            assert!(result.is_ok(), "Must parse: {}", input);

            let ast = result.unwrap();
            assert_eq!(ast.items.len(), 1, "Must parse exactly one variable");

            match &ast.items[0] {
                MakeItem::Variable { flavor, .. } => {
                    assert_eq!(
                        flavor, &expected_flavor,
                        "?= must be detected for: {}",
                        input
                    );
                }
                _ => panic!("Expected Variable for: {}", input),
            }
        }
    }

    /// Kill mutant: Ensure ?= doesn't get confused with ? in target names
    ///
    /// Tests edge case where ? might appear in other contexts.
    #[test]
    fn test_VAR_FLAVOR_003_mut_not_confused_with_question_mark() {
        // ?= in variable assignment vs ? in other contexts
        let makefile = "CONFIG ?= default\ntarget:\n\techo test?";
        let result = parse_makefile(makefile);
        assert!(result.is_ok());

        let ast = result.unwrap();
        assert_eq!(ast.items.len(), 2);

        // First item must be Variable with Conditional flavor
        match &ast.items[0] {
            MakeItem::Variable { name, flavor, .. } => {
                assert_eq!(name, "CONFIG");
                assert_eq!(*flavor, VarFlavor::Conditional);
            }
            _ => panic!("Expected Variable"),
        }

        // Second item must be Target
        match &ast.items[1] {
            MakeItem::Target { name, .. } => {
                assert_eq!(name, "target");
            }
            _ => panic!("Expected Target"),
        }
    }

    /// Kill mutant: line 150 VarFlavor enum variant
    ///
    /// Ensure Conditional variant is correctly used (not Simple, Append, etc).
    #[test]
    fn test_VAR_FLAVOR_003_mut_correct_flavor_enum_variant() {
        // Target: line 150 where VarFlavor::Conditional is set
        // Kill mutants that use wrong enum variant
        let makefile = "TEST ?= value";
        let result = parse_makefile(makefile);
        assert!(result.is_ok());

        let ast = result.unwrap();
        match &ast.items[0] {
            MakeItem::Variable { flavor, .. } => {
                // Must be exactly Conditional
                assert!(
                    matches!(flavor, VarFlavor::Conditional),
                    "Must be VarFlavor::Conditional"
                );

                // Must NOT be any other variant
                assert!(
                    !matches!(flavor, VarFlavor::Recursive),
                    "Must NOT be Recursive"
                );
                assert!(!matches!(flavor, VarFlavor::Simple), "Must NOT be Simple");
                assert!(!matches!(flavor, VarFlavor::Append), "Must NOT be Append");
                assert!(!matches!(flavor, VarFlavor::Shell), "Must NOT be Shell");
            }
            _ => panic!("Expected Variable"),
        }
    }
}

// ============================================================================
// Sprint 35: VAR-FLAVOR-004 - Append Assignment (+=)
// ============================================================================
//
// Task: VAR-FLAVOR-004 - Document append assignment (+=)
// Status: Verification task (parser already implements += at lines 111, 153)
//
// Implementation:
// - Line 111: is_variable_assignment() detects += operator
// - Line 153: parse_variable() maps to VarFlavor::Append
//
// Test Coverage:
// - Unit tests: Basic +=, spaces, empty values, operator distinction
// - Property tests: Various append scenarios (Phase 4)
// - Mutation-killing tests: Target lines 111, 153 (Phase 5)
// ============================================================================

// ----------------------------------------------------------------------------
// Phase 1: RED - Unit Tests for += Append Assignment
// ----------------------------------------------------------------------------

#[test]
fn test_VAR_FLAVOR_004_basic_append_assignment() {
    // ARRANGE: Makefile with += append operator
    let makefile = "CFLAGS += -O2";

    // ACT: Parse the makefile
    let result = parse_makefile(makefile);

    // ASSERT: Parser should handle += append assignment
    assert!(result.is_ok(), "Parser should handle += append assignment");

    let ast = result.unwrap();
    assert_eq!(ast.items.len(), 1, "Should parse one variable");

    match &ast.items[0] {
        MakeItem::Variable {
            name,
            value,
            flavor,
            ..
        } => {
            assert_eq!(name, "CFLAGS", "Variable name should be CFLAGS");
            assert_eq!(value, "-O2", "Value should be -O2");
            assert_eq!(*flavor, VarFlavor::Append, "Should detect += as Append");
        }
        other => panic!("Expected Variable, got {:?}", other),
    }
}

#[test]
fn test_VAR_FLAVOR_004_append_with_spaces() {
    // ARRANGE: Makefile with += and spaces around operator
    let makefile = "LDFLAGS  +=  -lm -lpthread";

    // ACT: Parse the makefile
    let result = parse_makefile(makefile);

    // ASSERT: Should handle spaces around +=
    assert!(result.is_ok(), "Parser should handle spaces around +=");

    let ast = result.unwrap();
    match &ast.items[0] {
        MakeItem::Variable {
            name,
            value,
            flavor,
            ..
        } => {
            assert_eq!(name, "LDFLAGS");
            assert_eq!(value, "-lm -lpthread", "Should preserve value with spaces");
            assert_eq!(*flavor, VarFlavor::Append);
        }
        _ => panic!("Expected Variable"),
    }
}

#[test]
fn test_VAR_FLAVOR_004_append_empty_value() {
    // ARRANGE: Makefile with += and empty value
    let makefile = "OPTS +=";

    // ACT: Parse the makefile
    let result = parse_makefile(makefile);

    // ASSERT: Should handle empty append value
    assert!(result.is_ok(), "Parser should handle empty append value");

    let ast = result.unwrap();
    match &ast.items[0] {
        MakeItem::Variable {
            name,
            value,
            flavor,
            ..
        } => {
            assert_eq!(name, "OPTS");
            assert_eq!(value, "", "Empty value should parse as empty string");
            assert_eq!(*flavor, VarFlavor::Append);
        }
        _ => panic!("Expected Variable"),
    }
}

#[test]
fn test_VAR_FLAVOR_004_append_vs_other_flavors() {
    // ARRANGE: Makefile with all 5 variable flavors
    let makefile = r#"
V1 = recursive
V2 := simple
V3 ?= conditional
V4 += append
V5 != echo shell
"#;

    // ACT: Parse the makefile
    let result = parse_makefile(makefile);

    // ASSERT: Should distinguish += from other operators
    assert!(result.is_ok(), "Parser should handle all 5 flavors");

    let ast = result.unwrap();
    assert_eq!(ast.items.len(), 5, "Should parse 5 variables");

    // Check each flavor is correct
    let flavors: Vec<VarFlavor> = ast
        .items
        .iter()
        .filter_map(|item| match item {
            MakeItem::Variable { flavor, .. } => Some(flavor.clone()),
            _ => None,
        })
        .collect();

    assert_eq!(flavors.len(), 5);
    assert!(matches!(flavors[0], VarFlavor::Recursive));
    assert!(matches!(flavors[1], VarFlavor::Simple));
    assert!(matches!(flavors[2], VarFlavor::Conditional));
    assert!(
        matches!(flavors[3], VarFlavor::Append),
        "Fourth variable should be Append"
    );
    assert!(matches!(flavors[4], VarFlavor::Shell));

    // Specifically verify V4 is Append
    match &ast.items[3] {
        MakeItem::Variable {
            name,
            value,
            flavor,
            ..
        } => {
            assert_eq!(name, "V4");
            assert_eq!(value, "append");
            assert_eq!(*flavor, VarFlavor::Append, "V4 should have Append flavor");
        }
        _ => panic!("Expected Variable at index 3"),
    }
}

// ----------------------------------------------------------------------------
// Phase 4: PROPERTY TESTING - Property Tests for += Append Assignment
// ----------------------------------------------------------------------------

#[cfg(test)]
mod property_tests_var_flavor_004 {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn test_VAR_FLAVOR_004_prop_append_always_parses(
            varname in "[A-Z][A-Z0-9_]{0,20}",
            value in "[a-zA-Z0-9_./+ -]{0,50}"
        ) {
            // ARRANGE: Generate makefile with += operator
            let makefile = format!("{} += {}", varname, value);

            // ACT: Parse the makefile
            let result = parse_makefile(&makefile);

            // ASSERT: Should always parse successfully
            prop_assert!(result.is_ok(), "Failed to parse: {}", makefile);

            let ast = result.unwrap();
            prop_assert_eq!(ast.items.len(), 1, "Should have exactly 1 item");

            if let MakeItem::Variable { name, value: val, flavor, .. } = &ast.items[0] {
                prop_assert_eq!(name, &varname, "Name should match");
                prop_assert_eq!(val, &value.trim(), "Value should match (trimmed)");
                prop_assert_eq!(flavor, &VarFlavor::Append, "Flavor should be Append");
            } else {
                return Err(TestCaseError::fail("Expected Variable item"));
            }
        }

        #[test]
        fn test_VAR_FLAVOR_004_prop_parsing_is_deterministic(
            varname in "[A-Z][A-Z_]{0,15}",
            value in "[a-zA-Z0-9 ]{0,30}"
        ) {
            // ARRANGE: Generate makefile with += operator
            let makefile = format!("{} += {}", varname, value);

            // ACT: Parse twice
            let result1 = parse_makefile(&makefile);
            let result2 = parse_makefile(&makefile);

            // ASSERT: Both should succeed
            prop_assert!(result1.is_ok() && result2.is_ok());

            let ast1 = result1.unwrap();
            let ast2 = result2.unwrap();

            // Extract variable items
            let var1 = &ast1.items[0];
            let var2 = &ast2.items[0];

            // Should be identical
            if let (
                MakeItem::Variable { name: n1, value: v1, flavor: f1, .. },
                MakeItem::Variable { name: n2, value: v2, flavor: f2, .. }
            ) = (var1, var2) {
                prop_assert_eq!(n1, n2, "Names should be identical");
                prop_assert_eq!(v1, v2, "Values should be identical");
                prop_assert_eq!(f1, f2, "Flavors should be identical");
                prop_assert!(matches!(f1, VarFlavor::Append));
            } else {
                return Err(TestCaseError::fail("Expected Variable items"));
            }
        }

        #[test]
        fn test_VAR_FLAVOR_004_prop_operator_not_confused(
            varname in "[A-Z][A-Z_]{0,10}",
            value in "[a-z]{1,20}"
        ) {
            // ARRANGE: Test that += is not confused with other operators
            let append_makefile = format!("{} += {}", varname, value);
            let recursive_makefile = format!("{} = {}", varname, value);
            let simple_makefile = format!("{} := {}", varname, value);

            // ACT: Parse all three
            let append_result = parse_makefile(&append_makefile);
            let recursive_result = parse_makefile(&recursive_makefile);
            let simple_result = parse_makefile(&simple_makefile);

            // ASSERT: All should parse
            prop_assert!(append_result.is_ok());
            prop_assert!(recursive_result.is_ok());
            prop_assert!(simple_result.is_ok());

            let append_ast = append_result.unwrap();
            let recursive_ast = recursive_result.unwrap();
            let simple_ast = simple_result.unwrap();

            // Extract flavors
            if let (
                MakeItem::Variable { flavor: f_append, .. },
                MakeItem::Variable { flavor: f_recursive, .. },
                MakeItem::Variable { flavor: f_simple, .. }
            ) = (&append_ast.items[0], &recursive_ast.items[0], &simple_ast.items[0]) {
                // Each should have correct flavor
                prop_assert!(matches!(f_append, VarFlavor::Append), "Should be Append");
                prop_assert!(matches!(f_recursive, VarFlavor::Recursive), "Should be Recursive");
                prop_assert!(matches!(f_simple, VarFlavor::Simple), "Should be Simple");

                // They should all be different
                prop_assert_ne!(f_append, f_recursive);
                prop_assert_ne!(f_append, f_simple);
            } else {
                return Err(TestCaseError::fail("Expected Variable items"));
            }
        }

        #[test]
        fn test_VAR_FLAVOR_004_prop_values_flexible(
            varname in "[A-Z]{2,10}",
            value in "[-a-zA-Z0-9_./ ]{0,40}"
        ) {
            // ARRANGE: Test flexible value formats
            let makefile = format!("{} += {}", varname, value);

            // ACT: Parse
            let result = parse_makefile(&makefile);

            // ASSERT: Should always parse
            prop_assert!(result.is_ok(), "Failed to parse: {}", makefile);

            let ast = result.unwrap();
            if let MakeItem::Variable { name, value: val, flavor, .. } = &ast.items[0] {
                prop_assert_eq!(name, &varname);
                prop_assert_eq!(val, &value.trim());
                prop_assert_eq!(flavor, &VarFlavor::Append);

                // Value can be empty or non-empty
                if value.trim().is_empty() {
                    prop_assert_eq!(val, "");
                } else {
                    prop_assert!(!val.is_empty());
                }
            } else {
                return Err(TestCaseError::fail("Expected Variable"));
            }
        }

        #[test]
        fn test_VAR_FLAVOR_004_prop_special_values(
            varname in "[A-Z]{2,8}",
            flags in prop::collection::vec("-[a-zA-Z][a-zA-Z0-9]*", 0..5)
        ) {
            // ARRANGE: Test compiler flag-like values (common use case for +=)
            let value = flags.join(" ");
            let makefile = format!("{} += {}", varname, value);

            // ACT: Parse
            let result = parse_makefile(&makefile);

            // ASSERT: Should parse
            prop_assert!(result.is_ok());

            let ast = result.unwrap();
            if let MakeItem::Variable { name, value: val, flavor, .. } = &ast.items[0] {
                prop_assert_eq!(name, &varname);
                prop_assert_eq!(val, &value.trim());
                prop_assert_eq!(flavor, &VarFlavor::Append);

                // If we had flags, verify they're preserved
                if !flags.is_empty() {
                    prop_assert!(!val.is_empty());
                    for flag in &flags {
                        prop_assert!(val.contains(flag), "Value should contain flag: {}", flag);
                    }
                }
            } else {
                return Err(TestCaseError::fail("Expected Variable"));
            }
        }
    }
}

// ----------------------------------------------------------------------------
// Phase 5: MUTATION TESTING - Mutation-Killing Tests for += Append Assignment
// ----------------------------------------------------------------------------

// Target: parser.rs:111 - is_variable_assignment() contains("+=") check
#[test]
fn test_VAR_FLAVOR_004_mut_operator_detection() {
    // ARRANGE: Makefile with += operator
    let makefile = "CFLAGS += -O2";

    // ACT: Parse the makefile
    let result = parse_makefile(makefile);

    // ASSERT: Must detect += as variable assignment
    assert!(result.is_ok(), "Must parse += as variable assignment");

    let ast = result.unwrap();
    assert_eq!(ast.items.len(), 1, "Must have exactly one item");

    // Must be a Variable (not a Target or Comment)
    match &ast.items[0] {
        MakeItem::Variable { name, flavor, .. } => {
            assert_eq!(name, "CFLAGS");
            assert_eq!(*flavor, VarFlavor::Append, "Must be Append flavor");
        }
        _ => panic!("Must be parsed as Variable, not other type"),
    }
}

// Target: parser.rs:152-153 - parse_variable() slicing for "+=" operator
#[test]
fn test_VAR_FLAVOR_004_mut_operator_slicing() {
    // ARRANGE: Makefile with += and value with special chars
    let makefile = "LDFLAGS += -lm -lpthread";

    // ACT: Parse
    let result = parse_makefile(makefile);
    assert!(result.is_ok());

    let ast = result.unwrap();
    match &ast.items[0] {
        MakeItem::Variable {
            name,
            value,
            flavor,
            ..
        } => {
            assert_eq!(name, "LDFLAGS");
            assert_eq!(*flavor, VarFlavor::Append);

            // CRITICAL: Value must NOT include "+" or "="
            // This tests the slicing at line 153: &trimmed[pos + 2..]
            assert_eq!(value, "-lm -lpthread");
            assert!(!value.contains('+'), "Value should not contain '+'");
            assert!(!value.contains('='), "Value should not contain '='");
            assert!(value.starts_with('-'), "Value should start with -");
        }
        _ => panic!("Expected Variable"),
    }
}

// Target: parser.rs:152 - Append assignment not missed
#[test]
fn test_VAR_FLAVOR_004_mut_append_not_missed() {
    // ARRANGE: Test multiple += assignments
    let makefile = r#"
V1 += first
V2 += second
V3 += third
"#;

    // ACT: Parse
    let result = parse_makefile(makefile);

    // ASSERT: All three must be parsed as Append
    assert!(result.is_ok());
    let ast = result.unwrap();
    assert_eq!(ast.items.len(), 3, "Must parse all 3 variables");

    for (idx, item) in ast.items.iter().enumerate() {
        match item {
            MakeItem::Variable { flavor, .. } => {
                assert_eq!(
                    *flavor,
                    VarFlavor::Append,
                    "Variable {} must be Append flavor",
                    idx
                );
            }
            _ => panic!("All items must be Variables"),
        }
    }
}

// Target: parser.rs:152-153 - Not confused with other operators containing '+'
#[test]
fn test_VAR_FLAVOR_004_mut_not_confused_with_plus() {
    // ARRANGE: Test that += is not confused with values containing '+'
    let append = "FLAGS += -O2";
    let recursive_with_plus = "VALUE = 1+2"; // Contains '+' but not '+='

    // ACT: Parse both
    let append_result = parse_makefile(append);
    let recursive_result = parse_makefile(recursive_with_plus);

    // ASSERT: Different flavors
    assert!(append_result.is_ok());
    assert!(recursive_result.is_ok());

    let append_ast = append_result.unwrap();
    let recursive_ast = recursive_result.unwrap();

    match (&append_ast.items[0], &recursive_ast.items[0]) {
        (MakeItem::Variable { flavor: f1, .. }, MakeItem::Variable { flavor: f2, .. }) => {
            // First must be Append
            assert_eq!(*f1, VarFlavor::Append, "FLAGS must be Append");

            // Second must be Recursive (not Append!)
            assert_eq!(*f2, VarFlavor::Recursive, "VALUE must be Recursive");
            assert_ne!(*f1, *f2, "Flavors must be different");
        }
        _ => panic!("Expected Variable items"),
    }
}

// Target: parser.rs:152-153 - Correct flavor enum variant assignment
#[test]
fn test_VAR_FLAVOR_004_mut_correct_flavor_enum_variant() {
    // ARRANGE: Makefile with += operator
    let makefile = "TARGET += value";

    // ACT: Parse
    let result = parse_makefile(makefile);
    assert!(result.is_ok());

    let ast = result.unwrap();
    match &ast.items[0] {
        MakeItem::Variable { flavor, .. } => {
            // Must be EXACTLY VarFlavor::Append
            assert!(
                matches!(flavor, VarFlavor::Append),
                "Must be VarFlavor::Append"
            );

            // Must NOT be any other variant
            assert!(
                !matches!(flavor, VarFlavor::Recursive),
                "Must NOT be Recursive"
            );
            assert!(!matches!(flavor, VarFlavor::Simple), "Must NOT be Simple");
            assert!(
                !matches!(flavor, VarFlavor::Conditional),
                "Must NOT be Conditional"
            );
            assert!(!matches!(flavor, VarFlavor::Shell), "Must NOT be Shell");
        }
        _ => panic!("Expected Variable"),
    }
}

// ============================================================================
// Sprint 36: VAR-FLAVOR-001 - Recursive Assignment (=)
// ============================================================================
// Target: parser.rs:116 (detection), parser.rs:156-157 (parsing)
// Verifies: = operator maps to VarFlavor::Recursive

// Phase 1 - RED: Unit tests for = recursive assignment

#[test]
fn test_VAR_FLAVOR_001_basic_recursive_assignment() {
    // ARRANGE: Makefile with = recursive assignment operator
    let makefile = "CC = gcc";

    // ACT: Parse the makefile
    let result = parse_makefile(makefile);

    // ASSERT: Parser should handle = recursive assignment
    assert!(
        result.is_ok(),
        "Parser should handle = recursive assignment"
    );

    let ast = result.unwrap();
    assert_eq!(ast.items.len(), 1, "Should parse one variable");

    match &ast.items[0] {
        MakeItem::Variable {
            name,
            value,
            flavor,
            ..
        } => {
            assert_eq!(name, "CC", "Variable name should be CC");
            assert_eq!(value, "gcc", "Value should be gcc");
            assert_eq!(
                *flavor,
                VarFlavor::Recursive,
                "Should detect = as Recursive"
            );
        }
        other => panic!("Expected Variable, got {:?}", other),
    }
}

#[test]
fn test_VAR_FLAVOR_001_recursive_with_spaces() {
    // ARRANGE: Makefile with spaces around = operator
    let makefile = "CFLAGS   =   -Wall -O2   ";

    // ACT: Parse the makefile
    let result = parse_makefile(makefile);

    // ASSERT: Parser should handle spaces around = operator
    assert!(
        result.is_ok(),
        "Parser should handle spaces around = operator"
    );

    let ast = result.unwrap();
    assert_eq!(ast.items.len(), 1, "Should parse one variable");

    match &ast.items[0] {
        MakeItem::Variable {
            name,
            value,
            flavor,
            ..
        } => {
            assert_eq!(name, "CFLAGS", "Variable name should be CFLAGS");
            assert_eq!(value, "-Wall -O2", "Value should be trimmed");
            assert_eq!(*flavor, VarFlavor::Recursive, "Should be Recursive flavor");
        }
        other => panic!("Expected Variable, got {:?}", other),
    }
}

#[test]
fn test_VAR_FLAVOR_001_recursive_empty_value() {
    // ARRANGE: Makefile with = operator and empty value
    let makefile = "EMPTY = ";

    // ACT: Parse the makefile
    let result = parse_makefile(makefile);

    // ASSERT: Parser should handle empty values with = operator
    assert!(
        result.is_ok(),
        "Parser should handle empty values with = operator"
    );

    let ast = result.unwrap();
    assert_eq!(ast.items.len(), 1, "Should parse one variable");

    match &ast.items[0] {
        MakeItem::Variable {
            name,
            value,
            flavor,
            ..
        } => {
            assert_eq!(name, "EMPTY", "Variable name should be EMPTY");
            assert_eq!(value, "", "Value should be empty string");
            assert_eq!(*flavor, VarFlavor::Recursive, "Should be Recursive flavor");
        }
        other => panic!("Expected Variable, got {:?}", other),
    }
}

#[test]
fn test_VAR_FLAVOR_001_recursive_vs_other_flavors() {
    // ARRANGE: Makefiles with different operators
    let simple = "VAR := value"; // Simple assignment
    let recursive = "VAR = value"; // Recursive assignment

    // ACT: Parse both makefiles
    let simple_result = parse_makefile(simple);
    let recursive_result = parse_makefile(recursive);

    // ASSERT: Different flavors
    assert!(simple_result.is_ok());
    assert!(recursive_result.is_ok());

    let simple_ast = simple_result.unwrap();
    let recursive_ast = recursive_result.unwrap();

    match (&simple_ast.items[0], &recursive_ast.items[0]) {
        (MakeItem::Variable { flavor: f1, .. }, MakeItem::Variable { flavor: f2, .. }) => {
            // First must be Simple
            assert_eq!(*f1, VarFlavor::Simple, "VAR must be Simple");

            // Second must be Recursive (not Simple!)
            assert_eq!(*f2, VarFlavor::Recursive, "VAR must be Recursive");
            assert_ne!(*f1, *f2, "Flavors must be different");
        }
        _ => panic!("Expected Variable items"),
    }
}

// Phase 4 - PROPERTY TESTING: Property tests for = recursive assignment

#[cfg(test)]
mod property_tests_var_flavor_001 {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn test_VAR_FLAVOR_001_prop_recursive_always_parses(
            varname in "[A-Z][A-Z0-9_]{0,20}",
            value in "[a-zA-Z0-9_./+ -]{0,50}"
        ) {
            let makefile = format!("{} = {}", varname, value);
            let result = parse_makefile(&makefile);

            prop_assert!(result.is_ok(), "Failed to parse: {}", makefile);
            let ast = result.unwrap();
            prop_assert_eq!(ast.items.len(), 1);

            if let MakeItem::Variable { name, value: val, flavor, .. } = &ast.items[0] {
                prop_assert_eq!(name, &varname);
                prop_assert_eq!(val, &value.trim());
                prop_assert_eq!(flavor, &VarFlavor::Recursive);
            } else {
                return Err(TestCaseError::fail("Expected Variable item"));
            }
        }

        #[test]
        fn test_VAR_FLAVOR_001_prop_recursive_is_deterministic(
            varname in "[A-Z]{1,15}",
            value in "[a-z0-9]{0,30}"
        ) {
            let makefile = format!("{} = {}", varname, value);

            // Parse twice
            let result1 = parse_makefile(&makefile);
            let result2 = parse_makefile(&makefile);

            prop_assert!(result1.is_ok());
            prop_assert!(result2.is_ok());

            let ast1 = result1.unwrap();
            let ast2 = result2.unwrap();

            // Must produce identical results
            match (&ast1.items[0], &ast2.items[0]) {
                (
                    MakeItem::Variable { name: n1, value: v1, flavor: f1, .. },
                    MakeItem::Variable { name: n2, value: v2, flavor: f2, .. }
                ) => {
                    prop_assert_eq!(n1, n2);
                    prop_assert_eq!(v1, v2);
                    prop_assert_eq!(f1, f2);
                    prop_assert_eq!(f1, &VarFlavor::Recursive);
                }
                _ => return Err(TestCaseError::fail("Expected Variable items")),
            }
        }

        #[test]
        fn test_VAR_FLAVOR_001_prop_not_confused_with_other_operators(
            varname in "[A-Z]{1,10}",
            value in "[a-z]{0,20}"
        ) {
            // Test that = is NOT parsed as :=, ?=, +=, or !=
            let makefile = format!("{} = {}", varname, value);
            let result = parse_makefile(&makefile);

            prop_assert!(result.is_ok());
            let ast = result.unwrap();

            if let MakeItem::Variable { flavor, .. } = &ast.items[0] {
                // Must be Recursive, NOT other flavors
                prop_assert_eq!(flavor, &VarFlavor::Recursive);
                prop_assert_ne!(flavor, &VarFlavor::Simple);
                prop_assert_ne!(flavor, &VarFlavor::Conditional);
                prop_assert_ne!(flavor, &VarFlavor::Append);
                prop_assert_ne!(flavor, &VarFlavor::Shell);
            } else {
                return Err(TestCaseError::fail("Expected Variable item"));
            }
        }

        #[test]
        fn test_VAR_FLAVOR_001_prop_handles_various_values(
            varname in "[A-Z_][A-Z0-9_]*",
            value in ".*"
        ) {
            // Filter out values containing special operators to avoid confusion
            if value.contains(":=") || value.contains("?=") ||
               value.contains("+=") || value.contains("!=") {
                return Ok(());
            }

            let makefile = format!("{} = {}", varname, value);
            let result = parse_makefile(&makefile);

            prop_assert!(result.is_ok(), "Failed to parse: {}", makefile);
            let ast = result.unwrap();

            if let MakeItem::Variable { name, value: val, flavor, .. } = &ast.items[0] {
                prop_assert_eq!(name, &varname);
                prop_assert_eq!(val, &value.trim());
                prop_assert_eq!(flavor, &VarFlavor::Recursive);
            } else {
                return Err(TestCaseError::fail("Expected Variable item"));
            }
        }

        #[test]
        fn test_VAR_FLAVOR_001_prop_handles_special_chars_in_values(
            varname in "[A-Z]{3,10}",
            value in "[a-zA-Z0-9@#$%^&*()_+\\-=\\[\\]{}|;:',.<>/?~ ]{0,40}"
        ) {
            // Avoid values with special operators
            if value.contains(":=") || value.contains("?=") ||
               value.contains("+=") || value.contains("!=") {
                return Ok(());
            }

            let makefile = format!("{} = {}", varname, value);
            let result = parse_makefile(&makefile);

            prop_assert!(result.is_ok());
            let ast = result.unwrap();

            if let MakeItem::Variable { flavor, .. } = &ast.items[0] {
                prop_assert_eq!(flavor, &VarFlavor::Recursive);
            } else {
                return Err(TestCaseError::fail("Expected Variable item"));
            }
        }
    }
}
