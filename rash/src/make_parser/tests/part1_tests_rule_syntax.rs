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
