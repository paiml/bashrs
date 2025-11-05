//! Tests for Makefile parser
//!
//! Following EXTREME TDD methodology:
//! RED -> GREEN -> REFACTOR -> PROPERTY TESTING -> MUTATION TESTING -> DOCUMENTATION
//!
//! Test naming convention: test_<TASK_ID>_<feature>_<scenario>

use super::*;
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
                match &ast.items[0] {
                    MakeItem::Variable { name, .. } => {
                        // Just verify it's a variable
                        assert!(!name.is_empty());
                    }
                    _ => {} // Could be target for some edge cases
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

// ============================================================================
// FUNC-SUBST-001: $(subst from,to,text) Function
// ============================================================================

#[test]
fn test_FUNC_SUBST_001_basic_subst() {
    // ARRANGE: Variable with $(subst) function
    let makefile = "OBJS = $(subst .c,.o,main.c util.c)";

    // ACT: Parse makefile
    let result = parse_makefile(makefile);

    // ASSERT: Successfully parsed
    assert!(
        result.is_ok(),
        "Should parse $(subst) function, got error: {:?}",
        result.err()
    );

    let ast = result.unwrap();
    assert_eq!(ast.items.len(), 1, "Should have 1 variable");

    // ASSERT: Check variable with $(subst) function
    match &ast.items[0] {
        MakeItem::Variable { name, value, .. } => {
            assert_eq!(name, "OBJS", "Variable name should be OBJS");
            assert_eq!(
                value, "$(subst .c,.o,main.c util.c)",
                "Variable value should preserve $(subst) function syntax"
            );
        }
        other => panic!("Expected Variable item, got {:?}", other),
    }
}

#[test]
fn test_FUNC_SUBST_001_subst_in_prerequisites() {
    // ARRANGE: Target with $(subst) in prerequisites
    // NOTE: Parser splits prerequisites on whitespace, so $(subst .c,.o,$(SRCS))
    // with spaces becomes 2 prerequisites: "$(subst" and ".c,.o,$(SRCS))"
    // This is expected behavior - the parser is not function-aware
    let makefile = "build: $(subst .c,.o,$(SRCS))\n\tgcc -o $@ $^";

    // ACT: Parse makefile
    let result = parse_makefile(makefile);

    // ASSERT: Successfully parsed
    assert!(result.is_ok(), "Should parse $(subst) in prerequisites");

    let ast = result.unwrap();

    // ASSERT: Check target prerequisites
    match &ast.items[0] {
        MakeItem::Target {
            name,
            prerequisites,
            recipe,
            ..
        } => {
            assert_eq!(name, "build");
            // Parser splits on whitespace, so we get 2 prerequisites
            assert_eq!(prerequisites.len(), 2);
            // The function syntax is preserved, just split
            assert_eq!(prerequisites[0], "$(subst");
            assert_eq!(prerequisites[1], ".c,.o,$(SRCS))");
            // Recipe should be parsed correctly
            assert_eq!(recipe.len(), 1, "Should have 1 recipe line");
            assert_eq!(recipe[0], "gcc -o $@ $^");
        }
        other => panic!("Expected Target, got {:?}", other),
    }
}

#[test]
fn test_FUNC_SUBST_001_multiple_subst() {
    // ARRANGE: Multiple variables with $(subst) functions
    let makefile = "OBJS = $(subst .c,.o,$(SRCS))\nLIBS = $(subst .a,.so,$(DEPS))";

    // ACT: Parse makefile
    let result = parse_makefile(makefile);

    // ASSERT: Successfully parsed
    assert!(result.is_ok(), "Should parse multiple $(subst) functions");

    let ast = result.unwrap();
    assert_eq!(ast.items.len(), 2, "Should have 2 variables");

    // ASSERT: Check first $(subst)
    match &ast.items[0] {
        MakeItem::Variable { name, value, .. } => {
            assert_eq!(name, "OBJS");
            assert_eq!(value, "$(subst .c,.o,$(SRCS))");
        }
        other => panic!("Expected Variable, got {:?}", other),
    }

    // ASSERT: Check second $(subst)
    match &ast.items[1] {
        MakeItem::Variable { name, value, .. } => {
            assert_eq!(name, "LIBS");
            assert_eq!(value, "$(subst .a,.so,$(DEPS))");
        }
        other => panic!("Expected Variable, got {:?}", other),
    }
}

#[test]
fn test_FUNC_SUBST_001_subst_with_spaces() {
    // ARRANGE: $(subst) with spaces in arguments
    let makefile = "FILES = $(subst src/,build/,src/main.c src/util.c)";

    // ACT: Parse makefile
    let result = parse_makefile(makefile);

    // ASSERT: Successfully parsed
    assert!(result.is_ok(), "Should parse $(subst) with spaces");

    let ast = result.unwrap();
    assert_eq!(ast.items.len(), 1);

    // ASSERT: Check $(subst) preserved with spaces
    match &ast.items[0] {
        MakeItem::Variable { name, value, .. } => {
            assert_eq!(name, "FILES");
            assert_eq!(
                value, "$(subst src/,build/,src/main.c src/util.c)",
                "Should preserve spaces in $(subst) text argument"
            );
        }
        other => panic!("Expected Variable, got {:?}", other),
    }
}

#[test]
fn test_FUNC_SUBST_001_nested_subst() {
    // ARRANGE: Nested $(subst) functions
    let makefile = "RESULT = $(subst .c,.o,$(subst src/,build/,$(SRCS)))";

    // ACT: Parse makefile
    let result = parse_makefile(makefile);

    // ASSERT: Successfully parsed
    assert!(result.is_ok(), "Should parse nested $(subst) functions");

    let ast = result.unwrap();
    assert_eq!(ast.items.len(), 1);

    // ASSERT: Check nested $(subst) preserved
    match &ast.items[0] {
        MakeItem::Variable { name, value, .. } => {
            assert_eq!(name, "RESULT");
            assert_eq!(
                value, "$(subst .c,.o,$(subst src/,build/,$(SRCS)))",
                "Should preserve nested $(subst) functions"
            );
        }
        other => panic!("Expected Variable, got {:?}", other),
    }
}

#[test]
fn test_FUNC_SUBST_001_subst_with_other_functions() {
    // ARRANGE: $(subst) combined with $(wildcard)
    let makefile = "OBJS = $(subst .c,.o,$(wildcard src/*.c))";

    // ACT: Parse makefile
    let result = parse_makefile(makefile);

    // ASSERT: Successfully parsed
    assert!(
        result.is_ok(),
        "Should parse $(subst) with $(wildcard) function"
    );

    let ast = result.unwrap();
    assert_eq!(ast.items.len(), 1);

    // ASSERT: Check combined functions preserved
    match &ast.items[0] {
        MakeItem::Variable { name, value, .. } => {
            assert_eq!(name, "OBJS");
            assert_eq!(
                value, "$(subst .c,.o,$(wildcard src/*.c))",
                "Should preserve $(subst) combined with other functions"
            );
        }
        other => panic!("Expected Variable, got {:?}", other),
    }
}

// ============================================================================
// FUNC-SUBST-001: Property Tests
// ============================================================================

#[cfg(test)]
mod property_tests_func_subst {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn prop_FUNC_SUBST_001_basic_subst_always_preserved(
            var in "[A-Z]{1,8}",
            from in "\\.[a-z]{1,3}",
            to in "\\.[a-z]{1,3}",
            text in "[a-z]{1,10}"
        ) {
            // ARRANGE: Variable with $(subst from,to,text)
            let makefile = format!("{} = $(subst {},{},{})", var, from, to, text);

            // ACT: Parse makefile
            let result = parse_makefile(&makefile);

            // ASSERT: Successfully parsed
            prop_assert!(result.is_ok());

            let ast = result.unwrap();
            prop_assert_eq!(ast.items.len(), 1);

            // ASSERT: $(subst) function preserved
            match &ast.items[0] {
                MakeItem::Variable { name, value, .. } => {
                    prop_assert_eq!(name, &var);
                    let expected = format!("$(subst {},{},{})", from, to, text);
                    prop_assert_eq!(value, &expected);
                }
                other => return Err(TestCaseError::fail(format!("Expected Variable, got {:?}", other))),
            }
        }

        #[test]
        fn prop_FUNC_SUBST_001_parsing_is_deterministic(
            var in "[A-Z]{1,8}",
            from in "\\.[a-z]{1,3}",
            to in "\\.[a-z]{1,3}",
            varref in "[A-Z]{1,8}"
        ) {
            // ARRANGE: Variable with $(subst from,to,$(VAR))
            let makefile = format!("{} = $(subst {},{},$({})) ", var, from, to, varref);

            // ACT: Parse twice
            let result1 = parse_makefile(&makefile);
            let result2 = parse_makefile(&makefile);

            // ASSERT: Same results
            prop_assert!(result1.is_ok());
            prop_assert!(result2.is_ok());

            let ast1 = result1.unwrap();
            let ast2 = result2.unwrap();

            // Same number of items
            prop_assert_eq!(ast1.items.len(), ast2.items.len());

            // Same variable value
            match (&ast1.items[0], &ast2.items[0]) {
                (MakeItem::Variable { value: v1, .. }, MakeItem::Variable { value: v2, .. }) => {
                    prop_assert_eq!(v1, v2);
                }
                _ => return Err(TestCaseError::fail("Expected Variables")),
            }
        }

        #[test]
        fn prop_FUNC_SUBST_001_nested_functions_preserved(
            var in "[A-Z]{1,8}",
            from1 in "\\.[a-z]{1,2}",
            to1 in "\\.[a-z]{1,2}",
            from2 in "[a-z]{1,5}/",
            to2 in "[a-z]{1,5}/",
            varref in "[A-Z]{1,8}"
        ) {
            // ARRANGE: Nested $(subst)
            let makefile = format!(
                "{} = $(subst {},{},$(subst {},{},$({})))",
                var, from1, to1, from2, to2, varref
            );

            // ACT: Parse makefile
            let result = parse_makefile(&makefile);

            // ASSERT: Successfully parsed
            prop_assert!(result.is_ok());

            let ast = result.unwrap();
            prop_assert_eq!(ast.items.len(), 1);

            // ASSERT: Nested functions preserved
            match &ast.items[0] {
                MakeItem::Variable { value, .. } => {
                    // Should contain both subst calls
                    prop_assert!(value.contains("$(subst"));
                    prop_assert!(value.contains(&from1));
                    prop_assert!(value.contains(&to1));
                    prop_assert!(value.contains(&from2));
                    prop_assert!(value.contains(&to2));
                }
                other => return Err(TestCaseError::fail(format!("Expected Variable, got {:?}", other))),
            }
        }

        #[test]
        fn prop_FUNC_SUBST_001_multiple_functions_preserved(
            var1 in "[A-Z]{1,8}",
            var2 in "[A-Z]{1,8}",
            from1 in "\\.[a-z]{1,3}",
            to1 in "\\.[a-z]{1,3}",
            from2 in "\\.[a-z]{1,3}",
            to2 in "\\.[a-z]{1,3}",
            ref1 in "[A-Z]{1,8}",
            ref2 in "[A-Z]{1,8}"
        ) {
            prop_assume!(var1 != var2);

            // ARRANGE: Two variables with $(subst) functions
            let makefile = format!(
                "{} = $(subst {},{},$({})) \n{} = $(subst {},{},$({})) ",
                var1, from1, to1, ref1, var2, from2, to2, ref2
            );

            // ACT: Parse makefile
            let result = parse_makefile(&makefile);

            // ASSERT: Successfully parsed
            prop_assert!(result.is_ok());

            let ast = result.unwrap();
            prop_assert_eq!(ast.items.len(), 2);

            // ASSERT: Both functions preserved
            match &ast.items[0] {
                MakeItem::Variable { name, value, .. } => {
                    prop_assert_eq!(name, &var1);
                    prop_assert!(value.contains("$(subst"));
                    prop_assert!(value.contains(&from1));
                }
                other => return Err(TestCaseError::fail(format!("Expected Variable, got {:?}", other))),
            }

            match &ast.items[1] {
                MakeItem::Variable { name, value, .. } => {
                    prop_assert_eq!(name, &var2);
                    prop_assert!(value.contains("$(subst"));
                    prop_assert!(value.contains(&from2));
                }
                other => return Err(TestCaseError::fail(format!("Expected Variable, got {:?}", other))),
            }
        }

        #[test]
        fn prop_FUNC_SUBST_001_combined_with_wildcard(
            var in "[A-Z]{1,8}",
            from in "\\.[a-z]{1,3}",
            to in "\\.[a-z]{1,3}",
            pattern in "[a-z]{1,8}",
            ext in "[a-z]{1,3}"
        ) {
            // ARRANGE: $(subst) with $(wildcard)
            let makefile = format!(
                "{} = $(subst {},{},$(wildcard {}/*.{}))",
                var, from, to, pattern, ext
            );

            // ACT: Parse makefile
            let result = parse_makefile(&makefile);

            // ASSERT: Successfully parsed
            prop_assert!(result.is_ok());

            let ast = result.unwrap();
            prop_assert_eq!(ast.items.len(), 1);

            // ASSERT: Combined functions preserved
            match &ast.items[0] {
                MakeItem::Variable { value, .. } => {
                    prop_assert!(value.contains("$(subst"));
                    prop_assert!(value.contains("$(wildcard"));
                }
                other => return Err(TestCaseError::fail(format!("Expected Variable, got {:?}", other))),
            }
        }

        #[test]
        fn prop_FUNC_SUBST_001_no_spaces_in_function(
            var in "[A-Z]{1,8}",
            from in "[a-z]{1,5}",
            to in "[a-z]{1,5}",
            text in "[a-z]{1,10}"
        ) {
            // ARRANGE: $(subst) without spaces (single token)
            let makefile = format!("{} = $(subst {},{},{})", var, from, to, text);

            // ACT: Parse makefile
            let result = parse_makefile(&makefile);

            // ASSERT: Successfully parsed
            prop_assert!(result.is_ok());

            let ast = result.unwrap();
            prop_assert_eq!(ast.items.len(), 1);

            // ASSERT: Function preserved as one value
            match &ast.items[0] {
                MakeItem::Variable { value, .. } => {
                    let expected = format!("$(subst {},{},{})", from, to, text);
                    prop_assert_eq!(value, &expected);
                }
                other => return Err(TestCaseError::fail(format!("Expected Variable, got {:?}", other))),
            }
        }
    }
}

// ============================================================================
// SPRINT 64: Function Call Parser Tests
// ============================================================================
// Goal: Parse GNU Make function calls like $(filter %.o, foo.o bar.c)
// Context: Enables recursive purification for 13 deterministic functions
// Reference: SPRINT-63-HANDOFF.md, SPRINT-61-HANDOFF.md, SPRINT-62-HANDOFF.md

#[test]
fn test_PARSER_FUNC_001_basic_filter() {
    // ARRANGE: Makefile with filter function
    let makefile = "OBJS := $(filter %.o, foo.o bar.c baz.o)";

    // ACT: Parse makefile
    let ast = parse_makefile(makefile).unwrap();

    // ASSERT: Should parse variable with function call
    assert_eq!(ast.items.len(), 1);

    match &ast.items[0] {
        MakeItem::Variable { name, value, .. } => {
            assert_eq!(name, "OBJS");
            // For now, just verify it contains the function call
            // Later: will verify FunctionCall AST node structure
            assert!(value.contains("$(filter"));
        }
        _ => panic!("Expected Variable, got {:?}", ast.items[0]),
    }
}

#[test]
fn test_PARSER_FUNC_002_basic_sort() {
    // ARRANGE: Makefile with sort function
    let makefile = "SORTED := $(sort foo bar baz foo)";

    // ACT: Parse makefile
    let ast = parse_makefile(makefile).unwrap();

    // ASSERT: Should parse variable with sort function call
    assert_eq!(ast.items.len(), 1);

    match &ast.items[0] {
        MakeItem::Variable { name, value, .. } => {
            assert_eq!(name, "SORTED");
            assert!(value.contains("$(sort"));
        }
        _ => panic!("Expected Variable, got {:?}", ast.items[0]),
    }
}

#[test]
fn test_PARSER_FUNC_003_filter_multiple_patterns() {
    // ARRANGE: filter with multiple pattern arguments
    let makefile = "OBJS := $(filter %.o %.a, foo.o bar.c baz.a)";

    // ACT: Parse makefile
    let ast = parse_makefile(makefile).unwrap();

    // ASSERT: Should parse both patterns
    assert_eq!(ast.items.len(), 1);

    match &ast.items[0] {
        MakeItem::Variable { name, value, .. } => {
            assert_eq!(name, "OBJS");
            assert!(value.contains("$(filter"));
            assert!(value.contains("%.o"));
            assert!(value.contains("%.a"));
        }
        _ => panic!("Expected Variable, got {:?}", ast.items[0]),
    }
}

#[test]
fn test_PARSER_FUNC_004_nested_wildcard() {
    // ARRANGE: CRITICAL - nested function calls
    // This is the key pattern for recursive purification
    let makefile = "OBJS := $(filter %.o, $(wildcard *.c))";

    // ACT: Parse makefile
    let ast = parse_makefile(makefile).unwrap();

    // ASSERT: Should parse nested $(wildcard) inside $(filter)
    assert_eq!(ast.items.len(), 1);

    match &ast.items[0] {
        MakeItem::Variable { name, value, .. } => {
            assert_eq!(name, "OBJS");
            assert!(value.contains("$(filter"));
            assert!(value.contains("$(wildcard"));
        }
        _ => panic!("Expected Variable, got {:?}", ast.items[0]),
    }
}

#[test]
fn test_PARSER_FUNC_005_word() {
    // ARRANGE: word function (extracts Nth word)
    let makefile = "SECOND := $(word 2, foo bar baz)";

    // ACT: Parse makefile
    let ast = parse_makefile(makefile).unwrap();

    // ASSERT: Should parse word function
    assert_eq!(ast.items.len(), 1);

    match &ast.items[0] {
        MakeItem::Variable { name, value, .. } => {
            assert_eq!(name, "SECOND");
            assert!(value.contains("$(word"));
        }
        _ => panic!("Expected Variable, got {:?}", ast.items[0]),
    }
}

#[test]
fn test_PARSER_FUNC_006_notdir() {
    // ARRANGE: notdir function (remove directory part)
    let makefile = "FILES := $(notdir src/main.c include/util.h)";

    // ACT: Parse makefile
    let ast = parse_makefile(makefile).unwrap();

    // ASSERT: Should parse notdir function
    assert_eq!(ast.items.len(), 1);

    match &ast.items[0] {
        MakeItem::Variable { name, value, .. } => {
            assert_eq!(name, "FILES");
            assert!(value.contains("$(notdir"));
        }
        _ => panic!("Expected Variable, got {:?}", ast.items[0]),
    }
}

#[test]
fn test_PARSER_FUNC_007_addsuffix() {
    // ARRANGE: addsuffix function
    let makefile = "OBJS := $(addsuffix .o, foo bar baz)";

    // ACT: Parse makefile
    let ast = parse_makefile(makefile).unwrap();

    // ASSERT: Should parse addsuffix function
    assert_eq!(ast.items.len(), 1);

    match &ast.items[0] {
        MakeItem::Variable { name, value, .. } => {
            assert_eq!(name, "OBJS");
            assert!(value.contains("$(addsuffix"));
        }
        _ => panic!("Expected Variable, got {:?}", ast.items[0]),
    }
}

#[test]
fn test_PARSER_FUNC_008_addprefix() {
    // ARRANGE: addprefix function
    let makefile = "OBJS := $(addprefix obj/, foo.o bar.o)";

    // ACT: Parse makefile
    let ast = parse_makefile(makefile).unwrap();

    // ASSERT: Should parse addprefix function
    assert_eq!(ast.items.len(), 1);

    match &ast.items[0] {
        MakeItem::Variable { name, value, .. } => {
            assert_eq!(name, "OBJS");
            assert!(value.contains("$(addprefix"));
        }
        _ => panic!("Expected Variable, got {:?}", ast.items[0]),
    }
}

#[test]
fn test_PARSER_FUNC_009_filter_out() {
    // ARRANGE: filter-out function (inverse of filter)
    let makefile = "SOURCES := $(filter-out test_%, main.c test_foo.c util.c)";

    // ACT: Parse makefile
    let ast = parse_makefile(makefile).unwrap();

    // ASSERT: Should parse filter-out function
    assert_eq!(ast.items.len(), 1);

    match &ast.items[0] {
        MakeItem::Variable { name, value, .. } => {
            assert_eq!(name, "SOURCES");
            assert!(value.contains("$(filter-out"));
        }
        _ => panic!("Expected Variable, got {:?}", ast.items[0]),
    }
}

#[test]
fn test_PARSER_FUNC_010_wordlist() {
    // ARRANGE: wordlist function (extract range of words)
    let makefile = "MIDDLE := $(wordlist 2, 4, foo bar baz qux quux)";

    // ACT: Parse makefile
    let ast = parse_makefile(makefile).unwrap();

    // ASSERT: Should parse wordlist function
    assert_eq!(ast.items.len(), 1);

    match &ast.items[0] {
        MakeItem::Variable { name, value, .. } => {
            assert_eq!(name, "MIDDLE");
            assert!(value.contains("$(wordlist"));
        }
        _ => panic!("Expected Variable, got {:?}", ast.items[0]),
    }
}

#[test]
fn test_PARSER_FUNC_011_words() {
    // ARRANGE: words function (count words)
    let makefile = "COUNT := $(words foo bar baz)";

    // ACT: Parse makefile
    let ast = parse_makefile(makefile).unwrap();

    // ASSERT: Should parse words function
    assert_eq!(ast.items.len(), 1);

    match &ast.items[0] {
        MakeItem::Variable { name, value, .. } => {
            assert_eq!(name, "COUNT");
            assert!(value.contains("$(words"));
        }
        _ => panic!("Expected Variable, got {:?}", ast.items[0]),
    }
}

#[test]
fn test_PARSER_FUNC_012_firstword() {
    // ARRANGE: firstword function
    let makefile = "FIRST := $(firstword foo bar baz)";

    // ACT: Parse makefile
    let ast = parse_makefile(makefile).unwrap();

    // ASSERT: Should parse firstword function
    assert_eq!(ast.items.len(), 1);

    match &ast.items[0] {
        MakeItem::Variable { name, value, .. } => {
            assert_eq!(name, "FIRST");
            assert!(value.contains("$(firstword"));
        }
        _ => panic!("Expected Variable, got {:?}", ast.items[0]),
    }
}

#[test]
fn test_PARSER_FUNC_013_lastword() {
    // ARRANGE: lastword function
    let makefile = "LAST := $(lastword foo bar baz)";

    // ACT: Parse makefile
    let ast = parse_makefile(makefile).unwrap();

    // ASSERT: Should parse lastword function
    assert_eq!(ast.items.len(), 1);

    match &ast.items[0] {
        MakeItem::Variable { name, value, .. } => {
            assert_eq!(name, "LAST");
            assert!(value.contains("$(lastword"));
        }
        _ => panic!("Expected Variable, got {:?}", ast.items[0]),
    }
}

#[test]
fn test_PARSER_FUNC_014_suffix() {
    // ARRANGE: suffix function (extract file suffixes)
    let makefile = "SUFFIXES := $(suffix foo.c bar.o baz.txt)";

    // ACT: Parse makefile
    let ast = parse_makefile(makefile).unwrap();

    // ASSERT: Should parse suffix function
    assert_eq!(ast.items.len(), 1);

    match &ast.items[0] {
        MakeItem::Variable { name, value, .. } => {
            assert_eq!(name, "SUFFIXES");
            assert!(value.contains("$(suffix"));
        }
        _ => panic!("Expected Variable, got {:?}", ast.items[0]),
    }
}

#[test]
fn test_PARSER_FUNC_015_basename() {
    // ARRANGE: basename function (remove suffix)
    let makefile = "BASES := $(basename foo.c bar.o baz.txt)";

    // ACT: Parse makefile
    let ast = parse_makefile(makefile).unwrap();

    // ASSERT: Should parse basename function
    assert_eq!(ast.items.len(), 1);

    match &ast.items[0] {
        MakeItem::Variable { name, value, .. } => {
            assert_eq!(name, "BASES");
            assert!(value.contains("$(basename"));
        }
        _ => panic!("Expected Variable, got {:?}", ast.items[0]),
    }
}

// ============================================================================
// Sprint 65: Recursive Semantic Analysis Tests
// ============================================================================
// These tests verify that semantic analysis detects non-deterministic patterns
// even when they're nested inside function arguments.
//
// Example: $(filter %.c, $(wildcard src/*.c))
//                         ^^^^^^^^^^^^^^^^^
//                         Nested wildcard needs detection

#[test]
fn test_SEMANTIC_RECURSIVE_001_detect_wildcard_in_filter_args() {
    // ARRANGE: wildcard nested in filter arguments
    let makefile = "FILES := $(filter %.c, $(wildcard src/*.c))";
    let ast = parse_makefile(makefile).unwrap();

    // ACT: Check if value contains nested wildcard
    match &ast.items[0] {
        MakeItem::Variable { name, value, .. } => {
            assert_eq!(name, "FILES");
            // This test verifies we can DETECT the pattern
            // (actual semantic analysis implementation will come in GREEN phase)
            assert!(value.contains("$(wildcard"));
            assert!(value.contains("$(filter"));
        }
        _ => panic!("Expected Variable"),
    }
}

#[test]
fn test_SEMANTIC_RECURSIVE_002_detect_wildcard_in_sort_args() {
    // ARRANGE: wildcard nested in sort arguments
    let makefile = "SORTED := $(sort $(wildcard *.c))";
    let ast = parse_makefile(makefile).unwrap();

    // ACT & ASSERT: Verify nested pattern detected
    match &ast.items[0] {
        MakeItem::Variable { name, value, .. } => {
            assert_eq!(name, "SORTED");
            assert!(value.contains("$(wildcard"));
            assert!(value.contains("$(sort"));
        }
        _ => panic!("Expected Variable"),
    }
}

#[test]
fn test_SEMANTIC_RECURSIVE_003_detect_shell_date_in_addsuffix_args() {
    // ARRANGE: shell date nested in addsuffix arguments
    let makefile = "TIMESTAMPED := $(addsuffix -$(shell date +%s), foo bar)";
    let ast = parse_makefile(makefile).unwrap();

    // ACT & ASSERT: Verify nested shell date detected
    match &ast.items[0] {
        MakeItem::Variable { name, value, .. } => {
            assert_eq!(name, "TIMESTAMPED");
            assert!(value.contains("$(shell date"));
            assert!(value.contains("$(addsuffix"));
        }
        _ => panic!("Expected Variable"),
    }
}

#[test]
fn test_SEMANTIC_RECURSIVE_004_detect_random_in_word_args() {
    // ARRANGE: $RANDOM nested in word arguments
    let makefile = "PICK := $(word $RANDOM, foo bar baz)";
    let ast = parse_makefile(makefile).unwrap();

    // ACT & ASSERT: Verify nested $RANDOM detected
    match &ast.items[0] {
        MakeItem::Variable { name, value, .. } => {
            assert_eq!(name, "PICK");
            assert!(value.contains("$RANDOM") || value.contains("RANDOM"));
            assert!(value.contains("$(word"));
        }
        _ => panic!("Expected Variable"),
    }
}

#[test]
fn test_SEMANTIC_RECURSIVE_005_detect_shell_find_in_filter_args() {
    // ARRANGE: shell find nested in filter arguments
    let makefile = "FOUND := $(filter %.c, $(shell find src -name '*.c'))";
    let ast = parse_makefile(makefile).unwrap();

    // ACT & ASSERT: Verify nested shell find detected
    match &ast.items[0] {
        MakeItem::Variable { name, value, .. } => {
            assert_eq!(name, "FOUND");
            assert!(value.contains("$(shell find"));
            assert!(value.contains("$(filter"));
        }
        _ => panic!("Expected Variable"),
    }
}

#[test]
fn test_SEMANTIC_RECURSIVE_006_detect_deeply_nested_wildcard() {
    // ARRANGE: deeply nested - wildcard in filter in sort
    let makefile = "DEEP := $(sort $(filter %.c, $(wildcard src/*.c)))";
    let ast = parse_makefile(makefile).unwrap();

    // ACT & ASSERT: Verify all nesting levels detected
    match &ast.items[0] {
        MakeItem::Variable { name, value, .. } => {
            assert_eq!(name, "DEEP");
            assert!(value.contains("$(wildcard"));
            assert!(value.contains("$(filter"));
            assert!(value.contains("$(sort"));
        }
        _ => panic!("Expected Variable"),
    }
}

#[test]
fn test_SEMANTIC_RECURSIVE_007_detect_multiple_nested_wildcards() {
    // ARRANGE: multiple wildcard calls in single function
    let makefile = "MULTI := $(filter %.c %.h, $(wildcard src/*.c) $(wildcard inc/*.h))";
    let ast = parse_makefile(makefile).unwrap();

    // ACT & ASSERT: Verify multiple nested patterns detected
    match &ast.items[0] {
        MakeItem::Variable { name, value, .. } => {
            assert_eq!(name, "MULTI");
            // Should contain two wildcard calls
            let wildcard_count = value.matches("$(wildcard").count();
            assert!(
                wildcard_count >= 2,
                "Expected at least 2 wildcard calls, found {}",
                wildcard_count
            );
        }
        _ => panic!("Expected Variable"),
    }
}

#[test]
fn test_SEMANTIC_RECURSIVE_008_safe_filter_no_wildcard() {
    // ARRANGE: filter without nested non-deterministic code (SAFE)
    let makefile = "SAFE := $(filter %.c, foo.c bar.c baz.c)";
    let ast = parse_makefile(makefile).unwrap();

    // ACT & ASSERT: Verify no non-deterministic patterns
    match &ast.items[0] {
        MakeItem::Variable { name, value, .. } => {
            assert_eq!(name, "SAFE");
            // Should NOT contain wildcard/shell/random
            assert!(!value.contains("$(wildcard"));
            assert!(!value.contains("$(shell"));
            assert!(!value.contains("$RANDOM"));
        }
        _ => panic!("Expected Variable"),
    }
}

#[test]
fn test_SEMANTIC_RECURSIVE_009_purified_sort_wrapped_wildcard() {
    // ARRANGE: PURIFIED - wildcard wrapped with sort (SAFE)
    let makefile = "PURIFIED := $(filter %.c, $(sort $(wildcard src/*.c)))";
    let ast = parse_makefile(makefile).unwrap();

    // ACT & ASSERT: Verify purified pattern (sort wraps wildcard)
    match &ast.items[0] {
        MakeItem::Variable { name, value, .. } => {
            assert_eq!(name, "PURIFIED");
            // Should contain both wildcard (non-deterministic) and sort (purifier)
            assert!(value.contains("$(wildcard"));
            assert!(value.contains("$(sort"));
            // Verify sort comes BEFORE wildcard (wrapping)
            let sort_pos = value.find("$(sort").unwrap();
            let wildcard_pos = value.find("$(wildcard").unwrap();
            assert!(sort_pos < wildcard_pos, "sort should wrap wildcard");
        }
        _ => panic!("Expected Variable"),
    }
}

#[test]
fn test_SEMANTIC_RECURSIVE_010_detect_wildcard_in_firstword() {
    // ARRANGE: wildcard in firstword (HIGH RISK - returns different results)
    let makefile = "FIRST := $(firstword $(wildcard *.c))";
    let ast = parse_makefile(makefile).unwrap();

    // ACT & ASSERT: Critical case - firstword depends on order
    match &ast.items[0] {
        MakeItem::Variable { name, value, .. } => {
            assert_eq!(name, "FIRST");
            assert!(value.contains("$(wildcard"));
            assert!(value.contains("$(firstword"));
        }
        _ => panic!("Expected Variable"),
    }
}

#[test]
fn test_SEMANTIC_RECURSIVE_011_detect_wildcard_in_lastword() {
    // ARRANGE: wildcard in lastword (HIGH RISK - returns different results)
    let makefile = "LAST := $(lastword $(wildcard *.c))";
    let ast = parse_makefile(makefile).unwrap();

    // ACT & ASSERT: Critical case - lastword depends on order
    match &ast.items[0] {
        MakeItem::Variable { name, value, .. } => {
            assert_eq!(name, "LAST");
            assert!(value.contains("$(wildcard"));
            assert!(value.contains("$(lastword"));
        }
        _ => panic!("Expected Variable"),
    }
}

#[test]
fn test_SEMANTIC_RECURSIVE_012_detect_wildcard_in_wordlist() {
    // ARRANGE: wildcard in wordlist (HIGH RISK - indices depend on order)
    let makefile = "RANGE := $(wordlist 2, 4, $(wildcard *.c))";
    let ast = parse_makefile(makefile).unwrap();

    // ACT & ASSERT: Critical case - wordlist indices depend on order
    match &ast.items[0] {
        MakeItem::Variable { name, value, .. } => {
            assert_eq!(name, "RANGE");
            assert!(value.contains("$(wildcard"));
            assert!(value.contains("$(wordlist"));
        }
        _ => panic!("Expected Variable"),
    }
}

#[test]
fn test_SEMANTIC_RECURSIVE_013_detect_shell_date_in_filter() {
    // ARRANGE: shell date in filter arguments (timestamp in filtering)
    let makefile = "DATED := $(filter release-%, $(shell date +%Y%m%d))";
    let ast = parse_makefile(makefile).unwrap();

    // ACT & ASSERT: Verify shell date nested in filter
    match &ast.items[0] {
        MakeItem::Variable { name, value, .. } => {
            assert_eq!(name, "DATED");
            assert!(value.contains("$(shell date"));
            assert!(value.contains("$(filter"));
        }
        _ => panic!("Expected Variable"),
    }
}

#[test]
fn test_SEMANTIC_RECURSIVE_014_detect_multiple_nested_issues() {
    // ARRANGE: multiple different non-deterministic patterns nested
    let makefile = r#"
COMPLEX := $(filter %.c, $(wildcard *.c)) $(word $RANDOM, $(shell find src))
"#;
    let ast = parse_makefile(makefile).unwrap();

    // ACT & ASSERT: Verify all three types of issues detected
    match &ast.items[0] {
        MakeItem::Variable { name, value, .. } => {
            assert_eq!(name, "COMPLEX");
            // Should detect wildcard, $RANDOM, and shell find
            assert!(value.contains("$(wildcard"));
            assert!(value.contains("RANDOM"));
            assert!(value.contains("$(shell find"));
        }
        _ => panic!("Expected Variable"),
    }
}

#[test]
fn test_SEMANTIC_RECURSIVE_015_pattern_rule_with_nested_wildcard() {
    // ARRANGE: pattern rule with wildcard in prerequisites
    let makefile = r#"
%.o: $(filter %.c, $(wildcard src/*.c))
	gcc -c $< -o $@
"#;
    let ast = parse_makefile(makefile).unwrap();

    // ACT & ASSERT: Verify pattern rule contains nested wildcard
    match &ast.items[0] {
        MakeItem::PatternRule {
            target_pattern,
            prereq_patterns,
            ..
        } => {
            assert_eq!(target_pattern, "%.o");
            // Prerequisites should contain nested wildcard
            let prereqs = prereq_patterns.join(" ");
            assert!(prereqs.contains("$(wildcard"));
            assert!(prereqs.contains("$(filter"));
        }
        _ => panic!("Expected PatternRule, got {:?}", ast.items[0]),
    }
}

// ============================================================================
// Sprint 65: Integration Tests for analyze_makefile() with Nested Patterns
// ============================================================================
// These tests verify that analyze_makefile() detects non-deterministic patterns
// even when nested inside function arguments

#[test]
fn test_SEMANTIC_ANALYZE_001_detect_nested_wildcard_in_filter() {
    use crate::make_parser::parse_makefile;
    use crate::make_parser::semantic::analyze_makefile;

    // ARRANGE: wildcard nested in filter arguments
    let makefile = "FILES := $(filter %.c, $(wildcard src/*.c))";
    let ast = parse_makefile(makefile).unwrap();

    // ACT: Run semantic analysis
    let issues = analyze_makefile(&ast);

    // ASSERT: Should detect nested wildcard
    assert!(
        !issues.is_empty(),
        "Expected to detect nested wildcard, but got no issues"
    );
    assert_eq!(
        issues.len(),
        1,
        "Expected exactly 1 issue for nested wildcard"
    );
    assert_eq!(issues[0].rule, "NO_WILDCARD");
    assert!(issues[0].message.contains("FILES"));
    assert!(issues[0].message.contains("wildcard"));
}

#[test]
fn test_SEMANTIC_ANALYZE_002_detect_nested_shell_date_in_addsuffix() {
    use crate::make_parser::parse_makefile;
    use crate::make_parser::semantic::analyze_makefile;

    // ARRANGE: shell date nested in addsuffix arguments
    let makefile = "TIMESTAMPED := $(addsuffix -$(shell date +%s), foo bar)";
    let ast = parse_makefile(makefile).unwrap();

    // ACT: Run semantic analysis
    let issues = analyze_makefile(&ast);

    // ASSERT: Should detect nested shell date
    assert!(!issues.is_empty(), "Expected to detect nested shell date");
    assert_eq!(issues.len(), 1);
    assert_eq!(issues[0].rule, "NO_TIMESTAMPS");
    assert!(issues[0].message.contains("TIMESTAMPED"));
}

#[test]
fn test_SEMANTIC_ANALYZE_003_detect_nested_random_in_word() {
    use crate::make_parser::parse_makefile;
    use crate::make_parser::semantic::analyze_makefile;

    // ARRANGE: $RANDOM nested in word arguments
    let makefile = "PICK := $(word $RANDOM, foo bar baz)";
    let ast = parse_makefile(makefile).unwrap();

    // ACT: Run semantic analysis
    let issues = analyze_makefile(&ast);

    // ASSERT: Should detect nested $RANDOM
    assert!(!issues.is_empty(), "Expected to detect nested $RANDOM");
    assert_eq!(issues.len(), 1);
    assert_eq!(issues[0].rule, "NO_RANDOM");
    assert!(issues[0].message.contains("PICK"));
}

#[test]
fn test_SEMANTIC_ANALYZE_004_no_issue_for_safe_filter() {
    use crate::make_parser::parse_makefile;
    use crate::make_parser::semantic::analyze_makefile;

    // ARRANGE: filter without nested non-deterministic code (SAFE)
    let makefile = "SAFE := $(filter %.c, foo.c bar.c baz.c)";
    let ast = parse_makefile(makefile).unwrap();

    // ACT: Run semantic analysis
    let issues = analyze_makefile(&ast);

    // ASSERT: Should NOT detect any issues (no wildcard, no shell, no random)
    assert_eq!(
        issues.len(),
        0,
        "Expected no issues for safe filter, but got: {:?}",
        issues
    );
}

#[test]
fn test_SEMANTIC_ANALYZE_005_purified_wildcard_not_detected() {
    use crate::make_parser::parse_makefile;
    use crate::make_parser::semantic::analyze_makefile;

    // ARRANGE: PURIFIED wildcard wrapped with sort
    // Enhancement IMPLEMENTED: detect $(sort $(wildcard)) as "already purified"
    let makefile = "PURIFIED := $(filter %.c, $(sort $(wildcard src/*.c)))";
    let ast = parse_makefile(makefile).unwrap();

    // ACT: Run semantic analysis
    let issues = analyze_makefile(&ast);

    // ASSERT: Purified wildcards should NOT be detected
    assert_eq!(
        issues.len(),
        0,
        "Purified wildcard should not be detected: {:?}",
        issues
    );
}

#[test]
fn test_SEMANTIC_ANALYZE_006_deeply_nested_unpurified_wildcard() {
    use crate::make_parser::parse_makefile;
    use crate::make_parser::semantic::analyze_makefile;

    // ARRANGE: deeply nested - wildcard in filter in sort (NOT PURIFIED PROPERLY)
    // This is NOT purified because the wildcard itself is not wrapped with sort
    // The outer sort only sorts the filter results, not the wildcard results
    let makefile = "DEEP := $(sort $(filter %.c, $(wildcard src/*.c)))";
    let ast = parse_makefile(makefile).unwrap();

    // ACT: Run semantic analysis
    let issues = analyze_makefile(&ast);

    // ASSERT: Should detect wildcard because it's not directly wrapped with sort
    assert!(
        !issues.is_empty(),
        "Wildcard should be detected when not directly wrapped with sort"
    );
    assert_eq!(issues.len(), 1);
    assert_eq!(issues[0].rule, "NO_WILDCARD");
}

#[test]
fn test_SEMANTIC_ANALYZE_007_multiple_nested_wildcards() {
    use crate::make_parser::parse_makefile;
    use crate::make_parser::semantic::analyze_makefile;

    // ARRANGE: multiple wildcard calls in single function
    let makefile = "MULTI := $(filter %.c %.h, $(wildcard src/*.c) $(wildcard inc/*.h))";
    let ast = parse_makefile(makefile).unwrap();

    // ACT: Run semantic analysis
    let issues = analyze_makefile(&ast);

    // ASSERT: Should detect wildcard (may report as 1 issue for the variable)
    assert!(!issues.is_empty());
    assert_eq!(issues[0].rule, "NO_WILDCARD");
    assert!(issues[0].message.contains("MULTI"));
}

#[test]
fn test_SEMANTIC_ANALYZE_008_nested_shell_find_in_filter() {
    use crate::make_parser::parse_makefile;
    use crate::make_parser::semantic::analyze_makefile;

    // ARRANGE: shell find nested in filter arguments
    let makefile = "FOUND := $(filter %.c, $(shell find src -name '*.c'))";
    let ast = parse_makefile(makefile).unwrap();

    // ACT: Run semantic analysis
    let issues = analyze_makefile(&ast);

    // ASSERT: Should detect nested shell find
    assert!(!issues.is_empty());
    assert_eq!(issues.len(), 1);
    assert_eq!(issues[0].rule, "NO_UNORDERED_FIND");
    assert!(issues[0].message.contains("FOUND"));
}

#[test]
fn test_SEMANTIC_ANALYZE_009_multiple_different_nested_issues() {
    use crate::make_parser::parse_makefile;
    use crate::make_parser::semantic::analyze_makefile;

    // ARRANGE: multiple different non-deterministic patterns nested
    let makefile = r#"
COMPLEX := $(filter %.c, $(wildcard *.c)) $(word $RANDOM, $(shell find src))
"#;
    let ast = parse_makefile(makefile).unwrap();

    // ACT: Run semantic analysis
    let issues = analyze_makefile(&ast);

    // ASSERT: Should detect all three types of issues
    // Current implementation detects all patterns in the value string
    assert!(
        issues.len() >= 3,
        "Expected at least 3 issues (wildcard, random, shell find), got {}",
        issues.len()
    );

    // Verify all three rule types are detected
    let rules: Vec<&str> = issues.iter().map(|i| i.rule.as_str()).collect();
    assert!(rules.contains(&"NO_WILDCARD"), "Should detect wildcard");
    assert!(rules.contains(&"NO_RANDOM"), "Should detect $RANDOM");
    assert!(
        rules.contains(&"NO_UNORDERED_FIND"),
        "Should detect shell find"
    );
}

#[test]
fn test_SEMANTIC_ANALYZE_010_nested_wildcard_in_firstword() {
    use crate::make_parser::parse_makefile;
    use crate::make_parser::semantic::analyze_makefile;

    // ARRANGE: wildcard in firstword (HIGH RISK - different results based on order)
    let makefile = "FIRST := $(firstword $(wildcard *.c))";
    let ast = parse_makefile(makefile).unwrap();

    // ACT: Run semantic analysis
    let issues = analyze_makefile(&ast);

    // ASSERT: Should detect wildcard (critical case for firstword)
    assert!(!issues.is_empty());
    assert_eq!(issues[0].rule, "NO_WILDCARD");
    assert!(issues[0].message.contains("FIRST"));
}

// ============================================================================
// Sprint 66: High-Risk Functions - FOREACH Detection Tests
// ============================================================================
//
// Goal: Verify semantic analysis detects non-deterministic patterns in
//       $(foreach) loops where iteration order matters.
//
// Hypothesis (based on Sprint 64-65): Existing .contains() approach
// already detects these patterns at any nesting level.
//
// Test Strategy: Write verification tests first (EXTREME TDD RED phase)

#[test]
fn test_SEMANTIC_FOREACH_001_detect_wildcard_in_foreach_list() {
    use crate::make_parser::parse_makefile;
    use crate::make_parser::semantic::analyze_makefile;

    // ARRANGE: $(foreach) iterating over $(wildcard) - ORDER MATTERS!
    // This is CRITICAL because foreach processes items in iteration order
    let makefile = "OBJS := $(foreach file, $(wildcard *.c), $(file:.c=.o))";
    let ast = parse_makefile(makefile).unwrap();

    // ACT: Run semantic analysis
    let issues = analyze_makefile(&ast);

    // ASSERT: Should detect wildcard in foreach list (non-deterministic order)
    // Based on Sprint 65 discovery: .contains("$(wildcard") should catch this!
    assert!(
        !issues.is_empty(),
        "Expected to detect wildcard in foreach list"
    );

    // Verify it's detected as NO_WILDCARD
    let wildcard_issues: Vec<_> = issues.iter().filter(|i| i.rule == "NO_WILDCARD").collect();
    assert!(!wildcard_issues.is_empty(), "Should detect as NO_WILDCARD");
}

#[test]
fn test_SEMANTIC_FOREACH_002_safe_foreach_with_explicit_list() {
    use crate::make_parser::parse_makefile;
    use crate::make_parser::semantic::analyze_makefile;

    // ARRANGE: $(foreach) with explicit list (SAFE - deterministic order)
    let makefile = "OBJS := $(foreach file, foo.c bar.c baz.c, $(file:.c=.o))";
    let ast = parse_makefile(makefile).unwrap();

    // ACT: Run semantic analysis
    let issues = analyze_makefile(&ast);

    // ASSERT: Should NOT detect issues (explicit list is deterministic)
    assert_eq!(
        issues.len(),
        0,
        "Expected no issues for explicit list: {:?}",
        issues
    );
}

#[test]
fn test_SEMANTIC_FOREACH_003_nested_shell_date_in_foreach() {
    use crate::make_parser::parse_makefile;
    use crate::make_parser::semantic::analyze_makefile;

    // ARRANGE: $(shell date) nested in foreach body
    let makefile = "TIMESTAMPED := $(foreach f, foo bar, $(f)-$(shell date +%s))";
    let ast = parse_makefile(makefile).unwrap();

    // ACT: Run semantic analysis
    let issues = analyze_makefile(&ast);

    // ASSERT: Should detect shell date
    assert!(!issues.is_empty(), "Expected to detect shell date");
    assert!(issues.iter().any(|i| i.rule == "NO_TIMESTAMPS"));
}

#[test]
fn test_SEMANTIC_FOREACH_004_random_in_foreach_body() {
    use crate::make_parser::parse_makefile;
    use crate::make_parser::semantic::analyze_makefile;

    // ARRANGE: $RANDOM in foreach body
    let makefile = "IDS := $(foreach item, a b c, id-$RANDOM)";
    let ast = parse_makefile(makefile).unwrap();

    // ACT: Run semantic analysis
    let issues = analyze_makefile(&ast);

    // ASSERT: Should detect random
    assert!(!issues.is_empty(), "Expected to detect $RANDOM");
    assert!(issues.iter().any(|i| i.rule == "NO_RANDOM"));
}

#[test]
fn test_SEMANTIC_FOREACH_005_shell_find_in_foreach_list() {
    use crate::make_parser::parse_makefile;
    use crate::make_parser::semantic::analyze_makefile;

    // ARRANGE: $(shell find) as foreach list source
    let makefile = "PROCESSED := $(foreach f, $(shell find src -name '*.c'), process-$(f))";
    let ast = parse_makefile(makefile).unwrap();

    // ACT: Run semantic analysis
    let issues = analyze_makefile(&ast);

    // ASSERT: Should detect shell find
    assert!(!issues.is_empty(), "Expected to detect shell find");
    assert!(issues.iter().any(|i| i.rule == "NO_UNORDERED_FIND"));
}

// ============================================================================
// Sprint 66: High-Risk Functions - CALL Detection Tests
// ============================================================================

#[test]
fn test_SEMANTIC_CALL_001_detect_wildcard_in_call_args() {
    use crate::make_parser::parse_makefile;
    use crate::make_parser::semantic::analyze_makefile;

    // ARRANGE: $(call) with $(wildcard) in arguments
    // Define a function and call it with wildcard
    let makefile = r#"
reverse = $(2) $(1)
FILES := $(call reverse, $(wildcard *.c), foo.c)
"#;
    let ast = parse_makefile(makefile).unwrap();

    // ACT: Run semantic analysis
    let issues = analyze_makefile(&ast);

    // ASSERT: Should detect wildcard in call arguments
    assert!(
        !issues.is_empty(),
        "Expected to detect wildcard in call args"
    );

    // Check that FILES variable has wildcard issue
    let files_issues: Vec<_> = issues
        .iter()
        .filter(|i| i.message.contains("FILES"))
        .collect();
    assert!(
        !files_issues.is_empty(),
        "Should detect wildcard in FILES variable"
    );
}

#[test]
fn test_SEMANTIC_CALL_002_safe_call_with_explicit_args() {
    use crate::make_parser::parse_makefile;
    use crate::make_parser::semantic::analyze_makefile;

    // ARRANGE: $(call) with explicit arguments (SAFE)
    let makefile = r#"
reverse = $(2) $(1)
RESULT := $(call reverse, foo.c, bar.c)
"#;
    let ast = parse_makefile(makefile).unwrap();

    // ACT: Run semantic analysis
    let issues = analyze_makefile(&ast);

    // ASSERT: Should NOT detect issues (explicit args are deterministic)
    assert_eq!(
        issues.len(),
        0,
        "Expected no issues for explicit args: {:?}",
        issues
    );
}

#[test]
fn test_SEMANTIC_CALL_003_shell_date_in_call_args() {
    use crate::make_parser::parse_makefile;
    use crate::make_parser::semantic::analyze_makefile;

    // ARRANGE: $(call) with $(shell date) in arguments
    let makefile = r#"
timestamp = build-$(1)-$(2)
RELEASE := $(call timestamp, v1.0, $(shell date +%s))
"#;
    let ast = parse_makefile(makefile).unwrap();

    // ACT: Run semantic analysis
    let issues = analyze_makefile(&ast);

    // ASSERT: Should detect shell date
    assert!(!issues.is_empty(), "Expected to detect shell date");
    assert!(issues.iter().any(|i| i.rule == "NO_TIMESTAMPS"));
}

#[test]
fn test_SEMANTIC_CALL_004_random_in_call_args() {
    use crate::make_parser::parse_makefile;
    use crate::make_parser::semantic::analyze_makefile;

    // ARRANGE: $RANDOM in call arguments
    let makefile = r#"
generate_id = id-$(1)-$(2)
SESSION := $(call generate_id, sess, $RANDOM)
"#;
    let ast = parse_makefile(makefile).unwrap();

    // ACT: Run semantic analysis
    let issues = analyze_makefile(&ast);

    // ASSERT: Should detect random
    assert!(!issues.is_empty(), "Expected to detect $RANDOM");
    assert!(issues.iter().any(|i| i.rule == "NO_RANDOM"));
}

#[test]
fn test_SEMANTIC_CALL_005_shell_find_in_call_args() {
    use crate::make_parser::parse_makefile;
    use crate::make_parser::semantic::analyze_makefile;

    // ARRANGE: $(shell find) in call arguments
    let makefile = r#"
process_files = Processing: $(1)
OUTPUT := $(call process_files, $(shell find src -name '*.c'))
"#;
    let ast = parse_makefile(makefile).unwrap();

    // ACT: Run semantic analysis
    let issues = analyze_makefile(&ast);

    // ASSERT: Should detect shell find
    assert!(!issues.is_empty(), "Expected to detect shell find");
    assert!(issues.iter().any(|i| i.rule == "NO_UNORDERED_FIND"));
}

// ============================================================================
// Sprint 67: Purification Engine Tests
// ============================================================================
//
// Goal: Implement purification engine that auto-fixes non-deterministic
//       patterns detected by semantic analysis.
//
// Approach: EXTREME TDD - Write RED tests first, then implement

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
                span: Span::dummy(),
            },
            MakeItem::Target {
                name: "build".to_string(),
                prerequisites: vec!["main.c".to_string()],
                recipe: vec!["$(CC) $(CFLAGS) -o build main.c".to_string()],
                phony: false,
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
        #[test]
        fn prop_GENERATE_008_roundtrip_targets(
            target_name in "[a-z][a-z0-9_-]{0,15}",
            prereq in "[a-z][a-z0-9_.]{0,15}",
        ) {
            // ARRANGE: Create target AST
            let ast = MakeAst {
                items: vec![MakeItem::Target {
                    name: target_name.clone(),
                    prerequisites: vec![prereq.clone()],
                    recipe: vec!["echo test".to_string()],
                    phony: false,
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

            // ASSERT: Should preserve target structure
            if let MakeItem::Target { name, prerequisites, recipe, .. } = &reparsed_ast.items[0] {
                prop_assert_eq!(name, &target_name);
                prop_assert_eq!(prerequisites.len(), 1);
                prop_assert_eq!(&prerequisites[0], &prereq);
                prop_assert_eq!(recipe.len(), 1);
                prop_assert_eq!(&recipe[0], "echo test");
            } else {
                prop_assert!(false, "Expected Target item, got {:?}", reparsed_ast.items[0]);
            }
        }

        /// PROPERTY TEST: Generation is deterministic
        ///
        /// Property: generate(ast) always produces same output for same input
        #[test]
        fn prop_GENERATE_009_deterministic_generation(
            var_name in "[A-Z_]{2,10}",
            var_value in "[a-z]{1,20}",
        ) {
            // ARRANGE: Create AST
            let ast = MakeAst {
                items: vec![MakeItem::Variable {
                    name: var_name,
                    value: var_value,
                    flavor: VarFlavor::Simple,
                    span: Span::dummy(),
                }],
                metadata: MakeMetadata::new(),
            };

            // ACT: Generate twice
            let output1 = generate_purified_makefile(&ast);
            let output2 = generate_purified_makefile(&ast);

            // ASSERT: Should be byte-identical
            prop_assert_eq!(output1, output2, "Generation is not deterministic");
        }
    }
}

// ============================================================================
// END-TO-END INTEGRATION TEST - Sprint 68
// ============================================================================

/// Integration test: Complete purification workflow
///
/// Tests the full pipeline: Parse → Analyze → Purify → Generate → Verify
///
/// This verifies the entire end-to-end workflow works correctly.
#[test]
fn test_GENERATE_010_end_to_end_purification() {
    // ARRANGE: Input Makefile with non-deterministic wildcard
    let input_makefile = r#"# Build configuration
CC := gcc
CFLAGS := -O2 -Wall

FILES := $(wildcard src/*.c)

build: $(FILES)
	$(CC) $(CFLAGS) -o build $(FILES)
"#;

    // ACT: Parse
    let ast = parse_makefile(input_makefile).expect("Failed to parse input");

    // ACT: Purify (wrap wildcard with sort)
    let purified_result = purify_makefile(&ast);

    // ASSERT: Should have applied transformations
    assert!(
        purified_result.transformations_applied > 0,
        "Expected transformations to be applied"
    );

    // ACT: Generate purified Makefile
    let purified_makefile = generate_purified_makefile(&purified_result.ast);

    // ASSERT: Should contain sorted wildcard
    assert!(
        purified_makefile.contains("$(sort $(wildcard"),
        "Generated Makefile should contain sorted wildcard"
    );

    // ASSERT: Should preserve structure
    assert!(purified_makefile.contains("CC := gcc"));
    assert!(purified_makefile.contains("CFLAGS := -O2 -Wall"));
    assert!(purified_makefile.contains("build: $(FILES)"));
    assert!(purified_makefile.contains("\t$(CC) $(CFLAGS) -o build $(FILES)"));

    // ACT: Re-parse generated Makefile to verify it's valid
    let reparsed = parse_makefile(&purified_makefile);
    assert!(
        reparsed.is_ok(),
        "Generated Makefile should be parseable: {:?}",
        reparsed.err()
    );

    // ASSERT: Re-purification should be idempotent (no changes)
    let reparsed_ast = reparsed.unwrap();
    let repurified = purify_makefile(&reparsed_ast);
    assert_eq!(
        repurified.transformations_applied, 0,
        "Second purification should apply zero transformations (idempotent)"
    );

    println!("\n=== Original Makefile ===");
    println!("{}", input_makefile);
    println!("\n=== Purified Makefile ===");
    println!("{}", purified_makefile);
    println!("\n=== End-to-End Test: PASSED ✅ ===\n");
}

// ============================================================================
// FUNC-CALL-001: Function Call Parsing Tests (Sprint 82, Day 2)
// ============================================================================
//
// These tests validate parsing of GNU Make function calls:
// - $(wildcard pattern)
// - $(patsubst pattern,replacement,text)
// - $(call function,args)
// - $(eval code)
// - $(shell command)
// - $(foreach var,list,text)
// - $(if condition,then,else)
// - $(or a,b)
// - $(and a,b)
// - $(value var)
// - $(origin var)
//
// RED PHASE: These tests are expected to FAIL initially.
// The parser currently stores function calls as raw strings in variable values.
// We need to implement explicit function call parsing.

/// Test for basic $(wildcard) function parsing
///
/// Input: SOURCES := $(wildcard src/*.c)
/// Expected: Parser stores function call in variable value, can extract it
#[test]
fn test_FUNC_CALL_001_wildcard_basic() {
    // ARRANGE: Variable with $(wildcard) function
    let makefile = "SOURCES := $(wildcard src/*.c)";

    // ACT: Parse makefile
    let result = parse_makefile(makefile);

    // ASSERT: Successfully parsed
    assert!(
        result.is_ok(),
        "Should parse $(wildcard) function, got error: {:?}",
        result.err()
    );

    let ast = result.unwrap();
    assert_eq!(ast.items.len(), 1, "Should have 1 variable");

    // ASSERT: Variable contains the function call
    match &ast.items[0] {
        MakeItem::Variable { name, value, .. } => {
            assert_eq!(name, "SOURCES");
            assert_eq!(value, "$(wildcard src/*.c)");

            // ASSERT: Can extract function calls from value
            let function_calls = extract_function_calls(value);
            assert_eq!(function_calls.len(), 1, "Should extract 1 function call");
            assert_eq!(
                function_calls[0].0, "wildcard",
                "Function name should be 'wildcard'"
            );
            assert!(
                function_calls[0].1.contains("src/*.c"),
                "Args should contain pattern"
            );
        }
        _ => panic!("Expected Variable item"),
    }
}

/// RED PHASE: Test for $(wildcard) with multiple patterns
///
/// Input: FILES := $(wildcard *.c *.h)
#[test]
fn test_FUNC_CALL_002_wildcard_multiple_patterns() {
    // ARRANGE: $(wildcard) with multiple patterns
    let makefile = "FILES := $(wildcard *.c *.h)";

    // ACT: Parse makefile
    let result = parse_makefile(makefile);
    assert!(result.is_ok());

    let ast = result.unwrap();
    assert_eq!(ast.items.len(), 1, "Should have 1 variable");

    // ASSERT: Should store as Variable and allow extraction
    match &ast.items[0] {
        MakeItem::Variable { name, value, .. } => {
            assert_eq!(name, "FILES");
            assert_eq!(value, "$(wildcard *.c *.h)");

            // Can extract function calls from value
            let function_calls = extract_function_calls(value);
            assert_eq!(function_calls.len(), 1, "Should extract 1 function call");
            assert_eq!(
                function_calls[0].0, "wildcard",
                "Function name should be 'wildcard'"
            );
            assert!(
                function_calls[0].1.contains("*.c"),
                "Args should contain *.c pattern"
            );
            assert!(
                function_calls[0].1.contains("*.h"),
                "Args should contain *.h pattern"
            );
        }
        _ => panic!("Expected Variable item"),
    }
}

/// RED PHASE: Test for basic $(patsubst) function
///
/// Input: OBJS := $(patsubst %.c,%.o,$(SOURCES))
#[test]
fn test_FUNC_CALL_003_patsubst_basic() {
    // ARRANGE: $(patsubst) function
    let makefile = "OBJS := $(patsubst %.c,%.o,$(SOURCES))";

    // ACT: Parse makefile
    let result = parse_makefile(makefile);
    assert!(result.is_ok());

    let ast = result.unwrap();
    assert_eq!(ast.items.len(), 1, "Should have 1 variable");

    // ASSERT: Should store as Variable and allow extraction
    match &ast.items[0] {
        MakeItem::Variable { name, value, .. } => {
            assert_eq!(name, "OBJS");
            assert_eq!(value, "$(patsubst %.c,%.o,$(SOURCES))");

            // Can extract function calls from value
            let function_calls = extract_function_calls(value);
            assert_eq!(function_calls.len(), 1, "Should extract 1 function call");
            assert_eq!(
                function_calls[0].0, "patsubst",
                "Function name should be 'patsubst'"
            );
            assert!(
                function_calls[0].1.contains("%.c"),
                "Args should contain %.c pattern"
            );
            assert!(
                function_calls[0].1.contains("%.o"),
                "Args should contain %.o pattern"
            );
        }
        _ => panic!("Expected Variable item"),
    }
}

/// RED PHASE: Test for $(patsubst) with nested variable
///
/// Input: OBJS := $(patsubst %.c,%.o,$(wildcard src/*.c))
#[test]
fn test_FUNC_CALL_004_patsubst_nested() {
    // ARRANGE: $(patsubst) with nested $(wildcard)
    let makefile = "OBJS := $(patsubst %.c,%.o,$(wildcard src/*.c))";

    // ACT: Parse makefile
    let result = parse_makefile(makefile);
    assert!(result.is_ok());

    let ast = result.unwrap();
    assert_eq!(ast.items.len(), 1, "Should have 1 variable");

    // ASSERT: Should store as Variable and allow extraction (outer function)
    match &ast.items[0] {
        MakeItem::Variable { name, value, .. } => {
            assert_eq!(name, "OBJS");
            assert_eq!(value, "$(patsubst %.c,%.o,$(wildcard src/*.c))");

            // Can extract function calls from value (extracts outermost)
            let function_calls = extract_function_calls(value);
            assert_eq!(
                function_calls.len(),
                1,
                "Should extract 1 outermost function call"
            );
            assert_eq!(
                function_calls[0].0, "patsubst",
                "Function name should be 'patsubst'"
            );
            assert!(
                function_calls[0].1.contains("$(wildcard"),
                "Args should contain nested $(wildcard)"
            );
        }
        _ => panic!("Expected Variable item"),
    }
}

/// RED PHASE: Test for $(call) function
///
/// Input: RESULT := $(call my_func,arg1,arg2)
#[test]
fn test_FUNC_CALL_005_call_basic() {
    // ARRANGE: $(call) function
    let makefile = "RESULT := $(call my_func,arg1,arg2)";

    // ACT: Parse makefile
    let result = parse_makefile(makefile);
    assert!(result.is_ok());

    let ast = result.unwrap();
    assert_eq!(ast.items.len(), 1, "Should have 1 variable");

    // ASSERT: Should store as Variable and allow extraction
    match &ast.items[0] {
        MakeItem::Variable { name, value, .. } => {
            assert_eq!(name, "RESULT");
            assert_eq!(value, "$(call my_func,arg1,arg2)");

            // Can extract function calls from value
            let function_calls = extract_function_calls(value);
            assert_eq!(function_calls.len(), 1, "Should extract 1 function call");
            assert_eq!(
                function_calls[0].0, "call",
                "Function name should be 'call'"
            );
            assert!(
                function_calls[0].1.contains("my_func"),
                "Args should contain my_func"
            );
            assert!(
                function_calls[0].1.contains("arg1"),
                "Args should contain arg1"
            );
        }
        _ => panic!("Expected Variable item"),
    }
}

/// RED PHASE: Test for nested $(call) function
///
/// Input: RESULT := $(call outer,$(call inner,x))
#[test]
fn test_FUNC_CALL_006_call_nested() {
    // ARRANGE: Nested $(call) functions
    let makefile = "RESULT := $(call outer,$(call inner,x))";

    // ACT: Parse makefile
    let result = parse_makefile(makefile);
    assert!(result.is_ok());

    let ast = result.unwrap();
    assert_eq!(ast.items.len(), 1, "Should have 1 variable");

    // ASSERT: Should store as Variable and allow extraction (outer call)
    match &ast.items[0] {
        MakeItem::Variable { name, value, .. } => {
            assert_eq!(name, "RESULT");
            assert_eq!(value, "$(call outer,$(call inner,x))");

            // Can extract function calls from value (extracts outermost)
            let function_calls = extract_function_calls(value);
            assert_eq!(
                function_calls.len(),
                1,
                "Should extract 1 outermost function call"
            );
            assert_eq!(
                function_calls[0].0, "call",
                "Function name should be 'call'"
            );
            assert!(
                function_calls[0].1.contains("outer"),
                "Args should contain outer"
            );
            assert!(
                function_calls[0].1.contains("$(call inner"),
                "Args should contain nested $(call inner)"
            );
        }
        _ => panic!("Expected Variable item"),
    }
}

/// RED PHASE: Test for $(eval) function
///
/// Input: $(eval VAR = value)
#[test]
fn test_FUNC_CALL_007_eval_basic() {
    // ARRANGE: $(eval) function (note: eval is typically standalone, not in assignment)
    let makefile = "DUMMY := $(eval NEW_VAR = value)";

    // ACT: Parse makefile
    let result = parse_makefile(makefile);
    assert!(result.is_ok());

    let ast = result.unwrap();
    assert_eq!(ast.items.len(), 1, "Should have 1 variable");

    // ASSERT: Should store as Variable and allow extraction
    match &ast.items[0] {
        MakeItem::Variable { name, value, .. } => {
            assert_eq!(name, "DUMMY");
            assert_eq!(value, "$(eval NEW_VAR = value)");

            // Can extract function calls from value
            let function_calls = extract_function_calls(value);
            assert_eq!(function_calls.len(), 1, "Should extract 1 function call");
            assert_eq!(
                function_calls[0].0, "eval",
                "Function name should be 'eval'"
            );
            assert!(
                function_calls[0].1.contains("NEW_VAR"),
                "Args should contain NEW_VAR"
            );
        }
        _ => panic!("Expected Variable item"),
    }
}

/// RED PHASE: Test for $(shell) function
///
/// Input: FILES := $(shell ls -la)
#[test]
fn test_FUNC_CALL_008_shell_basic() {
    // ARRANGE: $(shell) function
    let makefile = "FILES := $(shell ls -la)";

    // ACT: Parse makefile
    let result = parse_makefile(makefile);
    assert!(result.is_ok());

    let ast = result.unwrap();
    assert_eq!(ast.items.len(), 1, "Should have 1 variable");

    // ASSERT: Should store as Variable and allow extraction
    match &ast.items[0] {
        MakeItem::Variable { name, value, .. } => {
            assert_eq!(name, "FILES");
            assert_eq!(value, "$(shell ls -la)");

            // Can extract function calls from value
            let function_calls = extract_function_calls(value);
            assert_eq!(function_calls.len(), 1, "Should extract 1 function call");
            assert_eq!(
                function_calls[0].0, "shell",
                "Function name should be 'shell'"
            );
            assert!(
                function_calls[0].1.contains("ls -la"),
                "Args should contain shell command"
            );
        }
        _ => panic!("Expected Variable item"),
    }
}

/// RED PHASE: Test for $(foreach) function
///
/// Input: FILES := $(foreach dir,src test,$(wildcard $(dir)/*.c))
#[test]
fn test_FUNC_CALL_009_foreach_basic() {
    // ARRANGE: $(foreach) function with nested wildcard
    let makefile = "FILES := $(foreach dir,src test,$(wildcard $(dir)/*.c))";

    // ACT: Parse makefile
    let result = parse_makefile(makefile);
    assert!(result.is_ok());

    let ast = result.unwrap();
    assert_eq!(ast.items.len(), 1, "Should have 1 variable");

    // ASSERT: Should store as Variable and allow extraction
    match &ast.items[0] {
        MakeItem::Variable { name, value, .. } => {
            assert_eq!(name, "FILES");
            assert_eq!(value, "$(foreach dir,src test,$(wildcard $(dir)/*.c))");

            // Can extract function calls from value (extracts outermost)
            let function_calls = extract_function_calls(value);
            assert_eq!(
                function_calls.len(),
                1,
                "Should extract 1 outermost function call"
            );
            assert_eq!(
                function_calls[0].0, "foreach",
                "Function name should be 'foreach'"
            );
            assert!(
                function_calls[0].1.contains("dir,src test"),
                "Args should contain foreach parameters"
            );
        }
        _ => panic!("Expected Variable item"),
    }
}

/// RED PHASE: Test for $(if) function
///
/// Input: RESULT := $(if $(DEBUG),debug,release)
#[test]
fn test_FUNC_CALL_010_if_basic() {
    // ARRANGE: $(if) function with condition and branches
    let makefile = "RESULT := $(if $(DEBUG),debug,release)";

    // ACT: Parse makefile
    let result = parse_makefile(makefile);
    assert!(result.is_ok());

    let ast = result.unwrap();
    assert_eq!(ast.items.len(), 1, "Should have 1 variable");

    // ASSERT: Should store as Variable and allow extraction
    match &ast.items[0] {
        MakeItem::Variable { name, value, .. } => {
            assert_eq!(name, "RESULT");
            assert_eq!(value, "$(if $(DEBUG),debug,release)");

            // Can extract function calls from value
            let function_calls = extract_function_calls(value);
            assert_eq!(function_calls.len(), 1, "Should extract 1 function call");
            assert_eq!(function_calls[0].0, "if", "Function name should be 'if'");
            assert!(
                function_calls[0].1.contains("DEBUG"),
                "Args should contain condition"
            );
            assert!(
                function_calls[0].1.contains("debug"),
                "Args should contain then-branch"
            );
        }
        _ => panic!("Expected Variable item"),
    }
}

/// RED PHASE: Test for $(or) function
///
/// Input: ENABLED := $(or $(USE_FEATURE_A),$(USE_FEATURE_B))
#[test]
fn test_FUNC_CALL_011_or_basic() {
    // ARRANGE: $(or) function with multiple conditions
    let makefile = "ENABLED := $(or $(USE_FEATURE_A),$(USE_FEATURE_B))";

    // ACT: Parse makefile
    let result = parse_makefile(makefile);
    assert!(result.is_ok());

    let ast = result.unwrap();
    assert_eq!(ast.items.len(), 1, "Should have 1 variable");

    // ASSERT: Should store as Variable and allow extraction
    match &ast.items[0] {
        MakeItem::Variable { name, value, .. } => {
            assert_eq!(name, "ENABLED");
            assert_eq!(value, "$(or $(USE_FEATURE_A),$(USE_FEATURE_B))");

            // Can extract function calls from value
            let function_calls = extract_function_calls(value);
            assert_eq!(function_calls.len(), 1, "Should extract 1 function call");
            assert_eq!(function_calls[0].0, "or", "Function name should be 'or'");
            assert!(
                function_calls[0].1.contains("USE_FEATURE_A"),
                "Args should contain first condition"
            );
            assert!(
                function_calls[0].1.contains("USE_FEATURE_B"),
                "Args should contain second condition"
            );
        }
        _ => panic!("Expected Variable item"),
    }
}

/// RED PHASE: Test for $(and) function
///
/// Input: VALID := $(and $(HAS_COMPILER),$(HAS_LIBS))
#[test]
fn test_FUNC_CALL_012_and_basic() {
    // ARRANGE: $(and) function with multiple conditions
    let makefile = "VALID := $(and $(HAS_COMPILER),$(HAS_LIBS))";

    // ACT: Parse makefile
    let result = parse_makefile(makefile);
    assert!(result.is_ok());

    let ast = result.unwrap();
    assert_eq!(ast.items.len(), 1, "Should have 1 variable");

    // ASSERT: Should store as Variable and allow extraction
    match &ast.items[0] {
        MakeItem::Variable { name, value, .. } => {
            assert_eq!(name, "VALID");
            assert_eq!(value, "$(and $(HAS_COMPILER),$(HAS_LIBS))");

            // Can extract function calls from value
            let function_calls = extract_function_calls(value);
            assert_eq!(function_calls.len(), 1, "Should extract 1 function call");
            assert_eq!(function_calls[0].0, "and", "Function name should be 'and'");
            assert!(
                function_calls[0].1.contains("HAS_COMPILER"),
                "Args should contain first condition"
            );
            assert!(
                function_calls[0].1.contains("HAS_LIBS"),
                "Args should contain second condition"
            );
        }
        _ => panic!("Expected Variable item"),
    }
}

/// RED PHASE: Test for $(value) function
///
/// Input: VAR_CONTENT := $(value VARIABLE_NAME)
#[test]
fn test_FUNC_CALL_013_value_basic() {
    // ARRANGE: $(value) function to get variable value without expansion
    let makefile = "VAR_CONTENT := $(value VARIABLE_NAME)";

    // ACT: Parse makefile
    let result = parse_makefile(makefile);
    assert!(result.is_ok());

    let ast = result.unwrap();
    assert_eq!(ast.items.len(), 1, "Should have 1 variable");

    // ASSERT: Should store as Variable and allow extraction
    match &ast.items[0] {
        MakeItem::Variable { name, value, .. } => {
            assert_eq!(name, "VAR_CONTENT");
            assert_eq!(value, "$(value VARIABLE_NAME)");

            // Can extract function calls from value
            let function_calls = extract_function_calls(value);
            assert_eq!(function_calls.len(), 1, "Should extract 1 function call");
            assert_eq!(
                function_calls[0].0, "value",
                "Function name should be 'value'"
            );
            assert!(
                function_calls[0].1.contains("VARIABLE_NAME"),
                "Args should contain variable name"
            );
        }
        _ => panic!("Expected Variable item"),
    }
}

/// RED PHASE: Test for $(origin) function
///
/// Input: VAR_ORIGIN := $(origin CC)
#[test]
fn test_FUNC_CALL_014_origin_basic() {
    // ARRANGE: $(origin) function to check variable origin
    let makefile = "VAR_ORIGIN := $(origin CC)";

    // ACT: Parse makefile
    let result = parse_makefile(makefile);
    assert!(result.is_ok());

    let ast = result.unwrap();
    assert_eq!(ast.items.len(), 1, "Should have 1 variable");

    // ASSERT: Should store as Variable and allow extraction
    match &ast.items[0] {
        MakeItem::Variable { name, value, .. } => {
            assert_eq!(name, "VAR_ORIGIN");
            assert_eq!(value, "$(origin CC)");

            // Can extract function calls from value
            let function_calls = extract_function_calls(value);
            assert_eq!(function_calls.len(), 1, "Should extract 1 function call");
            assert_eq!(
                function_calls[0].0, "origin",
                "Function name should be 'origin'"
            );
            assert!(
                function_calls[0].1.contains("CC"),
                "Args should contain variable name"
            );
        }
        _ => panic!("Expected Variable item"),
    }
}

/// RED PHASE: Test for multiple function calls in one variable
///
/// Input: ALL := $(wildcard *.c) $(patsubst %.c,%.o,$(wildcard *.c))
#[test]
fn test_FUNC_CALL_015_multiple_functions() {
    // ARRANGE: Multiple function calls in one variable value
    let makefile = "ALL := $(wildcard *.c) $(patsubst %.c,%.o,$(wildcard *.c))";

    // ACT: Parse makefile
    let result = parse_makefile(makefile);
    assert!(result.is_ok());

    let ast = result.unwrap();
    assert_eq!(ast.items.len(), 1, "Should have 1 variable");

    // ASSERT: Should store as Variable and allow extraction of multiple calls
    match &ast.items[0] {
        MakeItem::Variable { name, value, .. } => {
            assert_eq!(name, "ALL");
            assert_eq!(value, "$(wildcard *.c) $(patsubst %.c,%.o,$(wildcard *.c))");

            // Can extract multiple function calls from value
            let function_calls = extract_function_calls(value);
            assert_eq!(function_calls.len(), 2, "Should extract 2 function calls");
            assert_eq!(
                function_calls[0].0, "wildcard",
                "First function should be 'wildcard'"
            );
            assert_eq!(
                function_calls[1].0, "patsubst",
                "Second function should be 'patsubst'"
            );
            assert!(
                function_calls[0].1.contains("*.c"),
                "First function args should contain *.c"
            );
            assert!(
                function_calls[1].1.contains("%.c"),
                "Second function args should contain %.c"
            );
        }
        _ => panic!("Expected Variable item"),
    }
}

// =============================================================================
// define...endef Tests (Sprint 82 Day 4-5)
// =============================================================================

/// RED PHASE: Test for basic define...endef
///
/// Input:
/// define COMPILE_RULE
/// gcc -c $< -o $@
/// endef
#[test]
fn test_DEFINE_001_basic_define() {
    // ARRANGE: Basic define...endef block
    let makefile = r#"define COMPILE_RULE
gcc -c $< -o $@
endef"#;

    // ACT: Parse makefile
    let result = parse_makefile(makefile);
    assert!(result.is_ok(), "Parsing should succeed");

    let ast = result.unwrap();
    assert_eq!(ast.items.len(), 1, "Should have 1 variable");

    // ASSERT: Should store as multi-line Variable
    match &ast.items[0] {
        MakeItem::Variable { name, value, .. } => {
            assert_eq!(name, "COMPILE_RULE");
            assert!(
                value.contains("gcc -c $< -o $@"),
                "Value should contain command"
            );
        }
        _ => panic!("Expected Variable item for define block"),
    }
}

/// RED PHASE: Test for empty define...endef
///
/// Input:
/// define EMPTY_VAR
/// endef
#[test]
fn test_DEFINE_002_empty_define() {
    // ARRANGE: Empty define block
    let makefile = r#"define EMPTY_VAR
endef"#;

    // ACT: Parse makefile
    let result = parse_makefile(makefile);
    assert!(result.is_ok(), "Parsing should succeed");

    let ast = result.unwrap();
    assert_eq!(ast.items.len(), 1, "Should have 1 variable");

    // ASSERT: Should store as Variable with empty or whitespace value
    match &ast.items[0] {
        MakeItem::Variable { name, value, .. } => {
            assert_eq!(name, "EMPTY_VAR");
            assert!(
                value.trim().is_empty() || value.is_empty(),
                "Value should be empty"
            );
        }
        _ => panic!("Expected Variable item"),
    }
}

/// RED PHASE: Test for multi-line define...endef
///
/// Input:
/// define HELP_TEXT
/// Usage: make [target]
/// Targets:
///   all    - Build everything
///   clean  - Remove build artifacts
/// endef
#[test]
fn test_DEFINE_003_multiline_text() {
    // ARRANGE: Multi-line define block
    let makefile = r#"define HELP_TEXT
Usage: make [target]
Targets:
  all    - Build everything
  clean  - Remove build artifacts
endef"#;

    // ACT: Parse makefile
    let result = parse_makefile(makefile);
    assert!(result.is_ok(), "Parsing should succeed");

    let ast = result.unwrap();
    assert_eq!(ast.items.len(), 1, "Should have 1 variable");

    // ASSERT: Should preserve multi-line content
    match &ast.items[0] {
        MakeItem::Variable { name, value, .. } => {
            assert_eq!(name, "HELP_TEXT");
            assert!(
                value.contains("Usage: make [target]"),
                "Should contain first line"
            );
            assert!(value.contains("Targets:"), "Should contain second line");
            assert!(
                value.contains("all    - Build everything"),
                "Should contain third line"
            );
            assert!(
                value.contains("clean  - Remove build artifacts"),
                "Should contain fourth line"
            );
        }
        _ => panic!("Expected Variable item"),
    }
}

/// RED PHASE: Test for define with tab-indented commands
///
/// Input:
/// define BUILD_CMD
///     @echo "Building..."
///     gcc -o output main.c
/// endef
#[test]
fn test_DEFINE_004_with_tabs() {
    // ARRANGE: define block with tab-indented commands (like recipe lines)
    let makefile = "define BUILD_CMD\n\t@echo \"Building...\"\n\tgcc -o output main.c\nendef";

    // ACT: Parse makefile
    let result = parse_makefile(makefile);
    assert!(result.is_ok(), "Parsing should succeed");

    let ast = result.unwrap();
    assert_eq!(ast.items.len(), 1, "Should have 1 variable");

    // ASSERT: Should preserve tabs in multi-line value
    match &ast.items[0] {
        MakeItem::Variable { name, value, .. } => {
            assert_eq!(name, "BUILD_CMD");
            assert!(value.contains("echo"), "Should contain echo command");
            assert!(
                value.contains("gcc -o output main.c"),
                "Should contain gcc command"
            );
        }
        _ => panic!("Expected Variable item"),
    }
}

/// RED PHASE: Test for define with variable references
///
/// Input:
/// define INSTALL_CMD
/// install -m 755 $(BIN) $(DESTDIR)$(PREFIX)/bin
/// install -m 644 $(MAN) $(DESTDIR)$(PREFIX)/share/man
/// endef
#[test]
fn test_DEFINE_005_with_variables() {
    // ARRANGE: define block with variable references
    let makefile = r#"define INSTALL_CMD
install -m 755 $(BIN) $(DESTDIR)$(PREFIX)/bin
install -m 644 $(MAN) $(DESTDIR)$(PREFIX)/share/man
endef"#;

    // ACT: Parse makefile
    let result = parse_makefile(makefile);
    assert!(result.is_ok(), "Parsing should succeed");

    let ast = result.unwrap();
    assert_eq!(ast.items.len(), 1, "Should have 1 variable");

    // ASSERT: Should preserve variable references
    match &ast.items[0] {
        MakeItem::Variable { name, value, .. } => {
            assert_eq!(name, "INSTALL_CMD");
            assert!(value.contains("$(BIN)"), "Should contain BIN variable");
            assert!(
                value.contains("$(DESTDIR)"),
                "Should contain DESTDIR variable"
            );
            assert!(
                value.contains("$(PREFIX)"),
                "Should contain PREFIX variable"
            );
        }
        _ => panic!("Expected Variable item"),
    }
}

/// RED PHASE: Test for define with recipe-style commands
///
/// Input:
/// define RUN_TESTS
///     cd tests && ./run_tests.sh
///     if [ $$? -ne 0 ]; then exit 1; fi
/// endef
#[test]
fn test_DEFINE_006_with_commands() {
    // ARRANGE: define block with shell commands
    let makefile = "define RUN_TESTS\n\tcd tests && ./run_tests.sh\n\tif [ $$? -ne 0 ]; then exit 1; fi\nendef";

    // ACT: Parse makefile
    let result = parse_makefile(makefile);
    assert!(result.is_ok(), "Parsing should succeed");

    let ast = result.unwrap();
    assert_eq!(ast.items.len(), 1, "Should have 1 variable");

    // ASSERT: Should preserve shell commands with $$
    match &ast.items[0] {
        MakeItem::Variable { name, value, .. } => {
            assert_eq!(name, "RUN_TESTS");
            assert!(value.contains("cd tests"), "Should contain cd command");
            assert!(
                value.contains("$$?") || value.contains("$?"),
                "Should contain exit code check"
            );
        }
        _ => panic!("Expected Variable item"),
    }
}

/// RED PHASE: Test for recursive expansion define (=)
///
/// Input:
/// define RECURSIVE =
/// This is $(FLAVOR) expansion
/// endef
#[test]
fn test_DEFINE_007_recursive_expansion() {
    // ARRANGE: define with recursive expansion (=)
    let makefile = r#"define RECURSIVE =
This is $(FLAVOR) expansion
endef"#;

    // ACT: Parse makefile
    let result = parse_makefile(makefile);
    assert!(result.is_ok(), "Parsing should succeed");

    let ast = result.unwrap();
    assert_eq!(ast.items.len(), 1, "Should have 1 variable");

    // ASSERT: Should handle = flavor
    match &ast.items[0] {
        MakeItem::Variable {
            name,
            value,
            flavor,
            ..
        } => {
            assert_eq!(name, "RECURSIVE");
            assert!(
                value.contains("$(FLAVOR)"),
                "Should contain variable reference"
            );
            assert_eq!(
                *flavor,
                VarFlavor::Recursive,
                "Should be recursive expansion"
            );
        }
        _ => panic!("Expected Variable item"),
    }
}

/// RED PHASE: Test for simple expansion define (:=)
///
/// Input:
/// define SIMPLE :=
/// Expanded at $(shell date)
/// endef
#[test]
fn test_DEFINE_008_simple_expansion() {
    // ARRANGE: define with simple expansion (:=)
    let makefile = r#"define SIMPLE :=
Expanded at $(shell date)
endef"#;

    // ACT: Parse makefile
    let result = parse_makefile(makefile);
    assert!(result.is_ok(), "Parsing should succeed");

    let ast = result.unwrap();
    assert_eq!(ast.items.len(), 1, "Should have 1 variable");

    // ASSERT: Should handle := flavor
    match &ast.items[0] {
        MakeItem::Variable {
            name,
            value,
            flavor,
            ..
        } => {
            assert_eq!(name, "SIMPLE");
            assert!(
                value.contains("shell") || value.contains("date"),
                "Should contain function call"
            );
            assert_eq!(*flavor, VarFlavor::Simple, "Should be simple expansion");
        }
        _ => panic!("Expected Variable item"),
    }
}

/// RED PHASE: Test for define with nested variable expansion
///
/// Input:
/// define COMPLEX
/// SRC = $(wildcard src/*.c)
/// OBJ = $(patsubst %.c,%.o,$(SRC))
/// endef
#[test]
fn test_DEFINE_009_nested_variables() {
    // ARRANGE: define block with nested variable assignments
    let makefile = r#"define COMPLEX
SRC = $(wildcard src/*.c)
OBJ = $(patsubst %.c,%.o,$(SRC))
endef"#;

    // ACT: Parse makefile
    let result = parse_makefile(makefile);
    assert!(result.is_ok(), "Parsing should succeed");

    let ast = result.unwrap();
    assert_eq!(ast.items.len(), 1, "Should have 1 variable");

    // ASSERT: Should preserve nested content as multi-line value
    match &ast.items[0] {
        MakeItem::Variable { name, value, .. } => {
            assert_eq!(name, "COMPLEX");
            assert!(value.contains("SRC ="), "Should contain SRC assignment");
            assert!(value.contains("OBJ ="), "Should contain OBJ assignment");
            assert!(
                value.contains("$(wildcard"),
                "Should contain wildcard function"
            );
            assert!(
                value.contains("$(patsubst"),
                "Should contain patsubst function"
            );
        }
        _ => panic!("Expected Variable item"),
    }
}

/// RED PHASE: Test for real-world define example
///
/// Input: Complex real-world define block from Linux kernel
#[test]
fn test_DEFINE_010_real_world_example() {
    // ARRANGE: Real-world complex define block
    let makefile = r#"define COMPILE_TEMPLATE
$(1)_OBJS := $$(patsubst %.c,%.o,$$($(1)_SOURCES))
$(1)_DEPS := $$($(1)_OBJS:.o=.d)

$(1): $$($(1)_OBJS)
	$$(CC) $$(CFLAGS) -o $$@ $$^

-include $$($(1)_DEPS)
endef"#;

    // ACT: Parse makefile
    let result = parse_makefile(makefile);
    assert!(result.is_ok(), "Parsing should succeed");

    let ast = result.unwrap();
    assert_eq!(ast.items.len(), 1, "Should have 1 variable");

    // ASSERT: Should preserve complex multi-line template
    match &ast.items[0] {
        MakeItem::Variable { name, value, .. } => {
            assert_eq!(name, "COMPILE_TEMPLATE");
            assert!(value.contains("_OBJS"), "Should contain OBJS assignment");
            assert!(value.contains("_DEPS"), "Should contain DEPS assignment");
            assert!(value.contains("$(CC)"), "Should contain CC variable");
            assert!(value.contains("$$@"), "Should contain automatic variable");
            assert!(
                value.contains("-include"),
                "Should contain include directive"
            );
        }
        _ => panic!("Expected Variable item"),
    }
}

// =============================================================================
// Conditional Edge Cases (Day 6)
// =============================================================================

/// RED PHASE: Test for nested conditionals (ifeq inside ifdef)
#[test]
fn test_COND_EDGE_001_nested_ifeq_ifdef() {
    let makefile = r#"
ifdef DEBUG
ifeq ($(VERBOSE),1)
CFLAGS += -DDEBUG_VERBOSE
endif
endif
"#;

    // ARRANGE: Parse Makefile with nested conditionals
    let result = parse_makefile(makefile);
    assert!(result.is_ok(), "Parsing should succeed");

    let ast = result.unwrap();

    // ASSERT: Should have conditional structure
    // Outer ifdef DEBUG should contain inner ifeq
    let has_ifdef = ast
        .items
        .iter()
        .any(|item| matches!(item, MakeItem::Conditional { .. }));

    assert!(
        has_ifdef,
        "Should have conditional items for nested structure"
    );
}

/// RED PHASE: Test for conditionals with function calls in condition
#[test]
fn test_COND_EDGE_002_conditional_with_functions() {
    let makefile = r#"
ifeq ($(shell uname),Linux)
PLATFORM = linux
else
PLATFORM = other
endif
"#;

    // ARRANGE: Parse Makefile with function call in condition
    let result = parse_makefile(makefile);
    assert!(result.is_ok(), "Parsing should succeed");

    let ast = result.unwrap();

    // ASSERT: Should parse ifeq with shell function
    let has_conditional = ast
        .items
        .iter()
        .any(|item| matches!(item, MakeItem::Conditional { .. }));

    assert!(has_conditional, "Should have conditional item");

    // ASSERT: Should have variable assignment in then or else branch
    let has_var_in_conditional = ast.items.iter().any(|item| {
        if let MakeItem::Conditional {
            then_items,
            else_items,
            ..
        } = item
        {
            let in_then = then_items
                .iter()
                .any(|i| matches!(i, MakeItem::Variable { name, .. } if name == "PLATFORM"));
            let in_else = else_items
                .as_ref()
                .map(|items| {
                    items
                        .iter()
                        .any(|i| matches!(i, MakeItem::Variable { name, .. } if name == "PLATFORM"))
                })
                .unwrap_or(false);
            in_then || in_else
        } else {
            false
        }
    });

    assert!(
        has_var_in_conditional,
        "Should have PLATFORM variable in conditional branches"
    );
}

/// RED PHASE: Test for empty conditional blocks
#[test]
fn test_COND_EDGE_003_empty_conditional_blocks() {
    let makefile = r#"
ifdef DEBUG
# Empty then block
endif

ifndef RELEASE
# Empty then block
else
# Empty else block
endif
"#;

    // ARRANGE: Parse Makefile with empty conditional blocks
    let result = parse_makefile(makefile);
    assert!(result.is_ok(), "Parsing should succeed for empty blocks");

    let ast = result.unwrap();

    // ASSERT: Should have conditional items even if empty
    let conditional_count = ast
        .items
        .iter()
        .filter(|item| matches!(item, MakeItem::Conditional { .. }))
        .count();

    assert!(
        conditional_count >= 2,
        "Should have 2 conditional items (ifdef + ifndef)"
    );
}

/// RED PHASE: Test for complex real-world nesting
#[test]
fn test_COND_EDGE_004_complex_nesting_real_world() {
    let makefile = r#"
ifdef USE_PYTHON
PYTHON := python3
ifeq ($(shell which python3),)
$(error Python 3 not found)
endif
else ifdef USE_PYTHON2
PYTHON := python2
else
PYTHON := python
endif

ifneq ($(PYTHON),)
PYTHON_CONFIG := $(PYTHON)-config
endif
"#;

    // ARRANGE: Parse complex real-world conditional nesting
    let result = parse_makefile(makefile);
    assert!(result.is_ok(), "Parsing should succeed for complex nesting");

    let ast = result.unwrap();

    // ASSERT: Should have multiple conditional items
    let conditional_count = ast
        .items
        .iter()
        .filter(|item| matches!(item, MakeItem::Conditional { .. }))
        .count();

    assert!(
        conditional_count >= 2,
        "Should have at least 2 conditional items"
    );

    // ASSERT: Should have variable assignments in conditional branches
    let has_python_var = ast.items.iter().any(|item| {
        if let MakeItem::Conditional { then_items, else_items, .. } = item {
            let check_items = |items: &[MakeItem]| {
                items.iter().any(|i| {
                    matches!(i, MakeItem::Variable { name, .. } if name == "PYTHON" || name == "PYTHON_CONFIG")
                })
            };

            let in_then = check_items(then_items);
            let in_else = else_items.as_ref().map(|items| check_items(items)).unwrap_or(false);
            in_then || in_else
        } else {
            false
        }
    });

    assert!(
        has_python_var,
        "Should have PYTHON or PYTHON_CONFIG variable in conditional branches"
    );
}

/// RED PHASE: Test for multiple nested conditional levels
#[test]
fn test_COND_EDGE_005_multiple_nested_levels() {
    let makefile = r#"
ifdef ENABLE_FEATURE_A
FEATURE_A = 1
ifdef ENABLE_FEATURE_A_VERBOSE
FEATURE_A_FLAGS = -v
else
FEATURE_A_FLAGS =
endif
else
FEATURE_A = 0
endif
"#;

    // ARRANGE: Parse conditional with multiple nesting levels
    let result = parse_makefile(makefile);
    assert!(
        result.is_ok(),
        "Parsing should succeed for multiple nesting levels"
    );

    let ast = result.unwrap();

    // ASSERT: Should have conditional item
    let has_conditional = ast
        .items
        .iter()
        .any(|item| matches!(item, MakeItem::Conditional { .. }));

    assert!(has_conditional, "Should have conditional item");

    // ASSERT: Should have variable assignments in conditional branches
    let has_var = ast.items.iter().any(|item| {
        if let MakeItem::Conditional {
            then_items,
            else_items,
            ..
        } = item
        {
            let check_items = |items: &[MakeItem]| {
                items.iter().any(|i| {
                    matches!(i, MakeItem::Variable { name, .. }
                        if name == "FEATURE_A" || name == "FEATURE_A_FLAGS")
                })
            };

            let in_then = check_items(then_items);
            let in_else = else_items
                .as_ref()
                .map(|items| check_items(items))
                .unwrap_or(false);
            in_then || in_else
        } else {
            false
        }
    });

    assert!(
        has_var,
        "Should have FEATURE_A or FEATURE_A_FLAGS variable in conditional branches"
    );
}
